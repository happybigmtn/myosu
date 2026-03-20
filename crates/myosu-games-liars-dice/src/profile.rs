//! Liar's Dice profile implementation.

use myosu_games::{Profile, Probability, Utility};
use rbp_core::{Entropy, Energy};
use rbp_mccfr::{CfrEdge, CfrGame, CfrInfo, CfrTurn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::edge::LiarsDiceEdge;
use crate::game::LiarsDiceGame;
use crate::info::LiarsDiceInfo;
use crate::turn::LiarsDiceTurn;

/// Liar's Dice CFR profile for training and strategy storage.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiarsDiceProfile {
    /// Accumulated encounter counts: info -> edge -> count
    encounters: HashMap<u64, HashMap<u64, u32>>,
    /// Accumulated regrets: info -> edge -> regret
    regrets: HashMap<u64, HashMap<u64, f32>>,
    /// Accumulated weights: info -> edge -> weight
    weights: HashMap<u64, HashMap<u64, f32>>,
    /// Accumulated expected values: info -> edge -> evalue
    evalues: HashMap<u64, HashMap<u64, f32>>,
    /// Number of training iterations completed
    epochs: usize,
}

impl LiarsDiceProfile {
    pub fn new() -> Self {
        Self {
            encounters: HashMap::new(),
            regrets: HashMap::new(),
            weights: HashMap::new(),
            evalues: HashMap::new(),
            epochs: 0,
        }
    }

    pub fn epochs_count(&self) -> usize {
        self.epochs
    }
}

impl Default for LiarsDiceProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple hash for LiarsDiceInfo
#[allow(clippy::derive_hash_xor_equal)]
fn hash_info(info: &LiarsDiceInfo) -> u64 {
    let mut h = u64::from(info.public.num_bids);
    h = h.wrapping_mul(31).wrapping_add(u64::from(info.secret.my_die));
    for (i, bid) in info.public.bid_history.iter().enumerate() {
        if i < info.public.num_bids as usize {
            h = h.wrapping_mul(17).wrapping_add(u64::from(bid.quantity));
            h = h.wrapping_mul(31).wrapping_add(u64::from(bid.face));
        }
    }
    h
}

/// Simple hash for LiarsDiceEdge
fn hash_edge(edge: &LiarsDiceEdge) -> u64 {
    match edge {
        LiarsDiceEdge::Bid(b) => (u64::from(b.quantity) << 8) | u64::from(b.face),
        LiarsDiceEdge::Challenge => 0xFFFF,
    }
}

impl Profile for LiarsDiceProfile {
    type T = LiarsDiceTurn;
    type E = LiarsDiceEdge;
    type G = LiarsDiceGame;
    type I = LiarsDiceInfo;

    fn increment(&mut self) {
        self.epochs += 1;
    }

    fn epochs(&self) -> usize {
        self.epochs
    }

    fn walker(&self) -> Self::T {
        match self.epochs % 2 {
            0 => LiarsDiceTurn::Player0,
            _ => LiarsDiceTurn::Player1,
        }
    }

    fn cum_weight(&self, info: &Self::I, edge: &Self::E) -> Probability {
        let info_h = hash_info(info);
        let edge_h = hash_edge(edge);
        self.weights
            .get(&info_h)
            .and_then(|m| m.get(&edge_h))
            .copied()
            .unwrap_or(1.0)
    }

    fn cum_regret(&self, info: &Self::I, edge: &Self::E) -> Utility {
        let info_h = hash_info(info);
        let edge_h = hash_edge(edge);
        self.regrets
            .get(&info_h)
            .and_then(|m| m.get(&edge_h))
            .copied()
            .unwrap_or(0.0)
    }

    fn cum_evalue(&self, info: &Self::I, edge: &Self::E) -> Utility {
        let info_h = hash_info(info);
        let edge_h = hash_edge(edge);
        self.evalues
            .get(&info_h)
            .and_then(|m| m.get(&edge_h))
            .copied()
            .unwrap_or(0.0)
    }

    fn cum_counts(&self, info: &Self::I, edge: &Self::E) -> u32 {
        let info_h = hash_info(info);
        let edge_h = hash_edge(edge);
        self.encounters
            .get(&info_h)
            .and_then(|m| m.get(&edge_h))
            .copied()
            .unwrap_or(0)
    }

    fn temperature(&self) -> Entropy {
        1.0
    }

    fn smoothing(&self) -> Energy {
        0.0
    }

    fn curiosity(&self) -> Probability {
        0.01
    }
}
