use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Executable stage-0 games supported by the miner bootstrap flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum GameSelection {
    Poker,
    LiarsDice,
}

/// Stage-0 bootstrap CLI for the Myosu miner process.
#[derive(Debug, Parser, Clone)]
#[command(name = "myosu-miner")]
#[command(about = "Bootstrap miner process for the Myosu stage-0 chain")]
pub struct Cli {
    /// Chain WebSocket RPC endpoint.
    #[arg(long, default_value = "ws://127.0.0.1:9944")]
    pub chain: String,

    /// Subnet to probe and later register on.
    #[arg(long)]
    pub subnet: u16,

    /// Hotkey seed or URI the operator intends to use for this miner.
    #[arg(long)]
    pub key: String,

    /// Planned HTTP port for the future miner axon server.
    #[arg(long, default_value_t = 8080)]
    pub port: u16,

    /// Register this miner hotkey on-chain before local work begins.
    #[arg(long, default_value_t = false)]
    pub register: bool,

    /// Publish the miner's planned axon endpoint on-chain.
    #[arg(long, default_value_t = false)]
    pub serve_axon: bool,

    /// Serve live HTTP strategy requests from the current checkpoint.
    #[arg(long, default_value_t = false)]
    pub serve_http: bool,

    /// Data directory reserved for future checkpoints and artifacts.
    #[arg(long, default_value = "./miner-data")]
    pub data_dir: PathBuf,

    /// Game contract used for local training and bounded strategy serving.
    #[arg(long, value_enum, default_value_t = GameSelection::Poker)]
    pub game: GameSelection,

    /// Manifest-backed encoder directory for bounded MCCFR training.
    #[arg(long)]
    pub encoder_dir: Option<PathBuf>,

    /// Optional checkpoint to resume before running a training batch.
    #[arg(long)]
    pub checkpoint: Option<PathBuf>,

    /// Number of MCCFR iterations to run before exiting.
    #[arg(long, default_value_t = 0)]
    pub train_iterations: usize,

    /// Wire-encoded strategy query file to answer once before exiting.
    #[arg(long)]
    pub query_file: Option<PathBuf>,

    /// Output path for the wire-encoded strategy response.
    #[arg(long)]
    pub response_file: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::Cli;
    use crate::cli::GameSelection;

    #[test]
    fn cli_parses_stage_zero_flags() {
        let cli = Cli::parse_from([
            "myosu-miner",
            "--chain",
            "ws://127.0.0.1:9944",
            "--subnet",
            "7",
            "--key",
            "//Alice",
            "--port",
            "8081",
            "--register",
            "--serve-axon",
            "--serve-http",
            "--data-dir",
            "/tmp/miner-data",
            "--game",
            "liars-dice",
            "--encoder-dir",
            "/tmp/encoder",
            "--checkpoint",
            "/tmp/miner-data/latest.bin",
            "--train-iterations",
            "256",
            "--query-file",
            "/tmp/miner-data/query.bin",
            "--response-file",
            "/tmp/miner-data/response.bin",
        ]);

        assert_eq!(cli.chain, "ws://127.0.0.1:9944");
        assert_eq!(cli.subnet, 7);
        assert_eq!(cli.key, "//Alice");
        assert_eq!(cli.port, 8081);
        assert!(cli.register);
        assert!(cli.serve_axon);
        assert!(cli.serve_http);
        assert_eq!(cli.data_dir.to_string_lossy(), "/tmp/miner-data");
        assert_eq!(cli.game, GameSelection::LiarsDice);
        assert_eq!(
            cli.encoder_dir.as_deref(),
            Some(std::path::Path::new("/tmp/encoder"))
        );
        assert_eq!(
            cli.checkpoint.as_deref(),
            Some(std::path::Path::new("/tmp/miner-data/latest.bin"))
        );
        assert_eq!(cli.train_iterations, 256);
        assert_eq!(
            cli.query_file.as_deref(),
            Some(std::path::Path::new("/tmp/miner-data/query.bin"))
        );
        assert_eq!(
            cli.response_file.as_deref(),
            Some(std::path::Path::new("/tmp/miner-data/response.bin"))
        );
    }
}
