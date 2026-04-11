mod backgammon;
mod cribbage;
mod gin_rummy;
mod hanafuda;
mod mahjong;
mod ofc;
mod poker_like;
mod shedding;
mod stratego;
mod trick_taking;

use crate::engine::{EngineAnswer, EngineTier};
use crate::game::ResearchGame;
use crate::protocol::{PortfolioAction, PortfolioStrategyResponse};
use crate::state::PortfolioChallenge;

/// Answer a legacy game-only query through the current engine dispatch.
pub(crate) fn answer_game(game: ResearchGame, epochs: usize) -> EngineAnswer {
    match PortfolioChallenge::bootstrap(game) {
        Some(challenge) => answer_challenge(&challenge, epochs),
        None => baseline_answer(game, &format!("{}:bootstrap-v1", game.chain_id())),
    }
}

/// Answer a typed portfolio challenge.
pub(crate) fn answer_challenge(challenge: &PortfolioChallenge, epochs: usize) -> EngineAnswer {
    match challenge {
        PortfolioChallenge::NlheSixMax(_)
        | PortfolioChallenge::Plo(_)
        | PortfolioChallenge::NlheTournament(_)
        | PortfolioChallenge::ShortDeck(_)
        | PortfolioChallenge::TeenPatti(_) => poker_like::answer(challenge),
        PortfolioChallenge::HanafudaKoiKoi(_) | PortfolioChallenge::HwatuGoStop(_) => {
            hanafuda::answer(challenge)
        }
        PortfolioChallenge::RiichiMahjong(_) => mahjong::answer(challenge),
        PortfolioChallenge::Bridge(_)
        | PortfolioChallenge::Spades(_)
        | PortfolioChallenge::CallBreak(_)
        | PortfolioChallenge::Hearts(_) => trick_taking::answer(challenge, epochs),
        PortfolioChallenge::GinRummy(_) => gin_rummy::answer(challenge),
        PortfolioChallenge::Stratego(_) => stratego::answer(challenge),
        PortfolioChallenge::OfcChinesePoker(_) => ofc::answer(challenge),
        PortfolioChallenge::DouDiZhu(_)
        | PortfolioChallenge::PusoyDos(_)
        | PortfolioChallenge::TienLen(_) => shedding::answer(challenge),
        PortfolioChallenge::Backgammon(_) => backgammon::answer(challenge),
        PortfolioChallenge::Cribbage(_) => cribbage::answer(challenge),
    }
}

pub(super) fn engine_answer(
    game: ResearchGame,
    challenge_id: &str,
    engine_family: &'static str,
    actions: Vec<(PortfolioAction, f32)>,
) -> EngineAnswer {
    EngineAnswer::new(
        game,
        challenge_id,
        engine_family,
        EngineTier::RuleAware,
        PortfolioStrategyResponse::new(normalize_actions(actions)),
    )
}

fn normalize_actions(actions: Vec<(PortfolioAction, f32)>) -> Vec<(PortfolioAction, f32)> {
    let total: f32 = actions.iter().map(|(_, weight)| weight.max(0.0)).sum();
    if total <= f32::EPSILON {
        return Vec::new();
    }

    actions
        .into_iter()
        .map(|(action, weight)| (action, weight.max(0.0) / total))
        .collect()
}

/// Static compatibility baseline used for quality comparisons and dedicated
/// compatibility routes.
pub(crate) fn baseline_answer(game: ResearchGame, challenge_id: &str) -> EngineAnswer {
    use PortfolioAction::{
        AcceptDouble, AdvancePiece, AnteSteal, AvoidPenalty, BearOff, BidContract, BidNil, CallGo,
        CallTrump, Challenge, DeclareRiichi, DefendBlind, DiscardDeadwood, DoubleDummyPlay,
        DrawToNuts, FoldDanger, FollowSuit, IcmFold, KeepCrib, Knock, KoiKoi, LandlordBid,
        LeadControl, PassControl, PegRun, PlaceSafe, PotControl, PotSizedRaise, PreserveBomb,
        PushFold, Scout, SeeCards, ShedLowest, ShootMoon, StopRound, TightOpen, TrumpControl,
        ValueBet,
    };

    let actions = match game {
        ResearchGame::NlheHeadsUp => vec![(TightOpen, 0.55), (DefendBlind, 0.30), (ValueBet, 0.15)],
        ResearchGame::NlheSixMax => vec![(TightOpen, 0.45), (PotControl, 0.35), (ValueBet, 0.20)],
        ResearchGame::Plo => vec![
            (PotSizedRaise, 0.40),
            (DrawToNuts, 0.35),
            (PotControl, 0.25),
        ],
        ResearchGame::NlheTournament => vec![(PushFold, 0.55), (IcmFold, 0.30), (ValueBet, 0.15)],
        ResearchGame::ShortDeck => vec![(AnteSteal, 0.45), (ValueBet, 0.35), (PotControl, 0.20)],
        ResearchGame::TeenPatti => vec![(SeeCards, 0.40), (PotControl, 0.35), (ValueBet, 0.25)],
        ResearchGame::HanafudaKoiKoi => vec![(StopRound, 0.50), (KoiKoi, 0.35), (CallGo, 0.15)],
        ResearchGame::HwatuGoStop => vec![(StopRound, 0.45), (CallGo, 0.35), (KoiKoi, 0.20)],
        ResearchGame::RiichiMahjong => vec![
            (DeclareRiichi, 0.45),
            (FoldDanger, 0.35),
            (PotControl, 0.20),
        ],
        ResearchGame::Bridge => vec![
            (DoubleDummyPlay, 0.45),
            (BidContract, 0.35),
            (FollowSuit, 0.20),
        ],
        ResearchGame::GinRummy => vec![(DiscardDeadwood, 0.45), (Knock, 0.35), (PotControl, 0.20)],
        ResearchGame::Stratego => vec![(Scout, 0.40), (AdvancePiece, 0.35), (PlaceSafe, 0.25)],
        ResearchGame::OfcChinesePoker => {
            vec![(PlaceSafe, 0.45), (DrawToNuts, 0.35), (PotControl, 0.20)]
        }
        ResearchGame::Spades => vec![(TrumpControl, 0.45), (BidNil, 0.30), (FollowSuit, 0.25)],
        ResearchGame::LiarsDice => vec![(Challenge, 0.40), (ValueBet, 0.35), (PotControl, 0.25)],
        ResearchGame::DouDiZhu => vec![
            (LandlordBid, 0.40),
            (PreserveBomb, 0.35),
            (ShedLowest, 0.25),
        ],
        ResearchGame::PusoyDos => {
            vec![(LeadControl, 0.45), (ShedLowest, 0.35), (PassControl, 0.20)]
        }
        ResearchGame::TienLen => vec![
            (PreserveBomb, 0.40),
            (LeadControl, 0.35),
            (ShedLowest, 0.25),
        ],
        ResearchGame::CallBreak => {
            vec![(CallTrump, 0.45), (TrumpControl, 0.35), (FollowSuit, 0.20)]
        }
        ResearchGame::Backgammon => {
            vec![(BearOff, 0.40), (AcceptDouble, 0.35), (AdvancePiece, 0.25)]
        }
        ResearchGame::Hearts => vec![(AvoidPenalty, 0.50), (ShootMoon, 0.25), (FollowSuit, 0.25)],
        ResearchGame::Cribbage => vec![(PegRun, 0.45), (KeepCrib, 0.35), (DiscardDeadwood, 0.20)],
    };

    EngineAnswer::new(
        game,
        challenge_id,
        game.solver_family(),
        EngineTier::StaticBaseline,
        PortfolioStrategyResponse::new(normalize_actions(actions)),
    )
}
