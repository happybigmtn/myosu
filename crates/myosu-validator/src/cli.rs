use clap::{ArgGroup, Parser, ValueEnum};
use myosu_games_portfolio::ResearchGame;
use myosu_keys::load_active_secret_uri_from_env;
use std::path::PathBuf;

/// Executable stage-0 games supported by the validator bootstrap flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum GameSelection {
    /// Existing robopoker-backed NLHE heads-up solver.
    #[value(alias = "nlhe-heads-up", alias = "nlhe-hu", alias = "nlhe_hu")]
    Poker,
    /// Existing exact Kuhn poker benchmark solver.
    #[value(alias = "kuhn_poker")]
    Kuhn,
    /// Existing dedicated Liar's Dice MCCFR solver.
    #[value(alias = "liars_dice")]
    LiarsDice,
    #[value(alias = "nlhe_6max")]
    NlheSixMax,
    Plo,
    #[value(alias = "nlhe_tournament")]
    NlheTournament,
    #[value(alias = "short_deck")]
    ShortDeck,
    #[value(alias = "teen_patti")]
    TeenPatti,
    #[value(alias = "hanafuda_koi_koi")]
    HanafudaKoiKoi,
    #[value(alias = "hwatu_go_stop")]
    HwatuGoStop,
    #[value(alias = "riichi_mahjong")]
    RiichiMahjong,
    Bridge,
    #[value(alias = "gin_rummy")]
    GinRummy,
    Stratego,
    #[value(alias = "ofc_chinese_poker")]
    OfcChinesePoker,
    Spades,
    #[value(alias = "dou_di_zhu")]
    DouDiZhu,
    #[value(alias = "pusoy_dos")]
    PusoyDos,
    #[value(alias = "tien_len")]
    TienLen,
    #[value(alias = "call_break")]
    CallBreak,
    Backgammon,
    Hearts,
    Cribbage,
}

impl GameSelection {
    pub const PORTFOLIO_SELECTIONS: [Self; 20] = [
        Self::NlheSixMax,
        Self::Plo,
        Self::NlheTournament,
        Self::ShortDeck,
        Self::TeenPatti,
        Self::HanafudaKoiKoi,
        Self::HwatuGoStop,
        Self::RiichiMahjong,
        Self::Bridge,
        Self::GinRummy,
        Self::Stratego,
        Self::OfcChinesePoker,
        Self::Spades,
        Self::DouDiZhu,
        Self::PusoyDos,
        Self::TienLen,
        Self::CallBreak,
        Self::Backgammon,
        Self::Hearts,
        Self::Cribbage,
    ];

    pub fn portfolio_game(self) -> Option<ResearchGame> {
        match self {
            Self::Poker | Self::Kuhn | Self::LiarsDice => None,
            Self::NlheSixMax => Some(ResearchGame::NlheSixMax),
            Self::Plo => Some(ResearchGame::Plo),
            Self::NlheTournament => Some(ResearchGame::NlheTournament),
            Self::ShortDeck => Some(ResearchGame::ShortDeck),
            Self::TeenPatti => Some(ResearchGame::TeenPatti),
            Self::HanafudaKoiKoi => Some(ResearchGame::HanafudaKoiKoi),
            Self::HwatuGoStop => Some(ResearchGame::HwatuGoStop),
            Self::RiichiMahjong => Some(ResearchGame::RiichiMahjong),
            Self::Bridge => Some(ResearchGame::Bridge),
            Self::GinRummy => Some(ResearchGame::GinRummy),
            Self::Stratego => Some(ResearchGame::Stratego),
            Self::OfcChinesePoker => Some(ResearchGame::OfcChinesePoker),
            Self::Spades => Some(ResearchGame::Spades),
            Self::DouDiZhu => Some(ResearchGame::DouDiZhu),
            Self::PusoyDos => Some(ResearchGame::PusoyDos),
            Self::TienLen => Some(ResearchGame::TienLen),
            Self::CallBreak => Some(ResearchGame::CallBreak),
            Self::Backgammon => Some(ResearchGame::Backgammon),
            Self::Hearts => Some(ResearchGame::Hearts),
            Self::Cribbage => Some(ResearchGame::Cribbage),
        }
    }
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

        let Some(config_dir) = self.key_config_dir.as_deref() else {
            return Err(myosu_keys::KeyError::MissingKeySource);
        };
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
    use clap::{Parser, ValueEnum};
    use myosu_games_portfolio::ResearchGame;

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

    #[test]
    fn resolve_key_uri_rejects_missing_key_source_after_cli_validation() {
        let mut cli = Cli::parse_from(["myosu-validator", "--subnet", "7", "--key", "//Bob"]);
        cli.key = None;

        let Err(error) = cli.resolve_key_uri() else {
            panic!("key source is missing");
        };
        assert!(matches!(error, myosu_keys::KeyError::MissingKeySource));
    }

    #[test]
    fn cli_parses_kuhn_game_selection() {
        let cli = Cli::parse_from([
            "myosu-validator",
            "--subnet",
            "7",
            "--key",
            "//Bob",
            "--game",
            "kuhn",
        ]);

        assert_eq!(cli.game, GameSelection::Kuhn);
    }

    #[test]
    fn cli_parses_research_slug_for_heads_up_poker() {
        let cli = Cli::parse_from([
            "myosu-validator",
            "--subnet",
            "7",
            "--key",
            "//Bob",
            "--game",
            "nlhe-heads-up",
        ]);

        assert_eq!(cli.game, GameSelection::Poker);
    }

    #[test]
    fn cli_parses_research_portfolio_game_selection() {
        let cli = Cli::parse_from([
            "myosu-validator",
            "--subnet",
            "7",
            "--key",
            "//Bob",
            "--game",
            "bridge",
        ]);

        assert_eq!(cli.game, GameSelection::Bridge);
        assert_eq!(cli.game.portfolio_game(), Some(ResearchGame::Bridge));
    }

    #[test]
    fn portfolio_selection_inventory_matches_research_manifest() {
        let mapped_games = GameSelection::PORTFOLIO_SELECTIONS
            .into_iter()
            .map(|selection| {
                let Some(possible_value) = selection.to_possible_value() else {
                    panic!("portfolio selection should expose a clap value");
                };
                if let Some(game) = selection.portfolio_game() {
                    assert_eq!(ResearchGame::parse(possible_value.get_name()), Ok(game));
                    let cli = Cli::parse_from([
                        "myosu-validator",
                        "--subnet",
                        "7",
                        "--key",
                        "//Bob",
                        "--game",
                        possible_value.get_name(),
                    ]);
                    assert_eq!(cli.game, selection);
                    let chain_id_cli = Cli::parse_from([
                        "myosu-validator",
                        "--subnet",
                        "7",
                        "--key",
                        "//Bob",
                        "--game",
                        game.chain_id(),
                    ]);
                    assert_eq!(chain_id_cli.game, selection);
                    game
                } else {
                    panic!("portfolio selection should map to a research game");
                }
            })
            .collect::<Vec<_>>();

        assert_eq!(
            mapped_games.as_slice(),
            myosu_games_portfolio::ALL_PORTFOLIO_ROUTED_GAMES.as_slice()
        );
    }

    #[test]
    fn cli_parses_framework_chain_id_aliases() {
        let heads_up = Cli::parse_from([
            "myosu-validator",
            "--subnet",
            "7",
            "--key",
            "//Bob",
            "--game",
            "nlhe_hu",
        ]);
        let kuhn = Cli::parse_from([
            "myosu-validator",
            "--subnet",
            "7",
            "--key",
            "//Bob",
            "--game",
            "kuhn_poker",
        ]);
        let liars_dice = Cli::parse_from([
            "myosu-validator",
            "--subnet",
            "7",
            "--key",
            "//Bob",
            "--game",
            "liars_dice",
        ]);

        assert_eq!(heads_up.game, GameSelection::Poker);
        assert_eq!(kuhn.game, GameSelection::Kuhn);
        assert_eq!(liars_dice.game, GameSelection::LiarsDice);
    }
}
