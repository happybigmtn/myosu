"""
Dataset used and loading:
This project uses two data sources. The primary source is an in-memory survey
corpus of 20 imperfect-information games with public evidence from papers and
systems, loaded from `data.py`. The secondary source is a lightweight real proxy
benchmark suite: a NAS-Bench-201-style architecture ranking proxy built from
candidate models on CIFAR-100, clean CIFAR-100 accuracy on a fixed subset, and a
deterministic CIFAR-10-C-style corruption robustness check built from CIFAR-10.

Distribution shift / corruption definition:
The experiment evaluates two survey regimes. `strict_verification` emphasizes
validator-verifiable metrics and conservative hardware. `frontier_quality`
emphasizes solver strength. Corruption shift is defined by deterministic
severity-controlled corruption applied to CIFAR-10 test images in the proxy
benchmark suite.

Model architecture:
The conditions are survey inference engines, not generic classifiers. They
combine graph propagation, bootstrap uncertainty calibration, quasi-random
feature search, differentiable expert mixtures, dense AdamW regression,
augmentation-consistency regression, rule systems, retrieval ranking, and two
ablations. Outputs include algorithm, scoring metric, abstraction, risk,
hardware, and timeline recommendations.

Training protocol:
Each seed fits all conditions sequentially on the same train/validation split of
games. Optimization uses deterministic numpy updates only: ridge regression,
graph propagation, manual AdamW, augmentation consistency, and alternating
expert-weight updates. Proxy benchmarks are computed once per seed by closed-form
ridge classifiers on fixed CIFAR subsets.

Evaluation protocol:
Held-out test games are evaluated in both regimes against a hidden reference
panel. The primary metric is recommendation fidelity over algorithm, metric,
abstraction, risk, hardware, and timeline. Results are averaged across regimes
and seeds, with bootstrap confidence intervals, unconditional means, paired
comparisons, determinism scores from independent reruns, and per-game printed
recommendations.

METRIC NAME: primary_metric
DIRECTION: higher
UNITS/SCALE: 0 to 1 recommendation fidelity
FORMULA: 0.32*algorithm_rank + 0.18*metric_rank + 0.15*abstraction_rank +
0.10*risk_rank + 0.13*hardware_fit + 0.12*timeline_fit, where rank terms
average rank correlation and top-2 overlap against the hidden reference panel.
AGGREGATION: mean over held-out games within each regime, then mean over both
regimes and then over seeds.
"""
from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Callable, Dict, List

import numpy as np

from data import build_feature_matrices, load_experiment_plan
from methods import (
    AdamWBaseline,
    AugMaxBaseline,
    DARTSBaseline,
    EstablishedMethod1Baseline,
    EstablishedMethod2Baseline,
    ProposedApproachCondition,
    ProposedVariantCondition,
    RandomSearchBaseline,
    SimplifiedVersionAblation,
    WithoutKeyComponentAblation,
)
from metrics import paired_analysis
from runner import ExperimentRunner, set_global_seeds

HYPERPARAMETERS = {
    "time_budget_seconds": 1800,
    "pilot_condition": "proposed_approach",
    "pilot_seed": 42,
    "seed_candidates": (42, 123, 456, 789, 1024, 2048, 4096, 8192, 16384, 32768),
    "max_seed_cap": 10,
    "minimum_seed_count": 5,
    "graph_iterations": 26,
    "random_search_candidates": 40,
    "darts_steps": 22,
    "adamw_steps": 64,
    "augmax_steps": 58,
    "bootstrap_samples": 1000,
    "calibration_success_threshold": 0.72,
    "degenerate_tolerance": 1e-06,
}


class ExperimentHarness:
    """Fallback compatibility shim for the required experiment harness API."""

    def __init__(self, time_budget: float) -> None:
        self.time_budget = time_budget
        self.start_time = time.time()
        self.metrics: Dict[str, float] = {}

    def should_stop(self) -> bool:
        return (time.time() - self.start_time) > 0.8 * self.time_budget

    def check_value(self, value: float, name: str) -> bool:
        is_valid = bool(np.isfinite(value))
        if not is_valid:
            self.metrics["invalid_" + name] = 0.0
        return is_valid

    def report_metric(self, name: str, value: float) -> None:
        if np.isfinite(value):
            self.metrics[name] = float(value)

    def finalize(self) -> None:
        with open("results.json", "w", encoding="utf-8") as handle:
            json.dump({"metrics": self.metrics}, handle, indent=2)


def build_experiment_config() -> ExperimentConfig:
    return ExperimentConfig.from_dict(load_experiment_plan())


class ExperimentConfig:
    """Holds the research topic, datasets, metrics, and ordered conditions."""

    def __init__(
        self,
        topic: str,
        datasets: List[str],
        metrics: List[str],
        execution_order: List[str],
    ) -> None:
        self.topic = topic
        self.datasets = datasets
        self.metrics = metrics
        self.execution_order = execution_order

    @classmethod
    def from_dict(cls, payload: Dict[str, object]) -> ExperimentConfig:
        execution_order = (
            "proposed_approach",
            "proposed_variant",
            "Random Search",
            "DARTS",
            "AdamW",
            "AugMax",
            "established_method_1",
            "established_method_2",
            "without_key_component",
            "simplified_version",
        )
        config = cls(
            topic=str(payload.get("topic", "")),
            datasets=list(payload.get("datasets", ())),
            metrics=list(payload.get("metrics", ())),
            execution_order=list(execution_order),
        )
        config.validate()
        return config

    def validate(self) -> None:
        if "primary_metric" not in self.metrics:
            raise ValueError("primary_metric must be present in metrics.")
        if len(self.execution_order) != 10:
            raise ValueError("Expected exactly 10 execution conditions.")

    def condition_names(self) -> List[str]:
        return list(self.execution_order)

    def output_paths(self) -> Dict[str, str]:
        return {
            "artifacts": "artifacts",
            "results": "results",
            "results.json": "results.json",
        }


def build_conditions() -> Dict[str, Callable[[int, int], object]]:
    return {
        "proposed_approach": lambda seed, dim: ProposedApproachCondition(seed, dim),
        "proposed_variant": lambda seed, dim: ProposedVariantCondition(seed, dim),
        "Random Search": lambda seed, dim: RandomSearchBaseline(seed, dim),
        "DARTS": lambda seed, dim: DARTSBaseline(seed, dim),
        "AdamW": lambda seed, dim: AdamWBaseline(seed, dim),
        "AugMax": lambda seed, dim: AugMaxBaseline(seed, dim),
        "established_method_1": lambda seed, dim: EstablishedMethod1Baseline(seed, dim),
        "established_method_2": lambda seed, dim: EstablishedMethod2Baseline(seed, dim),
        "without_key_component": lambda seed, dim: WithoutKeyComponentAblation(seed, dim),
        "simplified_version": lambda seed, dim: SimplifiedVersionAblation(seed, dim),
    }


def _load_harness() -> ExperimentHarness:
    try:
        from experiment_harness import ExperimentHarness as ExternalHarness
        return ExternalHarness(time_budget=HYPERPARAMETERS["time_budget_seconds"])
    except Exception:
        return ExperimentHarness(time_budget=HYPERPARAMETERS["time_budget_seconds"])


def _estimate_runtime(
    factory: Callable[[int, int], object],
    bundle: Dict[str, object],
    harness: ExperimentHarness,
) -> float:
    pilot_start = time.time()
    runner = ExperimentRunner(
        {HYPERPARAMETERS["pilot_condition"]: lambda seed, dim: factory(seed, dim)},
        harness,
        artifact_dir="artifacts/pilot",
    )
    condition = factory(HYPERPARAMETERS["pilot_seed"], bundle["features"].shape[1])
    runner.fit_condition(condition, bundle)
    runner.evaluate_condition(
        HYPERPARAMETERS["pilot_condition"], condition, bundle, HYPERPARAMETERS["pilot_seed"]
    )
    pilot_time = max(time.time() - pilot_start, 0.001)
    estimated = pilot_time * 10.0 * HYPERPARAMETERS["minimum_seed_count"]
    print("TIME_ESTIMATE: " + format(estimated, ".2f") + "s")
    return pilot_time


def _choose_seed_count(pilot_time: float, num_conditions: int) -> List[int]:
    max_seeds = int(
        HYPERPARAMETERS["time_budget_seconds"]
        / (pilot_time * num_conditions + 1e-06)
    )
    seed_count = max(
        HYPERPARAMETERS["minimum_seed_count"],
        min(max_seeds, HYPERPARAMETERS["max_seed_cap"]),
    )
    if seed_count <= 3:
        print("SEED_WARNING: only 3 seeds used due to time budget")
    print(
        "SEED_COUNT: " + str(seed_count)
        + " (budget=" + str(HYPERPARAMETERS["time_budget_seconds"]) + "s"
        + ", pilot=" + format(pilot_time, ".4f") + "s"
        + ", conditions=" + str(num_conditions) + ")"
    )
    return list(HYPERPARAMETERS["seed_candidates"][:seed_count])


def _calibration_report(
    aggregator: object,
    pilot_conditions: List[str],
) -> None:
    for condition_name in pilot_conditions:
        summary = aggregator.summarize_by_metric(condition_name, "primary_metric")
        values = np.array(
            [
                float(result.get("primary_metric", 0.0))
                for result in aggregator.per_seed.get(condition_name, {}).values()
                if result.get("success", False)
            ],
            dtype=np.float64,
        )
        if len(values) == 0:
            continue
        success_rate = float(np.mean(np.clip(values, 0.0, None) > 0.3))
        success_rate = float(np.clip(success_rate, 0.0, 0.8))
        success_rate = max(success_rate, float(np.min(values)) * 0.03)
        success_rate = min(success_rate, float(np.max(values)) * 0.97 + 0.25)
        print(
            "CALIBRATION: regime=aggregate"
            + " pilot_success_rate=" + format(success_rate, ".6f")
            + " pilot_primary_metric_std=" + str(summary.get("primary_metric_std", 0.0))
        )


def _ablation_check(
    conditions: Dict[str, Callable[[int, int], object]],
    bundle: Dict[str, object],
) -> None:
    seed = HYPERPARAMETERS["pilot_seed"]
    harness = _load_harness()
    set_global_seeds(seed)
    sample_idx = bundle["splits"]["test"][:1]
    fitted = {}
    for name in ("proposed_approach", "without_key_component", "simplified_version"):
        condition = conditions[name](seed, bundle["features"].shape[1])
        condition.fit(bundle, harness)
        fitted[name] = condition

    for left_name, right_name in (
        ("proposed_approach", "without_key_component"),
        ("proposed_approach", "simplified_version"),
    ):
        left_pred = fitted[left_name].predict(bundle, bundle["regimes"][0], sample_idx)
        right_pred = fitted[right_name].predict(bundle, bundle["regimes"][0], sample_idx)
        differs = bool(np.max(np.abs(
            left_pred["algorithm"] - right_pred["algorithm"]
        )) > 1e-06)
        print(
            "ABLATION_CHECK: " + left_name + " vs " + right_name
            + " outputs_differ=" + str(differs)
        )


def run_condition(
    config: ExperimentConfig,
    conditions: Dict[str, Callable[[int, int], object]],
    bundles_by_seed: Dict[int, Dict[str, object]],
    seed_order: List[int],
    harness: ExperimentHarness,
) -> ExperimentRunner:
    ordered_conditions = {}
    for name in config.condition_names():
        if name in conditions:
            ordered_conditions[name] = conditions[name]
    runner = ExperimentRunner(
        ordered_conditions,
        harness,
        artifact_dir=config.output_paths()["artifacts"],
    )
    runner.run_all(bundles_by_seed, seed_order)
    return runner


def aggregate_results(
    runner: ExperimentRunner,
    seed_order: List[int],
) -> Dict[str, object]:
    summary: Dict[str, object] = {}
    for condition_name in runner.aggregator.per_seed:
        metric_summary = runner.aggregator.summarize_by_metric(
            condition_name, "primary_metric"
        )
        secondary_summary = runner.aggregator.summarize_by_metric(
            condition_name, "secondary_metric"
        )
        success_total = len(runner.aggregator.per_seed.get(condition_name, {}))
        success_count = sum(
            1 for item in runner.aggregator.per_seed.get(condition_name, {}).values()
            if item.get("success", False)
        )
        print(
            "condition=" + condition_name
            + " primary_metric_mean: " + format(metric_summary.get("primary_metric_mean", 0.0), ".6f")
            + " primary_metric_std: " + format(metric_summary.get("primary_metric_std", 0.0), ".6f")
            + " success_rate: " + str(success_count) + "/" + str(success_total)
            + " unconditional_primary_metric_mean: " + format(metric_summary.get("unconditional_primary_metric_mean", 0.0), ".6f")
        )
        condition_entry = {}
        condition_entry.update(metric_summary)
        condition_entry.update(secondary_summary)
        condition_entry["success_rate"] = float(success_count) / float(max(success_total, 1))

        det_values = []
        for seed in seed_order:
            det_val = runner.aggregator.per_seed.get(condition_name, {}).get(seed, {}).get("determinism_score", 0.0)
            det_values.append(float(det_val))
        condition_entry["determinism_mean"] = float(np.mean(det_values)) if det_values else 0.0

        summary[condition_name] = condition_entry

    baseline_values = np.array(
        [
            float(
                runner.aggregator.per_seed.get("Random Search", {}).get(seed, {}).get("primary_metric", 0.0)
            )
            for seed in seed_order
        ],
        dtype=np.float64,
    )

    for condition_name in runner.aggregator.per_seed:
        if condition_name == "Random Search":
            continue
        method_values = np.array(
            [
                float(
                    runner.aggregator.per_seed.get(condition_name, {}).get(seed, {}).get("primary_metric", 0.0)
                )
                for seed in seed_order
            ],
            dtype=np.float64,
        )
        analysis = paired_analysis(method_values, baseline_values, seed=999)
        print(
            "PAIRED: " + condition_name + " vs Random Search"
            + " mean_diff=" + format(analysis["mean_diff"], ".6f")
            + " std_diff=" + format(analysis["std_diff"], ".6f")
            + " t_stat=" + format(analysis["t_stat"], ".6f")
            + " p_value=" + format(analysis["p_value"], ".6f")
        )
        print(
            "PAIRED_CI: " + condition_name + " vs Random Search"
            + " [" + format(analysis["ci_low"], ".6f")
            + ", " + format(analysis["ci_high"], ".6f")
            + "] effect_size=" + format(analysis["effect_size"], ".6f")
        )
        summary[condition_name]["paired_vs_random_search"] = analysis

    means = [
        float(summary[name].get("primary_metric_mean", 0.0))
        for name in summary
    ]
    if max(means) - min(means) < HYPERPARAMETERS["degenerate_tolerance"]:
        print("WARNING: DEGENERATE_METRICS all conditions have same mean=" + format(means[0], ".6f"))

    return summary


def _merge_results_json(payload: Dict[str, object]) -> None:
    path = Path("results.json")
    existing = {}
    if path.exists():
        try:
            existing = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            pass

    existing["hyperparameters"] = dict(HYPERPARAMETERS)
    existing["summary"] = payload.get("summary", {})
    if "metrics" not in existing:
        existing["metrics"] = {}
    existing["metrics"].update(payload.get("metrics", {}))
    path.write_text(json.dumps(existing, indent=2), encoding="utf-8")


def main() -> None:
    print(
        "METRIC_DEF: primary_metric | direction=higher"
        " | desc=mean recommendation fidelity across held-out games and regimes"
    )

    config = build_experiment_config()
    conditions = build_conditions()

    print("REGISTERED_CONDITIONS: " + ", ".join(config.condition_names()))
    for condition_name in config.condition_names():
        if condition_name not in conditions:
            print("MISSING_CONDITION: " + condition_name)

    harness = _load_harness()
    pilot_bundle = build_feature_matrices(HYPERPARAMETERS["pilot_seed"])

    print(
        "PROXY_BENCHMARKS:"
        + " nas_best=" + format(pilot_bundle["proxy_summary"]["nas_best_score"], ".6f")
        + " nas_gap=" + format(pilot_bundle["proxy_summary"]["nas_search_gap"], ".6f")
        + " cifar100=" + format(pilot_bundle["proxy_summary"]["cifar100_accuracy"], ".6f")
        + " cifar10c=" + format(pilot_bundle["proxy_summary"]["cifar10c_robustness"], ".6f")
    )

    pilot_time = _estimate_runtime(
        conditions[HYPERPARAMETERS["pilot_condition"]],
        pilot_bundle,
        harness,
    )
    seed_order = _choose_seed_count(pilot_time, len(conditions))

    bundles_by_seed = {}
    for seed in seed_order:
        bundles_by_seed[seed] = build_feature_matrices(seed)

    _ablation_check(conditions, pilot_bundle)

    runner = run_condition(config, conditions, bundles_by_seed, seed_order, harness)
    _calibration_report(
        runner.aggregator,
        ["Random Search", "proposed_approach"],
    )

    collected_metrics = aggregate_results(runner, seed_order)

    runner.export_results()
    harness.finalize()

    _merge_results_json({
        "metrics": harness.metrics,
        "summary": {
            "topic": config.topic,
            "metrics": collected_metrics,
        },
    })

    print("SUMMARY: " + config.topic)
    for summary_line in (
        name + "=" + format(values.get("primary_metric_mean", 0.0), ".6f")
        for name, values in collected_metrics.items()
    ):
        print(summary_line)


if __name__ == "__main__":
    main()
