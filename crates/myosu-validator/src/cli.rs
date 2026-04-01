use clap::{ArgGroup, Parser, ValueEnum};
use myosu_keys::load_active_secret_uri_from_env;
use std::path::PathBuf;

/// Executable stage-0 games supported by the validator bootstrap flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum GameSelection {
    Poker,
    LiarsDice,
}

/// Stage-0 bootstrap CLI for the Myosu validator process.
#[derive(Debug, Parser, Clone)]
#[command(name = "myosu-validator")]
#[command(about = "Bootstrap validator process for the Myosu stage-0 chain")]
#[command(group(
    ArgGroup::new("operator_key_source")
        .required(true)
        .args(["key", "key_config_dir"])
))]
pub struct Cli {
    /// Chain WebSocket RPC endpoint.
    #[arg(long, default_value = "ws://127.0.0.1:9944")]
    pub chain: String,

    /// Subnet to probe and later validate on.
    #[arg(long)]
    pub subnet: u16,

    /// Hotkey seed or URI the operator intends to use for this validator.
    #[arg(long)]
    pub key: Option<String>,

    /// Operator config directory containing `config.toml` and encrypted key files.
    #[arg(long)]
    pub key_config_dir: Option<PathBuf>,

    /// Environment variable holding the password for `--key-config-dir`.
    #[arg(long, default_value = "MYOSU_KEY_PASSWORD")]
    pub key_password_env: String,

    /// Register this validator hotkey on-chain before local scoring begins.
    #[arg(long, default_value_t = false)]
    pub register: bool,

    /// Start the subnet and enable staking before validator bootstrap continues.
    #[arg(long, default_value_t = false)]
    pub enable_subtoken: bool,

    /// Override the subnet tempo through the local sudo path before validation continues.
    #[arg(long)]
    pub sudo_tempo: Option<u16>,

    /// Override the subnet weights set rate limit through the local sudo path.
    #[arg(long)]
    pub sudo_weights_rate_limit: Option<u64>,

    /// Disable commit-reveal on the subnet before weight submission.
    #[arg(long, default_value_t = false)]
    pub disable_commit_reveal: bool,

    /// Submit one bootstrap weight vector on-chain after validation.
    #[arg(long, default_value_t = false)]
    pub submit_weights: bool,

    /// Minimum subnet stake to enforce before waiting for validator permit.
    #[arg(long)]
    pub stake_amount: Option<u64>,

    /// Hotkey to weight on-chain. Defaults to the validator hotkey for self-weight bootstrap.
    #[arg(long)]
    pub weight_hotkey: Option<String>,

    /// Game contract used for deterministic local response scoring.
    #[arg(long, value_enum, default_value_t = GameSelection::Poker)]
    pub game: GameSelection,

    /// Manifest-backed encoder directory for deterministic response scoring.
    #[arg(long)]
    pub encoder_dir: Option<PathBuf>,

    /// Checkpoint artifact used to derive the validator's expected response.
    #[arg(long)]
    pub checkpoint: Option<PathBuf>,

    /// Wire-encoded strategy query file to validate.
    #[arg(long)]
    pub query_file: Option<PathBuf>,

    /// Wire-encoded miner response file to score.
    #[arg(long)]
    pub response_file: Option<PathBuf>,
}

impl Cli {
    /// Resolves the operator signing key into the URI format accepted by the chain client.
    pub fn resolve_key_uri(&self) -> Result<String, myosu_keys::KeyError> {
        if let Some(key) = self.key.as_ref() {
            return Ok(key.clone());
        }

        let config_dir = self
            .key_config_dir
            .as_deref()
            .expect("clap should require one operator key source");
        load_active_secret_uri_from_env(config_dir, &self.key_password_env)
    }

    /// Returns a stable label for the current operator key source.
    pub fn key_source_label(&self) -> &'static str {
        if self.key.is_some() {
            "uri"
        } else {
            "config_dir"
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::Cli;
    use crate::cli::GameSelection;

    #[test]
    fn cli_parses_stage_zero_flags() {
        let cli = Cli::parse_from([
            "myosu-validator",
            "--chain",
            "ws://127.0.0.1:9944",
            "--subnet",
            "7",
            "--key",
            "//Bob",
            "--register",
            "--enable-subtoken",
            "--sudo-tempo",
            "2",
            "--sudo-weights-rate-limit",
            "0",
            "--disable-commit-reveal",
            "--submit-weights",
            "--stake-amount",
            "100000000000000",
            "--weight-hotkey",
            "//Alice",
            "--game",
            "liars-dice",
            "--encoder-dir",
            "/tmp/encoder",
            "--checkpoint",
            "/tmp/checkpoint.bin",
            "--query-file",
            "/tmp/query.bin",
            "--response-file",
            "/tmp/response.bin",
        ]);

        assert_eq!(cli.chain, "ws://127.0.0.1:9944");
        assert_eq!(cli.subnet, 7);
        assert_eq!(cli.key.as_deref(), Some("//Bob"));
        assert!(cli.key_config_dir.is_none());
        assert_eq!(cli.key_password_env, "MYOSU_KEY_PASSWORD");
        assert!(cli.register);
        assert!(cli.enable_subtoken);
        assert_eq!(cli.sudo_tempo, Some(2));
        assert_eq!(cli.sudo_weights_rate_limit, Some(0));
        assert!(cli.disable_commit_reveal);
        assert!(cli.submit_weights);
        assert_eq!(cli.stake_amount, Some(100_000_000_000_000));
        assert_eq!(cli.weight_hotkey.as_deref(), Some("//Alice"));
        assert_eq!(cli.game, GameSelection::LiarsDice);
        assert_eq!(
            cli.encoder_dir.as_deref(),
            Some(std::path::Path::new("/tmp/encoder"))
        );
        assert_eq!(
            cli.checkpoint.as_deref(),
            Some(std::path::Path::new("/tmp/checkpoint.bin"))
        );
        assert_eq!(
            cli.query_file.as_deref(),
            Some(std::path::Path::new("/tmp/query.bin"))
        );
        assert_eq!(
            cli.response_file.as_deref(),
            Some(std::path::Path::new("/tmp/response.bin"))
        );
    }

    #[test]
    fn cli_parses_config_backed_key_source() {
        let cli = Cli::parse_from([
            "myosu-validator",
            "--chain",
            "ws://127.0.0.1:9944",
            "--subnet",
            "7",
            "--key-config-dir",
            "/tmp/myosu",
            "--key-password-env",
            "MYOSU_PASSWORD",
        ]);

        assert!(cli.key.is_none());
        assert_eq!(
            cli.key_config_dir.as_deref(),
            Some(std::path::Path::new("/tmp/myosu"))
        );
        assert_eq!(cli.key_password_env, "MYOSU_PASSWORD");
    }
}
