use std::env;
use std::path::PathBuf;

use myosu_games_poker::benchmark_points_from_encoder_dir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let Some(encoder_dir) = args.next() else {
        return Err(usage().into());
    };

    let mut iterations = Vec::new();
    for value in args {
        iterations.push(value.parse::<usize>()?);
    }
    if iterations.is_empty() {
        return Err("expected at least one iteration count".into());
    }

    let encoder_dir = PathBuf::from(encoder_dir);
    let points = benchmark_points_from_encoder_dir(&encoder_dir, &iterations)?;

    println!("POKER_BENCHMARK encoder_dir={}", encoder_dir.display());
    for point in points {
        println!(
            "POKER_BENCHMARK iterations={} exploitability={:.6}",
            point.iterations, point.exploitability
        );
    }

    Ok(())
}

fn usage() -> &'static str {
    "usage: cargo run -p myosu-games-poker --example quality_benchmark -- <encoder-dir> <iterations...>"
}
