use myosu_games_portfolio::ResearchGame;
use myosu_games_portfolio::core::{CORE_CANONICAL_TEN, apply_action, bootstrap_state};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game = std::env::args()
        .nth(1)
        .ok_or("usage: core_roundtrip <canonical-ten-game-slug>")?;
    let game = ResearchGame::parse(&game)?;
    if !CORE_CANONICAL_TEN.contains(&game) {
        return Err(format!(
            "game `{}` is not in the canonical-ten core set",
            game.slug()
        )
        .into());
    }

    let state = bootstrap_state(game)?;
    let action = state
        .legal_actions
        .first()
        .cloned()
        .ok_or_else(|| format!("game `{}` has no legal core action", game.slug()))?;
    let transition = apply_action(&state, &action.action_id, json!({}))?;
    let payoff = transition
        .after
        .payoff
        .as_ref()
        .map(|payoff| format!("{payoff:?}"))
        .unwrap_or_else(|| "none".to_string());
    println!(
        "CORE_ROUNDTRIP game={} action_id={} terminal={} payoff={} transition=ok",
        game.slug(),
        action.action_id,
        transition.after.terminal,
        payoff
    );

    Ok(())
}
