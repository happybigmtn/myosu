//! Liar's Dice encoder — `Encoder` implementation.
//!
//! The encoder maps information states to indices for CFR table storage.
//! For Liar's Dice, this is trivial: direct enumeration of (die × bid_history).

/// LiarsDiceEncoder maps info states to table indices.
///
/// State space: 6 dice faces × ~2,048 bid history patterns ≈ 24,576 info sets.
/// This is small enough that a simple enumeration scheme works efficiently.
#[derive(Clone, Debug)]
pub struct LiarsDiceEncoder;

impl LiarsDiceEncoder {
    /// Create a new encoder.
    pub fn new() -> Self {
        Self
    }

    /// Encode an info set to an index.
    pub fn encode(&self, _info: &super::info::LiarsDiceInfo) -> usize {
        todo!("Slice 3: implement LiarsDiceEncoder::encode()")
    }

    /// Decode an index back to an info set.
    pub fn decode(&self, _index: usize) -> super::info::LiarsDiceInfo {
        todo!("Slice 3: implement LiarsDiceEncoder::decode()")
    }

    /// Return the total number of information sets.
    pub fn num_info_sets(&self) -> usize {
        todo!("Slice 3: implement LiarsDiceEncoder::num_info_sets()")
    }
}
