use std::io;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use myosu_games_portfolio::ResearchGame;

/// Executable game surfaces supported by `myosu-play`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum GameSelection {
    #[value(alias = "nlhe-heads-up", alias = "nlhe-hu", alias = "nlhe_hu")]
    Poker,
    #[value(alias = "kuhn_poker")]
    Kuhn,
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
    #[cfg(test)]
    pub(crate) const PORTFOLIO_SELECTIONS: [Self; 20] = [
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

    const fn supports_miner_discovery(self) -> bool {
        matches!(self, Self::Poker)
    }

    pub(crate) const fn portfolio_game(self) -> Option<ResearchGame> {
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

    pub(crate) const fn cli_label(self) -> &'static str {
        match self {
            Self::Poker => "poker",
            Self::Kuhn => "kuhn",
            Self::LiarsDice => "liars-dice",
            Self::NlheSixMax => "nlhe-six-max",
            Self::Plo => "plo",
            Self::NlheTournament => "nlhe-tournament",
            Self::ShortDeck => "short-deck",
            Self::TeenPatti => "teen-patti",
            Self::HanafudaKoiKoi => "hanafuda-koi-koi",
            Self::HwatuGoStop => "hwatu-go-stop",
            Self::RiichiMahjong => "riichi-mahjong",
            Self::Bridge => "bridge",
            Self::GinRummy => "gin-rummy",
            Self::Stratego => "stratego",
            Self::OfcChinesePoker => "ofc-chinese-poker",
            Self::Spades => "spades",
            Self::DouDiZhu => "dou-di-zhu",
            Self::PusoyDos => "pusoy-dos",
            Self::TienLen => "tien-len",
            Self::CallBreak => "call-break",
            Self::Backgammon => "backgammon",
            Self::Hearts => "hearts",
            Self::Cribbage => "cribbage",
        }
    }
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

    /// Run the live-read proof in place of the subcommand. Requires
    /// `--chain` and `--subnet`. Prints `bundle_hash`, `miner_uid`, and
    /// `emission` as key=value lines (the P0 #5 milestone).
    #[arg(long)]
    pub(crate) read_solved: bool,

    #[command(subcommand)]
    pub(crate) command: Option<Mode>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Mode {
    /// Run the interactive TUI against a generated demo hand.
    Train(AdviceArgs),

    /// Emit plain-text pipe output and handle a single input line.
    Pipe(AdviceArgs),

    /// Run the live-read proof: connect to a running chain, discover the
    /// miner axon, play one poker hand, and print `bundle_hash`,
    /// `miner_uid`, and `emission` as key=value lines. This is the P0 #5
    /// milestone; an external observer can replay the proof with curl +
    /// `state_getStorage` and confirm the three values match.
    LiveRead(LiveReadArgs),
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

/// CLI args for the `myosu-play live-read` subcommand and the
/// `--read-solved` top-level flag.
#[derive(Args, Debug, Clone)]
pub(crate) struct LiveReadArgs {
    /// Chain WebSocket RPC endpoint to read the live solved result from.
    /// Alias of the top-level `--chain` flag for ergonomic parity with
    /// the P0 row's literal `--chain-endpoint ws://...` wording.
    #[arg(long = "chain-endpoint")]
    pub(crate) chain_endpoint: Option<String>,

    /// Subnet to read the live solved result from. Alias of the
    /// top-level `--subnet` flag.
    #[arg(long)]
    pub(crate) subnet: Option<u16>,
}

#[derive(Clone, Debug)]
pub(crate) struct DiscoveryRequest {
    pub(crate) chain: Option<String>,
    pub(crate) subnet: Option<u16>,
}

impl DiscoveryRequest {
    pub(crate) fn from_cli(cli: &Cli) -> io::Result<Self> {
        if !cli.game.supports_miner_discovery() && cli.chain.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "--game {} does not support --chain/--subnet miner discovery yet",
                    cli.game.cli_label()
                ),
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
    use clap::{Parser, ValueEnum};
    use myosu_games_portfolio::ResearchGame;

    use crate::cli::{Cli, GameSelection};

    #[test]
    fn cli_parses_game_selection() {
        let cli = Cli::parse_from(["myosu-play", "--game", "liars-dice", "pipe"]);

        assert_eq!(cli.game, GameSelection::LiarsDice);
    }

    #[test]
    fn cli_parses_kuhn_selection() {
        let cli = Cli::parse_from(["myosu-play", "--game", "kuhn", "pipe"]);

        assert_eq!(cli.game, GameSelection::Kuhn);
    }

    #[test]
    fn cli_parses_research_portfolio_game_selection() {
        let cli = Cli::parse_from(["myosu-play", "--game", "bridge", "pipe"]);

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
                    assert_eq!(possible_value.get_name(), selection.cli_label());
                    assert_eq!(ResearchGame::parse(selection.cli_label()), Ok(game));
                    let cli = Cli::parse_from([
                        "myosu-play",
                        "--game",
                        possible_value.get_name(),
                        "pipe",
                    ]);
                    assert_eq!(cli.game, selection);
                    let chain_id_cli =
                        Cli::parse_from(["myosu-play", "--game", game.chain_id(), "pipe"]);
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
    fn cli_parses_research_slug_for_heads_up_poker() {
        let cli = Cli::parse_from(["myosu-play", "--game", "nlhe-heads-up", "pipe"]);

        assert_eq!(cli.game, GameSelection::Poker);
    }

    #[test]
    fn cli_parses_framework_chain_id_aliases() {
        let heads_up = Cli::parse_from(["myosu-play", "--game", "nlhe_hu", "pipe"]);
        let kuhn = Cli::parse_from(["myosu-play", "--game", "kuhn_poker", "pipe"]);
        let liars_dice = Cli::parse_from(["myosu-play", "--game", "liars_dice", "pipe"]);

        assert_eq!(heads_up.game, GameSelection::Poker);
        assert_eq!(kuhn.game, GameSelection::Kuhn);
        assert_eq!(liars_dice.game, GameSelection::LiarsDice);
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

    #[test]
    fn discovery_rejects_chain_flags_for_kuhn() {
        let cli = Cli::parse_from([
            "myosu-play",
            "--game",
            "kuhn",
            "--chain",
            "ws://127.0.0.1:9944",
            "--subnet",
            "7",
            "pipe",
        ]);

        let error = crate::cli::DiscoveryRequest::from_cli(&cli)
            .expect_err("kuhn should reject discovery flags");
        assert!(
            error
                .to_string()
                .contains("--game kuhn does not support --chain/--subnet")
        );
    }

    #[test]
    fn discovery_rejects_chain_flags_for_portfolio_games() {
        let cli = Cli::parse_from([
            "myosu-play",
            "--game",
            "bridge",
            "--chain",
            "ws://127.0.0.1:9944",
            "--subnet",
            "7",
            "pipe",
        ]);

        let error = crate::cli::DiscoveryRequest::from_cli(&cli)
            .expect_err("portfolio games should reject discovery flags");
        assert!(
            error
                .to_string()
                .contains("--game bridge does not support --chain/--subnet")
        );
    }

    #[test]
    fn cli_parses_read_solved_flag() {
        let cli = Cli::parse_from([
            "myosu-play",
            "--chain",
            "ws://127.0.0.1:9944",
            "--subnet",
            "7",
            "--read-solved",
        ]);

        assert!(cli.read_solved);
        assert_eq!(cli.chain.as_deref(), Some("ws://127.0.0.1:9944"));
        assert_eq!(cli.subnet, Some(7));
        assert!(cli.command.is_none());
    }

    #[test]
    fn cli_parses_live_read_subcommand_with_chain_endpoint_alias() {
        let cli = Cli::parse_from([
            "myosu-play",
            "live-read",
            "--chain-endpoint",
            "ws://127.0.0.1:9944",
            "--subnet",
            "7",
        ]);

        match cli.command.as_ref() {
            Some(crate::cli::Mode::LiveRead(args)) => {
                assert_eq!(args.chain_endpoint.as_deref(), Some("ws://127.0.0.1:9944"));
                assert_eq!(args.subnet, Some(7));
            }
            other => panic!("expected Mode::LiveRead, got {other:?}"),
        }
        assert!(!cli.read_solved);
    }
}
