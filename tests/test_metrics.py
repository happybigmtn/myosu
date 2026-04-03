from __future__ import annotations

import itertools
import math

import numpy as np
import pytest

from metrics import (
    MetricSuite,
    bootstrap_confidence_interval,
    paired_analysis,
    paired_sign_flip_test,
)


def _exact_sign_flip_p_value(differences: np.ndarray) -> float:
    differences = np.asarray(differences, dtype=np.float64).reshape(-1)
    observed = abs(float(np.mean(differences)))
    totals = []
    for pattern in itertools.product((-1.0, 1.0), repeat=len(differences)):
        signed = np.asarray(pattern, dtype=np.float64) * differences
        totals.append(abs(float(np.mean(signed))))
    return float(np.mean(np.asarray(totals, dtype=np.float64) >= observed))


def _prediction_bundle() -> dict[str, np.ndarray]:
    algorithm = np.array(
        [[0.1, 0.3, 0.6], [0.5, 0.2, 0.3]],
        dtype=np.float64,
    )
    metric = np.array(
        [[0.6, 0.2, 0.2], [0.1, 0.3, 0.6]],
        dtype=np.float64,
    )
    abstraction = np.array(
        [[0.7, 0.2, 0.1], [0.2, 0.6, 0.2]],
        dtype=np.float64,
    )
    risk = np.array(
        [[0.55, 0.35, 0.10], [0.25, 0.5, 0.25]],
        dtype=np.float64,
    )
    hardware = np.array([0.35, 0.8], dtype=np.float64)
    timeline = np.array([0.4, 0.9], dtype=np.float64)
    return {
        "algorithm": algorithm,
        "metric": metric,
        "abstraction": abstraction,
        "risk": risk,
        "hardware": hardware,
        "timeline": timeline,
    }


def test_paired_sign_flip_test_matches_bruteforce_for_small_input() -> None:
    differences = np.array([0.2, -0.4, 0.7, 1.1], dtype=np.float64)

    expected = _exact_sign_flip_p_value(differences)
    actual = paired_sign_flip_test(differences, seed=99)

    assert actual == pytest.approx(expected)


def test_paired_sign_flip_test_is_seeded_for_monte_carlo_branch() -> None:
    differences = np.linspace(-1.5, 1.5, 20, dtype=np.float64)

    first = paired_sign_flip_test(differences, seed=7, monte_carlo_samples=512)
    second = paired_sign_flip_test(differences, seed=7, monte_carlo_samples=512)

    assert first == second
    assert 0.0 <= first <= 1.0


def test_paired_sign_flip_test_handles_empty_and_single_inputs() -> None:
    assert paired_sign_flip_test(np.array([], dtype=np.float64)) == 1.0
    assert paired_sign_flip_test(np.array([2.0], dtype=np.float64)) == 1.0


def test_bootstrap_confidence_interval_handles_empty_input() -> None:
    low, high = bootstrap_confidence_interval(np.array([], dtype=np.float64))

    assert math.isnan(low)
    assert math.isnan(high)


def test_bootstrap_confidence_interval_handles_singleton_input() -> None:
    low, high = bootstrap_confidence_interval(np.array([3.5], dtype=np.float64), seed=3, samples=16)

    assert low == pytest.approx(3.5)
    assert high == pytest.approx(3.5)


def test_metric_suite_returns_perfect_scores_for_identical_predictions() -> None:
    bundle = _prediction_bundle()
    suite = MetricSuite()

    result = suite.evaluate_predictions(bundle, bundle)

    assert result["primary_metric"] == pytest.approx(1.0)
    assert result["secondary_metric"] == pytest.approx(0.0)
    assert result["recommendation_fidelity"] == pytest.approx(1.0)
    assert result["calibration_error"] == pytest.approx(0.0)
    assert np.allclose(result["per_game_primary"], 1.0)
    assert np.allclose(result["per_game_secondary"], 0.0)


def test_metric_suite_rejects_shape_mismatch() -> None:
    bundle = _prediction_bundle()
    broken_target = dict(bundle)
    broken_target["metric"] = np.array([[0.5, 0.5, 0.0]], dtype=np.float64)

    with pytest.raises(ValueError, match="Shape mismatch for metric"):
        MetricSuite().check_metric_inputs(bundle, broken_target)


def test_paired_analysis_returns_consistent_summary_fields() -> None:
    baseline = np.array([0.1, 0.2, 0.4, 0.3], dtype=np.float64)
    method = np.array([0.45, 0.55, 0.95, 0.8], dtype=np.float64)

    result = paired_analysis(method, baseline, seed=11)

    assert result["mean_diff"] > 0.0
    assert result["ci_low"] <= result["mean_diff"] <= result["ci_high"]
    assert 0.0 <= result["p_value"] <= 1.0
    assert result["effect_size"] > 0.0
