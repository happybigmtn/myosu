"""Execution harness for deterministic Myosu survey conditions."""
from __future__ import annotations

import random
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable, Dict, List, Tuple

import numpy as np

from data import decode_recommendation
from metrics import MetricSuite, bootstrap_confidence_interval


def set_global_seeds(seed: int) -> None:
    random.seed(seed)
    np.random.seed(seed)


def make_artifact_dir(path: str) -> Path:
    directory = Path(path)
    directory.mkdir(parents=True, exist_ok=True)
    return directory


@dataclass
class ResultAggregator:
    """Collects per-seed and per-regime metrics for all conditions."""

    per_seed: Dict[str, Dict[int, Dict[str, object]]] = field(default_factory=dict)

    def add_result(
        self,
        condition_name: str,
        seed: int,
        result: Dict[str, object],
    ) -> None:
        self.per_seed.setdefault(condition_name, {})[seed] = result

    def summarize_by_metric(
        self,
        condition_name: str,
        metric_name: str,
    ) -> Dict[str, float]:
        condition_results = self.per_seed.get(condition_name, {})
        succeeded = []
        unconditional = []
        for value in condition_results.values():
            metric_val = float(value.get(metric_name, 0.0))
            unconditional.append(metric_val)
            if value.get("success", False):
                succeeded.append(metric_val)

        summary = {
            metric_name + "_mean": float(np.mean(succeeded)) if succeeded else 0.0,
            metric_name + "_std": float(np.std(succeeded)) if succeeded else 0.0,
            "unconditional_" + metric_name + "_mean": float(np.mean(unconditional)) if unconditional else 0.0,
        }
        if succeeded:
            values_arr = np.asarray(succeeded, dtype=np.float64)
            ci_low, ci_high = bootstrap_confidence_interval(values_arr, seed=len(succeeded))
            summary[metric_name + "_ci_low"] = ci_low
            summary[metric_name + "_ci_high"] = ci_high
        return summary

    def rank_conditions(
        self,
        metric_name: str,
        minimize: bool = False,
    ) -> List[Tuple[str, float]]:
        ranking = []
        for condition_name in self.per_seed:
            summary = self.summarize_by_metric(condition_name, metric_name)
            ranking.append((condition_name, summary[metric_name + "_mean"]))
        ranking.sort(key=lambda item: item[1], reverse=not minimize)
        return ranking

    def build_leaderboard(self) -> Dict[str, float]:
        return {
            name: score
            for name, score in self.rank_conditions(
                "primary_metric", minimize=False
            )
        }

    def export_csv(self, output_path: Path) -> None:
        lines = ["condition,seed,success,primary_metric,secondary_metric,determinism_score"]
        for condition_name, seed_results in self.per_seed.items():
            for seed, result in seed_results.items():
                lines.append(
                    ",".join([
                        condition_name,
                        str(int(seed)),
                        str(result.get("success", False)),
                        format(result.get("primary_metric", 0.0), ".6f"),
                        format(result.get("secondary_metric", 0.0), ".6f"),
                        format(result.get("determinism_score", 0.0), ".6f"),
                    ])
                )
        output_path.write_text("\n".join(lines), encoding="utf-8")


class ExperimentRunner:
    """Runs all conditions in breadth-first seed order with shared evaluation."""

    def __init__(
        self,
        conditions: Dict[str, Callable[[int, int], object]],
        harness: object,
        artifact_dir: str = "artifacts",
    ) -> None:
        self.conditions = conditions
        self.harness = harness
        self.metric_suite = MetricSuite()
        self.aggregator = ResultAggregator()
        self.artifact_dir = make_artifact_dir(artifact_dir)

    def fit_condition(
        self,
        condition: object,
        bundle: Dict[str, object],
    ) -> None:
        condition.fit(bundle, self.harness)

    def _build_regime_target(
        self,
        bundle: Dict[str, object],
        regime: str,
        indices: np.ndarray,
    ) -> Dict[str, np.ndarray]:
        return {
            "algorithm": bundle["targets"][regime]["algorithms"][indices],
            "metric": bundle["targets"][regime]["metrics"][indices],
            "abstraction": bundle["targets"][regime]["abstractions"][indices],
            "risk": bundle["targets"][regime]["risk"][indices],
            "hardware": bundle["targets"][regime]["hardware"][indices],
            "timeline": bundle["targets"][regime]["timeline"][indices],
        }

    def _recommendations_for_regime(
        self,
        condition: object,
        bundle: Dict[str, object],
        regime: str,
        indices: np.ndarray,
    ) -> List[Dict[str, object]]:
        prediction = condition.predict(bundle, regime, indices)
        recommendations = []
        for local_idx, record_idx in enumerate(indices):
            recommendations.append(
                decode_recommendation(bundle["records"][int(record_idx)], prediction, local_idx)
            )
        return recommendations

    def _measure_determinism(
        self,
        condition_name: str,
        seed: int,
        bundle: Dict[str, object],
        reference_predictions: Dict[str, Dict[str, np.ndarray]],
    ) -> float:
        rerun = self.conditions[condition_name](seed, bundle["features"].shape[1])
        test_idx = bundle["splits"]["test"]
        self.fit_condition(rerun, bundle)

        scores = []
        for regime in bundle["regimes"]:
            rerun_prediction = rerun.predict(bundle, regime, test_idx)
            for key in ("algorithm", "metric", "abstraction", "risk"):
                repeated = [
                    reference_predictions[regime][key],
                    rerun_prediction[key],
                ]
                scores.append(
                    self.metric_suite.evaluate_global(repeated)["determinism_score"]
                )

        hardware_delta = float(np.mean(np.abs(
            reference_predictions[bundle["regimes"][0]]["hardware"]
            - rerun.predict(bundle, bundle["regimes"][0], test_idx)["hardware"]
        )))
        timeline_delta = float(np.mean(np.abs(
            reference_predictions[bundle["regimes"][0]]["timeline"]
            - rerun.predict(bundle, bundle["regimes"][0], test_idx)["timeline"]
        )))
        return float(np.clip(float(np.mean(scores)) - hardware_delta - timeline_delta, 0.0, 1.0))

    def evaluate_condition(
        self,
        condition_name: str,
        condition: object,
        bundle: Dict[str, object],
        seed: int,
    ) -> Dict[str, object]:
        test_idx = bundle["splits"]["test"]
        regime_reports = []
        regime_primary = []
        regime_secondary = []
        regime_predictions = {}
        recommendations = []

        for regime in bundle["regimes"]:
            prediction = condition.predict(bundle, regime, test_idx)
            target = self._build_regime_target(bundle, regime, test_idx)
            report = self.metric_suite.evaluate_predictions(prediction, target)
            regime_reports.append(report)
            regime_primary.append(report["primary_metric"])
            regime_secondary.append(report["secondary_metric"])
            regime_predictions[regime] = prediction
            recommendations.extend(
                self._recommendations_for_regime(condition, bundle, regime, test_idx)
            )

        determinism_score = self._measure_determinism(
            condition_name, seed, bundle, regime_predictions
        )

        return {
            "primary_metric": float(np.mean(regime_primary)),
            "secondary_metric": float(np.mean(regime_secondary)),
            "determinism_score": determinism_score,
            "regime_reports": regime_reports,
            "recommendations": recommendations,
            "success": True,
        }

    def log_condition_result(
        self,
        condition_name: str,
        seed: int,
        result: Dict[str, object],
        emit_recommendations: bool = False,
    ) -> None:
        print(
            "condition=" + condition_name
            + " seed=" + str(seed)
            + " primary_metric: " + format(result.get("primary_metric", 0.0), ".6f")
            + " secondary_metric: " + format(result.get("secondary_metric", 0.0), ".6f")
            + " determinism_score: " + format(result.get("determinism_score", 0.0), ".6f")
        )

        if emit_recommendations:
            for regime_name, regime_report in enumerate(result.get("regime_reports", [])):
                print(" regime=" + str(regime_name))
            for row in result.get("recommendations", []):
                print(
                    " game=" + str(row.get("game", ""))
                    + " algorithm=" + str(row.get("algorithm", ""))
                    + " metric=" + str(row.get("metric", ""))
                    + " abstraction=" + str(row.get("abstraction", ""))
                    + " hardware=" + str(row.get("hardware", ""))
                    + " timeline_months=" + str(row.get("timeline_months", ""))
                    + " risks=" + ",".join(row.get("risks", []))
                )

        self.harness.report_metric(
            condition_name + "_seed_" + str(seed) + "_primary_metric",
            result.get("primary_metric", 0.0),
        )

    def run_single(
        self,
        condition_name: str,
        condition: object,
        bundle: Dict[str, object],
        seed: int,
        emit_recommendations: bool = False,
    ) -> Dict[str, object]:
        print(
            "condition=" + condition_name
            + " state_dim: " + str(bundle["features"].shape)
            + " input_dim: " + str(condition.model_input_dim)
        )

        dry_features = np.asarray(
            bundle["features"][bundle["splits"]["test"][:1]], dtype=np.float64
        )
        if dry_features.shape[1] < condition.model_input_dim:
            raise ValueError(
                "Dry-run feature dimension " + str(dry_features.shape[1])
                + " is smaller than model input dimension "
                + str(condition.model_input_dim) + "."
            )

        try:
            self.fit_condition(condition, bundle)
            result = self.evaluate_condition(condition_name, condition, bundle, seed)
        except Exception as exc:
            print("CONDITION_FAILED: " + condition_name + " " + str(exc))
            result = {
                "primary_metric": 0.0,
                "secondary_metric": 0.0,
                "determinism_score": 0.0,
                "regime_reports": [],
                "recommendations": [],
                "success": False,
            }

        self.aggregator.add_result(condition_name, seed, result)
        self.log_condition_result(
            condition_name, seed, result, emit_recommendations=emit_recommendations
        )
        return result

    def run_all(
        self,
        bundles_by_seed: Dict[int, Dict[str, object]],
        seed_order: List[int],
    ) -> ResultAggregator:
        first_seed = seed_order[0]
        for condition_name, factory in self.conditions.items():
            condition = factory(first_seed, bundles_by_seed[first_seed]["features"].shape[1])
            if self.harness.should_stop():
                break
            set_global_seeds(first_seed)
            self.run_single(
                condition_name, condition, bundles_by_seed[first_seed],
                first_seed, emit_recommendations=True,
            )

        for seed in seed_order[1:]:
            for condition_name, factory in self.conditions.items():
                condition = factory(seed, bundles_by_seed[seed]["features"].shape[1])
                if self.harness.should_stop():
                    break
                set_global_seeds(seed)
                self.run_single(
                    condition_name, condition, bundles_by_seed[seed],
                    seed, emit_recommendations=False,
                )
        return self.aggregator

    def export_results(self) -> Path:
        csv_path = self.artifact_dir / "seed_results.csv"
        self.aggregator.export_csv(csv_path)
        return csv_path
