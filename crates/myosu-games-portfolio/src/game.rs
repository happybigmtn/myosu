use std::fmt;

use myosu_games::{GameConfig, GameParams, GameType};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Research games covered by `research/game-rules`.
///
/// The rules corpus has 21 files, but `21-hearts-cribbage.md` contains two
/// distinct games. The bootstrap portfolio tracks both so neither is silently
/// collapsed.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ResearchGame {
    NlheHeadsUp,
    NlheSixMax,
    Plo,
    NlheTournament,
    ShortDeck,
    TeenPatti,
    HanafudaKoiKoi,
    HwatuGoStop,
    RiichiMahjong,
    Bridge,
    GinRummy,
    Stratego,
    OfcChinesePoker,
    Spades,
    LiarsDice,
    DouDiZhu,
    PusoyDos,
    TienLen,
    CallBreak,
    Backgammon,
    Hearts,
    Cribbage,
}

/// All distinct games from the rules corpus in file order.
pub const ALL_RESEARCH_GAMES: [ResearchGame; 22] = [
    ResearchGame::NlheHeadsUp,
    ResearchGame::NlheSixMax,
    ResearchGame::Plo,
    ResearchGame::NlheTournament,
    ResearchGame::ShortDeck,
    ResearchGame::TeenPatti,
    ResearchGame::HanafudaKoiKoi,
    ResearchGame::HwatuGoStop,
    ResearchGame::RiichiMahjong,
    ResearchGame::Bridge,
    ResearchGame::GinRummy,
    ResearchGame::Stratego,
    ResearchGame::OfcChinesePoker,
    ResearchGame::Spades,
    ResearchGame::LiarsDice,
    ResearchGame::DouDiZhu,
    ResearchGame::PusoyDos,
    ResearchGame::TienLen,
    ResearchGame::CallBreak,
    ResearchGame::Backgammon,
    ResearchGame::Hearts,
    ResearchGame::Cribbage,
];

/// Research games routed through the portfolio wire surface.
///
/// Heads-up NLHE and Liar's Dice are omitted because they already have
/// dedicated solver crates with distinct wire formats.
pub const ALL_PORTFOLIO_ROUTED_GAMES: [ResearchGame; 20] = [
    ResearchGame::NlheSixMax,
    ResearchGame::Plo,
    ResearchGame::NlheTournament,
    ResearchGame::ShortDeck,
    ResearchGame::TeenPatti,
    ResearchGame::HanafudaKoiKoi,
    ResearchGame::HwatuGoStop,
    ResearchGame::RiichiMahjong,
    ResearchGame::Bridge,
    ResearchGame::GinRummy,
    ResearchGame::Stratego,
    ResearchGame::OfcChinesePoker,
    ResearchGame::Spades,
    ResearchGame::DouDiZhu,
    ResearchGame::PusoyDos,
    ResearchGame::TienLen,
    ResearchGame::CallBreak,
    ResearchGame::Backgammon,
    ResearchGame::Hearts,
    ResearchGame::Cribbage,
];

impl ResearchGame {
    /// Stable CLI / artifact slug.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::NlheHeadsUp => "nlhe-heads-up",
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
            Self::LiarsDice => "liars-dice",
            Self::DouDiZhu => "dou-di-zhu",
            Self::PusoyDos => "pusoy-dos",
            Self::TienLen => "tien-len",
            Self::CallBreak => "call-break",
            Self::Backgammon => "backgammon",
            Self::Hearts => "hearts",
            Self::Cribbage => "cribbage",
        }
    }

    /// Canonical on-chain game id.
    pub const fn chain_id(self) -> &'static str {
        match self {
            Self::NlheHeadsUp => "nlhe_hu",
            Self::NlheSixMax => "nlhe_6max",
            Self::Plo => "plo",
            Self::NlheTournament => "nlhe_tournament",
            Self::ShortDeck => "short_deck",
            Self::TeenPatti => "teen_patti",
            Self::HanafudaKoiKoi => "hanafuda_koi_koi",
            Self::HwatuGoStop => "hwatu_go_stop",
            Self::RiichiMahjong => "riichi_mahjong",
            Self::Bridge => "bridge",
            Self::GinRummy => "gin_rummy",
            Self::Stratego => "stratego",
            Self::OfcChinesePoker => "ofc_chinese_poker",
            Self::Spades => "spades",
            Self::LiarsDice => "liars_dice",
            Self::DouDiZhu => "dou_di_zhu",
            Self::PusoyDos => "pusoy_dos",
            Self::TienLen => "tien_len",
            Self::CallBreak => "call_break",
            Self::Backgammon => "backgammon",
            Self::Hearts => "hearts",
            Self::Cribbage => "cribbage",
        }
    }

    /// Shared framework game type for this research game.
    pub fn game_type(self) -> GameType {
        match self {
            Self::NlheHeadsUp => GameType::NlheHeadsUp,
            Self::NlheSixMax => GameType::NlheSixMax,
            Self::Plo => GameType::Plo,
            Self::NlheTournament => GameType::NlheTournament,
            Self::ShortDeck => GameType::ShortDeck,
            Self::TeenPatti => GameType::TeenPatti,
            Self::HanafudaKoiKoi => GameType::HanafudaKoiKoi,
            Self::HwatuGoStop => GameType::HwatuGoStop,
            Self::RiichiMahjong => GameType::RiichiMahjong,
            Self::Bridge => GameType::Bridge,
            Self::GinRummy => GameType::GinRummy,
            Self::Stratego => GameType::Stratego,
            Self::OfcChinesePoker => GameType::OfcChinesePoker,
            Self::Spades => GameType::Spades,
            Self::LiarsDice => GameType::LiarsDice,
            Self::DouDiZhu => GameType::DouDiZhu,
            Self::PusoyDos => GameType::PusoyDos,
            Self::TienLen => GameType::TienLen,
            Self::CallBreak => GameType::CallBreak,
            Self::Backgammon => GameType::Backgammon,
            Self::Hearts => GameType::Hearts,
            Self::Cribbage => GameType::Cribbage,
        }
    }

    /// Build a shared framework config for this research game.
    ///
    /// Dedicated games use the existing typed parameters. Portfolio-routed games
    /// use `GameParams::Custom` with stable metadata until richer per-game
    /// parameter structs exist.
    pub fn game_config(self) -> GameConfig {
        let game_type = self.game_type();
        let num_players = self.default_players();
        let params = match self {
            Self::NlheHeadsUp => GameParams::NlheHeadsUp {
                stack_bb: 100,
                ante_bb: None,
            },
            Self::LiarsDice => GameParams::LiarsDice {
                num_dice: 1,
                num_faces: 6,
            },
            _ => GameParams::Custom(serde_json::json!({
                "chain_id": self.chain_id(),
                "slug": self.slug(),
                "rule_file": self.rule_file(),
                "solver_family": self.solver_family(),
            })),
        };

        GameConfig::new(game_type, num_players, params)
    }

    /// Human-readable name from the rule corpus.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::NlheHeadsUp => "No-Limit Hold'em Heads-Up",
            Self::NlheSixMax => "No-Limit Hold'em 6-Max",
            Self::Plo => "Pot-Limit Omaha",
            Self::NlheTournament => "No-Limit Hold'em Tournament",
            Self::ShortDeck => "Short Deck Hold'em",
            Self::TeenPatti => "Teen Patti",
            Self::HanafudaKoiKoi => "Hanafuda Koi-Koi",
            Self::HwatuGoStop => "Hwatu Go-Stop",
            Self::RiichiMahjong => "Riichi Mahjong",
            Self::Bridge => "Contract Bridge",
            Self::GinRummy => "Gin Rummy",
            Self::Stratego => "Stratego",
            Self::OfcChinesePoker => "Open Face Chinese Poker",
            Self::Spades => "Spades",
            Self::LiarsDice => "Liar's Dice",
            Self::DouDiZhu => "Dou Di Zhu",
            Self::PusoyDos => "Pusoy Dos",
            Self::TienLen => "Tien Len",
            Self::CallBreak => "Call Break",
            Self::Backgammon => "Backgammon",
            Self::Hearts => "Hearts",
            Self::Cribbage => "Cribbage",
        }
    }

    /// Source rule file.
    pub const fn rule_file(self) -> &'static str {
        match self {
            Self::NlheHeadsUp => "research/game-rules/01-nlhe-heads-up.md",
            Self::NlheSixMax => "research/game-rules/02-nlhe-6max.md",
            Self::Plo => "research/game-rules/03-plo.md",
            Self::NlheTournament => "research/game-rules/04-nlhe-tournament.md",
            Self::ShortDeck => "research/game-rules/05-short-deck.md",
            Self::TeenPatti => "research/game-rules/06-teen-patti.md",
            Self::HanafudaKoiKoi => "research/game-rules/07-hanafuda-koi-koi.md",
            Self::HwatuGoStop => "research/game-rules/08-hwatu-go-stop.md",
            Self::RiichiMahjong => "research/game-rules/09-riichi-mahjong.md",
            Self::Bridge => "research/game-rules/10-bridge.md",
            Self::GinRummy => "research/game-rules/11-gin-rummy.md",
            Self::Stratego => "research/game-rules/12-stratego.md",
            Self::OfcChinesePoker => "research/game-rules/13-ofc-chinese-poker.md",
            Self::Spades => "research/game-rules/14-spades.md",
            Self::LiarsDice => "research/game-rules/15-liars-dice.md",
            Self::DouDiZhu => "research/game-rules/16-dou-di-zhu.md",
            Self::PusoyDos => "research/game-rules/17-pusoy-dos.md",
            Self::TienLen => "research/game-rules/18-tien-len.md",
            Self::CallBreak => "research/game-rules/19-call-break.md",
            Self::Backgammon => "research/game-rules/20-backgammon.md",
            Self::Hearts | Self::Cribbage => "research/game-rules/21-hearts-cribbage.md",
        }
    }

    /// Default player count for a bootstrap subnet instance.
    pub const fn default_players(self) -> u8 {
        match self {
            Self::NlheHeadsUp
            | Self::HanafudaKoiKoi
            | Self::GinRummy
            | Self::Stratego
            | Self::Backgammon
            | Self::Cribbage => 2,
            Self::Plo | Self::NlheSixMax | Self::ShortDeck | Self::TeenPatti => 6,
            Self::NlheTournament => 9,
            Self::HwatuGoStop | Self::DouDiZhu | Self::OfcChinesePoker => 3,
            Self::RiichiMahjong
            | Self::Bridge
            | Self::Spades
            | Self::PusoyDos
            | Self::TienLen
            | Self::CallBreak
            | Self::Hearts => 4,
            Self::LiarsDice => 2,
        }
    }

    /// Bootstrap solver family selected from the rule-corpus research notes.
    pub const fn solver_family(self) -> &'static str {
        match self {
            Self::NlheHeadsUp | Self::NlheSixMax | Self::Plo | Self::ShortDeck => {
                "abstracted MCCFR / blueprint poker policy"
            }
            Self::NlheTournament => "ICM-aware push/fold policy",
            Self::TeenPatti => "blind/seen poker heuristic policy",
            Self::HanafudaKoiKoi | Self::HwatuGoStop => "Monte Carlo yaku-value policy",
            Self::RiichiMahjong => "shanten/ukeire tile-efficiency policy",
            Self::Bridge => "PIMC plus double-dummy-inspired policy",
            Self::GinRummy => "meld-distance draw/discard policy",
            Self::Stratego => "belief-tracking deployment/play policy",
            Self::OfcChinesePoker => "foul-aware placement policy",
            Self::Spades | Self::CallBreak | Self::Hearts => {
                "trick-taking Monte Carlo control policy"
            }
            Self::LiarsDice => "belief-weighted challenge/bid policy",
            Self::DouDiZhu | Self::PusoyDos | Self::TienLen => "shedding-game control policy",
            Self::Backgammon => "race/contact equity policy",
            Self::Cribbage => "pegging and crib-equity policy",
        }
    }

    /// Representative bootstrap decision used by the query example.
    pub const fn bootstrap_decision(self) -> &'static str {
        match self {
            Self::NlheHeadsUp => "button open decision at 100bb",
            Self::NlheSixMax => "cutoff open decision at 100bb",
            Self::Plo => "single-raised-pot flop continuation decision",
            Self::NlheTournament => "final-table short-stack shove/fold spot",
            Self::ShortDeck => "ante-only button open decision",
            Self::TeenPatti => "blind player facing seen-player raise",
            Self::HanafudaKoiKoi => "new yaku made: koi-koi or stop",
            Self::HwatuGoStop => "three-point go/stop decision",
            Self::RiichiMahjong => "tenpai discard and riichi decision",
            Self::Bridge => "opening lead against notrump contract",
            Self::GinRummy => "draw from discard pile or stock",
            Self::Stratego => "unknown piece contact on central lane",
            Self::OfcChinesePoker => "middle-row placement with fantasyland draw",
            Self::Spades => "bid after partner has opened nil",
            Self::LiarsDice => "one-die-per-player opening bid response",
            Self::DouDiZhu => "landlord with initiative after pass-pass",
            Self::PusoyDos => "lead-control decision with five-card combo",
            Self::TienLen => "bomb/chop response to two-chain pressure",
            Self::CallBreak => "fixed-spade trump follow-suit decision",
            Self::Backgammon => "race/contact cube and checker-play decision",
            Self::Hearts => "queen-of-spades danger trick",
            Self::Cribbage => "pegging keep/run setup decision",
        }
    }

    /// Parse a CLI slug or canonical chain id into a research game.
    pub fn parse(value: &str) -> Result<Self, ParseResearchGameError> {
        ALL_RESEARCH_GAMES
            .iter()
            .copied()
            .find(|game| game.slug() == value || game.chain_id() == value)
            .ok_or_else(|| ParseResearchGameError {
                value: value.to_string(),
            })
    }

    /// Whether this research game should use the portfolio solver crate in
    /// miner/validator bootstrap flows.
    pub const fn is_portfolio_routed(self) -> bool {
        !matches!(self, Self::NlheHeadsUp | Self::LiarsDice)
    }
}

impl fmt::Display for ResearchGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

/// Error returned when parsing a research game identifier fails.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("unknown research game `{value}`")]
pub struct ParseResearchGameError {
    pub value: String,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;

    use myosu_games::GameParams;
    use serde_json::Value;

    use super::{ALL_RESEARCH_GAMES, ResearchGame};

    #[test]
    fn research_game_rule_files_match_corpus() {
        let repo_root = repo_root();
        let rules_dir = repo_root.join("research/game-rules");
        let entries = match fs::read_dir(&rules_dir) {
            Ok(entries) => entries,
            Err(error) => panic!("rule corpus should be readable: {error}"),
        };

        let mut corpus_files = BTreeSet::new();
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => panic!("rule corpus entry should be readable: {error}"),
            };
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
                let file_name = entry.file_name().to_string_lossy().into_owned();
                corpus_files.insert(format!("research/game-rules/{file_name}"));
            }
        }

        let mut mapped_files = BTreeSet::new();
        for game in ALL_RESEARCH_GAMES {
            let rule_file = game.rule_file();
            assert!(
                repo_root.join(rule_file).is_file(),
                "mapped rule file is missing: {rule_file}"
            );
            mapped_files.insert(rule_file.to_string());
        }

        assert_eq!(corpus_files.len(), 21);
        assert_eq!(ALL_RESEARCH_GAMES.len(), 22);
        assert_eq!(mapped_files, corpus_files);
        assert_eq!(
            ALL_RESEARCH_GAMES
                .iter()
                .filter(|game| game.rule_file() == "research/game-rules/21-hearts-cribbage.md")
                .count(),
            2
        );
    }

    #[test]
    fn shared_game_configs_match_research_identity() {
        for game in ALL_RESEARCH_GAMES {
            let config = game.game_config();
            assert_eq!(config.game_type, game.game_type());
            assert_eq!(config.num_players, game.default_players());
            assert_eq!(
                myosu_games::GameType::from_bytes(game.chain_id().as_bytes()),
                Some(game.game_type())
            );

            if game.is_portfolio_routed() {
                let GameParams::Custom(metadata) = config.params else {
                    panic!("portfolio-routed game should use custom metadata: {game:?}");
                };
                assert_json_string(&metadata, "chain_id", game.chain_id());
                assert_json_string(&metadata, "slug", game.slug());
                assert_json_string(&metadata, "rule_file", game.rule_file());
                assert_json_string(&metadata, "solver_family", game.solver_family());
            }
        }
    }

    #[test]
    fn parse_accepts_slugs_and_chain_ids() {
        for game in ALL_RESEARCH_GAMES {
            assert_eq!(ResearchGame::parse(game.slug()), Ok(game));
            assert_eq!(ResearchGame::parse(game.chain_id()), Ok(game));
        }
    }

    fn assert_json_string(metadata: &Value, field: &str, expected: &str) {
        let Some(actual) = metadata.get(field).and_then(Value::as_str) else {
            panic!("portfolio metadata field is missing or not a string: {field}");
        };
        assert_eq!(actual, expected);
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }
}
