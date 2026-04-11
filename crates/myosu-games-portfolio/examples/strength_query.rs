use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use myosu_games_portfolio::{PortfolioSolver, ResearchGame, encode_strength_query};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let game_arg = required_arg(&mut args, "<game>")?;
    let query_file = PathBuf::from(required_arg(&mut args, "<query-file>")?);
    if let Some(extra) = args.next() {
        return Err(format!("unexpected extra argument `{extra}`").into());
    }

    let game = ResearchGame::parse(&game_arg)?;
    let query = PortfolioSolver::strength_query(game)?;
    let query_parent = query_file
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&query_parent)?;
    fs::write(&query_file, encode_strength_query(&query)?)?;

    println!("STRENGTH game={}", game.slug());
    println!("STRENGTH query_file={}", query_file.display());
    println!(
        "STRENGTH challenge_id={}",
        query.info.challenge.spot().challenge_id
    );

    Ok(())
}

fn required_arg(
    args: &mut impl Iterator<Item = String>,
    label: &'static str,
) -> Result<String, Box<dyn Error>> {
    args.next()
        .ok_or_else(|| format!("missing required argument {label}").into())
}
