use std::env;
use std::fs;
use std::path::PathBuf;

use myosu_games_portfolio::{PortfolioSolver, ResearchGame, encode_strategy_query};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let Some(game_slug) = args.next() else {
        return Err(
            "usage: cargo run -p myosu-games-portfolio --example bootstrap_query -- <game> <query-file>"
                .into(),
        );
    };
    let Some(query_file) = args.next() else {
        return Err("missing <query-file>".into());
    };
    if args.next().is_some() {
        return Err("expected exactly two positional arguments".into());
    }

    let game = ResearchGame::parse(&game_slug)?;
    if !game.is_portfolio_routed() {
        return Err(format!(
            "`{}` has a dedicated solver wire format; use the dedicated bootstrap query surface",
            game.slug()
        )
        .into());
    }

    let query_file = PathBuf::from(query_file);
    let query = PortfolioSolver::bootstrap_query(game);
    let query_bytes = encode_strategy_query(&query)?;
    let query_parent = query_file
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&query_parent)?;
    fs::write(&query_file, query_bytes)?;

    println!("BOOTSTRAP game={}", game.slug());
    println!("BOOTSTRAP query_file={}", query_file.display());

    Ok(())
}
