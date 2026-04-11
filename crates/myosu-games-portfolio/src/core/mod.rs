pub mod backgammon;
pub mod cribbage;
pub mod gin_rummy;
pub mod hanafuda;
pub mod liars_dice;
pub mod mahjong;
pub mod model;
pub mod ofc;
pub mod poker_like;
pub mod shedding;
pub mod stratego;
pub mod trick_taking;

use crate::game::ResearchGame;

pub use model::{
    CoreAction, CoreGameError, CoreGameState, CoreTransition, apply_action, bootstrap_state,
    legal_actions,
};

pub const CORE_RESEARCH_GAMES: [ResearchGame; 22] = [
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

pub const CORE_CANONICAL_TEN: [ResearchGame; 10] = [
    ResearchGame::NlheSixMax,
    ResearchGame::HanafudaKoiKoi,
    ResearchGame::RiichiMahjong,
    ResearchGame::Bridge,
    ResearchGame::GinRummy,
    ResearchGame::Stratego,
    ResearchGame::OfcChinesePoker,
    ResearchGame::DouDiZhu,
    ResearchGame::Backgammon,
    ResearchGame::Cribbage,
];
