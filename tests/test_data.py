from __future__ import annotations

import numpy as np
import pytest

from data import (
    RAW_GAMES,
    REGIME_CONFIG,
    ProxyBenchmarkSuite,
    ResearchCorpusLoader,
    SurveyPreprocessor,
    build_feature_matrices,
    decode_recommendation,
    load_research_corpus,
    split_corpus_deterministically,
)


def test_load_research_corpus_preserves_game_count_and_schema() -> None:
    corpus = load_research_corpus()

    assert len(corpus) == len(RAW_GAMES) == 20
    for record in corpus:
        assert "name" in record
        assert "domain" in record
        assert "paper_algorithms" in record
        assert "system_algorithms" in record
        assert "paper_keywords" in record


def test_research_corpus_loader_returns_copies_of_raw_games() -> None:
    loader = ResearchCorpusLoader()
    public_rows = loader.load_games_table()

    public_rows[0]["name"] = "mutated"

    assert RAW_GAMES[0]["name"] == "NLHE Heads-Up"


def test_research_corpus_loader_validate_schema_rejects_missing_keys() -> None:
    invalid = [
        {
            "name": "broken",
            "domain": "poker",
            "circle": "domain",
            "players": "zero_sum",
            "papers": (),
            "infoset_log10": 1.0,
            "branching_factor": 2.0,
        }
    ]

    with pytest.raises(ValueError, match="missing keys"):
        ResearchCorpusLoader().validate_schema(invalid)


def test_proxy_benchmark_suite_builds_finite_summary_without_datasets(monkeypatch: pytest.MonkeyPatch) -> None:
    suite = ProxyBenchmarkSuite(max_train=6, max_test=4)
    monkeypatch.setattr(suite, "_dataset_candidates", lambda dataset_cls, train: [])

    summary = suite.build_proxy_summary()

    assert set(summary) == {
        "nas_search_gap",
        "nas_best_score",
        "cifar100_accuracy",
        "cifar10c_robustness",
        "latency_proxy",
    }
    assert all(np.isfinite(value) for value in summary.values())
    assert summary["latency_proxy"] > 0.0


def test_split_corpus_deterministically_is_stable_and_disjoint() -> None:
    first = split_corpus_deterministically(seed=17)
    second = split_corpus_deterministically(seed=17)

    for key in ("train", "val", "test"):
        assert np.array_equal(first[key], second[key])

    combined = np.concatenate([first["train"], first["val"], first["test"]])
    assert sorted(combined.tolist()) == list(range(len(RAW_GAMES)))
    assert set(first["train"]).isdisjoint(first["val"])
    assert set(first["train"]).isdisjoint(first["test"])
    assert set(first["val"]).isdisjoint(first["test"])


def test_build_feature_matrices_packages_expected_shapes(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        ProxyBenchmarkSuite,
        "build_proxy_summary",
        lambda self: {
            "nas_search_gap": 0.1,
            "nas_best_score": 0.8,
            "cifar100_accuracy": 0.7,
            "cifar10c_robustness": 1.2,
            "latency_proxy": 0.3,
        },
    )

    bundle = build_feature_matrices(seed=5)

    assert bundle["features"].shape[0] == len(RAW_GAMES)
    assert bundle["game_features"].shape[0] == len(RAW_GAMES)
    assert bundle["benchmark_matrix"].shape == (len(RAW_GAMES), 5)
    assert sorted(bundle["regimes"]) == sorted(REGIME_CONFIG.keys())
    assert sorted(bundle["public_evidence"].keys()) == ["abstraction", "algorithm", "metric", "risk"]


def test_decode_recommendation_uses_argmax_labels() -> None:
    record = RAW_GAMES[0]
    prediction = {
        "algorithm": np.array([[0.05, 0.9, 0.05] + [0.0] * 6], dtype=np.float64),
        "metric": np.array([[0.0, 1.0, 0.0, 0.0, 0.0, 0.0]], dtype=np.float64),
        "abstraction": np.array([[0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]], dtype=np.float64),
        "risk": np.array([[0.1, 0.2, 0.6, 0.05, 0.03, 0.02]], dtype=np.float64),
        "hardware": np.array([0.45], dtype=np.float64),
        "timeline": np.array([0.9], dtype=np.float64),
    }

    decoded = decode_recommendation(record, prediction, row_index=0)

    assert decoded["game"] == "NLHE Heads-Up"
    assert decoded["algorithm"] == "DCFR"
    assert decoded["metric"] == "best_response_oracle"
    assert decoded["abstraction"] == "emd_buckets"
    assert decoded["hardware"] == "cpu_128gb"
    assert decoded["timeline_months"] == 1
    assert decoded["risks"][:2] == ["abstraction_error", "metric_gaming"]


def test_survey_preprocessor_create_splits_covers_all_rows() -> None:
    splits = SurveyPreprocessor(REGIME_CONFIG).create_splits(size=len(RAW_GAMES), seed=23)

    assert sum(len(indices) for indices in splits.values()) == len(RAW_GAMES)
    assert min(len(indices) for indices in splits.values()) > 0
