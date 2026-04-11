use std::env;
use std::fs;
use std::path::PathBuf;

use myosu_games_portfolio::{PortfolioSolver, ResearchGame};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let Some(game_slug) = args.next() else {
        return Err(
            "usage: cargo run -p myosu-games-portfolio --example bootstrap_checkpoint -- <game> <checkpoint-file> [iterations]"
                .into(),
        );
    };
    let Some(checkpoint_file) = args.next() else {
        return Err("missing <checkpoint-file>".into());
    };
    let iterations = match args.next() {
        Some(raw) => raw.parse::<usize>()?,
        None => 0,
    };
    if args.next().is_some() {
        return Err("expected at most three positional arguments".into());
    }

    let game = ResearchGame::parse(&game_slug)?;
    if !game.is_portfolio_routed() {
        return Err(format!(
            "`{}` has a dedicated solver checkpoint format; use the dedicated solver surface",
            game.slug()
        )
        .into());
    }

    let checkpoint_file = PathBuf::from(checkpoint_file);
    let checkpoint_parent = checkpoint_file
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&checkpoint_parent)?;

    let mut solver = PortfolioSolver::for_game(game);
    solver.train(iterations);
    solver.save(&checkpoint_file)?;

    println!("BOOTSTRAP game={}", game.slug());
    println!("BOOTSTRAP checkpoint_file={}", checkpoint_file.display());
    println!("BOOTSTRAP iterations={}", solver.epochs());

    Ok(())
}
