use myosu_games::canonical_hash;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::core::{
    CoreGameState, backgammon as core_backgammon, cribbage as core_cribbage,
    gin_rummy as core_gin_rummy, hanafuda as core_hanafuda, mahjong as core_mahjong,
    ofc as core_ofc, poker_like as core_poker_like, shedding as core_shedding,
    stratego as core_stratego, trick_taking as core_trick_taking,
};
use crate::game::ResearchGame;

/// Typed challenge state for portfolio-routed research games.
///
/// Each variant keeps stable spot metadata plus compact state that the
/// matching engine family can use directly.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortfolioChallenge {
    NlheSixMax(PokerLikeChallenge),
    Plo(PokerLikeChallenge),
    NlheTournament(PokerLikeChallenge),
    ShortDeck(PokerLikeChallenge),
    TeenPatti(PokerLikeChallenge),
    HanafudaKoiKoi(HanafudaChallenge),
    HwatuGoStop(HanafudaChallenge),
    RiichiMahjong(MahjongChallenge),
    Bridge(TrickTakingChallenge),
    GinRummy(GinRummyChallenge),
    Stratego(StrategoChallenge),
    OfcChinesePoker(OfcChallenge),
    Spades(TrickTakingChallenge),
    DouDiZhu(SheddingChallenge),
    PusoyDos(SheddingChallenge),
    TienLen(SheddingChallenge),
    CallBreak(TrickTakingChallenge),
    Backgammon(BackgammonChallenge),
    Hearts(TrickTakingChallenge),
    Cribbage(CribbageChallenge),
}

impl PortfolioChallenge {
    /// Create the default typed challenge for a portfolio-routed game.
    pub fn bootstrap(game: ResearchGame) -> Option<Self> {
        Some(match game {
            ResearchGame::NlheSixMax => Self::NlheSixMax(PokerLikeChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "late-position-overpair-v2",
                    "Cutoff 3-bet pot with overpair and initiative",
                ),
                pot_bb: 18,
                effective_stack_bb: 72,
                made_strength: 4,
                draw_strength: 1,
                fold_equity: 2,
                to_call_bb: 4,
                active_players: 2,
                check_available: false,
                raise_available: true,
                in_position: true,
                icm_pressure: 0,
                has_seen_cards: true,
            }),
            ResearchGame::Plo => Self::Plo(PokerLikeChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "nut-wrap-flush-draw-v2",
                    "Turn spot with nut wrap, flush draw, and position",
                ),
                pot_bb: 28,
                effective_stack_bb: 85,
                made_strength: 2,
                draw_strength: 5,
                fold_equity: 2,
                to_call_bb: 4,
                active_players: 4,
                check_available: false,
                raise_available: true,
                in_position: true,
                icm_pressure: 0,
                has_seen_cards: true,
            }),
            ResearchGame::NlheTournament => Self::NlheTournament(PokerLikeChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "bubble-push-fold-v2",
                    "Button jam-or-fold spot under bubble ICM pressure",
                ),
                pot_bb: 4,
                effective_stack_bb: 12,
                made_strength: 1,
                draw_strength: 0,
                fold_equity: 4,
                to_call_bb: 2,
                active_players: 6,
                check_available: false,
                raise_available: true,
                in_position: true,
                icm_pressure: 5,
                has_seen_cards: true,
            }),
            ResearchGame::ShortDeck => Self::ShortDeck(PokerLikeChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "ante-pot-made-plus-draw-v2",
                    "Ante pot with pair plus redraw leverage",
                ),
                pot_bb: 14,
                effective_stack_bb: 38,
                made_strength: 3,
                draw_strength: 3,
                fold_equity: 3,
                to_call_bb: 0,
                active_players: 6,
                check_available: true,
                raise_available: true,
                in_position: true,
                icm_pressure: 0,
                has_seen_cards: true,
            }),
            ResearchGame::TeenPatti => Self::TeenPatti(PokerLikeChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "blind-to-seen-pair-pressure-v2",
                    "Blind decision whether to see cards facing pair pressure",
                ),
                pot_bb: 10,
                effective_stack_bb: 24,
                made_strength: 2,
                draw_strength: 0,
                fold_equity: 2,
                to_call_bb: 1,
                active_players: 6,
                check_available: false,
                raise_available: true,
                in_position: false,
                icm_pressure: 0,
                has_seen_cards: false,
            }),
            ResearchGame::HanafudaKoiKoi => Self::HanafudaKoiKoi(HanafudaChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "bright-stop-window-v2",
                    "Bright-heavy capture with a real stop-versus-koi-koi decision",
                ),
                points: 7,
                bright_count: 3,
                ribbon_yaku: 2,
                animal_yaku: 2,
                bonus_cards: 1,
                yaku_count: 3,
                bright_capture_options: 0,
                opponent_pressure: 2,
                hand_count: 2,
                decision_window: true,
                locked_points: 0,
                continuation_calls: 0,
                upside_capture_options: 1,
                max_upside_gain: 2,
            }),
            ResearchGame::HwatuGoStop => Self::HwatuGoStop(HanafudaChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "go-stop-bonus-press-v2",
                    "Go-Stop turn with scoring pressure and bonus continuation outs",
                ),
                points: 5,
                bright_count: 3,
                ribbon_yaku: 1,
                animal_yaku: 1,
                bonus_cards: 2,
                yaku_count: 1,
                bright_capture_options: 1,
                opponent_pressure: 2,
                hand_count: 2,
                decision_window: true,
                locked_points: 0,
                continuation_calls: 0,
                upside_capture_options: 2,
                max_upside_gain: 4,
            }),
            ResearchGame::RiichiMahjong => Self::RiichiMahjong(MahjongChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "one-shanten-threat-board-v2",
                    "One-shanten hand facing riichi pressure with limited safe tiles",
                ),
                shanten: 1,
                ukeire: 6,
                safe_discards: 2,
                discard_options: 6,
                pair_count: 1,
                run_bases: 4,
                dora_count: 1,
                yakuhai_pairs: 0,
                push_pressure: 0,
                riichi_threats: 2,
                riichi_available: false,
            }),
            ResearchGame::Bridge => Self::Bridge(TrickTakingChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "contract-play-follow-suit-v2",
                    "Trump contract play spot with winners and a follow-suit constraint",
                ),
                trump_count: 3,
                winners: 4,
                void_suits: 1,
                contract_pressure: 2,
                penalty_pressure: 0,
                cards_in_trick: 1,
                follow_suit_forced: true,
                nil_viable: false,
                moon_shot_viable: false,
            }),
            ResearchGame::GinRummy => Self::GinRummy(GinRummyChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "knock-window-v2",
                    "Low-deadwood hand with a real knock versus draw tension",
                ),
                deadwood: 8,
                meld_count: 3,
                live_draws: 2,
                knock_available: true,
                gin_available: false,
                discard_options: 3,
            }),
            ResearchGame::Stratego => Self::Stratego(StrategoChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "scout-lane-bomb-pressure-v2",
                    "Open scout lane with bomb suspicion and unrevealed miners",
                ),
                scout_lanes: 2,
                miners_remaining: 3,
                bombs_suspected: 2,
                attack_targets: 1,
                hidden_targets: 1,
                attack_is_forced: false,
            }),
            ResearchGame::OfcChinesePoker => Self::OfcChinesePoker(OfcChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "safe-placement-fantasyland-draw-v2",
                    "Row placement spot balancing foul safety against fantasy-land outs",
                ),
                front_strength: 2,
                middle_strength: 5,
                back_strength: 8,
                free_slots: 3,
                fantasyland_outs: 2,
                foul_pressure: 0,
            }),
            ResearchGame::Spades => Self::Spades(TrickTakingChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "late-hand-trump-control-v2",
                    "Late-hand spades decision with trump control and no nil window",
                ),
                trump_count: 4,
                winners: 3,
                void_suits: 1,
                contract_pressure: 1,
                penalty_pressure: 0,
                cards_in_trick: 1,
                follow_suit_forced: true,
                nil_viable: false,
                moon_shot_viable: false,
            }),
            ResearchGame::DouDiZhu => Self::DouDiZhu(SheddingChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "bomb-preservation-race-v2",
                    "Landlord race with a bomb available and opponents not yet desperate",
                ),
                bomb_count: 1,
                control_combos: 2,
                low_singles: 3,
                opponents_min_cards: 5,
                danger_opponents: 0,
                next_actor_cards: 5,
                on_lead: false,
                play_options: 2,
                finishing_plays: 0,
                bomb_only_escape: false,
                forced_pass: false,
                lead_rank_pressure: 8,
            }),
            ResearchGame::PusoyDos => Self::PusoyDos(SheddingChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "lead-control-endgame-v2",
                    "Lead spot with strong control combinations in a climbing endgame",
                ),
                bomb_count: 0,
                control_combos: 3,
                low_singles: 2,
                opponents_min_cards: 4,
                danger_opponents: 0,
                next_actor_cards: 6,
                on_lead: true,
                play_options: 1,
                finishing_plays: 0,
                bomb_only_escape: false,
                forced_pass: false,
                lead_rank_pressure: 9,
            }),
            ResearchGame::TienLen => Self::TienLen(SheddingChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "bomb-vs-exit-race-v2",
                    "Tien Len response deciding between bomb preservation and tempo",
                ),
                bomb_count: 1,
                control_combos: 2,
                low_singles: 4,
                opponents_min_cards: 5,
                danger_opponents: 0,
                next_actor_cards: 5,
                on_lead: false,
                play_options: 2,
                finishing_plays: 0,
                bomb_only_escape: false,
                forced_pass: false,
                lead_rank_pressure: 11,
            }),
            ResearchGame::CallBreak => Self::CallBreak(TrickTakingChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "need-trumps-to-hit-bid-v2",
                    "Call Break hand that still needs trump leverage to hit contract",
                ),
                trump_count: 4,
                winners: 3,
                void_suits: 0,
                contract_pressure: 2,
                penalty_pressure: 0,
                cards_in_trick: 1,
                follow_suit_forced: true,
                nil_viable: false,
                moon_shot_viable: false,
            }),
            ResearchGame::Backgammon => Self::Backgammon(BackgammonChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "take-point-in-bearoff-race-v5",
                    "Facing a double in a bear-off race with no remaining contact",
                ),
                race_lead_pips: 14,
                borne_off: 11,
                anchors: 0,
                cube_efficiency: 2,
                has_contact: false,
                bar_count: 0,
                bearoff_ready: true,
                cube_centered: true,
                cube_owned_by_actor: false,
                facing_double: true,
                move_options: 8,
                off_moves: 8,
                blot_count: 4,
                home_board_points: 0,
                prime_length: 0,
            }),
            ResearchGame::Hearts => Self::Hearts(TrickTakingChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "queen-risk-avoidance-v2",
                    "Hearts hand trying to dodge penalty cards rather than shoot the moon",
                ),
                trump_count: 0,
                winners: 2,
                void_suits: 1,
                contract_pressure: 0,
                penalty_pressure: 4,
                cards_in_trick: 1,
                follow_suit_forced: true,
                nil_viable: false,
                moon_shot_viable: false,
            }),
            ResearchGame::Cribbage => Self::Cribbage(CribbageChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    game,
                    "pegging-run-crib-edge-v2",
                    "Pegging count near thirty-one with run value and slight crib edge",
                ),
                pegging_count: 21,
                run_potential: 2,
                crib_edge: 1,
                pair_trap: true,
                go_window: true,
                fifteen_outs: 1,
                max_immediate_points: 4,
            }),
            ResearchGame::NlheHeadsUp | ResearchGame::LiarsDice => return None,
        })
    }

    /// Derive a typed portfolio challenge from the live bounded core state.
    pub fn from_core_state(state: &CoreGameState) -> Option<Self> {
        match state.game {
            ResearchGame::NlheHeadsUp | ResearchGame::LiarsDice => None,
            ResearchGame::NlheSixMax => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::NlheSixMax(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_poker_like(&mut challenge, state);
                Some(Self::NlheSixMax(challenge))
            }
            ResearchGame::Plo => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::Plo(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_poker_like(&mut challenge, state);
                Some(Self::Plo(challenge))
            }
            ResearchGame::NlheTournament => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::NlheTournament(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_poker_like(&mut challenge, state);
                Some(Self::NlheTournament(challenge))
            }
            ResearchGame::ShortDeck => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::ShortDeck(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_poker_like(&mut challenge, state);
                Some(Self::ShortDeck(challenge))
            }
            ResearchGame::TeenPatti => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::TeenPatti(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_poker_like(&mut challenge, state);
                Some(Self::TeenPatti(challenge))
            }
            ResearchGame::HanafudaKoiKoi => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::HanafudaKoiKoi(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_hanafuda(&mut challenge, state);
                Some(Self::HanafudaKoiKoi(challenge))
            }
            ResearchGame::HwatuGoStop => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::HwatuGoStop(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_hanafuda(&mut challenge, state);
                Some(Self::HwatuGoStop(challenge))
            }
            ResearchGame::RiichiMahjong => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::RiichiMahjong(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_mahjong(&mut challenge, state);
                Some(Self::RiichiMahjong(challenge))
            }
            ResearchGame::Bridge => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::Bridge(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_trick_taking(&mut challenge, state);
                Some(Self::Bridge(challenge))
            }
            ResearchGame::Spades => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::Spades(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_trick_taking(&mut challenge, state);
                Some(Self::Spades(challenge))
            }
            ResearchGame::CallBreak => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::CallBreak(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_trick_taking(&mut challenge, state);
                Some(Self::CallBreak(challenge))
            }
            ResearchGame::Hearts => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::Hearts(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_trick_taking(&mut challenge, state);
                Some(Self::Hearts(challenge))
            }
            ResearchGame::GinRummy => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::GinRummy(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_gin_rummy(&mut challenge, state);
                Some(Self::GinRummy(challenge))
            }
            ResearchGame::Stratego => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::Stratego(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_stratego(&mut challenge, state);
                Some(Self::Stratego(challenge))
            }
            ResearchGame::OfcChinesePoker => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::OfcChinesePoker(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_ofc(&mut challenge, state);
                Some(Self::OfcChinesePoker(challenge))
            }
            ResearchGame::DouDiZhu => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::DouDiZhu(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_shedding(&mut challenge, state);
                Some(Self::DouDiZhu(challenge))
            }
            ResearchGame::PusoyDos => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::PusoyDos(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_shedding(&mut challenge, state);
                Some(Self::PusoyDos(challenge))
            }
            ResearchGame::TienLen => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::TienLen(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_shedding(&mut challenge, state);
                Some(Self::TienLen(challenge))
            }
            ResearchGame::Backgammon => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::Backgammon(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_backgammon(&mut challenge, state);
                Some(Self::Backgammon(challenge))
            }
            ResearchGame::Cribbage => {
                let mut challenge = match Self::bootstrap(state.game)? {
                    Self::Cribbage(challenge) => challenge,
                    _ => unreachable!(),
                };
                challenge.spot = core_spot(state);
                hydrate_cribbage(&mut challenge, state);
                Some(Self::Cribbage(challenge))
            }
        }
    }

    /// Research game this typed challenge belongs to.
    pub const fn game(&self) -> ResearchGame {
        match self {
            Self::NlheSixMax(_) => ResearchGame::NlheSixMax,
            Self::Plo(_) => ResearchGame::Plo,
            Self::NlheTournament(_) => ResearchGame::NlheTournament,
            Self::ShortDeck(_) => ResearchGame::ShortDeck,
            Self::TeenPatti(_) => ResearchGame::TeenPatti,
            Self::HanafudaKoiKoi(_) => ResearchGame::HanafudaKoiKoi,
            Self::HwatuGoStop(_) => ResearchGame::HwatuGoStop,
            Self::RiichiMahjong(_) => ResearchGame::RiichiMahjong,
            Self::Bridge(_) => ResearchGame::Bridge,
            Self::GinRummy(_) => ResearchGame::GinRummy,
            Self::Stratego(_) => ResearchGame::Stratego,
            Self::OfcChinesePoker(_) => ResearchGame::OfcChinesePoker,
            Self::Spades(_) => ResearchGame::Spades,
            Self::DouDiZhu(_) => ResearchGame::DouDiZhu,
            Self::PusoyDos(_) => ResearchGame::PusoyDos,
            Self::TienLen(_) => ResearchGame::TienLen,
            Self::CallBreak(_) => ResearchGame::CallBreak,
            Self::Backgammon(_) => ResearchGame::Backgammon,
            Self::Hearts(_) => ResearchGame::Hearts,
            Self::Cribbage(_) => ResearchGame::Cribbage,
        }
    }

    /// Shared spot metadata for the challenge.
    pub fn spot(&self) -> &PortfolioChallengeSpot {
        match self {
            Self::NlheSixMax(challenge)
            | Self::Plo(challenge)
            | Self::NlheTournament(challenge)
            | Self::ShortDeck(challenge)
            | Self::TeenPatti(challenge) => &challenge.spot,
            Self::HanafudaKoiKoi(challenge) | Self::HwatuGoStop(challenge) => &challenge.spot,
            Self::RiichiMahjong(challenge) => &challenge.spot,
            Self::Bridge(challenge)
            | Self::Spades(challenge)
            | Self::CallBreak(challenge)
            | Self::Hearts(challenge) => &challenge.spot,
            Self::GinRummy(challenge) => &challenge.spot,
            Self::Stratego(challenge) => &challenge.spot,
            Self::OfcChinesePoker(challenge) => &challenge.spot,
            Self::DouDiZhu(challenge) | Self::PusoyDos(challenge) | Self::TienLen(challenge) => {
                &challenge.spot
            }
            Self::Backgammon(challenge) => &challenge.spot,
            Self::Cribbage(challenge) => &challenge.spot,
        }
    }
}

/// Shared metadata for one typed challenge state.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortfolioChallengeSpot {
    pub challenge_id: String,
    pub decision: String,
    pub rule_file: String,
    pub solver_family: String,
}

impl PortfolioChallengeSpot {
    /// Build the default typed challenge metadata for a research game.
    pub fn bootstrap(game: ResearchGame) -> Self {
        Self::scenario(game, "bootstrap-v1", game.bootstrap_decision())
    }

    /// Build a named representative spot for a research game.
    pub fn scenario(game: ResearchGame, scenario: &str, decision: &str) -> Self {
        Self {
            challenge_id: format!("{}:{scenario}", game.chain_id()),
            decision: decision.to_string(),
            rule_file: game.rule_file().to_string(),
            solver_family: game.solver_family().to_string(),
        }
    }
}

/// Compact state used by poker-like family engines.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PokerLikeChallenge {
    pub spot: PortfolioChallengeSpot,
    pub pot_bb: u16,
    pub effective_stack_bb: u16,
    pub made_strength: u8,
    pub draw_strength: u8,
    pub fold_equity: u8,
    pub to_call_bb: u16,
    pub active_players: u8,
    pub check_available: bool,
    pub raise_available: bool,
    pub in_position: bool,
    pub icm_pressure: u8,
    pub has_seen_cards: bool,
}

/// Compact state used by Hanafuda and Hwatu family engines.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HanafudaChallenge {
    pub spot: PortfolioChallengeSpot,
    pub points: u8,
    pub bright_count: u8,
    pub ribbon_yaku: u8,
    pub animal_yaku: u8,
    pub bonus_cards: u8,
    pub yaku_count: u8,
    pub bright_capture_options: u8,
    pub opponent_pressure: u8,
    pub hand_count: u8,
    pub decision_window: bool,
    pub locked_points: u8,
    pub continuation_calls: u8,
    pub upside_capture_options: u8,
    pub max_upside_gain: u8,
}

/// Compact state used by the Riichi Mahjong engine.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MahjongChallenge {
    pub spot: PortfolioChallengeSpot,
    pub shanten: u8,
    pub ukeire: u8,
    pub safe_discards: u8,
    pub discard_options: u8,
    pub pair_count: u8,
    pub run_bases: u8,
    pub dora_count: u8,
    pub yakuhai_pairs: u8,
    pub push_pressure: u8,
    pub riichi_threats: u8,
    pub riichi_available: bool,
}

/// Compact state used by trick-taking family engines.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrickTakingChallenge {
    pub spot: PortfolioChallengeSpot,
    pub trump_count: u8,
    pub winners: u8,
    pub void_suits: u8,
    pub contract_pressure: i8,
    pub penalty_pressure: u8,
    pub cards_in_trick: u8,
    pub follow_suit_forced: bool,
    pub nil_viable: bool,
    pub moon_shot_viable: bool,
}

/// Compact state used by the Gin Rummy engine.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GinRummyChallenge {
    pub spot: PortfolioChallengeSpot,
    pub deadwood: u8,
    pub meld_count: u8,
    pub live_draws: u8,
    pub knock_available: bool,
    pub gin_available: bool,
    pub discard_options: u8,
}

/// Compact state used by the Stratego engine.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StrategoChallenge {
    pub spot: PortfolioChallengeSpot,
    pub scout_lanes: u8,
    pub miners_remaining: u8,
    pub bombs_suspected: u8,
    pub attack_targets: u8,
    pub hidden_targets: u8,
    pub attack_is_forced: bool,
}

/// Compact state used by the OFC engine.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OfcChallenge {
    pub spot: PortfolioChallengeSpot,
    pub front_strength: u8,
    pub middle_strength: u8,
    pub back_strength: u8,
    pub free_slots: u8,
    pub fantasyland_outs: u8,
    pub foul_pressure: u8,
}

/// Compact state used by shedding-family engines.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SheddingChallenge {
    pub spot: PortfolioChallengeSpot,
    pub bomb_count: u8,
    pub control_combos: u8,
    pub low_singles: u8,
    pub opponents_min_cards: u8,
    pub danger_opponents: u8,
    pub next_actor_cards: u8,
    pub on_lead: bool,
    pub play_options: u8,
    pub finishing_plays: u8,
    pub bomb_only_escape: bool,
    pub forced_pass: bool,
    pub lead_rank_pressure: u8,
}

/// Compact state used by the Backgammon engine.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackgammonChallenge {
    pub spot: PortfolioChallengeSpot,
    pub race_lead_pips: i16,
    pub borne_off: u8,
    pub anchors: u8,
    pub cube_efficiency: u8,
    pub has_contact: bool,
    pub bar_count: u8,
    pub bearoff_ready: bool,
    pub cube_centered: bool,
    pub cube_owned_by_actor: bool,
    pub facing_double: bool,
    pub move_options: u8,
    pub off_moves: u8,
    pub blot_count: u8,
    pub home_board_points: u8,
    pub prime_length: u8,
}

/// Compact state used by the Cribbage engine.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CribbageChallenge {
    pub spot: PortfolioChallengeSpot,
    pub pegging_count: u8,
    pub run_potential: u8,
    pub crib_edge: i8,
    pub pair_trap: bool,
    pub go_window: bool,
    pub fifteen_outs: u8,
    pub max_immediate_points: u8,
}

fn core_spot(state: &CoreGameState) -> PortfolioChallengeSpot {
    let challenge_hash = canonical_hash(&json!({
        "actor": state.actor,
        "phase": state.phase,
        "public_state": state.public_state,
        "legal_action_ids": state
            .legal_actions
            .iter()
            .map(|action| action.action_id.as_str())
            .collect::<Vec<_>>(),
    }))
    .ok()
    .and_then(|hash| hash.get(..8).map(ToString::to_string))
    .unwrap_or_else(|| {
        format!(
            "a{}l{}",
            state.actor.unwrap_or(255),
            state.legal_actions.len()
        )
    });

    PortfolioChallengeSpot {
        challenge_id: format!(
            "{}:core-{}-{challenge_hash}",
            state.game.chain_id(),
            state.phase
        ),
        decision: format!("{} ({})", state.game.bootstrap_decision(), state.phase),
        rule_file: state.game.rule_file().to_string(),
        solver_family: state.game.solver_family().to_string(),
    }
}

fn hydrate_poker_like(challenge: &mut PokerLikeChallenge, state: &CoreGameState) {
    let Ok(view) = core_poker_like::feature_view(state) else {
        return;
    };
    challenge.pot_bb = u64_to_u16(view.pot);
    challenge.effective_stack_bb = u64_to_u16(view.effective_stack);
    challenge.fold_equity = usize_to_u8(
        usize::from(view.raise_legal)
            .saturating_add(view.active_players.saturating_sub(1).min(4))
            .saturating_add(usize::from(view.to_call == 0)),
    );
    challenge.to_call_bb = u64_to_u16(view.to_call);
    challenge.active_players = usize_to_u8(view.active_players);
    challenge.check_available = view.check_legal;
    challenge.raise_available = view.raise_legal;
    challenge.in_position = view.in_position;
    challenge.icm_pressure = view.icm_pressure;
    challenge.has_seen_cards = view.has_seen_cards;
    challenge.draw_strength = match state.game {
        ResearchGame::Plo => usize_to_u8(view.board_len.saturating_add(1).min(5)),
        ResearchGame::ShortDeck => {
            if view.board_len == 0 {
                2
            } else {
                usize_to_u8(view.board_len.saturating_add(1).min(5))
            }
        }
        ResearchGame::TeenPatti => 0,
        _ => usize_to_u8(view.board_len.min(3)),
    };
    challenge.made_strength = match state.game {
        ResearchGame::TeenPatti => {
            if challenge.has_seen_cards {
                3
            } else {
                1
            }
        }
        ResearchGame::NlheTournament => {
            if challenge.effective_stack_bb <= 12 {
                1
            } else {
                usize_to_u8(
                    view.board_len
                        .saturating_add(usize::from(view.check_legal))
                        .saturating_add(1)
                        .min(5),
                )
            }
        }
        _ => usize_to_u8(
            view.board_len
                .saturating_add(usize::from(view.check_legal))
                .saturating_add(usize::from(view.raise_legal && view.to_call == 0))
                .saturating_add(1)
                .min(5),
        ),
    };
}

fn hydrate_hanafuda(challenge: &mut HanafudaChallenge, state: &CoreGameState) {
    let Ok(view) = core_hanafuda::feature_view(state) else {
        return;
    };
    challenge.points = view.points;
    challenge.bright_count = view.bright_count;
    challenge.ribbon_yaku = view.ribbon_yaku;
    challenge.animal_yaku = view.animal_yaku;
    challenge.bonus_cards = view.bonus_cards;
    challenge.yaku_count = view.yaku_count;
    challenge.bright_capture_options = view.bright_capture_options;
    challenge.opponent_pressure = view.opponent_pressure;
    challenge.hand_count = view.hand_count;
    challenge.decision_window = view.decision_window;
    challenge.locked_points = view.locked_points;
    challenge.continuation_calls = view.continuation_calls;
    challenge.upside_capture_options = view.upside_capture_options;
    challenge.max_upside_gain = view.max_upside_gain;
}

fn hydrate_mahjong(challenge: &mut MahjongChallenge, state: &CoreGameState) {
    let Ok(view) = core_mahjong::feature_view(state) else {
        return;
    };
    challenge.shanten = view.shanten;
    challenge.safe_discards = view.safe_discards;
    challenge.discard_options = view.discard_options;
    challenge.ukeire = view.ukeire;
    challenge.pair_count = view.pair_count;
    challenge.run_bases = view.run_bases;
    challenge.dora_count = view.dora_count;
    challenge.yakuhai_pairs = view.yakuhai_pairs;
    challenge.push_pressure = view.push_pressure;
    challenge.riichi_threats = view.riichi_threats;
    challenge.riichi_available = view.riichi_available;
}

fn hydrate_trick_taking(challenge: &mut TrickTakingChallenge, state: &CoreGameState) {
    let Ok(view) = core_trick_taking::feature_view(state) else {
        return;
    };
    challenge.trump_count = view.trump_count;
    challenge.winners = view.winners;
    challenge.void_suits = view.void_suits;
    challenge.cards_in_trick = view.cards_in_trick;
    challenge.follow_suit_forced = view.follow_suit_forced;
    if state.game == ResearchGame::Hearts {
        challenge.contract_pressure = 0;
        challenge.penalty_pressure = view.penalty_pressure;
        challenge.nil_viable = view.nil_viable;
        challenge.moon_shot_viable = view.moon_shot_viable;
    } else {
        challenge.contract_pressure =
            i8::try_from(3_u8.saturating_sub(view.actor_tricks_won)).unwrap_or(i8::MAX);
        challenge.penalty_pressure = 0;
        challenge.nil_viable = view.nil_viable;
        challenge.moon_shot_viable = view.moon_shot_viable;
    }
}

fn hydrate_gin_rummy(challenge: &mut GinRummyChallenge, state: &CoreGameState) {
    let Ok(view) = core_gin_rummy::feature_view(state) else {
        return;
    };
    challenge.deadwood = view.deadwood;
    challenge.meld_count = view.meld_count;
    challenge.live_draws = view.live_draws;
    challenge.knock_available = view.knock_available;
    challenge.gin_available = view.gin_available;
    challenge.discard_options = view.discard_options;
}

fn hydrate_stratego(challenge: &mut StrategoChallenge, state: &CoreGameState) {
    let Ok(view) = core_stratego::feature_view(state) else {
        return;
    };
    challenge.scout_lanes = view.scout_lanes;
    challenge.miners_remaining = view.miners_remaining;
    challenge.bombs_suspected = view.bombs_suspected;
    challenge.attack_targets = view.attack_targets;
    challenge.hidden_targets = view.hidden_targets;
    challenge.attack_is_forced = view.attack_is_forced;
}

fn hydrate_ofc(challenge: &mut OfcChallenge, state: &CoreGameState) {
    let Ok(view) = core_ofc::feature_view(state) else {
        return;
    };
    challenge.front_strength = view.front_strength;
    challenge.middle_strength = view.middle_strength;
    challenge.back_strength = view.back_strength;
    challenge.free_slots = view.free_slots;
    challenge.fantasyland_outs = view.fantasyland_outs;
    challenge.foul_pressure = view.foul_pressure;
}

fn hydrate_shedding(challenge: &mut SheddingChallenge, state: &CoreGameState) {
    let Ok(view) = core_shedding::feature_view(state) else {
        return;
    };
    challenge.bomb_count = view.bomb_count;
    challenge.control_combos = view.control_combos;
    challenge.low_singles = view.low_singles;
    challenge.opponents_min_cards = view.opponents_min_cards;
    challenge.danger_opponents = view.danger_opponents;
    challenge.next_actor_cards = view.next_actor_cards;
    challenge.on_lead = view.on_lead;
    challenge.play_options = view.play_options;
    challenge.finishing_plays = view.finishing_plays;
    challenge.bomb_only_escape = view.bomb_only_escape;
    challenge.forced_pass = view.forced_pass;
    challenge.lead_rank_pressure = view.lead_rank_pressure;
}

fn hydrate_backgammon(challenge: &mut BackgammonChallenge, state: &CoreGameState) {
    let Ok(view) = core_backgammon::feature_view(state) else {
        return;
    };
    challenge.race_lead_pips = view.race_lead_pips;
    challenge.borne_off = view.borne_off;
    challenge.anchors = view.anchors;
    challenge.cube_efficiency = view.cube_efficiency;
    challenge.has_contact = view.has_contact;
    challenge.bar_count = view.bar_count;
    challenge.bearoff_ready = view.bearoff_ready;
    challenge.cube_centered = view.cube_centered;
    challenge.cube_owned_by_actor = view.cube_owned_by_actor;
    challenge.facing_double = view.facing_double;
    challenge.move_options = view.move_options;
    challenge.off_moves = view.off_moves;
    challenge.blot_count = view.blot_count;
    challenge.home_board_points = view.home_board_points;
    challenge.prime_length = view.prime_length;
}

fn hydrate_cribbage(challenge: &mut CribbageChallenge, state: &CoreGameState) {
    let Ok(view) = core_cribbage::feature_view(state) else {
        return;
    };
    challenge.pegging_count = view.pegging_count;
    challenge.run_potential = view.run_potential;
    challenge.crib_edge = view.crib_edge;
    challenge.pair_trap = view.pair_trap;
    challenge.go_window = view.go_window;
    challenge.fifteen_outs = view.fifteen_outs;
    challenge.max_immediate_points = view.max_immediate_points;
}

fn usize_to_u8(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

fn u64_to_u16(value: u64) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::core::{apply_action, bootstrap_state};
    use crate::engine::answer_typed_challenge;
    use crate::game::{ALL_PORTFOLIO_ROUTED_GAMES, ResearchGame};
    use crate::protocol::{PortfolioAction, recommended_action};
    use crate::state::{PortfolioChallenge, PortfolioChallengeSpot};

    #[test]
    fn every_portfolio_game_has_a_typed_bootstrap_challenge() {
        for game in ALL_PORTFOLIO_ROUTED_GAMES {
            let challenge = match PortfolioChallenge::bootstrap(game) {
                Some(challenge) => challenge,
                None => panic!("portfolio game should have challenge"),
            };

            assert_eq!(challenge.game(), game);
            assert_eq!(challenge.spot().rule_file, game.rule_file());
            assert!(challenge.spot().challenge_id.contains(game.chain_id()));
            let version_suffix = challenge
                .spot()
                .challenge_id
                .rsplit('-')
                .next()
                .unwrap_or_else(|| panic!("challenge id should include a version suffix"));
            assert!(version_suffix.starts_with('v'));
            assert!(version_suffix.chars().skip(1).all(|ch| ch.is_ascii_digit()));
            assert!(!challenge.spot().decision.is_empty());
        }
    }

    #[test]
    fn dedicated_games_do_not_use_portfolio_challenges() {
        assert!(PortfolioChallenge::bootstrap(ResearchGame::NlheHeadsUp).is_none());
        assert!(PortfolioChallenge::bootstrap(ResearchGame::LiarsDice).is_none());
    }

    #[test]
    fn scenario_builder_keeps_rule_file_and_family() {
        let spot = PortfolioChallengeSpot::scenario(
            ResearchGame::Bridge,
            "contract-play-follow-suit-v2",
            "Bridge follow-suit spot",
        );

        assert_eq!(spot.rule_file, ResearchGame::Bridge.rule_file());
        assert_eq!(spot.solver_family, ResearchGame::Bridge.solver_family());
        assert_eq!(spot.challenge_id, "bridge:contract-play-follow-suit-v2");
    }

    #[test]
    fn bootstrap_challenges_expose_real_state_not_just_metadata() {
        let bridge = PortfolioChallenge::bootstrap(ResearchGame::Bridge)
            .unwrap_or_else(|| panic!("bridge challenge missing"));
        let poker = PortfolioChallenge::bootstrap(ResearchGame::NlheSixMax)
            .unwrap_or_else(|| panic!("nlhe six max challenge missing"));
        let cribbage = PortfolioChallenge::bootstrap(ResearchGame::Cribbage)
            .unwrap_or_else(|| panic!("cribbage challenge missing"));

        match bridge {
            PortfolioChallenge::Bridge(challenge) => {
                assert!(challenge.trump_count > 0);
                assert!(challenge.contract_pressure > 0);
            }
            _ => panic!("bridge challenge should use trick-taking state"),
        }

        match poker {
            PortfolioChallenge::NlheSixMax(challenge) => {
                assert!(challenge.pot_bb > 0);
                assert!(challenge.in_position);
                assert!(challenge.made_strength > challenge.draw_strength);
            }
            _ => panic!("six-max challenge should use poker-like state"),
        }

        match cribbage {
            PortfolioChallenge::Cribbage(challenge) => {
                assert!(challenge.pegging_count > 0);
                assert!(challenge.run_potential > 0);
            }
            _ => panic!("cribbage challenge should use cribbage state"),
        }
    }

    #[test]
    fn teen_patti_core_state_updates_seen_cards_and_recommendation() {
        let blind_state = bootstrap_state(ResearchGame::TeenPatti)
            .unwrap_or_else(|error| panic!("teen patti core state should bootstrap: {error}"));
        let blind_challenge = PortfolioChallenge::from_core_state(&blind_state)
            .unwrap_or_else(|| panic!("teen patti core state should derive challenge"));
        let blind_answer = answer_typed_challenge(&blind_challenge, 0)
            .unwrap_or_else(|error| panic!("blind core challenge should answer: {error}"));

        let seen_transition = apply_action(&blind_state, "teen-patti.see-cards", json!({}))
            .unwrap_or_else(|error| panic!("see-cards action should apply: {error}"));
        let seen_challenge = PortfolioChallenge::from_core_state(&seen_transition.after)
            .unwrap_or_else(|| panic!("seen teen patti core state should derive challenge"));
        let seen_answer = answer_typed_challenge(&seen_challenge, 0)
            .unwrap_or_else(|error| panic!("seen core challenge should answer: {error}"));

        match blind_challenge {
            PortfolioChallenge::TeenPatti(challenge) => {
                assert!(!challenge.has_seen_cards);
                assert_eq!(challenge.to_call_bb, 1);
                assert_eq!(challenge.active_players, 6);
                assert!(challenge.raise_available);
                assert!(!challenge.check_available);
            }
            _ => panic!("blind teen patti state should derive teen patti challenge"),
        }
        match seen_challenge {
            PortfolioChallenge::TeenPatti(challenge) => {
                assert!(challenge.has_seen_cards);
                assert_eq!(challenge.to_call_bb, 1);
                assert_eq!(challenge.active_players, 6);
                assert!(challenge.raise_available);
                assert!(!challenge.check_available);
            }
            _ => panic!("seen teen patti state should derive teen patti challenge"),
        }
        assert_eq!(
            recommended_action(&blind_answer.response),
            Some(PortfolioAction::SeeCards)
        );
        assert_eq!(
            recommended_action(&seen_answer.response),
            Some(PortfolioAction::ValueBet)
        );
    }

    #[test]
    fn dou_di_zhu_core_state_updates_on_lead_after_pass_reset() {
        let contested_state = bootstrap_state(ResearchGame::DouDiZhu)
            .unwrap_or_else(|error| panic!("dou di zhu core state should bootstrap: {error}"));
        let contested_challenge = PortfolioChallenge::from_core_state(&contested_state)
            .unwrap_or_else(|| panic!("contested dou di zhu state should derive challenge"));

        let reset_state = apply_action(&contested_state, "dou-di-zhu.pass", json!({}))
            .unwrap_or_else(|error| panic!("pass reset should apply: {error}"))
            .after;
        let reset_challenge = PortfolioChallenge::from_core_state(&reset_state)
            .unwrap_or_else(|| panic!("reset dou di zhu state should derive challenge"));

        match contested_challenge {
            PortfolioChallenge::DouDiZhu(challenge) => {
                assert!(!challenge.on_lead);
                assert_eq!(challenge.opponents_min_cards, 5);
                assert_eq!(challenge.danger_opponents, 0);
                assert_eq!(challenge.next_actor_cards, 5);
                assert_eq!(challenge.play_options, 2);
                assert_eq!(challenge.finishing_plays, 0);
                assert!(!challenge.bomb_only_escape);
                assert!(!challenge.forced_pass);
                assert_eq!(challenge.lead_rank_pressure, 8);
            }
            _ => panic!("contested dou di zhu state should derive shedding challenge"),
        }
        match reset_challenge {
            PortfolioChallenge::DouDiZhu(challenge) => {
                assert!(challenge.on_lead);
                assert_eq!(challenge.lead_rank_pressure, 0);
            }
            _ => panic!("reset dou di zhu state should derive shedding challenge"),
        }
    }

    #[test]
    fn mahjong_core_state_derives_safe_discards_and_ukeire() {
        let state = bootstrap_state(ResearchGame::RiichiMahjong)
            .unwrap_or_else(|error| panic!("mahjong core state should bootstrap: {error}"));
        let challenge = PortfolioChallenge::from_core_state(&state)
            .unwrap_or_else(|| panic!("mahjong core state should derive challenge"));

        match challenge {
            PortfolioChallenge::RiichiMahjong(challenge) => {
                assert_eq!(challenge.shanten, 0);
                assert_eq!(challenge.safe_discards, 11);
                assert_eq!(challenge.discard_options, 12);
                assert_eq!(challenge.ukeire, 13);
                assert_eq!(challenge.pair_count, 2);
                assert_eq!(challenge.run_bases, 6);
                assert_eq!(challenge.dora_count, 2);
                assert_eq!(challenge.yakuhai_pairs, 1);
                assert_eq!(challenge.push_pressure, 5);
                assert!(challenge.riichi_available);
            }
            _ => panic!("mahjong state should derive mahjong challenge"),
        }
    }

    #[test]
    fn ofc_core_state_derives_slot_pressure() {
        let state = bootstrap_state(ResearchGame::OfcChinesePoker)
            .unwrap_or_else(|error| panic!("ofc core state should bootstrap: {error}"));
        let challenge = PortfolioChallenge::from_core_state(&state)
            .unwrap_or_else(|| panic!("ofc core state should derive challenge"));

        match challenge {
            PortfolioChallenge::OfcChinesePoker(challenge) => {
                assert_eq!(challenge.free_slots, 1);
                assert_eq!(challenge.fantasyland_outs, 0);
                assert!(challenge.back_strength > challenge.middle_strength);
                assert!(challenge.foul_pressure > 0);
            }
            _ => panic!("ofc state should derive ofc challenge"),
        }
    }

    #[test]
    fn gin_core_state_derives_finish_window_fields() {
        let state = bootstrap_state(ResearchGame::GinRummy)
            .unwrap_or_else(|error| panic!("gin core state should bootstrap: {error}"));
        let challenge = PortfolioChallenge::from_core_state(&state)
            .unwrap_or_else(|| panic!("gin core state should derive challenge"));

        match challenge {
            PortfolioChallenge::GinRummy(challenge) => {
                assert!(challenge.knock_available);
                assert!(!challenge.gin_available);
                assert_eq!(challenge.discard_options, 3);
            }
            _ => panic!("gin state should derive gin challenge"),
        }
    }

    #[test]
    fn stratego_core_state_derives_attack_pressure() {
        let state = bootstrap_state(ResearchGame::Stratego)
            .unwrap_or_else(|error| panic!("stratego core state should bootstrap: {error}"));
        let challenge = PortfolioChallenge::from_core_state(&state)
            .unwrap_or_else(|| panic!("stratego core state should derive challenge"));

        match challenge {
            PortfolioChallenge::Stratego(challenge) => {
                assert_eq!(challenge.attack_targets, 2);
                assert_eq!(challenge.hidden_targets, 2);
                assert!(!challenge.attack_is_forced);
            }
            _ => panic!("stratego state should derive stratego challenge"),
        }
    }

    #[test]
    fn cribbage_core_state_derives_immediate_scoring_pressure() {
        let state = bootstrap_state(ResearchGame::Cribbage)
            .unwrap_or_else(|error| panic!("cribbage core state should bootstrap: {error}"));
        let challenge = PortfolioChallenge::from_core_state(&state)
            .unwrap_or_else(|| panic!("cribbage core state should derive challenge"));

        match challenge {
            PortfolioChallenge::Cribbage(challenge) => {
                assert_eq!(challenge.fifteen_outs, 1);
                assert_eq!(challenge.max_immediate_points, 5);
            }
            _ => panic!("cribbage state should derive cribbage challenge"),
        }
    }

    #[test]
    fn hwatu_core_state_derives_decision_window() {
        let state = bootstrap_state(ResearchGame::HwatuGoStop)
            .unwrap_or_else(|error| panic!("hwatu core state should bootstrap: {error}"));
        let challenge = PortfolioChallenge::from_core_state(&state)
            .unwrap_or_else(|| panic!("hwatu core state should derive challenge"));

        match challenge {
            PortfolioChallenge::HwatuGoStop(challenge) => {
                assert_eq!(challenge.points, 5);
                assert_eq!(challenge.bright_count, 3);
                assert!(challenge.decision_window);
                assert_eq!(challenge.hand_count, 2);
                assert_eq!(challenge.yaku_count, 1);
                assert_eq!(challenge.bright_capture_options, 1);
                assert_eq!(challenge.opponent_pressure, 2);
                assert_eq!(challenge.locked_points, 0);
                assert_eq!(challenge.continuation_calls, 0);
                assert_eq!(challenge.upside_capture_options, 2);
                assert_eq!(challenge.max_upside_gain, 4);
            }
            _ => panic!("hwatu state should derive hanafuda challenge"),
        }
    }

    #[test]
    fn hanafuda_core_state_derives_stop_window_score() {
        let state = bootstrap_state(ResearchGame::HanafudaKoiKoi)
            .unwrap_or_else(|error| panic!("hanafuda core state should bootstrap: {error}"));
        let challenge = PortfolioChallenge::from_core_state(&state)
            .unwrap_or_else(|| panic!("hanafuda core state should derive challenge"));

        match challenge {
            PortfolioChallenge::HanafudaKoiKoi(challenge) => {
                assert_eq!(challenge.points, 7);
                assert_eq!(challenge.bright_count, 3);
                assert_eq!(challenge.ribbon_yaku, 2);
                assert_eq!(challenge.animal_yaku, 2);
                assert_eq!(challenge.bonus_cards, 1);
                assert_eq!(challenge.yaku_count, 3);
                assert_eq!(challenge.bright_capture_options, 0);
                assert_eq!(challenge.opponent_pressure, 2);
                assert_eq!(challenge.hand_count, 2);
                assert!(challenge.decision_window);
                assert_eq!(challenge.locked_points, 0);
                assert_eq!(challenge.continuation_calls, 0);
                assert_eq!(challenge.upside_capture_options, 1);
                assert_eq!(challenge.max_upside_gain, 2);
            }
            _ => panic!("hanafuda state should derive hanafuda challenge"),
        }
    }

    #[test]
    fn bridge_core_state_derives_follow_suit_pressure() {
        let state = bootstrap_state(ResearchGame::Bridge)
            .unwrap_or_else(|error| panic!("bridge core state should bootstrap: {error}"));
        let challenge = PortfolioChallenge::from_core_state(&state)
            .unwrap_or_else(|| panic!("bridge core state should derive challenge"));

        match challenge {
            PortfolioChallenge::Bridge(challenge) => {
                assert_eq!(challenge.cards_in_trick, 1);
                assert!(challenge.follow_suit_forced);
            }
            _ => panic!("bridge state should derive trick-taking challenge"),
        }
    }

    #[test]
    fn backgammon_core_state_derives_cube_phase_status() {
        let state = bootstrap_state(ResearchGame::Backgammon)
            .unwrap_or_else(|error| panic!("backgammon core state should bootstrap: {error}"));
        let challenge = PortfolioChallenge::from_core_state(&state)
            .unwrap_or_else(|| panic!("backgammon core state should derive challenge"));

        match challenge {
            PortfolioChallenge::Backgammon(challenge) => {
                assert_eq!(challenge.race_lead_pips, 14);
                assert_eq!(challenge.borne_off, 11);
                assert_eq!(challenge.cube_efficiency, 2);
                assert!(!challenge.has_contact);
                assert_eq!(challenge.bar_count, 0);
                assert!(challenge.bearoff_ready);
                assert!(challenge.cube_centered);
                assert!(!challenge.cube_owned_by_actor);
                assert!(challenge.facing_double);
                assert_eq!(challenge.move_options, 8);
                assert_eq!(challenge.off_moves, 8);
                assert_eq!(challenge.blot_count, 4);
                assert_eq!(challenge.home_board_points, 0);
                assert_eq!(challenge.prime_length, 0);
            }
            _ => panic!("backgammon state should derive backgammon challenge"),
        }
    }
}
