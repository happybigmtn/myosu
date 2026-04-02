"""Metric utilities for deterministic Myosu survey experiments."""
from __future__ import annotations

import math
from typing import Dict, List, Tuple

import numpy as np


EXACT_SIGN_FLIP_THRESHOLD = 15
MONTE_CARLO_SIGN_FLIP_SAMPLES = 10_000


def _exact_sign_flip_totals(differences: np.ndarray) -> np.ndarray:
    n = len(differences)
    permutation_count = 1 << n
    pattern_ids = np.arange(permutation_count, dtype=np.uint32)[:, None]
    bit_positions = np.arange(n, dtype=np.uint32)
    signs = (((pattern_ids >> bit_positions) & 1).astype(np.int8) * 2) - 1
    return np.abs((signs @ differences) / float(n))


def _monte_carlo_sign_flip_p_value(
    differences: np.ndarray,
    observed: float,
    seed: int,
    samples: int,
) -> float:
    total_samples = max(samples, 1)
    random_samples = max(total_samples - 1, 0)
    exceedances = 1  # Include the identity sign vector so the estimate is never spuriously zero.
    if random_samples > 0:
        rng = np.random.default_rng(seed)
        random_signs = (
            rng.integers(0, 2, size=(random_samples, len(differences)), dtype=np.int8) * 2
        ) - 1
        random_totals = np.abs((random_signs @ differences) / float(len(differences)))
        exceedances += int(np.count_nonzero(random_totals >= observed))
    return float(exceedances) / float(total_samples)


def compute_rank_correlation(predicted: np.ndarray, target: np.ndarray) -> float:
    pred_rank = np.argsort(np.argsort(predicted))
    target_rank = np.argsort(np.argsort(target))
    pred_centered = pred_rank - np.mean(pred_rank)
    target_centered = target_rank - np.mean(target_rank)
    denom = np.linalg.norm(pred_centered) * np.linalg.norm(target_centered)
    if denom == 0.0:
        return 0.0
    return float(np.dot(pred_centered, target_centered) / denom)


def compute_topk_overlap(predicted: np.ndarray, target: np.ndarray, k: int = 2) -> float:
    pred_top = set(np.argsort(predicted)[::-1][:k].tolist())
    target_top = set(np.argsort(target)[::-1][:k].tolist())
    return float(len(pred_top & target_top)) / float(len(pred_top | target_top))


def compute_calibration_error(predicted: np.ndarray, target: np.ndarray) -> float:
    return float(np.mean(np.abs(predicted - target)))


def compute_determinism_score(repeated_predictions: List[np.ndarray]) -> float:
    if len(repeated_predictions) < 2:
        return 1.0
    pairwise = []
    for first_idx in range(len(repeated_predictions)):
        for second_idx in range(first_idx + 1, len(repeated_predictions)):
            delta = float(np.mean(np.abs(
                repeated_predictions[first_idx] - repeated_predictions[second_idx]
            )))
            pairwise.append(delta)
    return float(np.clip(1.0 - float(np.mean(pairwise)), 0.0, 1.0))


def compute_primary_metric(per_game_scores: np.ndarray) -> float:
    return float(np.mean(per_game_scores))


def compute_secondary_metric(rank_scores: np.ndarray) -> float:
    return float(np.mean(rank_scores))


def summarize_game_level_metrics(per_game: Dict[str, np.ndarray]) -> Dict[str, float]:
    summary = {}
    for key, values in per_game.items():
        summary[key + "_mean"] = float(np.mean(values))
        summary[key + "_std"] = float(np.std(values))
    return summary


def bootstrap_confidence_interval(
    values: np.ndarray,
    seed: int = 42,
    samples: int = 1000,
) -> Tuple[float, float]:
    rng = np.random.default_rng(seed)
    draws = np.zeros(samples, dtype=np.float64)
    for idx in range(samples):
        indices = rng.integers(0, len(values), size=len(values))
        draws[idx] = float(np.mean(values[indices]))
    return float(np.quantile(draws, 0.025)), float(np.quantile(draws, 0.975))


def paired_sign_flip_test(
    differences: np.ndarray,
    seed: int = 42,
    exact_threshold: int = EXACT_SIGN_FLIP_THRESHOLD,
    monte_carlo_samples: int = MONTE_CARLO_SIGN_FLIP_SAMPLES,
) -> float:
    differences = np.asarray(differences, dtype=np.float64).reshape(-1)
    n = len(differences)
    if n == 0:
        return 1.0
    observed = float(abs(float(np.mean(differences))))
    if n <= exact_threshold:
        totals = _exact_sign_flip_totals(differences)
        return float(np.mean(totals >= observed))
    return _monte_carlo_sign_flip_p_value(
        differences,
        observed,
        seed=seed,
        samples=monte_carlo_samples,
    )


def compute_effect_size(differences: np.ndarray) -> float:
    std = float(np.std(differences))
    if std < 1e-12:
        return 0.0
    return float(np.mean(differences)) / std


class MetricSuite:
    """Evaluates predictions against deterministic hidden reference recommendations."""

    def __init__(self, regime_weights: Dict[str, float] | None = None):
        if regime_weights is None:
            self.regime_weights = {
                "algorithm_rank": 0.32,
                "metric_rank": 0.18,
                "abstraction_rank": 0.15,
                "risk_rank": 0.10,
                "hardware_fit": 0.13,
                "timeline_fit": 0.12,
            }
        else:
            self.regime_weights = regime_weights

    def check_metric_inputs(
        self,
        predicted: Dict[str, np.ndarray],
        target: Dict[str, np.ndarray],
    ) -> None:
        for key in ("algorithm", "metric", "abstraction", "risk"):
            if predicted[key].shape != target[key].shape:
                raise ValueError(
                    "Shape mismatch for " + key + ": "
                    + str(predicted[key].shape) + " vs " + str(target[key].shape)
                )

    def evaluate_predictions(
        self,
        predicted: Dict[str, np.ndarray],
        target: Dict[str, np.ndarray],
    ) -> Dict[str, object]:
        self.check_metric_inputs(predicted, target)
        num_games = predicted["algorithm"].shape[0]

        algorithm_scores = np.zeros(num_games, dtype=np.float64)
        metric_scores = np.zeros(num_games, dtype=np.float64)
        abstraction_scores = np.zeros(num_games, dtype=np.float64)
        risk_scores = np.zeros(num_games, dtype=np.float64)
        hardware_scores = np.zeros(num_games, dtype=np.float64)
        timeline_scores = np.zeros(num_games, dtype=np.float64)
        per_game_primary = np.zeros(num_games, dtype=np.float64)
        per_game_secondary = np.zeros(num_games, dtype=np.float64)
        per_game_fidelity = np.zeros(num_games, dtype=np.float64)

        for idx in range(num_games):
            algorithm_scores[idx] = (
                0.5 * compute_rank_correlation(predicted["algorithm"][idx], target["algorithm"][idx])
                + 0.5 * compute_topk_overlap(predicted["algorithm"][idx], target["algorithm"][idx], k=2)
            )
            metric_scores[idx] = (
                0.5 * compute_rank_correlation(predicted["metric"][idx], target["metric"][idx])
                + 0.5 * compute_topk_overlap(predicted["metric"][idx], target["metric"][idx], k=2)
            )
            abstraction_scores[idx] = (
                0.5 * compute_rank_correlation(predicted["abstraction"][idx], target["abstraction"][idx])
                + 0.5 * compute_topk_overlap(predicted["abstraction"][idx], target["abstraction"][idx], k=2)
            )
            risk_scores[idx] = (
                0.5 * compute_rank_correlation(predicted["risk"][idx], target["risk"][idx])
                + 0.5 * compute_topk_overlap(predicted["risk"][idx], target["risk"][idx], k=2)
            )
            hardware_scores[idx] = 1.0 - abs(
                float(predicted["hardware"][idx]) - float(target["hardware"][idx])
            )
            timeline_scores[idx] = 1.0 - abs(
                float(predicted["timeline"][idx]) - float(target["timeline"][idx])
            )

            per_game_primary[idx] = (
                self.regime_weights["algorithm_rank"] * algorithm_scores[idx]
                + self.regime_weights["metric_rank"] * metric_scores[idx]
                + self.regime_weights["abstraction_rank"] * abstraction_scores[idx]
                + self.regime_weights["risk_rank"] * risk_scores[idx]
                + self.regime_weights["hardware_fit"] * hardware_scores[idx]
                + self.regime_weights["timeline_fit"] * timeline_scores[idx]
            )
            per_game_secondary[idx] = 1.0 - per_game_primary[idx]

        return {
            "primary_metric": compute_primary_metric(per_game_primary),
            "secondary_metric": compute_secondary_metric(per_game_secondary),
            "recommendation_fidelity": float(np.mean(per_game_primary)),
            "per_game_primary": per_game_primary,
            "per_game_secondary": per_game_secondary,
            "per_game": {
                "algorithm_rank": algorithm_scores,
                "metric_rank": metric_scores,
                "abstraction_rank": abstraction_scores,
                "risk_rank": risk_scores,
                "hardware_fit": hardware_scores,
                "timeline_fit": timeline_scores,
            },
            "calibration_error": 0.25 * (
                compute_calibration_error(predicted["hardware"], target["hardware"])
                + compute_calibration_error(predicted["timeline"], target["timeline"])
            ),
        }

    def evaluate_per_game(
        self,
        predicted: Dict[str, np.ndarray],
        target: Dict[str, np.ndarray],
    ) -> Dict[str, np.ndarray]:
        return self.evaluate_predictions(predicted, target)["per_game"]

    def evaluate_global(
        self,
        repeated_predictions: List[np.ndarray],
    ) -> Dict[str, float]:
        return {"determinism_score": compute_determinism_score(repeated_predictions)}

    def build_metric_report(
        self,
        predicted: Dict[str, np.ndarray],
        target: Dict[str, np.ndarray],
    ) -> Dict[str, float]:
        report = self.evaluate_predictions(predicted, target)
        summary = summarize_game_level_metrics(report["per_game"])
        summary["primary_metric"] = float(report["primary_metric"])
        summary["secondary_metric"] = float(report["secondary_metric"])
        summary["calibration_error"] = float(report["calibration_error"])
        return summary


def paired_analysis(
    method_values: np.ndarray,
    baseline_values: np.ndarray,
    seed: int = 42,
) -> Dict[str, float]:
    differences = np.asarray(method_values, dtype=np.float64) - np.asarray(
        baseline_values, dtype=np.float64
    )
    mean_diff = float(np.mean(differences))
    std_diff = float(np.std(differences))
    stderr = std_diff / math.sqrt(max(len(differences), 1)) + 1e-12
    t_stat = mean_diff / stderr if stderr > 0.0 else 0.0
    p_value = paired_sign_flip_test(differences, seed=seed)
    ci_low, ci_high = bootstrap_confidence_interval(
        differences, seed=seed, samples=1000
    )
    effect_size = compute_effect_size(differences)
    return {
        "mean_diff": mean_diff,
        "std_diff": std_diff,
        "t_stat": t_stat,
        "p_value": p_value,
        "ci_low": ci_low,
        "ci_high": ci_high,
        "effect_size": effect_size,
    }
