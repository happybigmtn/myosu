use std::env;
use std::error::Error;

use myosu_games_portfolio::{ALL_PORTFOLIO_ROUTED_GAMES, PortfolioSolver};

fn main() -> Result<(), Box<dyn Error>> {
    let iterations = env_usize("MYOSU_ENGINE_ITERATIONS", 16)?;
    let min_score = env_f64("MYOSU_ENGINE_MIN_SCORE", 1.01)?;
    let mut failures = 0usize;

    println!(
        "ENGINE_QUALITY_BUDGET total={} iterations={} min_score={min_score:.6}",
        ALL_PORTFOLIO_ROUTED_GAMES.len(),
        iterations,
    );

    for game in ALL_PORTFOLIO_ROUTED_GAMES {
        let mut solver = PortfolioSolver::for_game(game);
        solver.train(iterations);
        let query = PortfolioSolver::strength_query(game)?;
        let quality = solver.strength_quality(query)?;
        let status = if quality.engine_tier.as_str() != "static-baseline"
            && quality.score >= min_score
            && quality.legal_action_count > 0
        {
            "pass"
        } else {
            failures = failures.saturating_add(1);
            "fail"
        };

        println!(
            "ENGINE_QUALITY game={} status={status} engine_tier={} engine_family={} challenge_id={} score={:.6} min_score={min_score:.6} baseline_l1_distance={:.6} legal_actions={} baseline_action={} engine_action={}",
            game.slug(),
            quality.engine_tier.as_str(),
            shell_token(&quality.engine_family),
            quality.challenge_id,
            quality.score,
            quality.baseline_l1_distance,
            quality.legal_action_count,
            quality.baseline_action,
            quality.engine_action,
        );
    }

    if failures == 0 {
        println!("ENGINE_QUALITY_BUDGET status=pass failures=0");
        Ok(())
    } else {
        Err(format!("engine quality budget failed for {failures} games").into())
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
