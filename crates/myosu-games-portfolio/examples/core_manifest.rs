use myosu_games_portfolio::core::{CORE_CANONICAL_TEN, bootstrap_state};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for game in CORE_CANONICAL_TEN {
        let state = bootstrap_state(game)?;
        let actor = state
            .actor
            .map(|actor| actor.to_string())
            .unwrap_or_else(|| "none".to_string());
        println!(
            "CORE_GAME slug={} phase={} actor={} actions={} state=ok",
            game.slug(),
            state.phase,
            actor,
            state.legal_actions.len()
        );
    }

    Ok(())
}
