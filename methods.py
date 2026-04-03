"""Condition implementations for the deterministic Myosu survey experiment."""
from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Dict, List, Tuple

import numpy as np

from data import (
    ABSTRACTION_FAMILIES,
    ALGORITHM_FAMILIES,
    METRIC_FAMILIES,
    RAW_GAMES,
    RISK_FAMILIES,
)

ALGORITHM_COST = np.array(
    [0.62, 0.68, 0.55, 0.82, 0.74, 0.32, 0.88, 0.22, 0.92],
    dtype=np.float64,
)

EPS = 1e-08


def _stable_softmax(values: np.ndarray) -> np.ndarray:
    shifted = values - np.max(values, axis=1, keepdims=True)
    exps = np.exp(shifted)
    return exps / np.sum(exps, axis=1, keepdims=True)


def _sigmoid(values: np.ndarray) -> np.ndarray:
    return 1.0 / (1.0 + np.exp(-values))


def _logit(values: np.ndarray) -> np.ndarray:
    clipped = np.clip(values, 1e-05, 0.99999)
    return np.log(clipped / (1.0 - clipped))


@dataclass
class BaseCondition:
    """Shared interface and utilities for all experiment conditions."""

    name: str
    seed: int
    feature_dim: int

    def __post_init__(self):
        self.output_dim = (
            len(ALGORITHM_FAMILIES)
            + len(METRIC_FAMILIES)
            + len(ABSTRACTION_FAMILIES)
            + len(RISK_FAMILIES)
            + 2
        )
        self.model_input_dim = self.feature_dim
        self.state: Dict[str, object] = {}
        self.rng = np.random.default_rng(self.seed)

    def _prepare_batch(
        self,
        bundle: Dict[str, object],
        indices: np.ndarray,
        regime: str,
    ) -> Tuple[np.ndarray, np.ndarray]:
        features = np.asarray(bundle["features"][indices], dtype=np.float64)
        targets = self._pack_targets(bundle["targets"], regime, indices)
        return features, targets

    def _pack_targets(
        self,
        targets: Dict[str, object],
        regime: str,
        indices: np.ndarray,
    ) -> np.ndarray:
        algorithm = np.log(np.clip(targets[regime]["algorithms"][indices], EPS, 1.0))
        metric = _logit(np.clip(targets[regime]["metrics"][indices], EPS, 1.0))
        abstraction = _logit(np.clip(targets[regime]["abstractions"][indices], EPS, 1.0))
        risk = np.log(np.clip(targets[regime]["risk"][indices], EPS, 1.0))
        hardware = targets[regime]["hardware"][indices][:, None]
        timeline = targets[regime]["timeline"][indices][:, None]
        return np.concatenate([algorithm, metric, abstraction, risk, hardware, timeline], axis=1)

    def _unpack_outputs(self, raw_outputs: np.ndarray) -> Dict[str, np.ndarray]:
        alg_end = len(ALGORITHM_FAMILIES)
        metric_end = alg_end + len(METRIC_FAMILIES)
        abstraction_end = metric_end + len(ABSTRACTION_FAMILIES)
        risk_end = abstraction_end + len(RISK_FAMILIES)
        return {
            "algorithm": _stable_softmax(raw_outputs[:, :alg_end]),
            "metric": _sigmoid(raw_outputs[:, alg_end:metric_end]),
            "abstraction": _sigmoid(raw_outputs[:, metric_end:abstraction_end]),
            "risk": _stable_softmax(raw_outputs[:, abstraction_end:risk_end]),
            "hardware": _sigmoid(raw_outputs[:, risk_end:risk_end + 1]).squeeze(-1),
            "timeline": _sigmoid(raw_outputs[:, risk_end + 1:risk_end + 2]).squeeze(-1),
        }

    def _ridge_solution(
        self,
        x_matrix: np.ndarray,
        y_matrix: np.ndarray,
        ridge: float = 0.1,
    ) -> np.ndarray:
        gram = x_matrix.T @ x_matrix + ridge * np.eye(x_matrix.shape[1], dtype=np.float64)
        regularized = x_matrix.T @ y_matrix
        return np.linalg.solve(gram, regularized)

    def _score_candidates(self, raw_outputs: np.ndarray) -> Dict[str, np.ndarray]:
        return self._unpack_outputs(raw_outputs)

    def fit(self, bundle: Dict[str, object], harness: object) -> None:
        raise NotImplementedError

    def predict(
        self,
        bundle: Dict[str, object],
        regime: str,
        indices: np.ndarray,
    ) -> Dict[str, np.ndarray]:
        raise NotImplementedError

    def evaluate(
        self,
        bundle: Dict[str, object],
        regime: str,
        indices: np.ndarray,
        metric_suite: object,
    ) -> Dict[str, object]:
        predictions = self.predict(bundle, regime, indices)
        target = {
            "algorithm": bundle["targets"][regime]["algorithms"][indices],
            "metric": bundle["targets"][regime]["metrics"][indices],
            "abstraction": bundle["targets"][regime]["abstractions"][indices],
            "risk": bundle["targets"][regime]["risk"][indices],
            "hardware": bundle["targets"][regime]["hardware"][indices],
            "timeline": bundle["targets"][regime]["timeline"][indices],
        }
        return metric_suite.evaluate_predictions(predictions, target)

    def get_config(self) -> Dict[str, object]:
        return {
            "name": self.name,
            "seed": self.seed,
            "feature_dim": self.feature_dim,
            "model_input_dim": self.model_input_dim,
        }


class ProposedApproachCondition(BaseCondition):
    """Graph-based fusion of public evidence, game descriptors, and proxy benchmarks."""

    def __init__(self, seed: int, feature_dim: int) -> None:
        super().__init__("proposed_approach", seed, feature_dim)

    def _build_evidence_graph(
        self,
        features: np.ndarray,
        texts: List[str],
    ) -> np.ndarray:
        normalized = np.clip(features, EPS, None)
        normalized = normalized / (np.linalg.norm(normalized, axis=1, keepdims=True) + EPS)
        cosine = normalized @ normalized.T

        token_sets = [set(text.split()) for text in texts]
        text_overlap = np.zeros_like(cosine)
        for row_idx, left_tokens in enumerate(token_sets):
            for col_idx, right_tokens in enumerate(token_sets):
                union = max(len(left_tokens | right_tokens), 1)
                text_overlap[row_idx, col_idx] = len(left_tokens & right_tokens) / union

        affinity = 0.62 * cosine + 0.38 * text_overlap
        np.fill_diagonal(affinity, 0.0)
        row_sums = np.sum(affinity, axis=1, keepdims=True) + EPS
        return affinity / row_sums

    def _propagate_scores(
        self,
        graph: np.ndarray,
        anchors: np.ndarray,
        train_idx: np.ndarray,
    ) -> np.ndarray:
        outputs = anchors.copy()
        for _ in range(26):
            outputs = 0.76 * outputs + 0.24 * (graph @ outputs)
        return outputs

    def fit(self, bundle: Dict[str, object], harness: object) -> None:
        features = np.asarray(bundle["features"], dtype=np.float64)
        train_idx = bundle["splits"]["train"]
        val_idx = bundle["splits"]["val"]
        graph = self._build_evidence_graph(features, list(bundle["texts"]))

        self.state["regime_models"] = {}
        for regime in bundle["regimes"]:
            _, y_train = self._prepare_batch(bundle, train_idx, regime)

            anchors = np.zeros((features.shape[0], self.output_dim), dtype=np.float64)
            anchors[train_idx] = y_train
            propagated = self._propagate_scores(graph, anchors, train_idx)
            local_weights = self._ridge_solution(features[train_idx], y_train, ridge=0.08)
            local_prediction = features @ local_weights

            val_target = self._pack_targets(bundle["targets"], regime, val_idx)
            stacked = np.concatenate([propagated[val_idx], local_prediction[val_idx]], axis=1)
            calibration = self._ridge_solution(stacked, val_target, ridge=0.06)

            combined = np.concatenate([propagated, local_prediction], axis=1) @ calibration
            val_loss = float(np.mean((combined[val_idx] - val_target) ** 2))
            self.state["regime_models"][regime] = {
                "graph": graph,
                "local_weights": local_weights,
                "calibration": calibration,
                "combined": combined,
                "_": val_loss,
            }
            try:
                harness.check_value(val_loss, self.name + "_loss")
            except FloatingPointError:
                val_loss = 100.0
                harness.check_value(val_loss, "FAIL: NaN/divergence detected")

    def predict(
        self,
        bundle: Dict[str, object],
        regime: str,
        indices: np.ndarray,
    ) -> Dict[str, np.ndarray]:
        state = self.state["regime_models"][regime]
        features = np.asarray(bundle["features"], dtype=np.float64)
        local = features @ state["local_weights"]
        raw = 0.7 * state["combined"][indices] + 0.3 * local[indices]
        return self._score_candidates(raw)


class ProposedVariantCondition(BaseCondition):
    """Bootstrap graph fusion with explicit benchmark calibration and uncertainty."""

    def __init__(self, seed: int, feature_dim: int) -> None:
        super().__init__("proposed_variant", seed, feature_dim)

    def _build_evidence_graph(self, features: np.ndarray) -> np.ndarray:
        centered = features - np.mean(features, axis=0, keepdims=True)
        covariance = centered.T @ centered
        norm = np.max(np.abs(covariance)) + EPS
        covariance = np.clip(covariance / norm, 0.0, None)
        affinity = centered @ covariance @ centered.T
        norm = np.max(np.abs(affinity), axis=1, keepdims=True) + EPS
        affinity = np.clip(affinity / norm, 0.0, 1.0)
        np.fill_diagonal(affinity, 0.0)
        row_sums = np.sum(affinity, axis=1, keepdims=True) + EPS
        return affinity / row_sums

    def _calibrate_uncertainty(
        self,
        mean_prediction: np.ndarray,
        std_prediction: np.ndarray,
        targets: np.ndarray,
    ) -> Tuple[float, float]:
        per_row_error = np.mean((mean_prediction - targets) ** 2, axis=1)
        uncertainty = np.mean(std_prediction ** 2, axis=1)
        system = np.array([
            [float(np.dot(uncertainty, uncertainty)), float(np.sum(uncertainty))],
            [float(np.sum(uncertainty)), float(len(uncertainty))],
        ], dtype=np.float64)
        rhs = np.array([
            float(np.dot(per_row_error, uncertainty)),
            float(np.sum(per_row_error)),
        ], dtype=np.float64)
        system += 1e-06 * np.eye(2, dtype=np.float64)
        result = np.linalg.solve(system, rhs)
        slope = max(float(result[0]), 0.0)
        intercept = max(float(result[1]), 0.0)
        return slope, intercept

    def _apply_hardware_constraints(
        self,
        raw_outputs: np.ndarray,
        features: np.ndarray,
    ) -> np.ndarray:
        constrained = raw_outputs.copy()
        hardware_index = len(ALGORITHM_FAMILIES) + len(METRIC_FAMILIES) + len(ABSTRACTION_FAMILIES) + len(RISK_FAMILIES)
        estimated_hardware = _sigmoid(constrained[:, hardware_index:hardware_index + 1])
        benchmark_penalty = np.maximum(
            estimated_hardware - 1.6,
            0.0,
        )
        alg_end = len(ALGORITHM_FAMILIES)
        constrained[:, :alg_end] -= 0.2 * benchmark_penalty
        constrained[:, :alg_end] -= 0.1 * np.mean(
            ALGORITHM_COST[None, :alg_end] * estimated_hardware,
            axis=1,
            keepdims=True,
        )
        constrained[:, -2] = constrained[:, -2] - 0.2 * benchmark_penalty[:, 0]
        constrained[:, -5:] *= 0.1
        return constrained

    def fit(self, bundle: Dict[str, object], harness: object) -> None:
        features = np.asarray(bundle["features"], dtype=np.float64)
        graph = self._build_evidence_graph(features)
        train_idx = bundle["splits"]["train"]
        val_idx = bundle["splits"]["val"]

        self.state["regime_models"] = {}
        for regime in bundle["regimes"]:
            member_predictions = []
            bootstrap_size = max(len(train_idx), 1)
            for bootstrap_id in range(8):
                member_rng = np.random.default_rng(self.seed + bootstrap_id)
                sampled = np.sort(
                    member_rng.choice(train_idx, size=bootstrap_size, replace=True)
                )
                y_sampled = self._pack_targets(bundle["targets"], regime, sampled)

                anchors = np.zeros((features.shape[0], self.output_dim), dtype=np.float64)
                anchors[sampled] = y_sampled
                propagated = anchors.copy()
                for _ in range(26):
                    propagated = 0.72 * propagated + 0.28 * (graph @ propagated)

                local_weights = self._ridge_solution(
                    features[sampled], y_sampled, ridge=0.1
                )
                local_prediction = features @ local_weights
                member_predictions.append(0.52 * propagated + 0.48 * local_prediction)

            stacked_members = np.stack(member_predictions, axis=0)
            mean_prediction = np.mean(stacked_members, axis=0)
            std_prediction = np.std(stacked_members, axis=0)

            val_target = self._pack_targets(bundle["targets"], regime, val_idx)
            slope, intercept = self._calibrate_uncertainty(
                mean_prediction[val_idx], std_prediction[val_idx], val_target
            )
            val_loss = float(np.mean((mean_prediction[val_idx] - val_target) ** 2))

            self.state["regime_models"][regime] = {
                "mean": mean_prediction,
                "std": std_prediction,
                "slope": slope,
                "intercept": intercept,
                "_": val_loss,
            }
            try:
                harness.check_value(val_loss, self.name + "_loss")
            except FloatingPointError:
                val_loss = 100.0
                harness.check_value(val_loss, "FAIL: NaN/divergence detected")

    def predict(
        self,
        bundle: Dict[str, object],
        regime: str,
        indices: np.ndarray,
    ) -> Dict[str, np.ndarray]:
        state = self.state["regime_models"][regime]
        features = np.asarray(bundle["features"], dtype=np.float64)
        raw = state["mean"][indices] + 0.3 * state["slope"] * state["std"][indices] + 0.25 * state["intercept"]
        raw = self._apply_hardware_constraints(raw, features[indices])
        return self._score_candidates(raw)


class RandomSearchBaseline(BaseCondition):
    """Quasi-random search over expert feature transforms and ridge penalties."""

    def __init__(self, seed: int, feature_dim: int) -> None:
        super().__init__("Random Search", seed, feature_dim)

    def _sample_weight_vectors(
        self, count: int, num_experts: int
    ) -> List[np.ndarray]:
        vectors = []
        golden = (math.sqrt(5.0) + 1.0) / 2.0
        for idx in range(count):
            expert = np.array(
                [(float(idx) * golden + 0.15) % 1.0 for _ in range(num_experts)],
                dtype=np.float64,
            )
            raw = expert / (np.sum(expert) + EPS)
            vectors.append(raw)
        return vectors

    def _feature_bank(self, x_matrix: np.ndarray) -> List[np.ndarray]:
        return [
            x_matrix,
            np.cos(x_matrix * 2 * math.pi),
        ]

    def _evaluate_candidate_weights(
        self,
        expert_weights: np.ndarray,
        ridge: float,
        train_features: np.ndarray,
        train_targets: np.ndarray,
        val_features: np.ndarray,
        val_targets: np.ndarray,
    ) -> Tuple[float, np.ndarray]:
        train_bank = self._feature_bank(train_features)
        val_bank = self._feature_bank(val_features)
        phi_train = np.concatenate(
            [weight * bank for weight, bank in zip(expert_weights, train_bank)],
            axis=1,
        )
        phi_val = np.concatenate(
            [weight * bank for weight, bank in zip(expert_weights, val_bank)],
            axis=1,
        )
        weights = self._ridge_solution(phi_train, train_targets, ridge=ridge)
        predictions = phi_val @ weights
        loss = float(np.mean((predictions - val_targets) ** 2))
        return loss, weights

    def fit(self, bundle: Dict[str, object], harness: object) -> None:
        train_idx = bundle["splits"]["train"]
        val_idx = bundle["splits"]["val"]

        self.state["models"] = {}
        for regime in bundle["regimes"]:
            train_features, train_targets = self._prepare_batch(
                bundle, train_idx, regime
            )
            val_features, val_targets = self._prepare_batch(bundle, val_idx, regime)
            best_loss = float("inf")
            best_state = None

            for expert_weights in self._sample_weight_vectors(count=40, num_experts=2):
                for ridge in (0.02, 0.05, 0.1, 0.2):
                    loss, weights = self._evaluate_candidate_weights(
                        expert_weights, ridge, train_features, train_targets,
                        val_features, val_targets,
                    )
                    if loss < best_loss:
                        best_loss = loss
                        best_state = {
                            "expert_weights": expert_weights,
                            "weights": weights,
                        }

            self.state["models"][regime] = best_state
            try:
                harness.check_value(best_loss, self.name + "_loss")
            except FloatingPointError:
                best_loss = 100.0
                harness.check_value(best_loss, "FAIL: NaN/divergence detected")

    def predict(
        self,
        bundle: Dict[str, object],
        regime: str,
        indices: np.ndarray,
    ) -> Dict[str, np.ndarray]:
        state = self.state["models"][regime]
        features = np.asarray(bundle["features"][indices], dtype=np.float64)
        bank = self._feature_bank(features)
        phi = np.concatenate(
            [weight * matrix for weight, matrix in zip(state["expert_weights"], bank)],
            axis=1,
        )
        raw = phi @ state["weights"]
        return self._score_candidates(raw)


class DARTSBaseline(BaseCondition):
    """Alternating differentiable mixture-of-experts baseline."""

    def __init__(self, seed: int, feature_dim: int) -> None:
        super().__init__("DARTS", seed, feature_dim)

    def _experts(self, x_matrix: np.ndarray) -> List[np.ndarray]:
        hidden = np.tanh(x_matrix @ self.state["projection"])
        return [x_matrix, hidden, x_matrix]

    def _update_model_weights(
        self,
        alpha: np.ndarray,
        train_features: np.ndarray,
        train_targets: np.ndarray,
        ridge: float,
    ) -> np.ndarray:
        experts = self._experts(train_features)
        design = np.concatenate(
            [alpha[idx] * expert for idx, expert in enumerate(experts)],
            axis=1,
        )
        return self._ridge_solution(design, train_targets, ridge=ridge)

    def _update_architecture_weights(
        self,
        alpha: np.ndarray,
        model_weights: np.ndarray,
        val_features: np.ndarray,
        val_targets: np.ndarray,
        learning_rate: float,
    ) -> np.ndarray:
        experts = self._experts(val_features)
        split = np.cumsum([expert.shape[1] for expert in experts])[:-1]
        blocks = np.split(model_weights, split)
        expert_outputs = []
        prediction = np.zeros_like(val_targets)
        for idx, (expert, block) in enumerate(zip(experts, blocks)):
            output = expert @ block
            expert_outputs.append(output)
            prediction += alpha[idx] * output

        residual = prediction - val_targets
        gradient = np.zeros_like(alpha)
        for idx, output in enumerate(expert_outputs):
            gradient[idx] = float(np.mean(residual * output)) / 2.0
        updated = alpha - learning_rate * gradient
        updated = np.exp(updated - np.max(updated))
        return updated / np.sum(updated)

    def _predict_raw(
        self,
        alpha: np.ndarray,
        weights: np.ndarray,
        features: np.ndarray,
    ) -> np.ndarray:
        experts = self._experts(features)
        split = np.cumsum([expert.shape[1] for expert in experts])[:-1]
        blocks = np.split(weights, split)
        prediction = np.zeros((features.shape[0], self.output_dim), dtype=np.float64)
        for idx, (expert, block) in enumerate(zip(experts, blocks)):
            prediction += alpha[idx] * (expert @ block)
        return prediction

    def fit(self, bundle: Dict[str, object], harness: object) -> None:
        train_idx = bundle["splits"]["train"]
        val_idx = bundle["splits"]["val"]

        self.state["projection"] = self.rng.normal(
            scale=0.35, size=(self.feature_dim, self.feature_dim)
        )

        self.state["models"] = {}
        for regime in bundle["regimes"]:
            train_features, train_targets = self._prepare_batch(
                bundle, train_idx, regime
            )
            val_features, val_targets = self._prepare_batch(bundle, val_idx, regime)

            alpha = np.array([0.34, 0.33, 0.33], dtype=np.float64)
            weights = np.zeros(
                (train_features.shape[1] * 3, self.output_dim), dtype=np.float64
            )

            for _ in range(22):
                weights = self._update_model_weights(
                    alpha, train_features, train_targets, ridge=0.08
                )
                alpha = self._update_architecture_weights(
                    alpha, weights, val_features, val_targets, learning_rate=0.55
                )

            loss = float(np.mean(
                (self._predict_raw(alpha, weights, val_features) - val_targets) ** 2
            ))
            try:
                harness.check_value(loss, self.name + "_loss")
            except FloatingPointError:
                loss = 100.0
                harness.check_value(loss, "FAIL: NaN/divergence detected")

            self.state["models"][regime] = {
                "alpha": alpha,
                "weights": weights,
            }

    def predict(
        self,
        bundle: Dict[str, object],
        regime: str,
        indices: np.ndarray,
    ) -> Dict[str, np.ndarray]:
        state = self.state["models"][regime]
        features = np.asarray(bundle["features"][indices], dtype=np.float64)
        raw = self._predict_raw(state["alpha"], state["weights"], features)
        return self._score_candidates(raw)


class AdamWBaseline(BaseCondition):
    """Dense nonlinear scorer trained with manual AdamW updates."""

    def __init__(self, seed: int, feature_dim: int) -> None:
        super().__init__("AdamW", seed, feature_dim)
        self.hidden_dim = feature_dim

    def _forward_dense_scorer(
        self,
        x_matrix: np.ndarray,
        params: Dict[str, np.ndarray],
    ) -> Tuple[np.ndarray, Dict[str, np.ndarray]]:
        hidden_linear = x_matrix @ params["w1"] + params["b1"]
        hidden = np.tanh(hidden_linear)
        output = hidden @ params["w2"] + params["b2"]
        return output, {"x": x_matrix, "hidden": hidden}

    def _compute_loss(
        self,
        prediction: np.ndarray,
        targets: np.ndarray,
        params: Dict[str, np.ndarray],
        weight_decay: float,
    ) -> float:
        mse = float(np.mean((prediction - targets) ** 2))
        penalty = 0.5 * weight_decay * (
            float(np.sum(params["w1"] ** 2)) + float(np.sum(params["w2"] ** 2))
        )
        return mse + penalty

    def fit(self, bundle: Dict[str, object], harness: object) -> None:
        self.state["models"] = {}
        train_idx = bundle["splits"]["train"]

        for regime in bundle["regimes"]:
            train_features, train_targets = self._prepare_batch(
                bundle, train_idx, regime
            )

            params = {
                "w1": self.rng.normal(
                    scale=0.16, size=(self.feature_dim, self.hidden_dim)
                ),
                "b1": np.zeros(self.hidden_dim, dtype=np.float64),
                "w2": self.rng.normal(
                    scale=0.16, size=(self.hidden_dim, self.output_dim)
                ),
                "b2": np.zeros(self.output_dim, dtype=np.float64),
            }

            moments = {name: np.zeros_like(value) for name, value in params.items()}
            velocities = {name: np.zeros_like(value) for name, value in params.items()}
            beta1 = 0.9
            beta2 = 0.999
            learning_rate = 0.025
            weight_decay = 0.01

            for step in range(64):
                prediction, cache = self._forward_dense_scorer(train_features, params)

                residual = (prediction - train_targets) * 2.0 / train_features.shape[0]
                grad_w2 = cache["hidden"].T @ residual
                grad_b2 = np.sum(residual, axis=0)
                hidden_grad = (residual @ params["w2"].T) * (1.0 - cache["hidden"] ** 2)
                grad_w1 = cache["x"].T @ hidden_grad
                grad_b1 = np.sum(hidden_grad, axis=0)

                grads = {"w1": grad_w1, "b1": grad_b1, "w2": grad_w2, "b2": grad_b2}

                for name in params:
                    moments[name] = beta1 * moments[name] + (1.0 - beta1) * grads[name]
                    velocities[name] = beta2 * velocities[name] + (1.0 - beta2) * grads[name] ** 2
                    m_hat = moments[name] / (1.0 - beta1 ** (step + 1))
                    v_hat = velocities[name] / (1.0 - beta2 ** (step + 1))
                    params[name] = params[name] - learning_rate * (
                        m_hat / (np.sqrt(v_hat) + 1e-08)
                        + weight_decay * params[name]
                    )

            loss = self._compute_loss(prediction, train_targets, params, weight_decay)
            try:
                harness.check_value(loss, self.name + "_loss")
            except FloatingPointError:
                loss = 100.0
                harness.check_value(loss, "FAIL: NaN/divergence detected")

            self.state["models"][regime] = dict(params)

    def predict(
        self,
        bundle: Dict[str, object],
        regime: str,
        indices: np.ndarray,
    ) -> Dict[str, np.ndarray]:
        params = self.state["models"][regime]
        features = np.asarray(bundle["features"][indices], dtype=np.float64)
        raw, _ = self._forward_dense_scorer(features, params)
        return self._score_candidates(raw)


class AugMaxBaseline(BaseCondition):
    """Feature augmentation baseline with consistency regularization."""

    def __init__(self, seed: int, feature_dim: int) -> None:
        super().__init__("AugMax", seed, feature_dim)
        self.hidden_dim = feature_dim

    def _augment_evidence(self, features: np.ndarray) -> np.ndarray:
        wave = np.sin(
            np.arange(features.shape[1], dtype=np.float64) / 3.0
        ) * 0.06
        return np.clip(features + wave[None, :] * 2.0, -2.0, 2.0)

    def _compute_consistency_loss(
        self, clean: np.ndarray, augmented: np.ndarray
    ) -> float:
        return float(np.mean((clean - augmented) ** 2))

    def fit(self, bundle: Dict[str, object], harness: object) -> None:
        self.state["models"] = {}
        train_idx = bundle["splits"]["train"]

        for regime in bundle["regimes"]:
            train_features, train_targets = self._prepare_batch(
                bundle, train_idx, regime
            )

            params = {
                "w1": self.rng.normal(
                    scale=0.15, size=(self.feature_dim, self.hidden_dim)
                ),
                "b1": np.zeros(self.hidden_dim, dtype=np.float64),
                "w2": self.rng.normal(
                    scale=0.15, size=(self.hidden_dim, self.output_dim)
                ),
                "b2": np.zeros(self.output_dim, dtype=np.float64),
            }
            learning_rate = 0.02

            for _ in range(58):
                augmented_features = self._augment_evidence(train_features)
                clean_hidden = np.tanh(train_features @ params["w1"] + params["b1"])
                aug_hidden = np.tanh(augmented_features @ params["w1"] + params["b1"])
                clean_output = clean_hidden @ params["w2"] + params["b2"]
                aug_output = aug_hidden @ params["w2"] + params["b2"]

                residual_clean = (clean_output - train_targets) * 2.0 / train_features.shape[0]
                residual_aug = (aug_output - train_targets) * 2.0 / train_features.shape[0]
                consistency = self._compute_consistency_loss(clean_output, aug_output)

                grad_w2 = (clean_hidden.T @ residual_clean + aug_hidden.T @ residual_aug) * 0.55
                grad_b2 = np.sum(residual_clean + residual_aug, axis=0) * 0.55
                hidden_grad_clean = (residual_clean @ params["w2"].T) * (1.0 - clean_hidden ** 2)
                hidden_grad_aug = (residual_aug @ params["w2"].T) * (1.0 - aug_hidden ** 2)
                grad_w1 = (
                    train_features.T @ hidden_grad_clean
                    + augmented_features.T @ hidden_grad_aug
                ) * 0.55 + 0.01 * consistency
                grad_b1 = np.sum(hidden_grad_clean + hidden_grad_aug, axis=0) * 0.55

                params["w1"] = params["w1"] - learning_rate * grad_w1
                params["b1"] = params["b1"] - learning_rate * grad_b1
                params["w2"] = params["w2"] - learning_rate * grad_w2
                params["b2"] = params["b2"] - learning_rate * grad_b2

            loss = float(np.mean((clean_output - train_targets) ** 2))
            try:
                harness.check_value(loss, self.name + "_loss")
            except FloatingPointError:
                loss = 100.0
                harness.check_value(loss, "FAIL: NaN/divergence detected")

            self.state["models"][regime] = dict(params)

    def predict(
        self,
        bundle: Dict[str, object],
        regime: str,
        indices: np.ndarray,
    ) -> Dict[str, np.ndarray]:
        params = self.state["models"][regime]
        features = np.asarray(bundle["features"][indices], dtype=np.float64)
        hidden = np.tanh(features @ params["w1"] + params["b1"])
        raw = hidden @ params["w2"] + params["b2"]
        return self._score_candidates(raw)


class EstablishedMethod1Baseline(BaseCondition):
    """Rule-based expert system from objective game descriptors."""

    def __init__(self, seed: int, feature_dim: int) -> None:
        super().__init__("established_method_1", seed, feature_dim)

    def _derive_rulebook(self) -> Dict[str, np.ndarray]:
        return {
            "algorithm": np.array(
                (0.7, 0.7, 0.6, 0.5, 0.3, 0.4, 0.6, 0.4, 0.8), dtype=np.float64
            ),
            "metric": np.array(
                (0.8, 0.7, 0.5, 0.7, 0.7, 0.4), dtype=np.float64
            ),
            "abstraction": np.array(
                (0.4, 0.6, 0.7, 0.7, 0.5, 0.5, 0.4), dtype=np.float64
            ),
            "risk": np.array(
                (0.6, 0.5, 0.6, 0.6, 0.4, 0.5), dtype=np.float64
            ),
        }

    def _apply_complexity_rules(
        self,
        features: np.ndarray,
        rulebook: Dict[str, np.ndarray],
    ) -> np.ndarray:
        outputs = np.zeros((features.shape[0], self.output_dim), dtype=np.float64)
        alg_end = len(ALGORITHM_FAMILIES)
        metric_end = alg_end + len(METRIC_FAMILIES)
        abstraction_end = metric_end + len(ABSTRACTION_FAMILIES)
        risk_end = abstraction_end + len(RISK_FAMILIES)

        # Extract game descriptor columns from features.
        # Domain one-hot occupies the first columns, numeric features follow.
        # The numeric features after one-hot: [circle==domain, zero_sum, partnership,
        #   stochasticity, hidden_info, perfect_recall, infoset/60, branching/40,
        #   action_granularity, compute_pressure, abstraction_gain, citation/8]
        # Indices below are relative to the full feature vector.
        num_domain = len(set(r["domain"] for r in RAW_GAMES))
        zero_sum = features[:, num_domain + 1:num_domain + 2]
        partnership = features[:, num_domain + 2:num_domain + 3]
        hidden = features[:, num_domain + 4:num_domain + 5]
        infoset = features[:, num_domain + 6:num_domain + 7]
        compute = features[:, num_domain + 9:num_domain + 10]
        abstraction = features[:, num_domain + 10:num_domain + 11]

        # Rule: weight algorithms by game complexity signals
        alg_mask_complex = np.array(
            (1, 1, 1, 0, 0, 0, 0, 0, 1), dtype=np.float64
        )
        alg_mask_search = np.array(
            (0, 1, 1, 0, 1, 0, 0, 0, 1), dtype=np.float64
        )
        alg_mask_rl = np.array(
            (0, 0, 0, 1, 0, 0, 1, 0, 1), dtype=np.float64
        )
        outputs[:, :alg_end] = (
            rulebook["algorithm"][None, :alg_end]
            + 0.7 * infoset * alg_mask_complex[None, :alg_end]
            + 0.6 * compute * alg_mask_search[None, :alg_end]
            + 0.5 * hidden * alg_mask_rl[None, :alg_end]
        )

        met_mask_exact = np.array((1, 1, 1, 0, 0, 0), dtype=np.float64)
        met_mask_dup = np.array((0, 0, 0, 0, 1, 0), dtype=np.float64)
        outputs[:, alg_end:metric_end] = (
            rulebook["metric"][None, :]
            + 0.8 * zero_sum * met_mask_exact[None, :]
            + 0.5 * partnership * met_mask_dup[None, :]
        )

        abs_mask_eq = np.array((0, 1, 1, 1, 1, 1, 1), dtype=np.float64)
        outputs[:, metric_end:abstraction_end] = (
            rulebook["abstraction"][None, :]
            + 0.1 * abstraction * abs_mask_eq[None, :]
        )

        risk_mask_conv = np.array((0, 1, 0, 1, 0, 0), dtype=np.float64)
        outputs[:, abstraction_end:risk_end] = (
            rulebook["risk"][None, :]
            + 0.05 * compute * risk_mask_conv[None, :]
        )

        outputs[:, risk_end] = np.clip(
            0.95 * float(np.mean(compute)) + 0.08 * features[:, 0],
            0.0,
            0.78,
        )
        outputs[:, risk_end + 1] = np.clip(
            0.95 * float(np.mean(compute)) + 0.08 * features[:, 0],
            0.0,
            0.78,
        )

        return _logit(np.clip(outputs, 0.05, 0.95))

    def fit(self, bundle: Dict[str, object], harness: object) -> None:
        self.state["rulebook"] = self._derive_rulebook()
        train_idx = bundle["splits"]["train"]

        for regime in bundle["regimes"]:
            features = np.asarray(bundle["features"][train_idx], dtype=np.float64)
            targets = self._pack_targets(bundle["targets"], regime, train_idx)
            prediction = self._apply_complexity_rules(features, self.state["rulebook"])
            loss = float(np.mean((prediction - targets) ** 2))
            try:
                harness.check_value(loss, self.name + "_loss")
            except FloatingPointError:
                loss = 100.0
                harness.check_value(loss, "FAIL: NaN/divergence detected")

    def predict(
        self,
        bundle: Dict[str, object],
        regime: str,
        indices: np.ndarray,
    ) -> Dict[str, np.ndarray]:
        features = np.asarray(bundle["features"][indices], dtype=np.float64)
        raw = self._apply_complexity_rules(features, self.state["rulebook"])
        if regime == "strict_verification":
            alg_end = len(ALGORITHM_FAMILIES)
            metric_end = alg_end + len(METRIC_FAMILIES)
            raw[:, :alg_end] += 0.2
            raw[:, alg_end:metric_end] += 0.2
        return self._score_candidates(raw)


class EstablishedMethod2Baseline(BaseCondition):
    """Retrieval-and-ranking baseline over public evidence text."""

    def __init__(self, seed: int, feature_dim: int) -> None:
        super().__init__("established_method_2", seed, feature_dim)

    def _index_documents(
        self,
        texts: List[str],
        train_idx: np.ndarray,
    ) -> Dict[str, object]:
        vocab: Dict[str, int] = {}
        for idx in train_idx:
            for token in texts[idx].split():
                vocab.setdefault(token, len(vocab))

        matrix = np.zeros((len(train_idx), len(vocab)), dtype=np.float64)
        for row_idx, source_idx in enumerate(train_idx):
            counts: Dict[int, float] = {}
            for token in texts[source_idx].split():
                token_id = vocab.get(token, -1)
                if token_id >= 0:
                    counts[token_id] = counts.get(token_id, 0.0) + 1.0
            for token_id, count in counts.items():
                matrix[row_idx, token_id] = count

        document_frequency = np.sum(matrix > 0.0, axis=0) + 1.0
        idf = np.log(float(len(train_idx)) / document_frequency + 1.0)
        matrix = matrix * idf[None, :]
        norms = np.linalg.norm(matrix, axis=1, keepdims=True) + EPS
        matrix = matrix / norms

        return {
            "vocab": vocab,
            "matrix": matrix,
            "idf": idf,
            "train_idx": train_idx,
        }

    def _retrieve_supporting_evidence(
        self,
        index: Dict[str, object],
        texts: List[str],
        query_indices: np.ndarray,
    ) -> np.ndarray:
        vocab = index["vocab"]
        query_matrix = np.zeros((len(query_indices), len(vocab)), dtype=np.float64)
        for row_idx, query_idx in enumerate(query_indices):
            counts: Dict[int, float] = {}
            for token in texts[query_idx].split():
                token_id = vocab.get(token, -1)
                if token_id >= 0:
                    counts[token_id] = counts.get(token_id, 0.0) + 1.0
            for token_id, count in counts.items():
                query_matrix[row_idx, token_id] = count

        query_matrix = query_matrix * index["idf"][None, :]
        norms = np.linalg.norm(query_matrix, axis=1, keepdims=True) + EPS
        query_matrix = query_matrix / norms
        return query_matrix @ index["matrix"].T

    def fit(self, bundle: Dict[str, object], harness: object) -> None:
        train_idx = bundle["splits"]["train"]
        texts = list(bundle["texts"])
        self.state["index"] = self._index_documents(texts, train_idx)
        self.state["targets"] = {}

        for regime in bundle["regimes"]:
            _, targets = self._prepare_batch(bundle, train_idx, regime)
            similarity = self._retrieve_supporting_evidence(
                self.state["index"], texts, train_idx
            )
            loss = float(np.mean((similarity.T @ targets - targets) ** 2))
            self.state["targets"][regime] = targets
            try:
                harness.check_value(loss, self.name + "_loss")
            except FloatingPointError:
                loss = 100.0
                harness.check_value(loss, "FAIL: NaN/divergence detected")

    def predict(
        self,
        bundle: Dict[str, object],
        regime: str,
        indices: np.ndarray,
    ) -> Dict[str, np.ndarray]:
        similarities = self._retrieve_supporting_evidence(
            self.state["index"], list(bundle["texts"]), indices
        )
        train_targets = self.state["targets"][regime]

        raw = np.zeros((len(indices), self.output_dim), dtype=np.float64)
        for row_idx in range(len(indices)):
            weights = similarities[row_idx]
            topk = np.argsort(weights)[::-1][:4]
            positive = np.maximum(weights[topk], 0.0)
            positive = positive / (float(np.sum(positive)) + EPS)
            raw[row_idx] = np.mean(
                train_targets[topk] * positive[:, None], axis=0
            )
        return self._score_candidates(raw)


class WithoutKeyComponentAblation(BaseCondition):
    """Ablation that removes public evidence text and benchmark conditioning."""

    def __init__(self, seed: int, feature_dim: int) -> None:
        super().__init__("without_key_component", seed, feature_dim)

    def _build_evidence_graph(self, features: np.ndarray) -> np.ndarray:
        distance = np.linalg.norm(
            features[:, None, :] - features[None, :, :], axis=-1
        )
        affinity = np.exp(-2.4 * distance)
        np.fill_diagonal(affinity, 0.0)
        denom = np.sum(affinity, axis=1, keepdims=True) + EPS
        return affinity / denom

    def fit(self, bundle: Dict[str, object], harness: object) -> None:
        reduced = np.asarray(bundle["game_features"], dtype=np.float64)
        train_idx = bundle["splits"]["train"]
        graph = self._build_evidence_graph(reduced)

        self.state["regime_models"] = {}
        for regime in bundle["regimes"]:
            y_train = self._pack_targets(bundle["targets"], regime, train_idx)
            anchors = np.zeros((reduced.shape[0], self.output_dim), dtype=np.float64)
            anchors[train_idx] = y_train
            propagated = anchors.copy()
            for _ in range(26):
                propagated = 0.86 * propagated + 0.14 * (graph @ propagated)

            loss = float(np.mean((propagated[train_idx] - y_train) ** 2))
            self.state["regime_models"][regime] = {
                "propagated": propagated,
                "train_mean": np.mean(y_train, axis=0, keepdims=True),
            }
            try:
                harness.check_value(loss, self.name + "_loss")
            except FloatingPointError:
                loss = 100.0
                harness.check_value(loss, "FAIL: NaN/divergence detected")

    def predict(
        self,
        bundle: Dict[str, object],
        regime: str,
        indices: np.ndarray,
    ) -> Dict[str, np.ndarray]:
        state = self.state["regime_models"][regime]
        raw = 0.66 * state["propagated"][indices] + 0.34 * state["train_mean"]
        return self._score_candidates(raw)


class SimplifiedVersionAblation(BaseCondition):
    """Reduced-capacity linear scorer over only the first six game features."""

    def __init__(self, seed: int, feature_dim: int) -> None:
        super().__init__("simplified_version", seed, feature_dim)
        self.model_input_dim = feature_dim

    def _build_linear_feature_stack(
        self,
        game_features: np.ndarray,
    ) -> np.ndarray:
        subset = game_features[:, :6]
        interactions = game_features[:, :3] * game_features[:, 3:6]
        return np.concatenate([subset, interactions], axis=1)

    def fit(self, bundle: Dict[str, object], harness: object) -> None:
        train_idx = bundle["splits"]["train"]

        self.state["models"] = {}
        for regime in bundle["regimes"]:
            features = self._build_linear_feature_stack(
                np.asarray(bundle["game_features"][train_idx], dtype=np.float64)
            )
            targets = self._pack_targets(bundle["targets"], regime, train_idx)
            weights = self._ridge_solution(features, targets, ridge=1.25)
            loss = float(np.mean((features @ weights - targets) ** 2))

            self.state["models"][regime] = {
                "weights": weights,
                "train_mean": np.mean(targets, axis=0, keepdims=True),
            }
            try:
                harness.check_value(loss, self.name + "_loss")
            except FloatingPointError:
                loss = 100.0
                harness.check_value(loss, "FAIL: NaN/divergence detected")

    def predict(
        self,
        bundle: Dict[str, object],
        regime: str,
        indices: np.ndarray,
    ) -> Dict[str, np.ndarray]:
        features = self._build_linear_feature_stack(
            np.asarray(bundle["game_features"][indices], dtype=np.float64)
        )
        state = self.state["models"][regime]
        raw = 0.58 * (features @ state["weights"]) + 0.42 * state["train_mean"]
        return self._score_candidates(raw)
