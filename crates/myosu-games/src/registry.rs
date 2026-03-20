//! Game registry and exploitability metrics.
//!
//! This module provides per-game metric descriptors that enable cross-game
//! scoring normalization. Each game type registers its exploitability unit,
//! scale, and baseline values.

use serde::{Deserialize, Serialize};

use crate::traits::GameType;

/// How to interpret the raw exploitability number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExploitScale {
    /// Raw exploitability value. Lower is better. 0 = Nash.
    /// Used for small games (Liar's Dice, Stratego).
    Absolute,
    /// Milli-units per hand/round. Lower is better. 0 = Nash.
    /// Used for games with per-hand utility (poker, backgammon).
    MilliPerHand,
    /// Normalized to [0, 1] where 0 = Nash, 1 = random.
    /// Used when absolute scale varies too much across configs.
    Normalized,
}

/// Descriptor for a game's exploitability metric.
///
/// Contains all information needed to format, normalize, and compare
/// exploitability values across different games.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExploitMetric {
    /// Display unit string (e.g., "mbb/h", "mcpw", "exploit").
    pub unit: &'static str,
    /// How to interpret the raw number.
    pub scale: ExploitScale,
    /// Number of decimal places in lobby display.
    pub display_precision: u8,
    /// Exploitability of a uniform random strategy.
    pub random_baseline: f64,
    /// Below this value = "good" solver in lobby.
    pub good_threshold: f64,
}

/// Global game registry mapping game types to their exploitability metrics.
pub struct GameRegistry;

impl GameRegistry {
    /// Get the exploitability metric for a game type.
    pub fn metric(game_type: &GameType) -> Option<ExploitMetric> {
        match game_type {
            GameType::NlheHeadsUp => Some(ExploitMetric {
                unit: "mbb/h",
                scale: ExploitScale::MilliPerHand,
                display_precision: 1,
                random_baseline: 300.0,
                good_threshold: 15.0,
            }),
            GameType::NlheSixMax => Some(ExploitMetric {
                unit: "mbb/h",
                scale: ExploitScale::MilliPerHand,
                display_precision: 1,
                random_baseline: 400.0,
                good_threshold: 25.0,
            }),
            GameType::LiarsDice => Some(ExploitMetric {
                unit: "exploit",
                scale: ExploitScale::Absolute,
                display_precision: 3,
                random_baseline: 1.0,
                good_threshold: 0.01,
            }),
            GameType::Custom(_) => None,
        }
    }

    /// Get all registered metrics.
    pub fn all_metrics() -> Vec<ExploitMetric> {
        vec![
            // NLHE Heads-Up
            ExploitMetric {
                unit: "mbb/h",
                scale: ExploitScale::MilliPerHand,
                display_precision: 1,
                random_baseline: 300.0,
                good_threshold: 15.0,
            },
            // NLHE 6-max
            ExploitMetric {
                unit: "mbb/h",
                scale: ExploitScale::MilliPerHand,
                display_precision: 1,
                random_baseline: 400.0,
                good_threshold: 25.0,
            },
            // Liar's Dice
            ExploitMetric {
                unit: "exploit",
                scale: ExploitScale::Absolute,
                display_precision: 3,
                random_baseline: 1.0,
                good_threshold: 0.01,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_game_types_have_metrics() {
        // Known game types should have metrics
        assert!(GameRegistry::metric(&GameType::NlheHeadsUp).is_some());
        assert!(GameRegistry::metric(&GameType::NlheSixMax).is_some());
        assert!(GameRegistry::metric(&GameType::LiarsDice).is_some());

        // Custom game types don't have built-in metrics
        assert!(GameRegistry::metric(&GameType::Custom("test".to_string())).is_none());
    }

    #[test]
    fn random_baseline_positive() {
        let metrics = GameRegistry::all_metrics();
        for metric in metrics {
            assert!(
                metric.random_baseline > 0.0,
                "random_baseline must be positive, got {}",
                metric.random_baseline
            );
        }
    }

    #[test]
    fn good_threshold_less_than_baseline() {
        let metrics = GameRegistry::all_metrics();
        for metric in metrics {
            assert!(
                metric.good_threshold < metric.random_baseline,
                "good_threshold ({}) must be less than random_baseline ({})",
                metric.good_threshold,
                metric.random_baseline
            );
        }
    }

    #[test]
    fn liars_dice_metric_correct() {
        let metric = GameRegistry::metric(&GameType::LiarsDice).unwrap();
        assert_eq!(metric.unit, "exploit");
        assert_eq!(metric.scale, ExploitScale::Absolute);
        assert_eq!(metric.display_precision, 3);
        assert!((metric.random_baseline - 1.0).abs() < f64::EPSILON);
        assert!((metric.good_threshold - 0.01).abs() < f64::EPSILON);
    }

    #[test]
    fn nlhe_metrics_use_milli_per_hand() {
        let hu = GameRegistry::metric(&GameType::NlheHeadsUp).unwrap();
        assert_eq!(hu.unit, "mbb/h");
        assert_eq!(hu.scale, ExploitScale::MilliPerHand);

        let six_max = GameRegistry::metric(&GameType::NlheSixMax).unwrap();
        assert_eq!(six_max.unit, "mbb/h");
        assert_eq!(six_max.scale, ExploitScale::MilliPerHand);
    }
}
