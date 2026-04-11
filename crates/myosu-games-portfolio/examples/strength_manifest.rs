use std::env;

use myosu_games_portfolio::{
    ALL_PORTFOLIO_ROUTED_GAMES, PortfolioChallenge, ResearchGame, answer_typed_challenge,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = env::args().nth(1).unwrap_or_else(|| "table".to_string());

    match mode.as_str() {
        "slugs" => print_slugs(ALL_PORTFOLIO_ROUTED_GAMES),
        "table" => print_table()?,
        _ => {
            return Err(
                "usage: cargo run -p myosu-games-portfolio --example strength_manifest -- [table|slugs]"
                    .into(),
            );
        }
    }

    Ok(())
}

fn print_slugs(games: impl IntoIterator<Item = ResearchGame>) {
    for game in games {
        println!("{}", game.slug());
    }
}

fn print_table() -> Result<(), Box<dyn std::error::Error>> {
    println!("STRENGTH_GAMES total={}", ALL_PORTFOLIO_ROUTED_GAMES.len());
    for game in ALL_PORTFOLIO_ROUTED_GAMES {
        let challenge = PortfolioChallenge::bootstrap(game)
            .ok_or_else(|| format!("missing typed strength challenge for `{}`", game.slug()))?;
        let answer = answer_typed_challenge(&challenge, 0)?;
        println!(
            "STRENGTH_GAME slug={} chain_id={} challenge_id={} engine_tier={} engine_family={} legal_actions={} rule_file={}",
            game.slug(),
            game.chain_id(),
            challenge.spot().challenge_id,
            answer.engine_tier.as_str(),
            shell_token(&answer.engine_family),
            answer.legal_actions.len(),
            game.rule_file(),
        );
    }

    Ok(())
}

fn shell_token(value: &str) -> String {
    value.replace(' ', "_")
}
