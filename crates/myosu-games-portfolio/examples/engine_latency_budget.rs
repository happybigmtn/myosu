use std::env;
use std::error::Error;
use std::time::Instant;

use myosu_games_portfolio::{ALL_PORTFOLIO_ROUTED_GAMES, PortfolioSolver};

fn main() -> Result<(), Box<dyn Error>> {
    let iterations = env_usize("MYOSU_ENGINE_ITERATIONS", 16)?;
    let budget_ms = env_f64("MYOSU_ENGINE_BUDGET_MS", 50.0)?;
    let mut failures = 0usize;

    println!(
        "ENGINE_LATENCY_BUDGET total={} iterations={} budget_ms={budget_ms:.3}",
        ALL_PORTFOLIO_ROUTED_GAMES.len(),
        iterations,
    );

    for game in ALL_PORTFOLIO_ROUTED_GAMES {
        let mut solver = PortfolioSolver::for_game(game);
        solver.train(iterations);
        let query = PortfolioSolver::strength_query(game)?;
        let started_at = Instant::now();
        let response = solver.answer_strength_checked(query.clone())?;
        let quality = solver.strength_quality(query)?;
        let elapsed_ms = started_at.elapsed().as_secs_f64() * 1_000.0;
        let status = if response.is_valid() && elapsed_ms <= budget_ms {
            "pass"
        } else {
            failures = failures.saturating_add(1);
            "fail"
        };

        println!(
            "ENGINE_LATENCY game={} status={status} engine_tier={} engine_family={} challenge_id={} elapsed_ms={elapsed_ms:.3} budget_ms={budget_ms:.3} action_count={}",
            game.slug(),
            quality.engine_tier.as_str(),
            shell_token(&quality.engine_family),
            quality.challenge_id,
            response.actions.len(),
        );
    }

    if failures == 0 {
        println!("ENGINE_LATENCY_BUDGET status=pass failures=0");
        Ok(())
    } else {
        Err(format!("engine latency budget failed for {failures} games").into())
    }
}

fn env_usize(name: &str, default: usize) -> Result<usize, Box<dyn Error>> {
    match env::var(name) {
        Ok(value) => Ok(value.parse::<usize>()?),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(error.into()),
    }
}

fn env_f64(name: &str, default: f64) -> Result<f64, Box<dyn Error>> {
    match env::var(name) {
        Ok(value) => Ok(value.parse::<f64>()?),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(error.into()),
    }
}

fn shell_token(value: &str) -> String {
    value.replace(' ', "_")
}
