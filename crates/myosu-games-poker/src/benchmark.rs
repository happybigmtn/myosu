use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use rbp_gameplay::Edge;
use rbp_mccfr::{Encounter, Metrics};
use rbp_nlhe::{NlheEncoder, NlheProfile};
use thiserror::Error;

use crate::artifacts::{NlheAbstractionStreet, NlheBootstrapScenario, bootstrap_scenarios};
use crate::request::{NlheStrategyRequest, StrategyRequestError};
use crate::robopoker::{NlheStrategyResponse, RbpNlheEdge, recommended_edge};
use crate::solver::{PokerSolver, PokerSolverError};
use crate::state::NlheTablePosition;
use crate::{ArtifactCodecError, decode_encoder, encode_encoder, load_encoder_dir};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReferenceEdgeKind {
    Fold,
    Check,
    Call,
    Aggro,
    Shove,
}

/// Per-scenario benchmark comparison row for the dedicated NLHE reference pack.
#[derive(Clone, Debug, PartialEq)]
pub struct NlheScenarioBenchmarkRow {
    pub label: &'static str,
    pub street: NlheAbstractionStreet,
    pub observation: &'static str,
    pub l1_distance: f64,
    pub exact_distribution_match: bool,
    pub exact_action_match: bool,
    pub reference_recommended: Option<RbpNlheEdge>,
    pub candidate_recommended: Option<RbpNlheEdge>,
}

/// Aggregate benchmark report for the dedicated NLHE reference pack.
#[derive(Clone, Debug, PartialEq)]
pub struct NlheScenarioBenchmarkReport {
    pub scenario_count: usize,
    pub unique_query_count: usize,
    pub exact_distribution_matches: usize,
    pub exact_action_matches: usize,
    pub mean_l1_distance: f64,
    pub max_l1_distance: f64,
    pub street_counts: BTreeMap<NlheAbstractionStreet, usize>,
    pub street_mean_l1: BTreeMap<NlheAbstractionStreet, f64>,
    pub street_action_matches: BTreeMap<NlheAbstractionStreet, usize>,
    pub rows: Vec<NlheScenarioBenchmarkRow>,
}

impl NlheScenarioBenchmarkReport {
    pub fn recommendation_agreement(&self) -> f64 {
        ratio(self.exact_action_matches, self.scenario_count)
    }

    pub fn street_mean_l1_token(&self) -> String {
        ordered_streets()
            .into_iter()
            .map(|street| {
                format!(
                    "{}={:.6}",
                    street.as_str(),
                    self.street_mean_l1.get(&street).copied().unwrap_or(0.0)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn street_action_match_token(&self) -> String {
        ordered_streets()
            .into_iter()
            .map(|street| {
                format!(
                    "{}={}/{}",
                    street.as_str(),
                    self.street_action_matches
                        .get(&street)
                        .copied()
                        .unwrap_or(0),
                    self.street_counts.get(&street).copied().unwrap_or(0)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// Errors returned while building or scoring the dedicated NLHE reference pack.
#[derive(Debug, Error)]
pub enum NlheScenarioBenchmarkError {
    #[error("{0}")]
    Request(#[from] StrategyRequestError),

    #[error("{0}")]
    Solver(#[from] PokerSolverError),

    #[error("bootstrap scenario `{label}` produced no legal NLHE edges")]
    EmptyChoices { label: &'static str },
}

/// One exploitability sample in a poker training benchmark.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PokerBenchmarkPoint {
    pub iterations: usize,
    pub exploitability: f32,
}

/// Errors returned while building poker exploitability benchmark points.
#[derive(Debug, Error)]
pub enum PokerBenchmarkError {
    #[error("failed to load encoder directory `{path}`: {source}")]
    LoadEncoder {
        path: String,
        #[source]
        source: ArtifactCodecError,
    },
    #[error("failed to clone encoder artifact bytes for benchmark reuse: {source}")]
    CloneEncoder {
        #[source]
        source: ArtifactCodecError,
    },
    #[error(transparent)]
    Solver(#[from] PokerSolverError),
}

/// Build exploitability samples for the requested training ladder.
pub fn benchmark_points_from_encoder_dir(
    directory: impl AsRef<Path>,
    iterations: &[usize],
) -> Result<Vec<PokerBenchmarkPoint>, PokerBenchmarkError> {
    let directory = directory.as_ref();
    let encoder =
        load_encoder_dir(directory).map_err(|source| PokerBenchmarkError::LoadEncoder {
            path: directory.display().to_string(),
            source,
        })?;

    benchmark_points_from_encoder(encoder, iterations)
}

fn benchmark_points_from_encoder(
    encoder: NlheEncoder,
    iterations: &[usize],
) -> Result<Vec<PokerBenchmarkPoint>, PokerBenchmarkError> {
    let encoder_bytes =
        encode_encoder(&encoder).map_err(|source| PokerBenchmarkError::CloneEncoder { source })?;
    let mut points = Vec::with_capacity(iterations.len());

    for iteration_count in iterations {
        let encoder = decode_encoder(&encoder_bytes)
            .map_err(|source| PokerBenchmarkError::CloneEncoder { source })?;
        let mut solver = PokerSolver::new(encoder);
        solver.train(*iteration_count)?;
        points.push(PokerBenchmarkPoint {
            iterations: *iteration_count,
            exploitability: solver.exploitability()?,
        });
    }

    Ok(points)
}

/// Build the repo-owned NLHE reference solver from a supplied encoder.
pub fn bootstrap_reference_solver(
    encoder: NlheEncoder,
) -> Result<PokerSolver, NlheScenarioBenchmarkError> {
    let profile = bootstrap_reference_profile(&encoder)?;
    Ok(PokerSolver::from_parts(profile, encoder))
}

/// Benchmark a candidate solver against the repo-owned reference checkpoint.
pub fn benchmark_against_bootstrap_reference(
    candidate: &PokerSolver,
) -> Result<NlheScenarioBenchmarkReport, NlheScenarioBenchmarkError> {
    let reference = bootstrap_reference_solver(candidate.snapshot_encoder()?)?;
    benchmark_solver_against_reference(candidate, &reference)
}

/// Benchmark a candidate solver against an explicit reference solver.
pub fn benchmark_solver_against_reference(
    candidate: &PokerSolver,
    reference: &PokerSolver,
) -> Result<NlheScenarioBenchmarkReport, NlheScenarioBenchmarkError> {
    let scenarios = bootstrap_scenarios();
    let mut rows = Vec::with_capacity(scenarios.len());
    let mut unique_queries = BTreeSet::new();
    let mut exact_distribution_matches = 0usize;
    let mut exact_action_matches = 0usize;
    let mut total_l1_distance = 0.0_f64;
    let mut max_l1_distance = 0.0_f64;
    let mut street_counts = BTreeMap::new();
    let mut street_l1_totals = BTreeMap::new();
    let mut street_action_matches = BTreeMap::new();

    for scenario in &scenarios {
        let request = NlheStrategyRequest::from_observation_text(
            NlheTablePosition::Button,
            scenario.observation,
            Vec::new(),
            0,
        )?;
        let query = request.query_with_encoder(candidate.encoder())?;
        unique_queries.insert((query.info.subgame, query.info.bucket, query.info.choices));

        let reference_response = reference.answer(query.clone());
        let candidate_response = candidate.answer(query);
        let l1_distance = l1_distance(&reference_response, &candidate_response);
        let exact_distribution_match = l1_distance < f64::EPSILON;
        let reference_recommended = recommended_edge(&reference_response);
        let candidate_recommended = recommended_edge(&candidate_response);
        let exact_action_match = reference_recommended == candidate_recommended;

        if exact_distribution_match {
            exact_distribution_matches += 1;
        }
        if exact_action_match {
            exact_action_matches += 1;
        }

        total_l1_distance += l1_distance;
        max_l1_distance = max_l1_distance.max(l1_distance);
        *street_counts.entry(scenario.street).or_insert(0usize) += 1;
        *street_l1_totals.entry(scenario.street).or_insert(0.0_f64) += l1_distance;
        if exact_action_match {
            *street_action_matches
                .entry(scenario.street)
                .or_insert(0usize) += 1;
        }

        rows.push(NlheScenarioBenchmarkRow {
            label: scenario.label,
            street: scenario.street,
            observation: scenario.observation,
            l1_distance,
            exact_distribution_match,
            exact_action_match,
            reference_recommended,
            candidate_recommended,
        });
    }

    let scenario_count = rows.len();
    let street_mean_l1 = ordered_streets()
        .into_iter()
        .map(|street| {
            let count = street_counts.get(&street).copied().unwrap_or(0);
            let total = street_l1_totals.get(&street).copied().unwrap_or(0.0);
            let mean = if count == 0 {
                0.0
            } else {
                total / count as f64
            };
            (street, mean)
        })
        .collect();

    Ok(NlheScenarioBenchmarkReport {
        scenario_count,
        unique_query_count: unique_queries.len(),
        exact_distribution_matches,
        exact_action_matches,
        mean_l1_distance: if scenario_count == 0 {
            0.0
        } else {
            total_l1_distance / scenario_count as f64
        },
        max_l1_distance,
        street_counts,
        street_mean_l1,
        street_action_matches,
        rows,
    })
}

fn bootstrap_reference_profile(
    encoder: &NlheEncoder,
) -> Result<NlheProfile, NlheScenarioBenchmarkError> {
    let mut encounters = BTreeMap::new();
    let mut seen_queries = BTreeSet::new();

    for scenario in bootstrap_scenarios() {
        let request = NlheStrategyRequest::from_observation_text(
            NlheTablePosition::Button,
            scenario.observation,
            Vec::new(),
            0,
        )?;
        let query = request.query_with_encoder(encoder)?;
        let query_key = (query.info.subgame, query.info.bucket, query.info.choices);
        if !seen_queries.insert(query_key) {
            continue;
        }

        let info = query.info.into_info();
        let legal_edges = info
            .choices()
            .into_iter()
            .map(RbpNlheEdge::from)
            .collect::<Vec<_>>();
        let policy = reference_policy_for_scenario(&scenario, &legal_edges)?;
        encounters.insert(info, policy);
    }

    Ok(NlheProfile {
        iterations: 1,
        encounters,
        metrics: Metrics::with_epoch(1),
    })
}

fn reference_policy_for_scenario(
    scenario: &NlheBootstrapScenario,
    legal_edges: &[RbpNlheEdge],
) -> Result<BTreeMap<RbpNlheEdge, Encounter>, NlheScenarioBenchmarkError> {
    let preferred = select_preferred_edge(scenario, legal_edges)?;
    let secondary = select_secondary_edge(scenario, legal_edges, preferred);

    if legal_edges.len() == 1 {
        return Ok(BTreeMap::from([(
            preferred,
            Encounter::new(1.0, 0.0, 0.0, 1),
        )]));
    }

    let preferred_weight = if secondary.is_some() {
        0.70_f32
    } else {
        0.80_f32
    };
    let secondary_weight = match secondary {
        Some(_) if legal_edges.len() == 2 => 1.0_f32 - preferred_weight,
        Some(_) => 0.20_f32,
        None => 0.0_f32,
    };
    let remainder_total = 1.0_f32 - preferred_weight - secondary_weight;
    let remainder_count = legal_edges
        .iter()
        .copied()
        .filter(|edge| Some(*edge) != secondary && *edge != preferred)
        .count();
    let remainder_share = if remainder_count == 0 {
        0.0_f32
    } else {
        remainder_total / remainder_count as f32
    };

    Ok(legal_edges
        .iter()
        .copied()
        .map(|edge| {
            let weight = if edge == preferred {
                preferred_weight
            } else if Some(edge) == secondary {
                secondary_weight
            } else {
                remainder_share
            };
            (edge, Encounter::new(weight, 0.0, 0.0, 1))
        })
        .collect())
}

fn select_preferred_edge(
    scenario: &NlheBootstrapScenario,
    legal_edges: &[RbpNlheEdge],
) -> Result<RbpNlheEdge, NlheScenarioBenchmarkError> {
    let preferences = match preferred_kind_for_scenario(scenario) {
        ReferenceEdgeKind::Aggro => [
            ReferenceEdgeKind::Aggro,
            ReferenceEdgeKind::Shove,
            ReferenceEdgeKind::Call,
            ReferenceEdgeKind::Check,
            ReferenceEdgeKind::Fold,
        ],
        ReferenceEdgeKind::Call => [
            ReferenceEdgeKind::Call,
            ReferenceEdgeKind::Check,
            ReferenceEdgeKind::Aggro,
            ReferenceEdgeKind::Shove,
            ReferenceEdgeKind::Fold,
        ],
        ReferenceEdgeKind::Check => [
            ReferenceEdgeKind::Check,
            ReferenceEdgeKind::Call,
            ReferenceEdgeKind::Aggro,
            ReferenceEdgeKind::Shove,
            ReferenceEdgeKind::Fold,
        ],
        ReferenceEdgeKind::Fold => [
            ReferenceEdgeKind::Fold,
            ReferenceEdgeKind::Check,
            ReferenceEdgeKind::Call,
            ReferenceEdgeKind::Aggro,
            ReferenceEdgeKind::Shove,
        ],
        ReferenceEdgeKind::Shove => [
            ReferenceEdgeKind::Shove,
            ReferenceEdgeKind::Aggro,
            ReferenceEdgeKind::Call,
            ReferenceEdgeKind::Check,
            ReferenceEdgeKind::Fold,
        ],
    };

    preferences
        .into_iter()
        .find_map(|kind| select_edge_by_kind(legal_edges, kind))
        .ok_or(NlheScenarioBenchmarkError::EmptyChoices {
            label: scenario.label,
        })
}

fn select_secondary_edge(
    scenario: &NlheBootstrapScenario,
    legal_edges: &[RbpNlheEdge],
    preferred: RbpNlheEdge,
) -> Option<RbpNlheEdge> {
    let preferences = match preferred_kind_for_scenario(scenario) {
        ReferenceEdgeKind::Aggro => [
            ReferenceEdgeKind::Call,
            ReferenceEdgeKind::Check,
            ReferenceEdgeKind::Fold,
            ReferenceEdgeKind::Shove,
            ReferenceEdgeKind::Aggro,
        ],
        ReferenceEdgeKind::Call => [
            ReferenceEdgeKind::Aggro,
            ReferenceEdgeKind::Check,
            ReferenceEdgeKind::Fold,
            ReferenceEdgeKind::Shove,
            ReferenceEdgeKind::Call,
        ],
        ReferenceEdgeKind::Check => [
            ReferenceEdgeKind::Call,
            ReferenceEdgeKind::Aggro,
            ReferenceEdgeKind::Fold,
            ReferenceEdgeKind::Shove,
            ReferenceEdgeKind::Check,
        ],
        ReferenceEdgeKind::Fold => [
            ReferenceEdgeKind::Check,
            ReferenceEdgeKind::Call,
            ReferenceEdgeKind::Aggro,
            ReferenceEdgeKind::Shove,
            ReferenceEdgeKind::Fold,
        ],
        ReferenceEdgeKind::Shove => [
            ReferenceEdgeKind::Aggro,
            ReferenceEdgeKind::Call,
            ReferenceEdgeKind::Check,
            ReferenceEdgeKind::Fold,
            ReferenceEdgeKind::Shove,
        ],
    };

    preferences
        .into_iter()
        .filter_map(|kind| select_edge_by_kind(legal_edges, kind))
        .find(|edge| *edge != preferred)
}

fn preferred_kind_for_scenario(scenario: &NlheBootstrapScenario) -> ReferenceEdgeKind {
    match scenario.street {
        NlheAbstractionStreet::Preflop => ReferenceEdgeKind::Aggro,
        NlheAbstractionStreet::Flop | NlheAbstractionStreet::Turn => {
            if scenario.label.contains("open_ender")
                || scenario.label.contains("wheel_draw")
                || scenario.label.contains("top_pair")
            {
                ReferenceEdgeKind::Call
            } else {
                ReferenceEdgeKind::Aggro
            }
        }
        NlheAbstractionStreet::River => {
            if scenario.label.contains("open_ender")
                || scenario.label.contains("wheel_draw")
                || scenario.label.contains("combo_draw")
            {
                ReferenceEdgeKind::Check
            } else if scenario.label.contains("top_pair") {
                ReferenceEdgeKind::Call
            } else {
                ReferenceEdgeKind::Aggro
            }
        }
    }
}

fn select_edge_by_kind(
    legal_edges: &[RbpNlheEdge],
    kind: ReferenceEdgeKind,
) -> Option<RbpNlheEdge> {
    legal_edges
        .iter()
        .copied()
        .find(|edge| edge_kind(*edge) == kind)
}

fn edge_kind(edge: RbpNlheEdge) -> ReferenceEdgeKind {
    let edge = Edge::from(edge);
    if edge.is_shove() {
        return ReferenceEdgeKind::Shove;
    }
    if edge.is_raise() {
        return ReferenceEdgeKind::Aggro;
    }

    match edge {
        Edge::Fold => ReferenceEdgeKind::Fold,
        Edge::Check => ReferenceEdgeKind::Check,
        Edge::Call | Edge::Draw => ReferenceEdgeKind::Call,
        Edge::Open(_) | Edge::Raise(_) | Edge::Shove => {
            unreachable!("raise and shove edges return early")
        }
    }
}

fn ordered_streets() -> [NlheAbstractionStreet; 4] {
    [
        NlheAbstractionStreet::Preflop,
        NlheAbstractionStreet::Flop,
        NlheAbstractionStreet::Turn,
        NlheAbstractionStreet::River,
    ]
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn l1_distance(expected: &NlheStrategyResponse, observed: &NlheStrategyResponse) -> f64 {
    expected
        .actions
        .iter()
        .map(|(action, _)| *action)
        .chain(observed.actions.iter().map(|(action, _)| *action))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|action| {
            let expected_probability = expected.probability_for(&action);
            let observed_probability = observed.probability_for(&action);
            f64::from((expected_probability - observed_probability).abs())
        })
        .sum()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::collections::BTreeMap;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use rbp_cards::{Isomorphism, Observation};
    use rbp_gameplay::Abstraction;

    use super::*;
    use crate::artifacts::{bootstrap_encoder_streets, encoder_from_lookup};
    use crate::{NlheAbstractionStreet, write_encoder_dir};

    #[test]
    fn benchmark_reports_sparse_encoder_failure_cleanly() {
        let directory = unique_artifact_dir();
        let observation = Observation::try_from("AcKh").expect("observation should parse");
        let streets = BTreeMap::from([(
            NlheAbstractionStreet::Preflop,
            BTreeMap::from([(Isomorphism::from(observation), Abstraction::from(42_i16))]),
        )]);
        write_encoder_dir(&directory, streets).expect("artifact dir should write");

        let error = benchmark_points_from_encoder_dir(&directory, &[1])
            .expect_err("sparse encoder should fail cleanly");
        let _ = fs::remove_dir_all(&directory);

        match error {
            PokerBenchmarkError::Solver(PokerSolverError::UpstreamPanic { operation, message }) => {
                assert_eq!(operation, "solver step");
                assert!(message.contains("isomorphism not found"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn reference_solver_self_matches_benchmark_pack() {
        let reference = bootstrap_reference_solver(merged_bootstrap_encoder())
            .expect("reference solver should build");
        let report =
            benchmark_against_bootstrap_reference(&reference).expect("self benchmark should work");

        assert_eq!(report.scenario_count, 80);
        assert_eq!(report.unique_query_count, 73);
        assert_eq!(report.exact_distribution_matches, 80);
        assert_eq!(report.exact_action_matches, 80);
        assert_eq!(report.mean_l1_distance, 0.0);
        assert_eq!(report.max_l1_distance, 0.0);
    }

    #[test]
    fn sparse_bootstrap_checkpoint_differs_from_reference_pack() {
        let candidate = PokerSolver::new(merged_bootstrap_encoder());
        let report = benchmark_against_bootstrap_reference(&candidate)
            .expect("reference benchmark should succeed");

        assert_eq!(report.scenario_count, 80);
        assert_eq!(report.unique_query_count, 73);
        assert!(report.mean_l1_distance > 0.0);
        assert!(report.max_l1_distance > 0.0);
        assert!(report.exact_distribution_matches < report.scenario_count);
        assert!(report.exact_action_matches < report.scenario_count);
    }

    fn merged_bootstrap_encoder() -> NlheEncoder {
        let lookup = bootstrap_encoder_streets()
            .into_values()
            .flat_map(|street| street.into_iter())
            .collect();

        encoder_from_lookup(lookup).expect("bootstrap lookup should build encoder")
    }

    fn unique_artifact_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("myosu-poker-benchmark-{nanos}"))
    }
}
