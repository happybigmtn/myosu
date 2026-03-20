//! `myosu-play` — Interactive NLHE poker gameplay binary.
//!
//! Entry point for the myosu poker gameplay experience:
//! - `myosu-play --train` — local practice against heuristic/blueprint bot
//! - `myosu-play --chain ws://...` — miner-connected play once chain integration lands
//! - `myosu-play --pipe` — plain-text agent protocol once pipe wiring lands

use clap::Parser;
use myosu_tui::shell::Shell;
use std::time::Duration;
use training::{HeuristicBackend, TrainingTable, bot_delay_from_env};
use tracing::info;
use tracing_subscriber::EnvFilter;

mod training;

#[derive(Parser, Debug)]
#[command(name = "myosu-play")]
#[command(about = "Interactive NLHE poker gameplay for myosu")]
struct Args {
    /// Run in training mode (local, no chain required)
    #[arg(long)]
    train: bool,

    /// Connect to a miner via WebSocket URL (future)
    #[arg(long)]
    chain: Option<String>,

    /// Run in pipe mode (plain-text agent protocol)
    #[arg(long)]
    pipe: bool,

    /// Bot thinking delay in milliseconds (0 to disable)
    #[arg(long, default_value = "300")]
    bot_delay_ms: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let args = Args::parse();

    if args.chain.is_some() {
        anyhow::bail!("--chain mode is pending chain integration work (Slice 7)");
    }

    if args.pipe {
        anyhow::bail!("--pipe mode is pending pipe integration work");
    }

    if !args.train {
        anyhow::bail!("myosu-play currently expects --train. Run: myosu-play --train");
    }

    run_training_mode(args.bot_delay_ms).await
}

/// Run training mode: heads-up NLHE practice against a local bot.
async fn run_training_mode(bot_delay_ms: u64) -> anyhow::Result<()> {
    info!("Starting training mode");

    let bot_delay_ms = bot_delay_from_env(bot_delay_ms);
    let backend = std::sync::Arc::new(HeuristicBackend);
    let mut table = TrainingTable::with_backend_and_delay(backend, bot_delay_ms);
    table.advance_until_hero_or_terminal().await?;
    let renderer = table.renderer();

    let mut shell = Shell::new();
    shell.log(format!("~ {}", table.strategy_status()));
    shell.log(format!("practice chips {}", table.practice_chips()));

    shell.update_completions(&renderer);

    let tick_rate = Duration::from_millis(16);

    shell.run(&renderer, tick_rate).await?;

    info!("Training session ended");
    Ok(())
}
