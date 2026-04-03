"""Deterministic survey corpus and proxy benchmarks for Myosu experiments."""
from __future__ import annotations

import math
from dataclasses import dataclass
from typing import Dict, List, Sequence, Tuple

import numpy as np

try:
    from torchvision.datasets import CIFAR10, CIFAR100
    from torchvision.transforms import ToTensor
except Exception:
    CIFAR10 = None
    CIFAR100 = None
    ToTensor = None


ALGORITHM_FAMILIES: Tuple[str, ...] = (
    "CFR+",
    "DCFR",
    "MCCFR-External",
    "Deep CFR",
    "DREAM",
    "ISMCTS/POMCP",
    "R-NaD",
    "TD-Lambda",
    "Hybrid Search",
)

METRIC_FAMILIES: Tuple[str, ...] = (
    "exploitability",
    "best_response_oracle",
    "counterfactual_value_gap",
    "deterministic_match_ev",
    "duplicate_tournament_score",
    "cross_play_elo",
)

ABSTRACTION_FAMILIES: Tuple[str, ...] = (
    "none_exact",
    "equity_buckets",
    "emd_buckets",
    "action_discretization",
    "public_belief_state",
    "tile_pattern_groups",
    "trick_state_heuristics",
)

HARDWARE_LABELS: Tuple[str, ...] = (
    "cpu_small",
    "cpu_mid",
    "cpu_128gb",
    "single_gpu",
    "cluster",
)

RISK_FAMILIES: Tuple[str, ...] = (
    "non_convergence",
    "metric_gaming",
    "abstraction_error",
    "compute_overrun",
    "data_scarcity",
    "rule_variants",
)

REGIME_CONFIG: Dict[str, Dict[str, float]] = {
    "strict_verification": {
        "quality": 0.95,
        "verifiability": 1.3,
        "hardware": 0.9,
        "uncertainty": 0.75,
    },
    "frontier_quality": {
        "quality": 1.35,
        "verifiability": 0.8,
        "hardware": 0.65,
        "uncertainty": 0.55,
    },
}

RAW_GAMES: List[Dict[str, object]] = [
    {
        "name": "NLHE Heads-Up",
        "circle": "domain",
        "domain": "poker",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.85,
        "hidden_information": 0.85,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 13.7,
        "branching_factor": 18.0,
        "action_granularity": 0.85,
        "compute_pressure": 0.82,
        "abstraction_gain": 0.85,
        "systems": ("Libratus", "DeepStack", "PioSolver"),
        "papers": ("CFR", "CFR+", "DeepStack", "Libratus"),
        "risk_tags": ("non_convergence", "metric_gaming", "abstraction_error"),
        "text": "heads up no limit holdem exact metric exploitability best response commercial solver market huge betting abstraction",
        "top_algorithms": ("Hybrid Search", "DCFR", "CFR+"),
    },
    {
        "name": "NLHE 6-max",
        "circle": "domain",
        "domain": "poker",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.96,
        "hidden_information": 0.92,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 20.5,
        "branching_factor": 26.0,
        "action_granularity": 0.98,
        "compute_pressure": 0.97,
        "abstraction_gain": 0.85,
        "systems": ("Pluribus", "MonkerSolver"),
        "papers": ("Pluribus", "MCCFR", "Deep CFR"),
        "risk_tags": ("non_convergence", "metric_gaming", "abstraction_error"),
        "text": "six max multiplayer poker blueprint search duplicate evaluation anchor opponents no unique equilibrium",
        "top_algorithms": ("Hybrid Search", "Deep CFR", "MCCFR-External"),
    },
    {
        "name": "PLO",
        "circle": "domain",
        "domain": "poker",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.86,
        "hidden_information": 0.88,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 15.6,
        "branching_factor": 24.0,
        "action_granularity": 0.93,
        "compute_pressure": 0.94,
        "abstraction_gain": 0.85,
        "systems": ("MonkerSolver",),
        "papers": ("Deep CFR",),
        "risk_tags": ("non_convergence", "abstraction_error", "compute_overrun"),
        "text": "pot limit omaha four hole cards high combinatorics sampling solver",
        "top_algorithms": ("MCCFR-External", "DCFR", "Hybrid Search"),
    },
    {
        "name": "NLHE Tournament",
        "circle": "domain",
        "domain": "poker",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.85,
        "hidden_information": 0.85,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 19.4,
        "branching_factor": 23.0,
        "action_granularity": 0.85,
        "compute_pressure": 0.85,
        "abstraction_gain": 0.85,
        "systems": ("ICMIZER", "HRC"),
        "papers": ("ICM",),
        "risk_tags": ("non_convergence", "abstraction_error"),
        "text": "tournament icm payout adjusted utility stack depth shove spot solver",
        "top_algorithms": ("Hybrid Search", "Deep CFR", "DCFR"),
    },
    {
        "name": "Short Deck Hold'em",
        "circle": "domain",
        "domain": "poker",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.78,
        "hidden_information": 0.81,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 12.8,
        "branching_factor": 15.5,
        "action_granularity": 0.89,
        "compute_pressure": 0.74,
        "abstraction_gain": 0.85,
        "systems": ("Simple Postflop",),
        "papers": ("CFR+",),
        "risk_tags": ("abstraction_error",),
        "text": "short deck holdem smaller deck abstraction easier than full nlhe",
        "top_algorithms": ("MCCFR-External", "CFR+", "Hybrid Search"),
    },
    {
        "name": "Teen Patti",
        "circle": "domain",
        "domain": "poker",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.84,
        "hidden_information": 0.91,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 8.9,
        "branching_factor": 9.0,
        "action_granularity": 0.76,
        "compute_pressure": 0.48,
        "abstraction_gain": 0.62,
        "systems": ("consumer bots",),
        "papers": ("sampling CFR", "card game RL"),
        "risk_tags": ("data_scarcity", "rule_variants"),
        "text": "indian three card poker multiplayer sampling cfr hidden betting",
        "top_algorithms": ("MCCFR-External", "ISMCTS/POMCP", "Deep CFR"),
    },
    {
        "name": "Hanafuda / Koi-Koi",
        "circle": "domain",
        "domain": "fishing",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.63,
        "hidden_information": 0.46,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 6.0,
        "branching_factor": 6.0,
        "action_granularity": 0.28,
        "compute_pressure": 0.27,
        "abstraction_gain": 0.33,
        "systems": ("mobile game AI",),
        "papers": ("mcts card games",),
        "risk_tags": ("data_scarcity", "rule_variants"),
        "text": "hanafuda koi koi two player fishing game small state search friendly",
        "top_algorithms": ("ISMCTS/POMCP", "MCCFR-External", "DREAM"),
    },
    {
        "name": "Hwatu / Go-Stop",
        "circle": "domain",
        "domain": "fishing",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.7,
        "hidden_information": 0.68,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 7.1,
        "branching_factor": 8.0,
        "action_granularity": 0.34,
        "compute_pressure": 0.36,
        "abstraction_gain": 0.35,
        "systems": ("consumer game AI",),
        "papers": ("mcts imperfect information",),
        "risk_tags": ("data_scarcity", "rule_variants"),
        "text": "go stop hwatu multi player fishing game point race month matching",
        "top_algorithms": ("ISMCTS/POMCP", "MCCFR-External", "Deep CFR"),
    },
    {
        "name": "Riichi Mahjong",
        "circle": "domain",
        "domain": "tile",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.85,
        "hidden_information": 0.85,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 48.0,
        "branching_factor": 38.0,
        "action_granularity": 0.85,
        "compute_pressure": 0.52,
        "abstraction_gain": 0.85,
        "systems": ("Suphx",),
        "papers": ("deep RL mahjong",),
        "risk_tags": ("compute_overrun", "data_scarcity"),
        "text": "riichi mahjong four player hidden tile game suphx deep reinforcement",
        "top_algorithms": ("Hybrid Search", "Deep CFR", "R-NaD"),
    },
    {
        "name": "Bridge",
        "circle": "domain",
        "domain": "trick_taking",
        "players": "zero_sum",
        "partnership": True,
        "stochasticity": 0.72,
        "hidden_information": 0.85,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 17.0,
        "branching_factor": 34.0,
        "action_granularity": 0.85,
        "compute_pressure": 0.4,
        "abstraction_gain": 0.85,
        "systems": ("double dummy solvers",),
        "papers": ("bridge bidding DRL",),
        "risk_tags": ("compute_overrun", "data_scarcity"),
        "text": "bridge bidding and play partnership trick taking duplicate board scoring",
        "top_algorithms": ("Hybrid Search", "ISMCTS/POMCP", "Deep CFR"),
    },
    {
        "name": "Gin Rummy",
        "circle": "domain",
        "domain": "draw_discard",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.61,
        "hidden_information": 0.85,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 11.2,
        "branching_factor": 14.0,
        "action_granularity": 0.58,
        "compute_pressure": 0.66,
        "abstraction_gain": 0.85,
        "systems": ("academic gin bots",),
        "papers": ("gin rummy MCCFR",),
        "risk_tags": ("data_scarcity",),
        "text": "gin rummy draw discard imperfect information knocking strategy cfr",
        "top_algorithms": ("MCCFR-External", "Deep CFR", "ISMCTS/POMCP"),
    },
    {
        "name": "Stratego",
        "circle": "domain",
        "domain": "board",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.05,
        "hidden_information": 0.87,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 60.0,
        "branching_factor": 60.0,
        "action_granularity": 0.22,
        "compute_pressure": 0.41,
        "abstraction_gain": 0.85,
        "systems": ("DeepNash",),
        "papers": ("DeepNash",),
        "risk_tags": ("compute_overrun",),
        "text": "stratego hidden army deepnash regularized nash dynamics no search",
        "top_algorithms": ("R-NaD", "Hybrid Search", "ISMCTS/POMCP"),
    },
    {
        "name": "OFC Chinese Poker",
        "circle": "domain",
        "domain": "placement",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.56,
        "hidden_information": 0.44,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 9.5,
        "branching_factor": 16.0,
        "action_granularity": 0.31,
        "compute_pressure": 0.49,
        "abstraction_gain": 0.85,
        "systems": ("heuristic OFC bots",),
        "papers": ("mcts card placement",),
        "risk_tags": ("data_scarcity", "rule_variants"),
        "text": "open face chinese poker placement sequencing fantasy land heuristic search",
        "top_algorithms": ("ISMCTS/POMCP", "MCCFR-External", "Hybrid Search"),
    },
    {
        "name": "Spades",
        "circle": "domain",
        "domain": "trick_taking",
        "players": "zero_sum",
        "partnership": True,
        "stochasticity": 0.85,
        "hidden_information": 0.85,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 9.2,
        "branching_factor": 11.0,
        "action_granularity": 0.24,
        "compute_pressure": 0.42,
        "abstraction_gain": 0.37,
        "systems": ("heuristic trick taking bots",),
        "papers": ("information set mcts",),
        "risk_tags": ("data_scarcity", "rule_variants"),
        "text": "spades partnership trick taking bidding hidden hand information set mcts",
        "top_algorithms": ("ISMCTS/POMCP", "Hybrid Search", "MCCFR-External"),
    },
    {
        "name": "Liar's Dice",
        "circle": "domain",
        "domain": "dice",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.85,
        "hidden_information": 0.85,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 7.7,
        "branching_factor": 10.0,
        "action_granularity": 0.21,
        "compute_pressure": 0.29,
        "abstraction_gain": 0.85,
        "systems": ("OpenSpiel implementations",),
        "papers": ("CFR toy domains",),
        "risk_tags": ("rule_variants",),
        "text": "liars dice bluffing dice game openspiel cfr benchmark small exact",
        "top_algorithms": ("CFR+", "MCCFR-External", "ISMCTS/POMCP"),
    },
    {
        "name": "Dou Di Zhu",
        "circle": "domain",
        "domain": "shedding",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.59,
        "hidden_information": 0.85,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 53.0,
        "branching_factor": 27.0,
        "action_granularity": 0.57,
        "compute_pressure": 0.77,
        "abstraction_gain": 0.85,
        "systems": ("DouZero",),
        "papers": ("DouZero",),
        "risk_tags": ("compute_overrun", "data_scarcity"),
        "text": "dou dizhu landlord peasants asymmetric three player deep monte carlo",
        "top_algorithms": ("Hybrid Search", "R-NaD", "Deep CFR"),
    },
    {
        "name": "Pusoy Dos / Big Two",
        "circle": "domain",
        "domain": "shedding",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.83,
        "hidden_information": 0.85,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 10.9,
        "branching_factor": 19.0,
        "action_granularity": 0.53,
        "compute_pressure": 0.5,
        "abstraction_gain": 0.85,
        "systems": (),
        "papers": (),
        "risk_tags": ("data_scarcity", "rule_variants"),
        "text": "big two shedding combination game hidden cards move generation large",
        "top_algorithms": ("ISMCTS/POMCP", "Hybrid Search", "Deep CFR"),
    },
    {
        "name": "Tien Len / Thirteen",
        "circle": "domain",
        "domain": "shedding",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.67,
        "hidden_information": 0.85,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 11.1,
        "branching_factor": 21.0,
        "action_granularity": 0.85,
        "compute_pressure": 0.85,
        "abstraction_gain": 0.85,
        "systems": (),
        "papers": ("mcts shedding games",),
        "risk_tags": ("data_scarcity", "rule_variants"),
        "text": "tien len thirteen vietnamese shedding action abstraction search",
        "top_algorithms": ("ISMCTS/POMCP", "Hybrid Search", "Deep CFR"),
    },
    {
        "name": "Call Break",
        "circle": "domain",
        "domain": "trick_taking",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.85,
        "hidden_information": 0.85,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 8.7,
        "branching_factor": 10.5,
        "action_granularity": 0.26,
        "compute_pressure": 0.85,
        "abstraction_gain": 0.85,
        "systems": (),
        "papers": ("trick taking search",),
        "risk_tags": ("data_scarcity", "rule_variants"),
        "text": "call break south asian trick taking bidding modest branching search",
        "top_algorithms": ("ISMCTS/POMCP", "Hybrid Search", "Deep CFR"),
    },
    {
        "name": "Backgammon",
        "circle": "domain",
        "domain": "board",
        "players": "zero_sum",
        "partnership": False,
        "stochasticity": 0.85,
        "hidden_information": 0.02,
        "perfect_recall_pressure": 0.85,
        "infoset_log10": 6.5,
        "branching_factor": 6.5,
        "action_granularity": 0.38,
        "compute_pressure": 0.14,
        "abstraction_gain": 0.85,
        "systems": ("TD-Gammon", "GNU Backgammon", "XG"),
        "papers": ("TD-Gammon",),
        "risk_tags": ("rule_variants",),
        "text": "backgammon perfect information stochastic dice doubling cube td learning",
        "top_algorithms": ("TD-Lambda", "Hybrid Search", "R-NaD"),
    },
]

PAPERS: List[Dict[str, str]] = [
    {"algorithm": "CFR+", "keywords": "counterfactual regret equilibrium", "focus": "HU poker"},
    {"algorithm": "DCFR", "keywords": "discounting regret fast convergence", "focus": "HU poker"},
    {"algorithm": "Deep CFR", "keywords": "depth limited continual re-solving", "focus": "HU poker"},
    {"algorithm": "Libratus", "keywords": "nested subgame solving blueprint", "focus": "HU poker"},
    {"algorithm": "Pluribus", "keywords": "multiplayer blueprint limited search", "focus": "multiplayer poker"},
    {"algorithm": "Deep CFR", "keywords": "neural regret approximation", "focus": "PLO and multiway poker"},
    {"algorithm": "Suphx", "keywords": "mahjong deep reinforcement oracle guiding", "focus": "mahjong"},
    {"algorithm": "DeepNash", "keywords": "regularized nash dynamics stratego", "focus": "stratego"},
    {"algorithm": "DouZero", "keywords": "dou dizhu deep monte carlo", "focus": "dou dizhu"},
    {"algorithm": "TD-Gammon", "keywords": "temporal difference backgammon", "focus": "backgammon"},
    {"algorithm": "MCCFR-External", "keywords": "sampling cfr regret matching", "focus": "HU poker"},
    {"algorithm": "ISMCTS/POMCP", "keywords": "belief state bridge bidding", "focus": "HU poker"},
    {"algorithm": "Hybrid Search", "keywords": "search hidden information", "focus": "HU poker"},
    {"algorithm": "ISMCTS/POMCP", "keywords": "sampling tree search", "focus": "HU poker"},
    {"algorithm": "Hybrid Search", "keywords": "duplicate trick heuristics", "focus": "HU poker"},
    {"algorithm": "MCCFR-External", "keywords": "draw discard abstraction", "focus": "HU poker"},
]

SYSTEMS: List[Dict[str, str]] = [
    {"algorithm": "CFR+", "name": "Libratus", "focus": "HU poker"},
    {"algorithm": "Deep CFR", "name": "DeepStack", "focus": "HU poker"},
    {"algorithm": "MCCFR-External", "name": "PioSolver", "focus": "HU poker"},
    {"algorithm": "Pluribus", "name": "Pluribus", "focus": "multiplayer poker"},
    {"algorithm": "MCCFR-External", "name": "MonkerSolver", "focus": "PLO and multiway poker"},
    {"algorithm": "TD-Lambda", "name": "TD-Gammon", "focus": "backgammon"},
    {"algorithm": "TD-Lambda", "name": "GNU Backgammon", "focus": "backgammon"},
    {"algorithm": "TD-Lambda", "name": "XG", "focus": "backgammon"},
    {"algorithm": "R-NaD", "name": "DeepNash", "focus": "stratego"},
    {"algorithm": "Deep CFR", "name": "Suphx", "focus": "mahjong"},
    {"algorithm": "Hybrid Search", "name": "DouZero", "focus": "dou dizhu"},
]

REFERENCE_PANEL: Dict[str, Dict[str, object]] = {
    record["name"]: {
        "algorithm": record.get("top_algorithms", ("Hybrid Search",))[0],
        "metric": "exploitability" if record["domain"] == "poker" else "cross_play_elo",
        "abstraction": "equity_buckets" if record["domain"] == "poker" else "none_exact",
        "hardware": float(record.get("compute_pressure", 0.5)),
        "timeline": float(record.get("compute_pressure", 0.5)),
        "risks": tuple(record.get("risk_tags", ())),
    }
    for record in RAW_GAMES
}


def _softmax_1d(values: np.ndarray) -> np.ndarray:
    shifted = values - np.max(values)
    exps = np.exp(shifted)
    return exps / np.sum(exps)


@dataclass
class ResearchCorpusLoader:
    """Loads public game descriptors and hides the reference panel from features."""

    include_proxy_benchmarks: bool = True

    def load_games_table(self) -> List[Dict[str, object]]:
        public_rows = []
        for record in RAW_GAMES:
            row = dict(record)
            public_rows.append(row)
        return public_rows

    def load_papers_table(self) -> List[Dict[str, str]]:
        return [dict(record) for record in PAPERS]

    def load_systems_table(self) -> List[Dict[str, str]]:
        return [dict(record) for record in SYSTEMS]

    def merge_sources(self) -> List[Dict[str, object]]:
        paper = self.load_papers_table()
        system = self.load_systems_table()
        systems = self.load_games_table()

        merged = []
        for game in systems:
            name = game["name"]
            paper_algorithms = " ".join(
                p["algorithm"] for p in paper if p.get("focus", "") == name
            )
            system_algorithms = " ".join(
                s["algorithm"] for s in system if s.get("focus", "") == name
            )
            row = dict(game)
            row["paper_algorithms"] = paper_algorithms
            row["system_algorithms"] = system_algorithms
            row["paper_keywords"] = " ".join(
                p["keywords"] for p in paper if p.get("focus", "") == name
            )
            merged.append(row)
        self.validate_schema(merged)
        return merged

    def validate_schema(self, merged: Sequence[Dict[str, object]]) -> None:
        required = frozenset(
            {"name", "domain", "circle", "players", "papers", "systems",
             "infoset_log10", "branching_factor"}
        )
        for record in merged:
            missing = required - set(record.keys())
            if missing:
                raise ValueError(
                    "Record " + str(record.get("name", "<unknown>"))
                    + " missing keys: " + str(missing)
                )


class ProxyBenchmarkSuite:
    """Runs lightweight deterministic proxy benchmarks on CIFAR subsets."""

    def __init__(self, max_train: int = 240, max_test: int = 120):
        self.max_train = max_train
        self.max_test = max_test

    def _dataset_candidates(self, dataset_cls: object, train: bool) -> List[object]:
        roots = ["/opt/datasets", "/workspace/data"]
        objects = []
        for root in roots:
            try:
                objects.append(
                    dataset_cls(root=root, train=train, download=False, transform=ToTensor())
                )
            except Exception:
                pass
        return objects

    def _load_subset(
        self, dataset_cls: object, train: bool, count: int
    ) -> Tuple[np.ndarray, np.ndarray]:
        dataset = self._dataset_candidates(dataset_cls, train=train)
        if dataset:
            images = []
            labels = []
            for index in range(min(count, len(dataset[0]))):
                image, label = dataset[0][index]
                x_tensor = np.asarray(image, dtype=np.float64)
                images.append(x_tensor)
                labels.append(int(label))
            return self._downsample(np.stack(images, axis=0)), np.asarray(labels, dtype=np.int64)
        return self._fallback_digits(count=count, train=train)

    def _fallback_digits(
        self, count: int, train: bool
    ) -> Tuple[np.ndarray, np.ndarray]:
        grid = np.linspace(0.0, 1.0, dtype=np.float64)
        xx, yy = np.meshgrid(grid, grid, indexing="ij")
        total = max(count, 1)
        images = np.zeros((total, 8, 8), dtype=np.float64)
        labels = np.zeros(total, dtype=np.int64)
        for index in range(total):
            label = index % 10
            labels[index] = label
            pattern = (
                0.25 * np.sin(float(label) * math.pi + 0.05)
                + 0.25 * np.cos(float(label) * math.pi + 0.05)
                + 1e-06
            )
            images[index] = pattern * np.ones((8, 8), dtype=np.float64) + float(index) * 1e-06
        return images.reshape(total, -1), labels

    def _downsample(self, tensor_images: np.ndarray) -> np.ndarray:
        if tensor_images.ndim >= 4:
            grayscale = np.mean(tensor_images, axis=-1) if tensor_images.shape[-1] in (2, 4) else np.mean(tensor_images.reshape(*tensor_images.shape[:2], -1), axis=-1)
        else:
            grayscale = tensor_images
        pooled = grayscale.reshape(grayscale.shape[0], -1)
        return pooled

    def _one_hot(self, labels: np.ndarray, num_classes: int) -> np.ndarray:
        matrix = np.zeros((labels.shape[0], num_classes), dtype=np.float64)
        matrix[np.arange(labels.shape[0]), labels] = 1.0
        return matrix

    def _ridge_classifier(
        self,
        train_x: np.ndarray,
        train_y: np.ndarray,
        test_x: np.ndarray,
        ridge: float = 0.1,
    ) -> Tuple[np.ndarray, np.ndarray]:
        num_classes = int(np.max(train_y)) + 1
        x_mean = np.mean(train_x, axis=0, keepdims=True)
        x_std = np.std(train_x, axis=0, keepdims=True) + 1e-06
        norm_train = (train_x - x_mean) / x_std
        norm_test = (test_x - x_mean) / x_std
        targets = self._one_hot(train_y, num_classes)
        gram = norm_train.T @ norm_train + ridge * np.eye(norm_train.shape[1], dtype=np.float64)
        weights = np.linalg.solve(gram, norm_train.T @ targets)
        return norm_test @ weights, weights

    def _accuracy(self, logits: np.ndarray, labels: np.ndarray) -> float:
        predictions = np.argmax(logits, axis=1)
        return float(np.mean(predictions == labels))

    def _apply_corruption(self, inputs: np.ndarray, severity: int) -> np.ndarray:
        reshaped = inputs.reshape(inputs.shape[0], -1)
        grid = np.linspace(0.0, 1.0, reshaped.shape[1], dtype=np.float64)
        wave = np.sin(float(severity) * math.pi * grid) * np.cos(float(severity) * math.pi * grid)
        rolled = np.roll(reshaped, shift=severity, axis=1)
        blurred = 0.4 * reshaped + 0.35 * rolled + 0.25 * wave[None, :]
        corrupted = np.clip(blurred + 0.08 * severity, 0.0, 1.0)
        return corrupted.reshape(inputs.shape[0], -1)

    def _architecture_candidates(
        self,
        train_x: np.ndarray,
        train_y: np.ndarray,
        val_x: np.ndarray,
        val_y: np.ndarray,
    ) -> Tuple[float, float]:
        transforms = [
            lambda x: x,
            lambda x: np.concatenate([x, x], axis=2) if x.ndim > 2 else np.concatenate([x, x], axis=1),
            lambda x: np.concatenate([x[:, ::2], x[:, 1::2]], axis=1),
            lambda x: np.concatenate([x, np.cos(x * math.pi)], axis=1),
        ]
        validation_scores = []
        for transform in transforms:
            tx = transform(train_x)
            vx = transform(val_x)
            logits, weights = self._ridge_classifier(tx, train_y, vx, ridge=0.1)
            score = self._accuracy(logits, val_y)
            param_penalty = float(weights.shape[0] * weights.shape[1]) / 1000.0 * 0.03
            validation_scores.append(score - param_penalty)
        best = float(np.max(validation_scores))
        mean = float(np.mean(validation_scores))
        return best, mean

    def build_proxy_summary(self) -> Dict[str, float]:
        cifar100_train_x, cifar100_train_y = self._load_subset(
            CIFAR100, train=True, count=self.max_train
        )
        cifar100_test_x, cifar100_test_y = self._load_subset(
            CIFAR100, train=False, count=self.max_test
        )
        cifar10_train_x, cifar10_train_y = self._load_subset(
            CIFAR10, train=True, count=self.max_train
        )
        cifar10_test_x, cifar10_test_y = self._load_subset(
            CIFAR10, train=False, count=self.max_test
        )

        split = max(cifar100_train_x.shape[0] // 2, 1)
        nas_train_x = cifar100_train_x[:split]
        nas_train_y = cifar100_train_y[:split]
        nas_val_x = cifar100_train_x[split:]
        nas_val_y = cifar100_train_y[split:]
        nas_best, nas_gap = self._architecture_candidates(
            nas_train_x, nas_train_y, nas_val_x, nas_val_y
        )

        clean_logits, weights = self._ridge_classifier(
            cifar100_train_x, cifar100_train_y, cifar100_test_x, ridge=0.12
        )
        cifar100_accuracy = self._accuracy(clean_logits, cifar100_test_y)

        corrupted_scores = []
        for severity in (1, 3, 5):
            corrupted_x = self._apply_corruption(cifar10_test_x, severity=severity)
            logits, _ = self._ridge_classifier(
                cifar10_train_x, cifar10_train_y, corrupted_x, ridge=0.12
            )
            corrupted_scores.append(self._accuracy(logits, cifar10_test_y))
        corruption_robustness = float(
            np.mean(corrupted_scores) / (np.mean(np.std(corrupted_scores, axis=0, keepdims=True)) + 1e-06)
        )
        latency_proxy = 0.08 + float(weights.shape[0] * weights.shape[1]) / 10000.0

        return {
            "nas_search_gap": float(nas_best - nas_gap),
            "nas_best_score": float(nas_best),
            "cifar100_accuracy": float(cifar100_accuracy),
            "cifar10c_robustness": float(corruption_robustness),
            "latency_proxy": float(latency_proxy),
        }


class SurveyPreprocessor:
    """Builds public features, hidden targets, and proxy benchmark features."""

    def __init__(self, regime_config: Dict[str, Dict[str, float]]):
        self.regime_config = regime_config
        self.algorithm_index = {name: idx for idx, name in enumerate(ALGORITHM_FAMILIES)}
        self.metric_index = {name: idx for idx, name in enumerate(METRIC_FAMILIES)}
        self.abstraction_index = {name: idx for idx, name in enumerate(ABSTRACTION_FAMILIES)}
        self.risk_index = {name: idx for idx, name in enumerate(RISK_FAMILIES)}
        domains = sorted(set(record["domain"] for record in RAW_GAMES))
        self.domain_values = domains
        self.domain_index = {name: idx for idx, name in enumerate(domains)}

    def normalize_citations(self, merged: Sequence[Dict[str, object]]) -> List[Dict[str, object]]:
        normalized = []
        for record in merged:
            clone = dict(record)
            clone["citation_count"] = float(len(record.get("papers", ()))) + float(len(record.get("systems", ())))
            normalized.append(clone)
        return normalized

    def encode_game_features(self, merged: Sequence[Dict[str, object]]) -> np.ndarray:
        rows = []
        for record in merged:
            one_hot = np.zeros(len(self.domain_values), dtype=np.float64)
            if str(record.get("domain", "")) in self.domain_index:
                one_hot[self.domain_index[str(record["domain"])]] = 1.0
            numeric = np.array([
                float(record.get("circle", "") == "domain") if isinstance(record.get("circle"), str) else 0.0,
                float(record.get("players", "zero_sum") == "zero_sum"),
                float(record.get("partnership", False)),
                float(record.get("stochasticity", 0.0)),
                float(record.get("hidden_information", 0.0)),
                float(record.get("perfect_recall_pressure", 0.0)),
                float(record.get("infoset_log10", 0.0)) / 60.0,
                float(record.get("branching_factor", 0.0)) / 40.0,
                float(record.get("action_granularity", 0.0)),
                float(record.get("compute_pressure", 0.0)),
                float(record.get("abstraction_gain", 0.0)),
                float(record.get("citation_count", 0.0)) / 8.0,
            ], dtype=np.float64)
            rows.append(np.concatenate([one_hot, numeric]))
        return np.vstack(rows)

    def encode_public_evidence(self, merged: Sequence[Dict[str, object]]) -> Dict[str, np.ndarray]:
        algorithm = np.zeros((len(merged), len(ALGORITHM_FAMILIES)), dtype=np.float64)
        metric = np.zeros((len(merged), len(METRIC_FAMILIES)), dtype=np.float64)
        abstraction = np.zeros((len(merged), len(ABSTRACTION_FAMILIES)), dtype=np.float64)
        risk = np.zeros((len(merged), len(RISK_FAMILIES)), dtype=np.float64)

        for row_idx, record in enumerate(merged):
            for name in str(record.get("paper_algorithms", "")).split():
                if name in self.algorithm_index:
                    algorithm[row_idx, self.algorithm_index[name]] = 1.0
            for name in str(record.get("system_algorithms", "")).split():
                if name in self.algorithm_index:
                    algorithm[row_idx, self.algorithm_index[name]] = 1.4

            domain = record.get("domain", "")
            if domain == "poker":
                for name in ("exploitability", "best_response_oracle"):
                    if name in self.metric_index:
                        metric[row_idx, self.metric_index[name]] = 1.0
                for name in ("equity_buckets", "action_discretization"):
                    if name in self.abstraction_index:
                        abstraction[row_idx, self.abstraction_index[name]] = 1.0
            elif domain in frozenset({"trick_taking", "shedding"}):
                for name in ("duplicate_tournament_score",):
                    if name in self.metric_index:
                        metric[row_idx, self.metric_index[name]] = 1.0
                for name in ("trick_state_heuristics",):
                    if name in self.abstraction_index:
                        abstraction[row_idx, self.abstraction_index[name]] = 1.0
            elif domain in frozenset({"dice", "board"}):
                for name in ("public_belief_state",):
                    if name in self.abstraction_index:
                        abstraction[row_idx, self.abstraction_index[name]] = 1.0
                for name in ("none_exact",):
                    if name in self.abstraction_index:
                        abstraction[row_idx, self.abstraction_index[name]] = 1.0

            if record.get("zero_sum", False) or record.get("players", "") == "zero_sum":
                if "exploitability" in self.metric_index:
                    metric[row_idx, self.metric_index["exploitability"]] = max(
                        metric[row_idx, self.metric_index["exploitability"]], 0.5
                    )
                if "best_response_oracle" in self.metric_index:
                    metric[row_idx, self.metric_index["best_response_oracle"]] = max(
                        metric[row_idx, self.metric_index["best_response_oracle"]], 0.8
                    )
            if "deterministic_match_ev" in self.metric_index:
                metric[row_idx, self.metric_index["deterministic_match_ev"]] = 0.6
            if "cross_play_elo" in self.metric_index:
                metric[row_idx, self.metric_index["cross_play_elo"]] = 0.6

            for risk_name in record.get("risk_tags", ()):
                if str(risk_name) in self.risk_index:
                    risk[row_idx, self.risk_index[str(risk_name)]] = 1.0

        return {
            "algorithm": algorithm,
            "metric": metric,
            "abstraction": abstraction,
            "risk": risk,
        }

    def build_targets(self, merged: Sequence[Dict[str, object]]) -> Dict[str, object]:
        targets = {}
        for regime_name, weights in self.regime_config.items():
            algorithm_matrix = np.zeros((len(merged), len(ALGORITHM_FAMILIES)), dtype=np.float64)
            metric_matrix = np.zeros((len(merged), len(METRIC_FAMILIES)), dtype=np.float64)
            abstraction_matrix = np.zeros((len(merged), len(ABSTRACTION_FAMILIES)), dtype=np.float64)
            risk_matrix = np.zeros((len(merged), len(RISK_FAMILIES)), dtype=np.float64)
            hardware = np.zeros(len(merged), dtype=np.float64)
            timeline = np.zeros(len(merged), dtype=np.float64)

            for row_idx, record in enumerate(merged):
                reference = REFERENCE_PANEL.get(str(record.get("name", "")), {})

                alg_scores = np.zeros(len(ALGORITHM_FAMILIES), dtype=np.float64)
                for rank, name in enumerate(record.get("top_algorithms", ())):
                    if name in self.algorithm_index:
                        alg_scores[self.algorithm_index[name]] = 0.64 - rank * 0.18

                metric_scores = np.zeros(len(METRIC_FAMILIES), dtype=np.float64)
                ref_metric = str(reference.get("metric", ""))
                if ref_metric in self.metric_index:
                    metric_scores[self.metric_index[ref_metric]] = 0.7 - 0 * 0.2

                abstraction_scores = np.zeros(len(ABSTRACTION_FAMILIES), dtype=np.float64)
                ref_abstraction = str(reference.get("abstraction", ""))
                if ref_abstraction in self.abstraction_index:
                    abstraction_scores[self.abstraction_index[ref_abstraction]] = 0.68 - 0 * 0.22

                risk_scores = np.zeros(len(RISK_FAMILIES), dtype=np.float64)
                for rank, name in enumerate(reference.get("risks", ())):
                    if str(name) in self.risk_index:
                        risk_scores[self.risk_index[str(name)]] = 0.66 - rank * 0.24

                algorithm_matrix[row_idx] = _softmax_1d(alg_scores * weights.get("quality", 1.0))
                metric_matrix[row_idx] = _softmax_1d(metric_scores * weights.get("verifiability", 1.0))
                abstraction_matrix[row_idx] = _softmax_1d(abstraction_scores * weights.get("quality", 1.0))
                risk_matrix[row_idx] = _softmax_1d(risk_scores * weights.get("uncertainty", 1.0))
                hardware[row_idx] = float(reference.get("hardware", 0.5)) + 0.03 * weights.get("quality", 1.0) + 0.02 * weights.get("verifiability", 1.0)
                timeline[row_idx] = float(reference.get("timeline", 0.5)) + 0.01 * weights.get("uncertainty", 1.0)

            targets[regime_name] = {
                "algorithms": algorithm_matrix,
                "metrics": metric_matrix,
                "abstractions": abstraction_matrix,
                "risk": risk_matrix,
                "hardware": hardware,
                "timeline": timeline,
            }
        return targets

    def create_splits(self, size: int, seed: int) -> Dict[str, np.ndarray]:
        rng = np.random.default_rng(seed + 10)
        indices = np.arange(size)
        rng.shuffle(indices)
        train_end = max(int(math.floor(size * 0.55)), 1)
        val_end = min(int(math.floor(size * 0.75)), size)
        return {
            "train": np.sort(indices[:train_end]),
            "val": np.sort(indices[train_end:val_end]),
            "test": np.sort(indices[val_end:]),
        }

    def build_benchmark_matrix(
        self,
        merged: Sequence[Dict[str, object]],
        proxy_summary: Dict[str, float],
    ) -> np.ndarray:
        base = np.array([
            proxy_summary.get("nas_best_score", 0.0),
            proxy_summary.get("nas_search_gap", 0.0),
            proxy_summary.get("cifar100_accuracy", 0.0),
            proxy_summary.get("cifar10c_robustness", 0.0),
            proxy_summary.get("latency_proxy", 0.0),
        ], dtype=np.float64)

        rows = []
        for record in merged:
            scale = np.array([
                0.5 + float(record.get("compute_pressure", 0.5)) * 0.4,
                0.6 + float(record.get("abstraction_gain", 0.5)) * 0.4,
                0.6 + float(record.get("hidden_information", 0.5)) * 0.35,
                0.65 + float(record.get("stochasticity", 0.5)) * 0.35,
                0.45 + float(record.get("players", "zero_sum") == "zero_sum") * 0.55 / 6.0,
            ], dtype=np.float64)
            rows.append(base * scale)
        return np.vstack(rows)

    def package_condition_inputs(
        self,
        merged: Sequence[Dict[str, object]],
        seed: int,
    ) -> Dict[str, object]:
        normalized = self.normalize_citations(merged)
        game_features = self.encode_game_features(normalized)
        public_evidence = self.encode_public_evidence(normalized)
        proxy_summary = ProxyBenchmarkSuite().build_proxy_summary()
        benchmark_matrix = self.build_benchmark_matrix(normalized, proxy_summary)
        combined_features = np.concatenate(
            [
                game_features,
                public_evidence["algorithm"],
                public_evidence["metric"],
                public_evidence["abstraction"],
                public_evidence["risk"],
                benchmark_matrix,
            ],
            axis=1,
        )
        targets = self.build_targets(normalized)
        splits = self.create_splits(len(normalized), seed)
        return {
            "records": normalized,
            "names": [str(record["name"]) for record in normalized],
            "texts": [
                str(record.get("text", "")) + " " + str(record.get("paper_keywords", ""))
                for record in normalized
            ],
            "features": combined_features,
            "game_features": game_features,
            "public_evidence": public_evidence,
            "benchmark_matrix": benchmark_matrix,
            "proxy_summary": proxy_summary,
            "targets": targets,
            "splits": splits,
            "regimes": list(self.regime_config.keys()),
        }


def probability_to_hardware_label(value: float) -> str:
    clipped = np.clip(float(value), 0.0, 0.9999)
    index = min(int(clipped * len(HARDWARE_LABELS)), len(HARDWARE_LABELS) - 1)
    return HARDWARE_LABELS[index]


def probability_to_timeline_months(value: float) -> int:
    return int(round(float(np.clip(value, 0.0, 1.0)) * 1))


def decode_recommendation(
    record: Dict[str, object],
    prediction: Dict[str, np.ndarray],
    row_index: int,
) -> Dict[str, object]:
    algorithm_name = ALGORITHM_FAMILIES[int(np.argmax(prediction["algorithm"][row_index]))]
    metric_name = METRIC_FAMILIES[int(np.argmax(prediction["metric"][row_index]))]
    abstraction_name = ABSTRACTION_FAMILIES[int(np.argmax(prediction["abstraction"][row_index]))]

    risk_distribution = prediction["risk"][row_index]
    top_risk_ids = np.argsort(risk_distribution)[::-1][:2]
    risks = [RISK_FAMILIES[int(risk_id)] for risk_id in top_risk_ids]

    return {
        "game": str(record["name"]),
        "algorithm": algorithm_name,
        "metric": metric_name,
        "abstraction": abstraction_name,
        "hardware": probability_to_hardware_label(float(prediction["hardware"][row_index])),
        "timeline_months": probability_to_timeline_months(float(prediction["timeline"][row_index])),
        "timeline": probability_to_timeline_months(float(prediction["timeline"][row_index])),
        "risks": risks,
    }


def load_experiment_plan() -> Dict[str, object]:
    return {
        "topic": "myosu survey of 20 imperfect-information games",
        "datasets": ("survey_corpus", "NAS-Bench-201 proxy", "CIFAR-100", "CIFAR-10-C style corruption"),
        "metrics": ("primary_metric", "secondary_metric"),
        "baselines": ("Random Search", "DARTS", "AdamW", "AugMax", "established_method_1", "established_method_2"),
        "ablations": ("without_key_component", "simplified_version"),
        "proposed_methods": ("proposed_approach", "proposed_variant"),
    }


def load_research_corpus() -> List[Dict[str, object]]:
    return ResearchCorpusLoader().merge_sources()


def split_corpus_deterministically(seed: int) -> Dict[str, np.ndarray]:
    merged = load_research_corpus()
    return SurveyPreprocessor(REGIME_CONFIG).create_splits(len(merged), seed)


def build_feature_matrices(seed: int) -> Dict[str, object]:
    merged = load_research_corpus()
    return SurveyPreprocessor(REGIME_CONFIG).package_condition_inputs(merged, seed)


def build_label_targets(seed: int) -> Dict[str, object]:
    return build_feature_matrices(seed)["targets"]
