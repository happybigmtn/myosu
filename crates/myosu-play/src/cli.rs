use std::io;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Executable game surfaces supported by `myosu-play`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum GameSelection {
    Poker,
    LiarsDice,
}

/// CLI entrypoint for the Myosu play surface.
#[derive(Parser, Debug)]
#[command(name = "myosu-play")]
#[command(about = "Interactive TUI and pipe-mode demos for Myosu games")]
pub(crate) struct Cli {
    /// Game surface to launch.
    #[arg(long, value_enum, default_value_t = GameSelection::Poker)]
    pub(crate) game: GameSelection,

    /// Run the deterministic smoke proof instead of entering an interactive mode.
    #[arg(long)]
    pub(crate) smoke_test: bool,

    /// Require the smoke proof to use artifact-backed advice instead of fallback generation.
    #[arg(long)]
    pub(crate) require_artifact: bool,

    /// Require the smoke proof to discover one chain-visible miner.
    #[arg(long)]
    pub(crate) require_discovery: bool,

    /// Require the smoke proof to execute one live miner health and strategy query.
    #[arg(long)]
    pub(crate) require_live_query: bool,

    /// Optional checkpoint used only by `--smoke-test`.
    #[arg(long = "smoke-checkpoint")]
    pub(crate) smoke_checkpoint: Option<PathBuf>,

    /// Optional encoder directory paired with `--smoke-checkpoint`.
    #[arg(long = "smoke-encoder-dir")]
    pub(crate) smoke_encoder_dir: Option<PathBuf>,

    /// Optional chain WebSocket endpoint used for miner discovery.
    #[arg(long)]
    pub(crate) chain: Option<String>,

    /// Optional subnet id used for miner discovery.
    #[arg(long)]
    pub(crate) subnet: Option<u16>,

    #[command(subcommand)]
    pub(crate) command: Option<Mode>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Mode {
    /// Run the interactive TUI against a generated demo hand.
    Train(AdviceArgs),

    /// Emit plain-text pipe output and handle a single input line.
    Pipe(AdviceArgs),
}

#[derive(Args, Debug, Clone)]
pub(crate) struct AdviceArgs {
    /// Optional MCCFR checkpoint to use for live advice.
    #[arg(long)]
    pub(crate) checkpoint: Option<PathBuf>,

    /// Optional manifest-backed encoder directory paired with --checkpoint.
    #[arg(long = "encoder-dir")]
    pub(crate) encoder_dir: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub(crate) struct DiscoveryRequest {
    pub(crate) chain: Option<String>,
    pub(crate) subnet: Option<u16>,
}

impl DiscoveryRequest {
    pub(crate) fn from_cli(cli: &Cli) -> io::Result<Self> {
        if cli.game == GameSelection::LiarsDice && cli.chain.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--game liars-dice does not support --chain/--subnet miner discovery yet",
            ));
        }
        if cli.chain.is_some() != cli.subnet.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--chain requires --subnet and --subnet requires --chain",
            ));
        }
        if cli.require_discovery && cli.chain.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--require-discovery requires --chain and --subnet",
            ));
        }
        if cli.require_live_query && cli.chain.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--require-live-query requires --chain and --subnet",
            ));
        }
        Ok(Self {
            chain: cli.chain.clone(),
            subnet: cli.subnet,
        })
    }

    pub(crate) const fn is_requested(&self) -> bool {
        self.chain.is_some()
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::{Cli, GameSelection};

    #[test]
    fn cli_parses_game_selection() {
        let cli = Cli::parse_from(["myosu-play", "--game", "liars-dice", "pipe"]);

        assert_eq!(cli.game, GameSelection::LiarsDice);
    }

    #[test]
    fn discovery_rejects_chain_flags_for_liars_dice() {
        let cli = Cli::parse_from([
            "myosu-play",
            "--game",
            "liars-dice",
            "--chain",
            "ws://127.0.0.1:9944",
            "--subnet",
            "7",
            "pipe",
        ]);

        let error = crate::cli::DiscoveryRequest::from_cli(&cli)
            .expect_err("liar's dice should reject discovery flags");
        assert!(
            error
                .to_string()
                .contains("does not support --chain/--subnet")
        );
    }
}
