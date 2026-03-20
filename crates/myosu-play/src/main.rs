//! `myosu-play` — Interactive NLHE poker gameplay binary.
//!
//! Entry point for the myosu poker gameplay experience:
//! - `myosu-play --train` — local practice against heuristic/blueprint bot
//! - `myosu-play --chain ws://...` — miner-connected play (future slice)
//! - `myosu-play --pipe` — plain-text agent protocol

use clap::Parser;
use myosu_games_poker::NlheRenderer;
use myosu_tui::shell::Shell;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::EnvFilter;

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
        anyhow::bail!("--chain mode is not yet implemented (Slice 7)");
    }

    if args.pipe {
        anyhow::bail!("--pipe mode is not yet implemented (future slice)");
    }

    if !args.train {
        anyhow::bail!("myosu-play requires --train flag for now. Run: myosu-play --train");
    }

    run_training_mode(args.bot_delay_ms).await
}

/// Run training mode: heads-up NLHE practice against a local bot.
async fn run_training_mode(_bot_delay_ms: u64) -> anyhow::Result<()> {
    info!("Starting training mode");

    // Create the poker renderer with initial preflop state
    let renderer = NlheRenderer::preflop(1);

    // Create and configure the shell
    let mut shell = Shell::new();

    // Update completions from the renderer
    shell.update_completions(&renderer);

    // Use a tick rate of ~60fps
    let tick_rate = Duration::from_millis(16);

    // Run the shell event loop
    // Note: in a full implementation, this would be wrapped in a ratatui Terminal
    // For Slice 1, we verify the binary builds by running the shell with the renderer
    shell.run(&renderer, tick_rate).await?;

    info!("Training session ended");
    Ok(())
}
