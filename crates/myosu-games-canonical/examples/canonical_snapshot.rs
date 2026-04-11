use std::env;
use std::io;

use myosu_games::canonical_hash;
use myosu_games_canonical::{
    canonical_bootstrap_snapshot, canonical_bootstrap_strategy_binding, is_canonical_ten,
};
use myosu_games_portfolio::ResearchGame;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let slug = env::args().nth(1).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: canonical_snapshot <canonical-game-slug>",
        )
    })?;
    let game = ResearchGame::parse(&slug)?;
    if !is_canonical_ten(game) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "game `{}` is not in the canonical-ten manifest",
                game.slug()
            ),
        )
        .into());
    }

    let snapshot = canonical_bootstrap_snapshot(game)?;
    let binding = canonical_bootstrap_strategy_binding(game)?;
    let state_hash = canonical_hash(&snapshot)?;

    println!(
        "CANONICAL_SNAPSHOT game={} trace_id={} state_hash={} actions={} query_hash={} response_hash={}",
        game.slug(),
        snapshot.trace_id,
        state_hash,
        snapshot.legal_actions.len(),
        binding.query_hash,
        binding.response_hash,
    );

    Ok(())
}
