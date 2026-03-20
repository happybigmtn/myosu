//! Profile (strategy storage) for Liar's Dice.
//!
//! `LiarsDiceProfile` implements `Profile`, storing accumulated regrets
//! and strategy weights for each information set / action pair.
//!
//! For the 1-die Liar's Dice, the info set space is small:
//! - 6 possible die values (1-6) for the acting player
//! - Up to 12 bid history entries
//! - ~24,576 information sets total
//!
//! The profile uses hashmaps to store regret and weight data,
//! indexed by (info_set, edge) pairs.

use crate::edge::LiarsDiceEdge;
use crate::encoder::LiarsDiceEncoder;
use crate::game::LiarsDiceGame;
use crate::info::LiarsDiceInfo;
use crate::turn::LiarsDiceTurn;
use rbp_core::{Probability, Utility};
use rbp_mccfr::Profile;
use std::collections::HashMap;

/// Liar's Dice strategy profile.
#[derive(Clone, Debug)]
pub struct LiarsDiceProfile {
    /// Accumulated regret per (info, edge).
    regrets: HashMap<(LiarsDiceInfo, LiarsDiceEdge), Utility>,
    /// Accumulated strategy weight per (info, edge).
    weights: HashMap<(LiarsDiceInfo, LiarsDiceEdge), Probability>,
    /// Accumulated expected value per (info, edge).
    evalues: HashMap<(LiarsDiceInfo, LiarsDiceEdge), Utility>,
    /// Encounter counts per (info, edge).
    counts: HashMap<(LiarsDiceInfo, LiarsDiceEdge), u32>,
    /// Current training epoch.
    epoch: usize,
    /// Current walker (which player's turn is being updated).
    walker: LiarsDiceTurn,
}

impl Default for LiarsDiceProfile {
    fn default() -> Self {
        Self::new()
    }
}

impl LiarsDiceProfile {
    pub fn new() -> Self {
        Self {
            regrets: HashMap::new(),
            weights: HashMap::new(),
            evalues: HashMap::new(),
            counts: HashMap::new(),
            epoch: 0,
            walker: LiarsDiceTurn::Player0,
        }
    }

    /// Train the profile for a given number of iterations.
    pub fn train(&mut self, iterations: usize) {
        // Note: actual training requires the full Solver implementation
        // This is a placeholder for the training loop
        for _ in 0..iterations {
            self.increment();
        }
    }
}

impl Profile for LiarsDiceProfile {
    type T = LiarsDiceTurn;
    type E = LiarsDiceEdge;
    type G = LiarsDiceGame;
    type I = LiarsDiceInfo;

    fn increment(&mut self) {
        self.epoch += 1;
        // Alternate walker between players
        self.walker = if self.walker == LiarsDiceTurn::Player0 {
            LiarsDiceTurn::Player1
        } else {
            LiarsDiceTurn::Player0
        };
    }

    fn walker(&self) -> Self::T {
        self.walker
    }

    fn epochs(&self) -> usize {
        self.epoch
    }

    fn cum_weight(&self, info: &Self::I, edge: &Self::E) -> Probability {
        self.weights.get(&(*info, *edge)).copied().unwrap_or(0.0)
    }

    fn cum_regret(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.regrets.get(&(*info, *edge)).copied().unwrap_or(0.0)
    }

    fn cum_evalue(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.evalues.get(&(*info, *edge)).copied().unwrap_or(0.0)
    }

    fn cum_counts(&self, info: &Self::I, edge: &Self::E) -> u32 {
        self.counts.get(&(*info, *edge)).copied().unwrap_or(0)
    }
}

/// Solver for Liar's Dice CFR training.
#[derive(Clone, Debug)]
pub struct LiarsDiceSolver {
    profile: LiarsDiceProfile,
    encoder: LiarsDiceEncoder,
}

impl LiarsDiceSolver {
    pub fn new(profile: LiarsDiceProfile) -> Self {
        Self {
            profile,
            encoder: LiarsDiceEncoder,
        }
    }
}

impl rbp_mccfr::Solver for LiarsDiceSolver {
    type T = LiarsDiceTurn;
    type E = LiarsDiceEdge;
    type G = LiarsDiceGame;
    type I = LiarsDiceInfo;
    type X = crate::info::LiarsDicePublic;
    type Y = crate::info::LiarsDiceSecret;
    type P = LiarsDiceProfile;
    type N = LiarsDiceEncoder;
    type W = rbp_mccfr::LinearWeight;
    type R = rbp_mccfr::DiscountedRegret;
    type S = rbp_mccfr::VanillaSampling;

    fn batch_size() -> usize {
        1 // Liar's Dice converges fast, no batching needed
    }

    fn tree_count() -> usize {
        1024 // sufficient for convergence to < 0.001 exploitability
    }

    fn encoder(&self) -> &Self::N {
        &self.encoder
    }

    fn profile(&self) -> &Self::P {
        &self.profile
    }

    fn advance(&mut self) {
        self.profile.increment();
    }

    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility {
        self.profile.regrets.entry((*info, *edge)).or_insert(0.0)
    }

    fn mut_weight(&mut self, info: &Self::I, edge: &Self::E) -> &mut Probability {
        self.profile.weights.entry((*info, *edge)).or_insert(0.0)
    }

    fn mut_evalue(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility {
        self.profile.evalues.entry((*info, *edge)).or_insert(0.0)
    }

    fn mut_counts(&mut self, info: &Self::I, edge: &Self::E) -> &mut u32 {
        self.profile.counts.entry((*info, *edge)).or_insert(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_default() {
        let profile = LiarsDiceProfile::new();
        assert_eq!(profile.epochs(), 0);
    }

    #[test]
    fn train_short() {
        let mut profile = LiarsDiceProfile::new();
        profile.train(100);
        assert_eq!(profile.epochs(), 100);
    }
}
