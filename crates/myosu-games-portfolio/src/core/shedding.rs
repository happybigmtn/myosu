use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::core::model::{CoreAction, CoreGameError, CoreGameState, CoreTransition};
use crate::game::ResearchGame;

const DOU_DI_ZHU_PLAY_PREFIX: &str = "dou-di-zhu.play.";
const PUSOY_DOS_PLAY_PREFIX: &str = "pusoy-dos.play.";
const TIEN_LEN_PLAY_PREFIX: &str = "tien-len.play.";

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SheddingVariant {
    DouDiZhu,
    PusoyDos,
    TienLen,
}

impl SheddingVariant {
    const fn game(self) -> ResearchGame {
        match self {
            Self::DouDiZhu => ResearchGame::DouDiZhu,
            Self::PusoyDos => ResearchGame::PusoyDos,
            Self::TienLen => ResearchGame::TienLen,
        }
    }

    const fn action_prefix(self) -> &'static str {
        match self {
            Self::DouDiZhu => DOU_DI_ZHU_PLAY_PREFIX,
            Self::PusoyDos => PUSOY_DOS_PLAY_PREFIX,
            Self::TienLen => TIEN_LEN_PLAY_PREFIX,
        }
    }

    const fn pass_action_id(self) -> &'static str {
        match self {
            Self::DouDiZhu => "dou-di-zhu.pass",
            Self::PusoyDos => "pusoy-dos.pass",
            Self::TienLen => "tien-len.pass",
        }
    }

    const fn player_count(self) -> u8 {
        match self {
            Self::DouDiZhu => 3,
            Self::PusoyDos | Self::TienLen => 4,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct SheddingPublicState {
    variant: SheddingVariant,
    current_lead: Option<Combination>,
    hands: Vec<Vec<u8>>,
    actor: u8,
    last_player: u8,
    pass_count: u8,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
enum CombinationClass {
    Single,
    Pair,
    Straight,
    Bomb,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Combination {
    class: CombinationClass,
    rank: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SheddingFeatureView {
    pub bomb_count: u8,
    pub control_combos: u8,
    pub low_singles: u8,
    pub on_lead: bool,
    pub opponents_min_cards: u8,
    pub danger_opponents: u8,
    pub next_actor_cards: u8,
    pub play_options: u8,
    pub finishing_plays: u8,
    pub bomb_only_escape: bool,
    pub forced_pass: bool,
    pub lead_rank_pressure: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SheddingAction {
    Pass,
    Play(Combination),
}

pub fn dou_di_zhu_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(SheddingPublicState {
        variant: SheddingVariant::DouDiZhu,
        current_lead: Some(Combination {
            class: CombinationClass::Single,
            rank: 8,
        }),
        hands: vec![
            vec![10, 10, 11, 12, 12, 13, 14],
            vec![3, 3, 3, 3, 5, 9],
            vec![4, 6, 6, 7, 8],
        ],
        actor: 1,
        last_player: 0,
        pass_count: 1,
    })
}

pub fn pusoy_dos_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(SheddingPublicState {
        variant: SheddingVariant::PusoyDos,
        current_lead: Some(Combination {
            class: CombinationClass::Straight,
            rank: 8,
        }),
        hands: vec![
            vec![3, 4, 4, 5, 5, 6, 10, 12, 14],
            vec![6, 7, 8, 9, 10, 12, 13],
            vec![5, 6, 7, 8, 9, 11, 11, 13],
            vec![3, 3, 4, 8, 12, 13],
        ],
        actor: 2,
        last_player: 1,
        pass_count: 0,
    })
}

pub fn tien_len_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(SheddingPublicState {
        variant: SheddingVariant::TienLen,
        current_lead: Some(Combination {
            class: CombinationClass::Pair,
            rank: 11,
        }),
        hands: vec![
            vec![4, 4, 5, 5, 9, 10, 11, 14],
            vec![8, 8, 10, 11, 12, 13],
            vec![3, 3, 3, 3, 12, 12, 13],
            vec![6, 7, 7, 9, 12],
        ],
        actor: 2,
        last_player: 1,
        pass_count: 1,
    })
}

pub fn apply_dou_di_zhu_action(
    state: &CoreGameState,
    action_id: &str,
    params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    apply_variant_action(state, action_id, params, SheddingVariant::DouDiZhu)
}

/// One row of the canonical 22-scenario Dou-Di-Zhu benchmark pack.
///
/// The fields mirror the live `SheddingChallenge` feature struct so the
/// dossier's typed `PortfolioChallenge::DouDiZhu(SheddingChallenge { ... })`
/// is a field-for-field copy of the scenario. Keeping the mirror stable
/// means the dossier hash is also a regression guard for the
/// `feature_view` extraction: a refactor that changes how `feature_view`
/// reads `bomb_count` / `control_combos` / etc out of a `CoreGameState`
/// will flip the dossier's recommendation map and the unit tests will
/// fail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DouDiZhuScenario {
    pub scenario_id: &'static str,
    pub decision: &'static str,
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

/// Return the canonical 22-scenario Dou-Di-Zhu benchmark pack.
pub fn dou_di_zhu_scenario_pack() -> &'static [DouDiZhuScenario] {
    DOU_DI_ZHU_SCENARIO_PACK
}

const DOU_DI_ZHU_SCENARIO_PACK: &[DouDiZhuScenario] = &[
    // ---- preserve-bomb bucket (×8) — bomb available, no escape / forced-pass ----
    DouDiZhuScenario {
        scenario_id: "bomb-rich-mid-race",
        decision: "Two bombs mid-race, no danger, no finish — preserve-bomb dominates (pb=3.30, lb=1.90, sl=1.20)",
        bomb_count: 2,
        control_combos: 2,
        low_singles: 1,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 5,
        on_lead: false,
        play_options: 2,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 8,
    },
    DouDiZhuScenario {
        scenario_id: "bomb-bomb-only-just-once",
        decision: "One bomb with one shot left and a tiny finish line — preserve-bomb dominates (pb=2.20, lb=1.90, sl=1.90)",
        bomb_count: 1,
        control_combos: 2,
        low_singles: 1,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 5,
        on_lead: false,
        play_options: 2,
        finishing_plays: 1,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 8,
    },
    DouDiZhuScenario {
        scenario_id: "bomb-rich-no-finish",
        decision: "Two bombs with high lead rank pressure, no finish — preserve-bomb dominates (pb=3.45, lb=1.80, sl=1.10)",
        bomb_count: 2,
        control_combos: 2,
        low_singles: 1,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 4,
        on_lead: false,
        play_options: 1,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 12,
    },
    DouDiZhuScenario {
        scenario_id: "bomb-solo-high-pressure",
        decision: "Two bombs with one danger opponent, no finish — preserve-bomb dominates (pb=2.75, lb=1.80, sl=1.28)",
        bomb_count: 2,
        control_combos: 2,
        low_singles: 1,
        opponents_min_cards: 5,
        danger_opponents: 1,
        next_actor_cards: 5,
        on_lead: false,
        play_options: 2,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 10,
    },
    DouDiZhuScenario {
        scenario_id: "bomb-clean-mid-rank",
        decision: "One bomb at clean mid rank, next-actor still has cards — preserve-bomb dominates (pb=2.40, lb=1.45, sl=1.20)",
        bomb_count: 1,
        control_combos: 1,
        low_singles: 1,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 3,
        on_lead: false,
        play_options: 2,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 8,
    },
    DouDiZhuScenario {
        scenario_id: "bomb-rich-deep",
        decision: "Three bombs deep stack on the lead — preserve-bomb dominates (pb=4.20, lb=2.10, sl=1.20)",
        bomb_count: 3,
        control_combos: 2,
        low_singles: 1,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 4,
        on_lead: true,
        play_options: 1,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 8,
    },
    DouDiZhuScenario {
        scenario_id: "bomb-early-soft",
        decision: "One bomb early-soft, no danger — preserve-bomb dominates (pb=2.40, lb=1.90, sl=1.20)",
        bomb_count: 1,
        control_combos: 2,
        low_singles: 1,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 5,
        on_lead: false,
        play_options: 2,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 4,
    },
    DouDiZhuScenario {
        scenario_id: "bomb-late-bomb-only",
        decision: "One bomb with one option and high lead pressure — preserve-bomb dominates (pb=2.55, lb=1.35, sl=1.10)",
        bomb_count: 1,
        control_combos: 1,
        low_singles: 1,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 5,
        on_lead: false,
        play_options: 1,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 10,
    },
    // ---- landlord-bid bucket (×8) — no bombs, on the lead, control-rich ----
    DouDiZhuScenario {
        scenario_id: "landlord-rich-controls",
        decision: "Four control combos on the lead, no danger — landlord-bid dominates (pb=1.50, lb=3.20, sl=0.90)",
        bomb_count: 0,
        control_combos: 4,
        low_singles: 0,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 5,
        on_lead: true,
        play_options: 3,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 4,
    },
    DouDiZhuScenario {
        scenario_id: "landlord-mid-controls",
        decision: "Two control combos on the lead, two options — landlord-bid dominates (pb=1.25, lb=2.20, sl=0.90)",
        bomb_count: 0,
        control_combos: 2,
        low_singles: 0,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 5,
        on_lead: true,
        play_options: 2,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 4,
    },
    DouDiZhuScenario {
        scenario_id: "landlord-rich-options",
        decision: "Three control combos and four play options on the lead — landlord-bid dominates (pb=1.25, lb=2.85, sl=0.90)",
        bomb_count: 0,
        control_combos: 3,
        low_singles: 0,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 5,
        on_lead: true,
        play_options: 4,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 6,
    },
    DouDiZhuScenario {
        scenario_id: "landlord-rich-no-double",
        decision: "Three control combos on the lead, mid rank — landlord-bid dominates (pb=1.25, lb=2.75, sl=0.90)",
        bomb_count: 0,
        control_combos: 3,
        low_singles: 0,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 5,
        on_lead: true,
        play_options: 3,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 8,
    },
    DouDiZhuScenario {
        scenario_id: "landlord-mid-pressure",
        decision: "Two control combos on the lead, mid rank — landlord-bid dominates (pb=1.25, lb=2.20, sl=0.90)",
        bomb_count: 0,
        control_combos: 2,
        low_singles: 0,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 5,
        on_lead: true,
        play_options: 2,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 8,
    },
    DouDiZhuScenario {
        scenario_id: "landlord-bid-rich-combo",
        decision: "Five control combos on the lead — landlord-bid dominates (pb=1.25, lb=3.65, sl=0.90)",
        bomb_count: 0,
        control_combos: 5,
        low_singles: 0,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 5,
        on_lead: true,
        play_options: 3,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 4,
    },
    DouDiZhuScenario {
        scenario_id: "landlord-clean",
        decision: "Two control combos on the lead with next-actor at 4 — landlord-bid dominates (pb=1.25, lb=2.20, sl=0.90)",
        bomb_count: 0,
        control_combos: 2,
        low_singles: 0,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 4,
        on_lead: true,
        play_options: 2,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 4,
    },
    DouDiZhuScenario {
        scenario_id: "landlord-deep-stack",
        decision: "Three control combos on the lead, low rank — landlord-bid dominates (pb=1.25, lb=2.75, sl=0.90)",
        bomb_count: 0,
        control_combos: 3,
        low_singles: 0,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 5,
        on_lead: true,
        play_options: 3,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 2,
    },
    // ---- shed-lowest bucket (×6) — no bombs, on-lead off, finishing line OR many low singles ----
    DouDiZhuScenario {
        scenario_id: "shed-finishing-line",
        decision: "Two finishing plays and a single play option — shed-lowest dominates (pb=1.10, lb=0.90, sl=2.30)",
        bomb_count: 0,
        control_combos: 0,
        low_singles: 0,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 5,
        on_lead: false,
        play_options: 1,
        finishing_plays: 2,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 8,
    },
    DouDiZhuScenario {
        scenario_id: "shed-many-low-singles",
        decision: "Two low singles plus a danger opponent — shed-lowest dominates (pb=0.80, lb=0.90, sl=1.68)",
        bomb_count: 0,
        control_combos: 0,
        low_singles: 2,
        opponents_min_cards: 5,
        danger_opponents: 1,
        next_actor_cards: 5,
        on_lead: false,
        play_options: 2,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 8,
    },
    DouDiZhuScenario {
        scenario_id: "shed-next-actor-short",
        decision: "One finishing play with next-actor at 2 cards — shed-lowest dominates (pb=0.90, lb=0.80, sl=1.85)",
        bomb_count: 0,
        control_combos: 0,
        low_singles: 0,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 2,
        on_lead: false,
        play_options: 2,
        finishing_plays: 1,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 8,
    },
    DouDiZhuScenario {
        scenario_id: "shed-low-singles-rich",
        decision: "Three low singles and three play options — shed-lowest dominates (pb=1.25, lb=1.10, sl=1.80)",
        bomb_count: 0,
        control_combos: 0,
        low_singles: 3,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 5,
        on_lead: false,
        play_options: 3,
        finishing_plays: 0,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 8,
    },
    DouDiZhuScenario {
        scenario_id: "shed-deep-exit",
        decision: "Three finishing plays and one play option — shed-lowest dominates (pb=0.90, lb=0.90, sl=3.00)",
        bomb_count: 0,
        control_combos: 0,
        low_singles: 0,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 5,
        on_lead: false,
        play_options: 1,
        finishing_plays: 3,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 8,
    },
    DouDiZhuScenario {
        scenario_id: "shed-double-finish",
        decision: "Two finishing plays and one play option, low rank — shed-lowest dominates (pb=0.95, lb=0.90, sl=2.40)",
        bomb_count: 0,
        control_combos: 0,
        low_singles: 0,
        opponents_min_cards: 5,
        danger_opponents: 0,
        next_actor_cards: 5,
        on_lead: false,
        play_options: 1,
        finishing_plays: 2,
        bomb_only_escape: false,
        forced_pass: false,
        lead_rank_pressure: 4,
    },
];

pub fn apply_pusoy_dos_action(
    state: &CoreGameState,
    action_id: &str,
    params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    apply_variant_action(state, action_id, params, SheddingVariant::PusoyDos)
}

pub fn apply_tien_len_action(
    state: &CoreGameState,
    action_id: &str,
    params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    apply_variant_action(state, action_id, params, SheddingVariant::TienLen)
}

fn apply_variant_action(
    state: &CoreGameState,
    action_id: &str,
    _params: serde_json::Value,
    variant: SheddingVariant,
) -> Result<CoreTransition, CoreGameError> {
    let before_public: SheddingPublicState = serde_json::from_value(state.public_state.clone())
        .map_err(|source| CoreGameError::InvalidParams {
            action_id: action_id.to_string(),
            reason: source.to_string(),
        })?;
    if before_public.variant != variant {
        return Err(invalid_shedding_state(
            variant,
            action_id,
            "state variant does not match shedding dispatch target",
        ));
    }
    let action = parse_shedding_action(variant, action_id)?;
    if let SheddingAction::Play(combination) = action {
        validate_play_combination(&before_public, combination, action_id)?;
    }
    if !state
        .legal_actions
        .iter()
        .any(|candidate| candidate.action_id == action_id)
    {
        return Err(illegal_shedding_action(
            variant,
            action_id,
            "action is not legal in this climbing round",
        ));
    }

    let mut after_public = before_public.clone();
    match action {
        SheddingAction::Pass => {
            after_public.pass_count = after_public.pass_count.saturating_add(1);
            if after_public.pass_count >= after_public.variant.player_count().saturating_sub(1) {
                after_public.actor = after_public.last_player;
                after_public.current_lead = None;
                after_public.pass_count = 0;
            } else {
                after_public.actor = next_actor(after_public.variant, after_public.actor);
            }
        }
        SheddingAction::Play(combination) => {
            let actor_index = usize::from(after_public.actor);
            let Some(actor_hand) = after_public.hands.get_mut(actor_index) else {
                return Err(invalid_shedding_state(
                    variant,
                    action_id,
                    "actor hand is missing from seat hand table",
                ));
            };
            remove_combination_cards(actor_hand, combination);
            let terminal = actor_hand.is_empty();
            after_public.current_lead = Some(combination);
            after_public.pass_count = 0;
            after_public.last_player = after_public.actor;
            after_public.actor = next_actor(after_public.variant, after_public.actor);

            let payoff = terminal.then(|| terminal_payoff(after_public.variant));
            let after = state_from_public_with_terminal(after_public, terminal, payoff)?;
            let action = core_action_for_shedding(variant, action);

            return Ok(CoreTransition {
                before: state.clone(),
                action,
                after,
            });
        }
    }

    let terminal = false;
    let payoff = terminal.then(|| terminal_payoff(after_public.variant));
    let after = state_from_public_with_terminal(after_public, terminal, payoff)?;
    let action = core_action_for_shedding(variant, action);

    Ok(CoreTransition {
        before: state.clone(),
        action,
        after,
    })
}

fn state_from_public(public: SheddingPublicState) -> Result<CoreGameState, CoreGameError> {
    state_from_public_with_terminal(public, false, None)
}

fn state_from_public_with_terminal(
    public: SheddingPublicState,
    terminal: bool,
    payoff: Option<Vec<i64>>,
) -> Result<CoreGameState, CoreGameError> {
    validate_shedding_state(&public, "shedding.bootstrap")?;
    let variant = public.variant;
    let actor = (!terminal).then_some(public.actor);
    let legal_actions = if terminal {
        Vec::new()
    } else {
        legal_shedding_actions(&public)
            .into_iter()
            .map(|action| core_action_for_shedding(variant, action))
            .collect()
    };
    let public_state =
        serde_json::to_value(public).map_err(|source| CoreGameError::InvalidParams {
            action_id: format!("{}.bootstrap", variant.game().slug()),
            reason: source.to_string(),
        })?;

    Ok(CoreGameState {
        game: variant.game(),
        phase: "climbing".to_string(),
        actor,
        public_state,
        private_state_commitments: vec![format!(
            "{}.other-hands.bootstrap-v1",
            variant.game().slug()
        )],
        legal_actions,
        terminal,
        payoff,
    })
}

pub(crate) fn feature_view(state: &CoreGameState) -> Result<SheddingFeatureView, CoreGameError> {
    let public: SheddingPublicState =
        serde_json::from_value(state.public_state.clone()).map_err(|source| {
            CoreGameError::InvalidParams {
                action_id: format!("{}.feature-view", state.game.slug()),
                reason: source.to_string(),
            }
        })?;
    validate_shedding_state(&public, "shedding.feature-view")?;
    let legal_actions = legal_shedding_actions(&public);
    let play_actions = legal_actions
        .iter()
        .filter_map(|action| match action {
            SheddingAction::Play(combination) => Some(*combination),
            SheddingAction::Pass => None,
        })
        .collect::<Vec<_>>();
    let play_options = play_actions.len();
    let actor_index = usize::from(public.actor);
    let Some(actor_hand) = actor_hand(&public) else {
        return Err(invalid_shedding_state(
            public.variant,
            "shedding.feature-view",
            "actor hand is missing from seat hand table",
        ));
    };
    let opponent_sizes = public
        .hands
        .iter()
        .enumerate()
        .filter_map(|(index, hand)| (index != actor_index).then_some(usize_to_u8(hand.len())))
        .collect::<Vec<_>>();
    let next_actor_cards = public
        .hands
        .get(usize::from(next_actor(public.variant, public.actor)))
        .map(|hand| usize_to_u8(hand.len()))
        .unwrap_or(0);
    let finishing_plays = play_actions
        .iter()
        .filter(|combination| {
            actor_hand
                .len()
                .saturating_sub(combination_card_count(**combination))
                <= 1
        })
        .count();
    let bomb_only_escape = play_options > 0
        && play_actions
            .iter()
            .all(|combination| combination.class == CombinationClass::Bomb);

    Ok(SheddingFeatureView {
        bomb_count: usize_to_u8(
            play_actions
                .iter()
                .filter(|combination| combination.class == CombinationClass::Bomb)
                .count(),
        ),
        control_combos: usize_to_u8(
            play_actions
                .iter()
                .filter(|combination| match combination.class {
                    CombinationClass::Single => combination.rank >= 12,
                    CombinationClass::Pair
                    | CombinationClass::Straight
                    | CombinationClass::Bomb => true,
                })
                .count(),
        ),
        low_singles: usize_to_u8(actor_hand.iter().filter(|rank| **rank <= 5).count()),
        on_lead: public.current_lead.is_none(),
        opponents_min_cards: opponent_sizes.iter().copied().min().unwrap_or(0),
        danger_opponents: usize_to_u8(opponent_sizes.iter().filter(|size| **size <= 3).count()),
        next_actor_cards,
        play_options: usize_to_u8(play_options),
        finishing_plays: usize_to_u8(finishing_plays),
        bomb_only_escape,
        forced_pass: play_options == 0,
        lead_rank_pressure: public.current_lead.map_or(0, |lead| {
            let class_bonus = match lead.class {
                CombinationClass::Bomb => 2_u8,
                CombinationClass::Straight => 1_u8,
                CombinationClass::Single | CombinationClass::Pair => 0_u8,
            };
            lead.rank.saturating_add(class_bonus)
        }),
    })
}

fn legal_shedding_actions(public: &SheddingPublicState) -> Vec<SheddingAction> {
    let mut actions = vec![SheddingAction::Pass];
    let Some(actor_hand) = actor_hand(public) else {
        return actions;
    };
    for combination in candidate_combinations(actor_hand) {
        if beats_current_lead(public.current_lead, combination) {
            actions.push(SheddingAction::Play(combination));
        }
    }

    actions
}

fn candidate_combinations(hand: &[u8]) -> Vec<Combination> {
    let mut ranks = hand.to_vec();
    ranks.sort_unstable();
    ranks.dedup();
    let mut combinations = Vec::new();
    for rank in &ranks {
        let count = hand.iter().filter(|candidate| **candidate == *rank).count();
        combinations.push(Combination {
            class: CombinationClass::Single,
            rank: *rank,
        });
        if count >= 2 {
            combinations.push(Combination {
                class: CombinationClass::Pair,
                rank: *rank,
            });
        }
        if count >= 4 {
            combinations.push(Combination {
                class: CombinationClass::Bomb,
                rank: *rank,
            });
        }
    }

    for window in ranks.windows(5) {
        if is_consecutive(window) {
            let Some(high) = window.last().copied() else {
                continue;
            };
            combinations.push(Combination {
                class: CombinationClass::Straight,
                rank: high,
            });
        }
    }

    combinations
}

fn is_consecutive(window: &[u8]) -> bool {
    if window.len() < 2 {
        return false;
    }
    for pair in window.windows(2) {
        let Some(next) = pair.first().copied().and_then(|value| value.checked_add(1)) else {
            return false;
        };
        if pair.get(1).copied() != Some(next) {
            return false;
        }
    }
    true
}

fn beats_current_lead(current: Option<Combination>, candidate: Combination) -> bool {
    let Some(current) = current else {
        return true;
    };
    if candidate.class == CombinationClass::Bomb && current.class != CombinationClass::Bomb {
        return true;
    }
    candidate.class == current.class && candidate.rank > current.rank
}

fn validate_shedding_state(
    public: &SheddingPublicState,
    action_id: &str,
) -> Result<(), CoreGameError> {
    if public.hands.len() != usize::from(public.variant.player_count()) {
        return Err(invalid_shedding_state(
            public.variant,
            action_id,
            "seat hand table does not match player count",
        ));
    }
    if public.actor >= public.variant.player_count()
        || public.last_player >= public.variant.player_count()
    {
        return Err(invalid_shedding_state(
            public.variant,
            action_id,
            "actor and last-player must fit inside the variant player count",
        ));
    }
    Ok(())
}

fn validate_play_combination(
    public: &SheddingPublicState,
    combination: Combination,
    action_id: &str,
) -> Result<(), CoreGameError> {
    validate_shedding_state(public, action_id)?;
    let Some(actor_hand) = actor_hand(public) else {
        return Err(invalid_shedding_state(
            public.variant,
            action_id,
            "actor hand is missing from seat hand table",
        ));
    };
    if !hand_contains_combination(actor_hand, combination) {
        return Err(illegal_shedding_action(
            public.variant,
            action_id,
            "actor hand does not contain the played combination",
        ));
    }
    if !beats_current_lead(public.current_lead, combination) {
        return Err(illegal_shedding_action(
            public.variant,
            action_id,
            "played combination does not beat the current lead",
        ));
    }

    Ok(())
}

fn hand_contains_combination(hand: &[u8], combination: Combination) -> bool {
    match combination.class {
        CombinationClass::Single => hand.contains(&combination.rank),
        CombinationClass::Pair => {
            hand.iter()
                .filter(|candidate| **candidate == combination.rank)
                .count()
                >= 2
        }
        CombinationClass::Bomb => {
            hand.iter()
                .filter(|candidate| **candidate == combination.rank)
                .count()
                >= 4
        }
        CombinationClass::Straight => {
            for offset in 0..5 {
                let Some(needed) = combination
                    .rank
                    .checked_sub(4)
                    .and_then(|base| base.checked_add(offset))
                else {
                    return false;
                };
                if !hand.contains(&needed) {
                    return false;
                }
            }
            true
        }
    }
}

fn remove_combination_cards(hand: &mut Vec<u8>, combination: Combination) {
    match combination.class {
        CombinationClass::Single | CombinationClass::Pair | CombinationClass::Bomb => {
            let mut remaining = combination_card_count(combination);
            hand.retain(|candidate| {
                if *candidate == combination.rank && remaining > 0 {
                    remaining = remaining.saturating_sub(1);
                    false
                } else {
                    true
                }
            });
        }
        CombinationClass::Straight => {
            for offset in 0..5 {
                let Some(needed) = combination
                    .rank
                    .checked_sub(4)
                    .and_then(|base| base.checked_add(offset))
                else {
                    return;
                };
                let Some(position) = hand.iter().position(|candidate| *candidate == needed) else {
                    return;
                };
                hand.remove(position);
            }
        }
    }
}

fn combination_card_count(combination: Combination) -> usize {
    match combination.class {
        CombinationClass::Single => 1,
        CombinationClass::Pair => 2,
        CombinationClass::Bomb => 4,
        CombinationClass::Straight => 5,
    }
}

fn actor_hand(public: &SheddingPublicState) -> Option<&[u8]> {
    public
        .hands
        .get(usize::from(public.actor))
        .map(Vec::as_slice)
}

fn core_action_for_shedding(variant: SheddingVariant, action: SheddingAction) -> CoreAction {
    match action {
        SheddingAction::Pass => CoreAction {
            action_id: variant.pass_action_id().to_string(),
            display_label: "pass".to_string(),
            params: json!({}),
        },
        SheddingAction::Play(combination) => CoreAction {
            action_id: format!(
                "{}{}-{}",
                variant.action_prefix(),
                class_token(combination.class),
                combination.rank
            ),
            display_label: format!(
                "play-{}-{}",
                class_token(combination.class),
                combination.rank
            ),
            params: json!({"class": class_token(combination.class), "rank": combination.rank}),
        },
    }
}

fn parse_shedding_action(
    variant: SheddingVariant,
    action_id: &str,
) -> Result<SheddingAction, CoreGameError> {
    if action_id == variant.pass_action_id() {
        return Ok(SheddingAction::Pass);
    }
    let Some(play_token) = action_id.strip_prefix(variant.action_prefix()) else {
        return Err(CoreGameError::UnknownAction {
            game: variant.game(),
            action_id: action_id.to_string(),
        });
    };
    let Some((class, rank)) = play_token.split_once('-') else {
        return Err(CoreGameError::UnknownAction {
            game: variant.game(),
            action_id: action_id.to_string(),
        });
    };
    let class = parse_class(variant, action_id, class)?;
    let rank = rank
        .parse::<u8>()
        .map_err(|_| CoreGameError::UnknownAction {
            game: variant.game(),
            action_id: action_id.to_string(),
        })?;

    Ok(SheddingAction::Play(Combination { class, rank }))
}

fn parse_class(
    variant: SheddingVariant,
    action_id: &str,
    class: &str,
) -> Result<CombinationClass, CoreGameError> {
    match class {
        "single" => Ok(CombinationClass::Single),
        "pair" => Ok(CombinationClass::Pair),
        "straight" => Ok(CombinationClass::Straight),
        "bomb" => Ok(CombinationClass::Bomb),
        _ => Err(CoreGameError::UnknownAction {
            game: variant.game(),
            action_id: action_id.to_string(),
        }),
    }
}

fn class_token(class: CombinationClass) -> &'static str {
    match class {
        CombinationClass::Single => "single",
        CombinationClass::Pair => "pair",
        CombinationClass::Straight => "straight",
        CombinationClass::Bomb => "bomb",
    }
}

fn usize_to_u8(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

fn next_actor(variant: SheddingVariant, actor: u8) -> u8 {
    let next = actor.saturating_add(1);
    if next >= variant.player_count() {
        0
    } else {
        next
    }
}

fn terminal_payoff(variant: SheddingVariant) -> Vec<i64> {
    match variant {
        SheddingVariant::DouDiZhu => vec![1, -1, 0],
        SheddingVariant::PusoyDos | SheddingVariant::TienLen => vec![1, -1, 0, 0],
    }
}

fn illegal_shedding_action(
    variant: SheddingVariant,
    action_id: &str,
    reason: &str,
) -> CoreGameError {
    CoreGameError::IllegalAction {
        game: variant.game(),
        action_id: action_id.to_string(),
        reason: reason.to_string(),
    }
}

fn invalid_shedding_state(
    variant: SheddingVariant,
    action_id: &str,
    reason: &str,
) -> CoreGameError {
    CoreGameError::InvalidParams {
        action_id: format!("{}:{action_id}", variant.game().slug()),
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::core::model::{apply_action, bootstrap_state};

    #[test]
    fn shedding_bootstrap_state_has_legal_actions() {
        let state = shedding_state(ResearchGame::DouDiZhu);

        assert_eq!(state.game, ResearchGame::DouDiZhu);
        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "dou-di-zhu.play.single-9")
        );
    }

    #[test]
    fn shedding_two_passes_return_control() {
        let state = shedding_state(ResearchGame::DouDiZhu);
        let transition = apply_action(&state, "dou-di-zhu.pass", json!({}))
            .unwrap_or_else(|error| panic!("pass should apply: {error}"));
        let public: SheddingPublicState = serde_json::from_value(transition.after.public_state)
            .unwrap_or_else(|error| panic!("shedding public state should decode: {error}"));

        assert_eq!(public.actor, 0);
        assert_eq!(public.current_lead, None);
        assert_eq!(public.pass_count, 0);
    }

    #[test]
    fn pusoy_dos_supports_five_card_straight_control() {
        let state = shedding_state(ResearchGame::PusoyDos);

        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "pusoy-dos.play.straight-9")
        );
    }

    #[test]
    fn tien_len_bomb_beats_pair_pressure() {
        let state = shedding_state(ResearchGame::TienLen);

        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "tien-len.play.bomb-3")
        );
    }

    #[test]
    fn shedding_rejects_weaker_combination() {
        let state = shedding_state(ResearchGame::DouDiZhu);

        assert!(matches!(
            apply_action(&state, "dou-di-zhu.play.single-5", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("does not beat")
        ));
    }

    #[test]
    fn shedding_transition_is_deterministic() {
        let state = shedding_state(ResearchGame::DouDiZhu);
        let first = apply_action(&state, "dou-di-zhu.play.single-9", json!({}));
        let second = apply_action(&state, "dou-di-zhu.play.single-9", json!({}));

        assert_eq!(first, second);
    }

    #[test]
    fn shedding_feature_view_tracks_lead_reset_after_pass() {
        let state = shedding_state(ResearchGame::DouDiZhu);
        let contested_view = feature_view(&state)
            .unwrap_or_else(|error| panic!("contested shedding view should decode: {error}"));
        let reset_state = apply_action(&state, "dou-di-zhu.pass", json!({}))
            .unwrap_or_else(|error| panic!("pass should apply: {error}"))
            .after;
        let reset_view = feature_view(&reset_state)
            .unwrap_or_else(|error| panic!("reset shedding view should decode: {error}"));

        assert!(!contested_view.on_lead);
        assert!(reset_view.on_lead);
        assert!(reset_view.control_combos >= contested_view.control_combos);
        assert_eq!(contested_view.opponents_min_cards, 5);
        assert_eq!(contested_view.danger_opponents, 0);
        assert_eq!(contested_view.next_actor_cards, 5);
        assert_eq!(contested_view.play_options, 2);
        assert_eq!(contested_view.finishing_plays, 0);
        assert!(!contested_view.bomb_only_escape);
        assert!(!contested_view.forced_pass);
        assert_eq!(contested_view.lead_rank_pressure, 8);
        assert_eq!(reset_view.lead_rank_pressure, 0);
    }

    #[test]
    fn shedding_feature_view_detects_bomb_only_escape_and_finishing_line() {
        let state = state_from_public(SheddingPublicState {
            variant: SheddingVariant::TienLen,
            current_lead: Some(Combination {
                class: CombinationClass::Pair,
                rank: 13,
            }),
            hands: vec![vec![9], vec![5, 5, 6], vec![3, 3, 3, 3, 14], vec![7, 8]],
            actor: 2,
            last_player: 1,
            pass_count: 0,
        })
        .unwrap_or_else(|error| panic!("custom shedding state should build: {error}"));
        let view = feature_view(&state)
            .unwrap_or_else(|error| panic!("custom shedding view should decode: {error}"));

        assert_eq!(view.play_options, 1);
        assert_eq!(view.finishing_plays, 1);
        assert!(view.bomb_only_escape);
        assert_eq!(view.next_actor_cards, 2);
        assert!(!view.forced_pass);
    }

    #[test]
    fn shedding_pass_uses_next_seat_continuation_hand() {
        let state = shedding_state(ResearchGame::DouDiZhu);
        let reset_state = apply_action(&state, "dou-di-zhu.pass", json!({}))
            .unwrap_or_else(|error| panic!("pass should apply: {error}"))
            .after;

        assert!(
            reset_state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "dou-di-zhu.play.single-14")
        );
        assert!(
            reset_state
                .legal_actions
                .iter()
                .all(|action| action.action_id != "dou-di-zhu.play.bomb-3")
        );
    }

    fn shedding_state(game: ResearchGame) -> CoreGameState {
        bootstrap_state(game)
            .unwrap_or_else(|error| panic!("{} bootstrap should succeed: {error}", game.slug()))
    }
}

#[cfg(test)]
mod dou_di_zhu_scenario_pack_tests {
    use super::*;

    #[test]
    fn dou_di_zhu_scenario_pack_is_22_rows() {
        assert_eq!(dou_di_zhu_scenario_pack().len(), 22);
    }

    #[test]
    fn dou_di_zhu_scenario_ids_are_unique() {
        let pack = dou_di_zhu_scenario_pack();
        let mut seen = std::collections::HashSet::new();
        for scenario in pack {
            assert!(
                seen.insert(scenario.scenario_id),
                "duplicate scenario_id: {}",
                scenario.scenario_id
            );
        }
    }

    #[test]
    fn dou_di_zhu_scenario_buckets_have_expected_counts() {
        let pack = dou_di_zhu_scenario_pack();
        let bomb = pack
            .iter()
            .filter(|s| s.scenario_id.starts_with("bomb-"))
            .count();
        let landlord = pack
            .iter()
            .filter(|s| s.scenario_id.starts_with("landlord-"))
            .count();
        let shed = pack
            .iter()
            .filter(|s| s.scenario_id.starts_with("shed-"))
            .count();
        assert_eq!(bomb, 8, "preserve-bomb bucket should be 8");
        assert_eq!(landlord, 8, "landlord-bid bucket should be 8");
        assert_eq!(shed, 6, "shed-lowest bucket should be 6");
        assert_eq!(bomb + landlord + shed, 22);
    }

    #[test]
    fn dou_di_zhu_scenario_math_holds_for_every_row() {
        // The dossier's recommendation map is built from
        // `answer_typed_challenge` on the typed challenge, but the
        // pack's own `decision` docstring names the expected
        // `pb=…`/`lb=…`/`sl=…` values, so this test re-derives the
        // expected dominant action and confirms it matches the bucket
        // label. A regression that changes the engine's heuristic
        // math would flip the dominant action on at least one row
        // and this test would fail.
        fn dominant(scenario: &DouDiZhuScenario) -> &'static str {
            let preserve_bomb = 0.9_f32
                + (scenario.bomb_count as f32) * 0.90
                + if scenario.danger_opponents > 0 {
                    -0.10
                } else {
                    0.35
                }
                + if scenario.lead_rank_pressure >= 10 {
                    0.15
                } else {
                    0.0
                }
                + if scenario.play_options <= 1 {
                    0.25
                } else {
                    0.0
                }
                - (scenario.finishing_plays as f32) * 0.20
                - if scenario.bomb_only_escape { 0.60 } else { 0.0 }
                - if scenario.next_actor_cards <= 2 {
                    0.15
                } else {
                    0.0
                };
            let landlord_bid = 0.8_f32
                + (scenario.control_combos as f32) * 0.45
                + if scenario.on_lead { 0.30 } else { 0.0 }
                + (scenario.play_options as f32) * 0.10
                - if scenario.forced_pass { 0.40 } else { 0.0 }
                - (scenario.danger_opponents as f32) * 0.10
                - if scenario.next_actor_cards <= 2 {
                    0.20
                } else {
                    0.0
                };
            let shed_lowest = 0.8_f32
                + (scenario.low_singles as f32) * 0.30
                + if scenario.opponents_min_cards <= 2 {
                    0.25
                } else {
                    0.0
                }
                + (scenario.danger_opponents as f32) * 0.18
                + if scenario.lead_rank_pressure <= 8 {
                    0.10
                } else {
                    0.0
                }
                + (scenario.finishing_plays as f32) * 0.70
                + if scenario.next_actor_cards <= 2 {
                    0.25
                } else {
                    0.0
                }
                - if scenario.bomb_only_escape { 0.20 } else { 0.0 }
                - if scenario.forced_pass { 0.50 } else { 0.0 };
            if preserve_bomb >= landlord_bid && preserve_bomb >= shed_lowest {
                "preserve-bomb"
            } else if landlord_bid >= shed_lowest {
                "landlord-bid"
            } else {
                "shed-lowest"
            }
        }

        for scenario in dou_di_zhu_scenario_pack() {
            let derived = dominant(scenario);
            let expected = if scenario.scenario_id.starts_with("bomb-") {
                "preserve-bomb"
            } else if scenario.scenario_id.starts_with("landlord-") {
                "landlord-bid"
            } else {
                "shed-lowest"
            };
            assert_eq!(
                derived, expected,
                "scenario {} expected {} but engine math derives {}",
                scenario.scenario_id, expected, derived
            );
        }
    }
}
