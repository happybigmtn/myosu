use crate::game::ResearchGame;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Deterministically derive a seed from a game, challenge, epoch, and caller seed.
pub fn seed_for(game: ResearchGame, challenge_id: &str, epochs: usize, caller_seed: u64) -> u64 {
    let epochs = u64::try_from(epochs).unwrap_or(u64::MAX);
    let mut hash = FNV_OFFSET;
    hash = hash_bytes(hash, game.chain_id().as_bytes());
    hash = hash_bytes(hash, b":");
    hash = hash_bytes(hash, challenge_id.as_bytes());
    hash = hash_bytes(hash, b":");
    hash = hash_bytes(hash, &epochs.to_le_bytes());
    hash = hash_bytes(hash, b":");
    hash_bytes(hash, &caller_seed.to_le_bytes())
}

fn hash_bytes(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use crate::game::ResearchGame;
    use crate::rng::seed_for;

    #[test]
    fn seed_derivation_is_stable_and_game_specific() {
        let bridge = seed_for(ResearchGame::Bridge, "bridge:opening-lead", 3, 17);
        let bridge_again = seed_for(ResearchGame::Bridge, "bridge:opening-lead", 3, 17);
        let cribbage = seed_for(ResearchGame::Cribbage, "bridge:opening-lead", 3, 17);

        assert_eq!(bridge, bridge_again);
        assert_ne!(bridge, cribbage);
    }
}
