use std::env;

use myosu_games_portfolio::{ALL_RESEARCH_GAMES, ResearchGame};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = env::args().nth(1).unwrap_or_else(|| "table".to_string());

    match mode.as_str() {
        "all-slugs" => print_slugs(ALL_RESEARCH_GAMES),
        "portfolio-slugs" => print_slugs(portfolio_games()),
        "dedicated-slugs" => print_slugs(dedicated_games()),
        "table" => print_table(),
        _ => {
            return Err(
                "usage: cargo run -p myosu-games-portfolio --example bootstrap_manifest -- [table|all-slugs|portfolio-slugs|dedicated-slugs]"
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

fn print_table() {
    println!("RESEARCH_GAMES total={}", ALL_RESEARCH_GAMES.len());
    for game in ALL_RESEARCH_GAMES {
        let route = if game.is_portfolio_routed() {
            "portfolio"
        } else {
            "dedicated"
        };
        println!(
            "RESEARCH_GAME slug={} chain_id={} route={} players={} rule_file={} solver_family={}",
            game.slug(),
            game.chain_id(),
            route,
            game.default_players(),
            game.rule_file(),
            shell_token(game.solver_family()),
        );
    }
}

fn portfolio_games() -> impl Iterator<Item = ResearchGame> {
    ALL_RESEARCH_GAMES
        .into_iter()
        .filter(|game| game.is_portfolio_routed())
}

fn dedicated_games() -> impl Iterator<Item = ResearchGame> {
    ALL_RESEARCH_GAMES
        .into_iter()
        .filter(|game| !game.is_portfolio_routed())
}

fn shell_token(value: &str) -> String {
    value.replace(' ', "_")
}
