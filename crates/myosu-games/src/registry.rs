//! Game registry and exploitability metrics.
//!
//! Central registration of `ExploitMetric` descriptors for each `GameType`.
//! The validator oracle uses these to normalize game-specific exploitability
//! into comparable u16 consensus weights.

use crate::traits::{ExploitMetric, ExploitScale};

/// All registered exploitability metrics, keyed by game type.
pub struct ExploitMetricRegistry {
    pub nlhe_heads_up: ExploitMetric,
    pub nlhe_six_max: ExploitMetric,
    pub liars_dice: ExploitMetric,
}

impl ExploitMetricRegistry {
    const fn new() -> Self {
        // NLHE heads-up: exploitability in milli-big-blinds per hand.
        // Baseline: ~500 mbb/h for random, "good" = <50 mbb/h.
        let nlhe_hu = ExploitMetric {
            unit: "mbb/h",
            scale: ExploitScale::MilliPerHand,
            display_precision: 1,
            random_baseline: 500.0,
            good_threshold: 50.0,
        };
        // NLHE 6-max: similar scale but slightly higher baseline due to more players.
        let nlhe_6max = ExploitMetric {
            unit: "mbb/h",
            scale: ExploitScale::MilliPerHand,
            display_precision: 1,
            random_baseline: 600.0,
            good_threshold: 60.0,
        };
        // Liar's Dice: absolute exploitability in game payoff units.
        // The game payoff is ±1 per episode. Random strategy gets ~0.5 exploitability.
        // "Good" means near-zero exploitability after training.
        let liars_dice = ExploitMetric {
            unit: "exploit",
            scale: ExploitScale::Absolute,
            display_precision: 4,
            random_baseline: 1.0,
            good_threshold: 0.01,
        };
        Self {
            nlhe_heads_up: nlhe_hu,
            nlhe_six_max: nlhe_6max,
            liars_dice,
        }
    }
}

/// Static registry of all known exploitability metrics.
static EXPLOIT_METRICS: ExploitMetricRegistry = ExploitMetricRegistry::new();

/// Access the static metric registry.
pub fn all_metrics() -> &'static ExploitMetricRegistry {
    &EXPLOIT_METRICS
}

/// Compute a normalized weight from raw exploitability using the given metric.
///
/// The normalization formula is:
/// `weight = (1.0 - min(1.0, exploit / baseline)) * u16::MAX`
///
/// This maps random strategy → 0, Nash equilibrium → u16::MAX.
pub fn compute_weight(exploit: f64, metric: &ExploitMetric) -> u16 {
    let normalized = (1.0 - (exploit / metric.random_baseline).min(1.0)).clamp(0.0, 1.0);
    (normalized * u16::MAX as f64) as u16
}

/// Format an exploitability value for lobby display.
pub fn format_exploit(exploit: f64, metric: &ExploitMetric) -> String {
    if exploit < 1e-9 {
        return "solved".to_string();
    }
    format!(
        "{:.prec$} {}",
        exploit,
        metric.unit,
        prec = metric.display_precision as usize
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::GameType;

    #[test]
    fn all_game_types_have_metrics() {
        assert!(GameType::NlheHeadsUp.exploit_metric().is_some());
        assert!(GameType::NlheSixMax.exploit_metric().is_some());
        assert!(GameType::LiarsDice.exploit_metric().is_some());
    }

    #[test]
    fn random_baseline_positive() {
        for variant in [
            GameType::NlheHeadsUp,
            GameType::NlheSixMax,
            GameType::LiarsDice,
        ] {
            let metric = variant.exploit_metric().unwrap();
            assert!(metric.random_baseline > 0.0);
        }
    }

    #[test]
    fn good_threshold_less_than_baseline() {
        for variant in [
            GameType::NlheHeadsUp,
            GameType::NlheSixMax,
            GameType::LiarsDice,
        ] {
            let metric = variant.exploit_metric().unwrap();
            assert!(metric.good_threshold < metric.random_baseline);
        }
    }

    #[test]
    fn weight_zero_for_random_strategy() {
        for variant in [
            GameType::NlheHeadsUp,
            GameType::NlheSixMax,
            GameType::LiarsDice,
        ] {
            let metric = variant.exploit_metric().unwrap();
            let weight = compute_weight(metric.random_baseline, metric);
            assert_eq!(weight, 0);
        }
    }

    #[test]
    fn weight_max_for_nash_strategy() {
        for variant in [
            GameType::NlheHeadsUp,
            GameType::NlheSixMax,
            GameType::LiarsDice,
        ] {
            let metric = variant.exploit_metric().unwrap();
            let weight = compute_weight(0.0, metric);
            assert_eq!(weight, u16::MAX);
        }
    }

    #[test]
    fn weight_scales_linearly() {
        let metric = GameType::LiarsDice.exploit_metric().unwrap();
        // At baseline/2, we should get approximately half of MAX
        let weight = compute_weight(metric.random_baseline / 2.0, metric);
        let expected = u16::MAX / 2;
        // Allow ±1% tolerance
        let tolerance = u16::MAX as f64 * 0.01;
        assert!((weight as f64 - expected as f64).abs() < tolerance);
    }
}
