use std::collections::BTreeMap;
use std::sync::Mutex;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use rbp_cards::{Hand, Strength};
use rbp_mccfr::Encounter;
use rbp_nlhe::{NlheEdge, NlheEncoder, NlheProfile};

use myosu_tui::GameRenderer;

use crate::action::{NlheAction, ParseActionError};
use crate::codexpoker::CodexpokerBlueprint;
use crate::request::NlheStrategyRequest;
use crate::robopoker::NlheBlueprint;
use crate::state::{NlheActor, NlhePlayerState, NlheSnapshot, NlheStreet, NlheTablePosition};

/// NLHE implementation of the shared TUI game-renderer contract.
pub struct NlheRenderer {
    mode: Mutex<NlheRendererMode>,
    advisor: NlheDemoAdvisor,
    declaration: Mutex<&'static str>,
    live_advice: Mutex<Option<LiveAdviceOverlay>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LiveAdviceOverlay {
    action: Option<String>,
    declaration: &'static str,
    status: LiveAdviceStatus,
    age_seconds: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LiveAdviceStatus {
    Fresh,
    Stale,
    Offline,
}

enum NlheRendererMode {
    Static(Option<NlheSnapshot>),
    Demo(NlheDemoState),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NlheDemoPhase {
    Preflop,
    Flop,
    Turn,
    River,
}

impl NlheDemoPhase {
    const fn street(self) -> NlheStreet {
        match self {
            Self::Preflop => NlheStreet::Preflop,
            Self::Flop => NlheStreet::Flop,
            Self::Turn => NlheStreet::Turn,
            Self::River => NlheStreet::River,
        }
    }

    const fn board_len(self) -> usize {
        match self {
            Self::Preflop => 0,
            Self::Flop => 3,
            Self::Turn => 4,
            Self::River => 5,
        }
    }

    const fn lower_label(self) -> &'static str {
        match self {
            Self::Preflop => "preflop",
            Self::Flop => "flop",
            Self::Turn => "turn",
            Self::River => "river",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NlheDemoResult {
    Folded(NlheDemoPhase),
    Showdown(NlheDemoWinner),
    VillainFolded,
}

struct NlheDemoState {
    hand_number: u32,
    phase: Option<NlheDemoPhase>,
    result: Option<NlheDemoResult>,
}

struct NlheDemoHand {
    hand_number: u32,
    hero_hole: [String; 2],
    villain_hole: [String; 2],
    board_runout: [String; 5],
}

enum NlheDemoAdvisor {
    Generated,
    Blueprint(NlheBlueprint),
    Codexpoker(CodexpokerBlueprint),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NlheDemoWinner {
    Hero,
    Villain,
    Split,
}

impl NlheDemoAdvisor {
    const fn label(&self) -> &'static str {
        match self {
            Self::Generated => "generated",
            Self::Blueprint(_) | Self::Codexpoker(_) => "artifact",
        }
    }
}

impl NlheRenderer {
    /// Create a renderer with an optional active hand snapshot.
    pub fn new(snapshot: Option<NlheSnapshot>) -> Self {
        let declaration = match &snapshot {
            Some(snapshot) => declaration_for(snapshot),
            None => "NO ACTIVE HAND",
        };

        Self {
            mode: Mutex::new(NlheRendererMode::Static(snapshot)),
            advisor: NlheDemoAdvisor::Generated,
            declaration: Mutex::new(declaration),
            live_advice: Mutex::new(None),
        }
    }

    /// Create a deterministic playable demo hand.
    pub fn demo() -> Self {
        Self::demo_with_generated_advice()
    }

    /// Create a demo hand with generated demo advice.
    pub fn demo_with_generated_advice() -> Self {
        Self {
            mode: Mutex::new(NlheRendererMode::Demo(NlheDemoState::new())),
            advisor: NlheDemoAdvisor::Generated,
            declaration: Mutex::new("GENERATED ADVICE: CALL"),
            live_advice: Mutex::new(None),
        }
    }

    /// Create a demo hand with artifact-backed blueprint advice.
    pub fn demo_with_blueprint(blueprint: NlheBlueprint) -> Self {
        Self {
            mode: Mutex::new(NlheRendererMode::Demo(NlheDemoState::new())),
            advisor: NlheDemoAdvisor::Blueprint(blueprint),
            declaration: Mutex::new("ARTIFACT ADVICE: CALL"),
            live_advice: Mutex::new(None),
        }
    }

    /// Create a demo hand with a codexpoker-backed mmap blueprint advisor.
    pub fn demo_with_codexpoker_blueprint(blueprint: CodexpokerBlueprint) -> Self {
        Self {
            mode: Mutex::new(NlheRendererMode::Demo(NlheDemoState::new())),
            advisor: NlheDemoAdvisor::Codexpoker(blueprint),
            declaration: Mutex::new("ARTIFACT ADVICE: CALL"),
            live_advice: Mutex::new(None),
        }
    }

    fn render_lines(&self) -> Vec<String> {
        let live_advice = self.live_advice_line();
        let mode = self.mode.lock().expect("renderer state poisoned");
        let mut lines = match &*mode {
            NlheRendererMode::Static(snapshot) => static_lines(snapshot.as_ref()),
            NlheRendererMode::Demo(demo) => demo.render_lines(),
        };
        if let Some(live_advice) = live_advice {
            lines.push(live_advice);
        }
        lines
    }

    /// Describe where live advice is currently coming from.
    pub fn advice_source(&self) -> &'static str {
        self.advisor.label()
    }

    /// Build the current strategy request for the active snapshot, if one exists.
    pub fn strategy_request(&self) -> Option<NlheStrategyRequest> {
        let mode = self.mode.lock().expect("renderer state poisoned");
        match &*mode {
            NlheRendererMode::Static(Some(snapshot)) => Some(strategy_request_for(snapshot)),
            NlheRendererMode::Static(None) => None,
            NlheRendererMode::Demo(demo) => demo.request(),
        }
    }

    /// Format the current pipe state with an injected recommendation label.
    pub fn pipe_output_with_recommendation(&self, recommendation: &str, advisor: &str) -> String {
        let mode = self.mode.lock().expect("renderer state poisoned");
        match &*mode {
            NlheRendererMode::Static(Some(snapshot)) => format!(
                "{} recommend={} advisor={advisor}",
                snapshot_pipe_output(snapshot),
                recommendation
            ),
            NlheRendererMode::Static(None) => "STATE idle actions=/quit".to_string(),
            NlheRendererMode::Demo(demo) => {
                demo.pipe_output_with_recommendation(recommendation, advisor)
            }
        }
    }

    /// Map a live strategy edge onto the current human-facing action label.
    pub fn recommendation_action(&self, edge: NlheEdge) -> Option<&'static str> {
        let mode = self.mode.lock().expect("renderer state poisoned");
        match &*mode {
            NlheRendererMode::Static(Some(snapshot)) => map_edge_to_action(snapshot, edge),
            NlheRendererMode::Static(None) => None,
            NlheRendererMode::Demo(demo) => demo
                .snapshot()
                .as_ref()
                .and_then(|snapshot| map_edge_to_action(snapshot, edge)),
        }
    }

    /// Update the cached live recommendation displayed in interactive mode.
    pub fn set_live_recommendation(&self, recommendation: &str) {
        self.set_live_recommendation_with_age(recommendation, 0);
    }

    /// Update the cached live recommendation with an explicit age in seconds.
    pub fn set_live_recommendation_with_age(&self, recommendation: &str, age_seconds: u64) {
        let mut overlay = self
            .live_advice
            .lock()
            .expect("renderer live advice poisoned");
        *overlay = Some(LiveAdviceOverlay {
            action: Some(recommendation.to_string()),
            declaration: live_declaration_for_recommendation(recommendation),
            status: LiveAdviceStatus::Fresh,
            age_seconds,
        });
    }

    /// Remove any live-advice overlay from the interactive renderer.
    pub fn clear_live_advice(&self) {
        let mut overlay = self
            .live_advice
            .lock()
            .expect("renderer live advice poisoned");
        *overlay = None;
    }

    /// Mark the last live recommendation as stale after a refresh failure.
    pub fn mark_live_recommendation_stale(&self) {
        self.mark_live_recommendation_stale_with_age(0);
    }

    /// Mark the last live recommendation as stale with an explicit age in seconds.
    pub fn mark_live_recommendation_stale_with_age(&self, age_seconds: u64) {
        let mut overlay = self
            .live_advice
            .lock()
            .expect("renderer live advice poisoned");
        let Some(current) = overlay.as_mut() else {
            *overlay = Some(LiveAdviceOverlay {
                action: None,
                declaration: "LIVE ADVICE OFFLINE",
                status: LiveAdviceStatus::Offline,
                age_seconds: 0,
            });
            return;
        };

        if let Some(action) = current.action.as_deref() {
            current.status = LiveAdviceStatus::Stale;
            current.declaration = stale_live_declaration_for_recommendation(action);
            current.age_seconds = age_seconds;
        } else {
            current.status = LiveAdviceStatus::Offline;
            current.declaration = "LIVE ADVICE OFFLINE";
            current.age_seconds = 0;
        }
    }

    /// Clear any stale state and show that live advice is currently offline.
    pub fn set_live_advice_offline(&self) {
        let mut overlay = self
            .live_advice
            .lock()
            .expect("renderer live advice poisoned");
        *overlay = Some(LiveAdviceOverlay {
            action: None,
            declaration: "LIVE ADVICE OFFLINE",
            status: LiveAdviceStatus::Offline,
            age_seconds: 0,
        });
    }

    fn live_advice_line(&self) -> Option<String> {
        let overlay = self
            .live_advice
            .lock()
            .expect("renderer live advice poisoned");
        overlay
            .as_ref()
            .map(|overlay| match (&overlay.action, overlay.status) {
                (Some(action), LiveAdviceStatus::Fresh) => {
                    format!(
                        "LIVE ADVICE {} [FRESH {}s]",
                        action.to_ascii_uppercase(),
                        overlay.age_seconds
                    )
                }
                (Some(action), LiveAdviceStatus::Stale) => {
                    format!(
                        "LIVE ADVICE {} [STALE {}s]",
                        action.to_ascii_uppercase(),
                        overlay.age_seconds
                    )
                }
                (_, LiveAdviceStatus::Offline) => "LIVE ADVICE OFFLINE".to_string(),
                (None, LiveAdviceStatus::Fresh | LiveAdviceStatus::Stale) => {
                    "LIVE ADVICE READY".to_string()
                }
            })
    }
}

impl GameRenderer for NlheRenderer {
    fn render_state(&self, area: Rect, buf: &mut Buffer) {
        let lines = self.render_lines();
        for (row, line) in lines.into_iter().enumerate() {
            let y = area.y + row as u16;
            if y >= area.bottom() {
                break;
            }

            for (column, ch) in line.chars().enumerate() {
                let x = area.x + column as u16;
                if x >= area.right() {
                    break;
                }
                buf[(x, y)].set_char(ch);
            }
        }
    }

    fn desired_height(&self, _width: u16) -> u16 {
        self.render_lines().len() as u16
    }

    fn declaration(&self) -> &str {
        if let Some(declaration) = self
            .live_advice
            .lock()
            .expect("renderer live advice poisoned")
            .as_ref()
            .map(|overlay| overlay.declaration)
        {
            return declaration;
        }
        *self
            .declaration
            .lock()
            .expect("renderer declaration poisoned")
    }

    fn completions(&self) -> Vec<String> {
        let mode = self.mode.lock().expect("renderer state poisoned");
        match &*mode {
            NlheRendererMode::Static(Some(snapshot)) => snapshot.action_labels(),
            NlheRendererMode::Static(None) => vec!["/quit".to_string()],
            NlheRendererMode::Demo(demo) => demo.completions(),
        }
    }

    fn parse_input(&self, input: &str) -> Option<String> {
        let mut mode = self.mode.lock().expect("renderer state poisoned");
        let action = match &mut *mode {
            NlheRendererMode::Static(_) => NlheAction::parse(input).ok()?.to_string(),
            NlheRendererMode::Demo(demo) => demo.apply_input(input)?,
        };

        let declaration = match &*mode {
            NlheRendererMode::Static(Some(snapshot)) => declaration_for(snapshot),
            NlheRendererMode::Static(None) => "NO ACTIVE HAND",
            NlheRendererMode::Demo(demo) => demo.declaration(&self.advisor),
        };
        *self
            .declaration
            .lock()
            .expect("renderer declaration poisoned") = declaration;

        Some(action)
    }

    fn clarify(&self, input: &str) -> Option<String> {
        match NlheAction::parse(input) {
            Err(ParseActionError::MissingAmount) => Some("raise or bet to how much?".to_string()),
            _ => None,
        }
    }

    fn pipe_output(&self) -> String {
        let mode = self.mode.lock().expect("renderer state poisoned");
        match &*mode {
            NlheRendererMode::Static(snapshot) => static_pipe_output(snapshot.as_ref()),
            NlheRendererMode::Demo(demo) => demo.pipe_output(&self.advisor),
        }
    }

    fn game_label(&self) -> &str {
        "NLHE-HU"
    }

    fn context_label(&self) -> String {
        let mode = self.mode.lock().expect("renderer state poisoned");
        match &*mode {
            NlheRendererMode::Static(Some(snapshot)) => snapshot.context_label(),
            NlheRendererMode::Static(None) => "NO HAND".to_string(),
            NlheRendererMode::Demo(demo) => demo.context_label(),
        }
    }
}

impl NlheDemoState {
    fn new() -> Self {
        Self {
            hand_number: 47,
            phase: Some(NlheDemoPhase::Preflop),
            result: None,
        }
    }

    fn declaration(&self, advisor: &NlheDemoAdvisor) -> &'static str {
        declaration_for_recommendation(self.recommended_action(advisor), advisor.label())
    }

    fn snapshot(&self) -> Option<NlheSnapshot> {
        let hand = self.hand();
        self.phase.map(|phase| hand.snapshot(phase))
    }

    fn completions(&self) -> Vec<String> {
        match self.snapshot() {
            Some(snapshot) => snapshot.action_labels(),
            None => vec!["new".to_string(), "/quit".to_string()],
        }
    }

    fn render_lines(&self) -> Vec<String> {
        if let Some(snapshot) = self.snapshot() {
            return snapshot_lines(&snapshot);
        }

        let Some(result) = self.result else {
            return vec!["idle".to_string()];
        };
        let hand = self.hand();

        match result {
            NlheDemoResult::Folded(phase) => vec![
                "SHOWDOWN".to_string(),
                format!("HERO FOLDS ON {}", phase.street().label()),
                format!("BOARD {}", hand.board_label(phase)),
                format!("HERO {} {}", hand.hero_hole[0], hand.hero_hole[1]),
                "VILLAIN SCOOPS THE POT".to_string(),
            ],
            NlheDemoResult::Showdown(winner) => vec![
                "SHOWDOWN".to_string(),
                format!("BOARD {}", hand.board_label(NlheDemoPhase::River)),
                format!("HERO {} {}", hand.hero_hole[0], hand.hero_hole[1]),
                format!("VILLAIN {} {}", hand.villain_hole[0], hand.villain_hole[1]),
                format!("HERO HAND {}", hand.pretty_strength(&hand.hero_hole)),
                format!("VILLAIN HAND {}", hand.pretty_strength(&hand.villain_hole)),
                format!("RESULT {}", winner.label()),
            ],
            NlheDemoResult::VillainFolded => vec![
                "SHOWDOWN".to_string(),
                "VILLAIN FOLDS TO THE RIVER JAM".to_string(),
                format!("BOARD {}", hand.board_label(NlheDemoPhase::River)),
                format!("HERO {} {}", hand.hero_hole[0], hand.hero_hole[1]),
            ],
        }
    }

    fn pipe_output(&self, advisor: &NlheDemoAdvisor) -> String {
        if let Some(snapshot) = self.snapshot() {
            return format!(
                "{} recommend={}",
                snapshot_pipe_output(&snapshot),
                self.recommended_action(advisor)
            ) + &format!(" advisor={}", advisor.label());
        }

        let Some(result) = self.result else {
            return "STATE idle".to_string();
        };
        let hand = self.hand();

        match result {
            NlheDemoResult::Folded(phase) => {
                format!(
                    "STATE complete hand={} result=hero_folded street={} board={} hero_cards={} {} actions=new|/quit",
                    self.hand_number,
                    phase.lower_label(),
                    hand.board_label(phase),
                    hand.hero_hole[0],
                    hand.hero_hole[1]
                )
            }
            NlheDemoResult::Showdown(winner) => format!(
                "STATE complete hand={} result=showdown winner={} board={} hero_cards={} {} villain_cards={} {} hero_strength={} villain_strength={} actions=new|/quit",
                self.hand_number,
                winner.token(),
                hand.board_label(NlheDemoPhase::River),
                hand.hero_hole[0],
                hand.hero_hole[1],
                hand.villain_hole[0],
                hand.villain_hole[1],
                hand.pipe_strength(&hand.hero_hole),
                hand.pipe_strength(&hand.villain_hole)
            ),
            NlheDemoResult::VillainFolded => format!(
                "STATE complete hand={} result=villain_folded street=river board={} hero_cards={} {} actions=new|/quit",
                self.hand_number,
                hand.board_label(NlheDemoPhase::River),
                hand.hero_hole[0],
                hand.hero_hole[1]
            ),
        }
    }

    fn pipe_output_with_recommendation(&self, recommendation: &str, advisor: &str) -> String {
        if let Some(snapshot) = self.snapshot() {
            return format!(
                "{} recommend={} advisor={advisor}",
                snapshot_pipe_output(&snapshot),
                recommendation
            );
        }

        self.pipe_output(&NlheDemoAdvisor::Generated)
    }

    fn apply_input(&mut self, input: &str) -> Option<String> {
        if self.phase.is_none() {
            let normalized = input.trim().to_ascii_lowercase();
            if normalized == "new" || normalized == "again" {
                self.start_next_hand();
                return Some("new hand".to_string());
            }
            return None;
        }

        let snapshot = self.snapshot()?;
        let action = NlheAction::parse(input).ok()?;
        if !snapshot.legal_actions.iter().any(|legal| legal == &action) {
            return None;
        }

        match (self.phase, &action) {
            (_, NlheAction::Fold) => {
                let street = self.phase?;
                self.phase = None;
                self.result = Some(NlheDemoResult::Folded(street));
            }
            (Some(NlheDemoPhase::Preflop), _) => {
                self.phase = Some(NlheDemoPhase::Flop);
            }
            (Some(NlheDemoPhase::Flop), _) => {
                self.phase = Some(NlheDemoPhase::Turn);
            }
            (Some(NlheDemoPhase::Turn), _) => {
                self.phase = Some(NlheDemoPhase::River);
            }
            (Some(NlheDemoPhase::River), NlheAction::AllIn) => {
                self.phase = None;
                self.result = Some(NlheDemoResult::VillainFolded);
            }
            (Some(NlheDemoPhase::River), _) => {
                self.phase = None;
                self.result = Some(NlheDemoResult::Showdown(self.hand().showdown_winner()));
            }
            (None, _) => return None,
        }

        Some(action.to_string())
    }

    fn recommended_action(&self, advisor: &NlheDemoAdvisor) -> &'static str {
        let Some(snapshot) = self.snapshot() else {
            return "complete";
        };
        let Some(request) = self.request() else {
            return fallback_recommendation(snapshot.street);
        };

        let recommended = match advisor {
            NlheDemoAdvisor::Generated => {
                let Ok(info) = request.info() else {
                    return fallback_recommendation(snapshot.street);
                };
                let blueprint = demo_blueprint(&snapshot, &info);
                blueprint.recommend(info)
            }
            NlheDemoAdvisor::Blueprint(blueprint) => request
                .query_with_encoder(blueprint.encoder())
                .ok()
                .and_then(|query| blueprint.recommend_query(query)),
            NlheDemoAdvisor::Codexpoker(blueprint) => blueprint.recommend_request(&request),
        };

        let Some(edge) = recommended else {
            return fallback_recommendation(snapshot.street);
        };

        map_edge_to_action(&snapshot, edge)
            .unwrap_or_else(|| fallback_recommendation(snapshot.street))
    }

    fn start_next_hand(&mut self) {
        self.hand_number += 1;
        self.phase = Some(NlheDemoPhase::Preflop);
        self.result = None;
    }

    fn hand(&self) -> NlheDemoHand {
        NlheDemoHand::new(self.hand_number)
    }

    fn request(&self) -> Option<NlheStrategyRequest> {
        let snapshot = self.snapshot()?;
        Some(NlheStrategyRequest::from_snapshot(
            &snapshot,
            Vec::new(),
            demo_bucket(&snapshot),
        ))
    }

    fn context_label(&self) -> String {
        format!("HAND {}", self.hand_number)
    }
}

impl NlheDemoHand {
    fn new(hand_number: u32) -> Self {
        let cards = generated_cards(hand_number);
        Self {
            hand_number,
            hero_hole: [cards[0].clone(), cards[1].clone()],
            villain_hole: [cards[2].clone(), cards[3].clone()],
            board_runout: [
                cards[4].clone(),
                cards[5].clone(),
                cards[6].clone(),
                cards[7].clone(),
                cards[8].clone(),
            ],
        }
    }

    fn board_for(&self, phase: NlheDemoPhase) -> Vec<String> {
        self.board_runout[..phase.board_len()].to_vec()
    }

    fn board_label(&self, phase: NlheDemoPhase) -> String {
        let board = self.board_for(phase);
        if board.is_empty() {
            return "--".to_string();
        }
        board.join(" ")
    }

    fn snapshot(&self, phase: NlheDemoPhase) -> NlheSnapshot {
        match phase {
            NlheDemoPhase::Preflop => NlheSnapshot {
                hand_number: self.hand_number,
                street: NlheStreet::Preflop,
                pot_bb: 3,
                board: Vec::new(),
                hero_hole: self.hero_hole.clone(),
                action_on: NlheActor::Hero,
                to_call_bb: 1,
                min_raise_to_bb: Some(6),
                legal_actions: vec![
                    NlheAction::Fold,
                    NlheAction::Call,
                    NlheAction::RaiseTo { amount_bb: 6 },
                ],
                hero: player("Hero", NlheTablePosition::Button, 99, 1),
                villain: player("Villain", NlheTablePosition::BigBlind, 98, 2),
            },
            NlheDemoPhase::Flop => NlheSnapshot {
                hand_number: self.hand_number,
                street: NlheStreet::Flop,
                pot_bb: 12,
                board: self.board_for(NlheDemoPhase::Flop),
                hero_hole: self.hero_hole.clone(),
                action_on: NlheActor::Hero,
                to_call_bb: 4,
                min_raise_to_bb: Some(15),
                legal_actions: vec![
                    NlheAction::Fold,
                    NlheAction::Call,
                    NlheAction::RaiseTo { amount_bb: 15 },
                ],
                hero: player("Hero", NlheTablePosition::Button, 88, 4),
                villain: player("Villain", NlheTablePosition::BigBlind, 96, 8),
            },
            NlheDemoPhase::Turn => NlheSnapshot {
                hand_number: self.hand_number,
                street: NlheStreet::Turn,
                pot_bb: 27,
                board: self.board_for(NlheDemoPhase::Turn),
                hero_hole: self.hero_hole.clone(),
                action_on: NlheActor::Hero,
                to_call_bb: 9,
                min_raise_to_bb: Some(18),
                legal_actions: vec![
                    NlheAction::Fold,
                    NlheAction::Call,
                    NlheAction::RaiseTo { amount_bb: 18 },
                ],
                hero: player("Hero", NlheTablePosition::Button, 73, 9),
                villain: player("Villain", NlheTablePosition::BigBlind, 73, 18),
            },
            NlheDemoPhase::River => NlheSnapshot {
                hand_number: self.hand_number,
                street: NlheStreet::River,
                pot_bb: 45,
                board: self.board_for(NlheDemoPhase::River),
                hero_hole: self.hero_hole.clone(),
                action_on: NlheActor::Hero,
                to_call_bb: 0,
                min_raise_to_bb: Some(30),
                legal_actions: vec![
                    NlheAction::Check,
                    NlheAction::Bet { amount_bb: 30 },
                    NlheAction::AllIn,
                ],
                hero: player("Hero", NlheTablePosition::Button, 55, 0),
                villain: player("Villain", NlheTablePosition::BigBlind, 55, 0),
            },
        }
    }

    fn showdown_winner(&self) -> NlheDemoWinner {
        let (hero_strength, villain_strength) = self.showdown_strengths();

        match hero_strength.cmp(&villain_strength) {
            std::cmp::Ordering::Greater => NlheDemoWinner::Hero,
            std::cmp::Ordering::Less => NlheDemoWinner::Villain,
            std::cmp::Ordering::Equal => NlheDemoWinner::Split,
        }
    }

    fn showdown_strengths(&self) -> (Strength, Strength) {
        let hero = self
            .showdown_hand(&self.hero_hole)
            .expect("generated hero showdown hand should parse");
        let villain = self
            .showdown_hand(&self.villain_hole)
            .expect("generated villain showdown hand should parse");
        (Strength::from(hero), Strength::from(villain))
    }

    fn showdown_hand(&self, hole: &[String; 2]) -> Result<Hand, String> {
        let text = format!(
            "{}{}{}{}{}{}{}",
            hole[0],
            hole[1],
            self.board_runout[0],
            self.board_runout[1],
            self.board_runout[2],
            self.board_runout[3],
            self.board_runout[4]
        );
        Hand::try_from(text.as_str())
    }

    fn pretty_strength(&self, hole: &[String; 2]) -> String {
        let strength = self
            .showdown_hand(hole)
            .expect("generated showdown hand should parse");
        Strength::from(strength).to_string().trim().to_string()
    }

    fn pipe_strength(&self, hole: &[String; 2]) -> String {
        self.pretty_strength(hole)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("_")
    }
}

impl NlheDemoWinner {
    const fn label(self) -> &'static str {
        match self {
            Self::Hero => "HERO WINS",
            Self::Villain => "VILLAIN WINS",
            Self::Split => "POT SPLIT",
        }
    }

    const fn token(self) -> &'static str {
        match self {
            Self::Hero => "hero",
            Self::Villain => "villain",
            Self::Split => "split",
        }
    }
}

fn static_lines(snapshot: Option<&NlheSnapshot>) -> Vec<String> {
    match snapshot {
        Some(snapshot) => snapshot_lines(snapshot),
        None => Vec::new(),
    }
}

fn static_pipe_output(snapshot: Option<&NlheSnapshot>) -> String {
    match snapshot {
        Some(snapshot) => snapshot_pipe_output(snapshot),
        None => "STATE idle actions=/quit".to_string(),
    }
}

fn snapshot_lines(snapshot: &NlheSnapshot) -> Vec<String> {
    vec![
        format!(
            "{}  POT {}bb  ACTION {}",
            snapshot.street.label(),
            snapshot.pot_bb,
            snapshot.action_on.label()
        ),
        format!(
            "{} {}  stack={}bb committed={}bb",
            snapshot.hero.position.label(),
            snapshot.hero.label,
            snapshot.hero.stack_bb,
            snapshot.hero.committed_bb
        ),
        format!(
            "{} {}  stack={}bb committed={}bb",
            snapshot.villain.position.label(),
            snapshot.villain.label,
            snapshot.villain.stack_bb,
            snapshot.villain.committed_bb
        ),
        format!("BOARD {}", snapshot.board_label()),
        format!("HERO {} {}", snapshot.hero_hole[0], snapshot.hero_hole[1]),
        format!("TO CALL {}bb", snapshot.to_call_bb),
        action_prompt(snapshot),
    ]
}

fn snapshot_pipe_output(snapshot: &NlheSnapshot) -> String {
    let min_raise = snapshot
        .min_raise_to_bb
        .map(|amount| amount.to_string())
        .unwrap_or_else(|| "-".to_string());
    let mut actions = snapshot.action_labels();
    actions.push("/quit".to_string());

    format!(
        "STATE hand={} street={} pot_bb={} hero_cards={} {} board={} to_call_bb={} min_raise_to_bb={} actions={}",
        snapshot.hand_number,
        snapshot.street.label(),
        snapshot.pot_bb,
        snapshot.hero_hole[0],
        snapshot.hero_hole[1],
        snapshot.board_label(),
        snapshot.to_call_bb,
        min_raise,
        actions.join("|")
    )
}

fn strategy_request_for(snapshot: &NlheSnapshot) -> NlheStrategyRequest {
    NlheStrategyRequest::from_snapshot(snapshot, Vec::new(), demo_bucket(snapshot))
}

fn declaration_for(snapshot: &NlheSnapshot) -> &'static str {
    match snapshot.action_on {
        NlheActor::Hero => "THE SYSTEM AWAITS YOUR DECISION",
        NlheActor::Villain => "TRACKING OPPONENT ACTION",
    }
}

fn declaration_for_recommendation(action: &str, advisor: &str) -> &'static str {
    match (advisor, action) {
        ("generated", "call") => "GENERATED ADVICE: CALL",
        ("generated", "all-in") => "GENERATED ADVICE: ALL-IN",
        ("generated", "fold") => "GENERATED ADVICE: FOLD",
        ("generated", "check") => "GENERATED ADVICE: CHECK",
        ("generated", "raise 6") => "GENERATED ADVICE: RAISE 6",
        ("generated", "raise 15") => "GENERATED ADVICE: RAISE 15",
        ("generated", "raise 18") => "GENERATED ADVICE: RAISE 18",
        ("generated", "bet 30") => "GENERATED ADVICE: BET 30",
        ("artifact", "call") => "ARTIFACT ADVICE: CALL",
        ("artifact", "all-in") => "ARTIFACT ADVICE: ALL-IN",
        ("artifact", "fold") => "ARTIFACT ADVICE: FOLD",
        ("artifact", "check") => "ARTIFACT ADVICE: CHECK",
        ("artifact", "raise 6") => "ARTIFACT ADVICE: RAISE 6",
        ("artifact", "raise 15") => "ARTIFACT ADVICE: RAISE 15",
        ("artifact", "raise 18") => "ARTIFACT ADVICE: RAISE 18",
        ("artifact", "bet 30") => "ARTIFACT ADVICE: BET 30",
        _ => "TYPE NEW FOR NEXT HAND",
    }
}

fn live_declaration_for_recommendation(action: &str) -> &'static str {
    match action {
        "call" => "LIVE ADVICE: CALL",
        "all-in" => "LIVE ADVICE: ALL-IN",
        "fold" => "LIVE ADVICE: FOLD",
        "check" => "LIVE ADVICE: CHECK",
        "raise 6" => "LIVE ADVICE: RAISE 6",
        "raise 15" => "LIVE ADVICE: RAISE 15",
        "raise 18" => "LIVE ADVICE: RAISE 18",
        "bet 30" => "LIVE ADVICE: BET 30",
        _ => "LIVE ADVICE READY",
    }
}

fn stale_live_declaration_for_recommendation(action: &str) -> &'static str {
    match action {
        "call" => "LIVE ADVICE STALE: CALL",
        "all-in" => "LIVE ADVICE STALE: ALL-IN",
        "fold" => "LIVE ADVICE STALE: FOLD",
        "check" => "LIVE ADVICE STALE: CHECK",
        "raise 6" => "LIVE ADVICE STALE: RAISE 6",
        "raise 15" => "LIVE ADVICE STALE: RAISE 15",
        "raise 18" => "LIVE ADVICE STALE: RAISE 18",
        "bet 30" => "LIVE ADVICE STALE: BET 30",
        _ => "LIVE ADVICE STALE",
    }
}

fn action_prompt(snapshot: &NlheSnapshot) -> String {
    format!("ACTIONS {}", snapshot.action_labels().join(", "))
}

fn fallback_recommendation(street: NlheStreet) -> &'static str {
    match street {
        NlheStreet::Preflop => "call",
        NlheStreet::Flop => "call",
        NlheStreet::Turn => "call",
        NlheStreet::River => "all-in",
    }
}

fn demo_bucket(snapshot: &NlheSnapshot) -> i16 {
    let mut score = rank_score(&snapshot.hero_hole[0]) + rank_score(&snapshot.hero_hole[1]);
    for card in &snapshot.board {
        score += rank_score(card);
    }
    let street_offset = match snapshot.street {
        NlheStreet::Preflop => 11,
        NlheStreet::Flop => 22,
        NlheStreet::Turn => 33,
        NlheStreet::River => 44,
    };
    ((street_offset + score) % 97) as i16
}

fn demo_blueprint(snapshot: &NlheSnapshot, info: &rbp_nlhe::NlheInfo) -> NlheBlueprint {
    let preferred = preferred_edge_kind(snapshot);
    let choices = info
        .choices()
        .into_iter()
        .map(NlheEdge::from)
        .collect::<Vec<_>>();
    let resolved_preferred = choices
        .first()
        .copied()
        .map(|first| {
            if choices.iter().any(|edge| edge_kind(*edge) == preferred) {
                preferred
            } else {
                edge_kind(first)
            }
        })
        .unwrap_or(AdviceKind::Call);
    let preferred_count = choices
        .iter()
        .filter(|edge| edge_kind(**edge) == resolved_preferred)
        .count();
    let other_count = choices.len().saturating_sub(preferred_count);
    let preferred_weight = if preferred_count > 0 {
        0.80 / preferred_count as f32
    } else {
        0.0
    };
    let other_weight = if other_count > 0 {
        0.20 / other_count as f32
    } else {
        0.0
    };
    let encounters = BTreeMap::from([(
        *info,
        choices
            .into_iter()
            .map(|edge| {
                let weight = if edge_kind(edge) == resolved_preferred {
                    preferred_weight
                } else {
                    other_weight
                };
                (edge, Encounter::new(weight, 0.0, 0.0, 1))
            })
            .collect::<BTreeMap<_, _>>(),
    )]);

    let profile = NlheProfile {
        iterations: 1,
        encounters,
        metrics: Default::default(),
    };

    NlheBlueprint::new(NlheEncoder::default(), profile)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdviceKind {
    Fold,
    Check,
    Call,
    Aggro,
    Shove,
}

fn edge_kind(edge: NlheEdge) -> AdviceKind {
    let edge = rbp_gameplay::Edge::from(edge);
    if edge.is_shove() {
        return AdviceKind::Shove;
    }
    if edge.is_raise() {
        return AdviceKind::Aggro;
    }
    match edge {
        rbp_gameplay::Edge::Fold => AdviceKind::Fold,
        rbp_gameplay::Edge::Check => AdviceKind::Check,
        rbp_gameplay::Edge::Call => AdviceKind::Call,
        rbp_gameplay::Edge::Draw => AdviceKind::Call,
        rbp_gameplay::Edge::Open(_) | rbp_gameplay::Edge::Raise(_) | rbp_gameplay::Edge::Shove => {
            unreachable!("raise and shove edges return early")
        }
    }
}

fn preferred_edge_kind(snapshot: &NlheSnapshot) -> AdviceKind {
    match snapshot.street {
        NlheStreet::Preflop => {
            if is_pair(&snapshot.hero_hole) || broadway_count(&snapshot.hero_hole) == 2 {
                AdviceKind::Aggro
            } else if high_card_points(&snapshot.hero_hole) >= 18 || is_suited(&snapshot.hero_hole)
            {
                AdviceKind::Call
            } else {
                AdviceKind::Fold
            }
        }
        NlheStreet::Flop | NlheStreet::Turn => {
            if hero_pairs_board(&snapshot.hero_hole, &snapshot.board) || has_flush_draw(snapshot) {
                AdviceKind::Aggro
            } else if has_overcard_pressure(snapshot) {
                AdviceKind::Call
            } else {
                AdviceKind::Fold
            }
        }
        NlheStreet::River => {
            if hero_pairs_board(&snapshot.hero_hole, &snapshot.board) {
                AdviceKind::Shove
            } else if high_card_points(&snapshot.hero_hole) >= 20 {
                AdviceKind::Aggro
            } else {
                AdviceKind::Check
            }
        }
    }
}

fn map_edge_to_action(snapshot: &NlheSnapshot, edge: NlheEdge) -> Option<&'static str> {
    match edge_kind(edge) {
        AdviceKind::Fold => Some("fold"),
        AdviceKind::Check => {
            if snapshot
                .legal_actions
                .iter()
                .any(|action| matches!(action, NlheAction::Check))
            {
                Some("check")
            } else {
                Some("call")
            }
        }
        AdviceKind::Call => {
            if snapshot
                .legal_actions
                .iter()
                .any(|action| matches!(action, NlheAction::Call))
            {
                Some("call")
            } else {
                Some("check")
            }
        }
        AdviceKind::Aggro => snapshot
            .legal_actions
            .iter()
            .find_map(|action| match action {
                NlheAction::RaiseTo { amount_bb: 6 } => Some("raise 6"),
                NlheAction::RaiseTo { amount_bb: 15 } => Some("raise 15"),
                NlheAction::RaiseTo { amount_bb: 18 } => Some("raise 18"),
                NlheAction::Bet { amount_bb: 30 } => Some("bet 30"),
                NlheAction::AllIn => Some("all-in"),
                _ => None,
            }),
        AdviceKind::Shove => {
            if snapshot
                .legal_actions
                .iter()
                .any(|action| matches!(action, NlheAction::AllIn))
            {
                Some("all-in")
            } else {
                snapshot
                    .legal_actions
                    .iter()
                    .find_map(|action| match action {
                        NlheAction::RaiseTo { amount_bb: 6 } => Some("raise 6"),
                        NlheAction::RaiseTo { amount_bb: 15 } => Some("raise 15"),
                        NlheAction::RaiseTo { amount_bb: 18 } => Some("raise 18"),
                        NlheAction::Bet { amount_bb: 30 } => Some("bet 30"),
                        _ => None,
                    })
            }
        }
    }
}

fn player(
    label: &str,
    position: NlheTablePosition,
    stack_bb: u32,
    committed_bb: u32,
) -> NlhePlayerState {
    NlhePlayerState {
        label: label.to_string(),
        position,
        stack_bb,
        committed_bb,
    }
}

fn generated_cards(hand_number: u32) -> Vec<String> {
    const DECK: [&str; 52] = [
        "As", "Ah", "Ad", "Ac", "Ks", "Kh", "Kd", "Kc", "Qs", "Qh", "Qd", "Qc", "Js", "Jh", "Jd",
        "Jc", "Ts", "Th", "Td", "Tc", "9s", "9h", "9d", "9c", "8s", "8h", "8d", "8c", "7s", "7h",
        "7d", "7c", "6s", "6h", "6d", "6c", "5s", "5h", "5d", "5c", "4s", "4h", "4d", "4c", "3s",
        "3h", "3d", "3c", "2s", "2h", "2d", "2c",
    ];
    let start = (hand_number as usize * 7) % DECK.len();
    let step = 9;
    (0..9)
        .map(|offset| DECK[(start + offset * step) % DECK.len()].to_string())
        .collect()
}

fn rank_score(card: &str) -> u32 {
    match card.chars().next().unwrap_or('2') {
        'A' => 14,
        'K' => 13,
        'Q' => 12,
        'J' => 11,
        'T' => 10,
        '9' => 9,
        '8' => 8,
        '7' => 7,
        '6' => 6,
        '5' => 5,
        '4' => 4,
        '3' => 3,
        _ => 2,
    }
}

fn card_suit(card: &str) -> char {
    card.chars().nth(1).unwrap_or('s')
}

fn is_pair(hero_hole: &[String; 2]) -> bool {
    hero_hole[0].chars().next() == hero_hole[1].chars().next()
}

fn is_suited(hero_hole: &[String; 2]) -> bool {
    card_suit(&hero_hole[0]) == card_suit(&hero_hole[1])
}

fn broadway_count(hero_hole: &[String; 2]) -> usize {
    hero_hole
        .iter()
        .filter(|card| {
            matches!(
                card.chars().next().unwrap_or('2'),
                'A' | 'K' | 'Q' | 'J' | 'T'
            )
        })
        .count()
}

fn high_card_points(hero_hole: &[String; 2]) -> u32 {
    hero_hole.iter().map(|card| rank_score(card)).sum()
}

fn hero_pairs_board(hero_hole: &[String; 2], board: &[String]) -> bool {
    board.iter().any(|board_card| {
        let board_rank = board_card.chars().next();
        hero_hole
            .iter()
            .any(|hero_card| hero_card.chars().next() == board_rank)
    })
}

fn has_flush_draw(snapshot: &NlheSnapshot) -> bool {
    let mut suit_counts = BTreeMap::new();
    for suit in snapshot
        .hero_hole
        .iter()
        .chain(snapshot.board.iter())
        .map(|card| card_suit(card))
    {
        *suit_counts.entry(suit).or_insert(0_usize) += 1;
    }
    suit_counts.values().any(|count| *count >= 4)
}

fn has_overcard_pressure(snapshot: &NlheSnapshot) -> bool {
    let Some(highest_board_rank) = snapshot.board.iter().map(|card| rank_score(card)).max() else {
        return high_card_points(&snapshot.hero_hole) >= 20;
    };
    snapshot
        .hero_hole
        .iter()
        .any(|card| rank_score(card) > highest_board_rank)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_renderer_matches_tui_contract() {
        let renderer = NlheRenderer::new(Some(NlheDemoHand::new(47).snapshot(NlheDemoPhase::Turn)));

        assert_eq!(renderer.desired_height(80), 7);
        assert_eq!(renderer.declaration(), "THE SYSTEM AWAITS YOUR DECISION");
        assert_eq!(renderer.game_label(), "NLHE-HU");
        assert_eq!(renderer.context_label(), "HAND 47");
        assert_eq!(renderer.completions(), vec!["fold", "call", "raise 18"]);
    }

    #[test]
    fn render_state_writes_poker_lines() {
        let renderer = NlheRenderer::new(Some(NlheDemoHand::new(47).snapshot(NlheDemoPhase::Turn)));
        let mut buffer = Buffer::empty(Rect::new(0, 0, 60, 8));

        renderer.render_state(Rect::new(0, 0, 60, 8), &mut buffer);

        let first_line: String = (0..24)
            .map(|x| buffer[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(first_line.contains("TURN"));
        assert!(first_line.contains("POT 27bb"));
    }

    #[test]
    fn parse_input_normalizes_actions() {
        let renderer = NlheRenderer::new(Some(NlheDemoHand::new(47).snapshot(NlheDemoPhase::Turn)));

        assert_eq!(renderer.parse_input("f"), Some("fold".to_string()));
        assert_eq!(renderer.parse_input("bet 12"), Some("bet 12".to_string()));
        assert_eq!(renderer.parse_input("r 30"), Some("raise 30".to_string()));
        assert_eq!(renderer.parse_input("???"), None);
    }

    #[test]
    fn clarify_requires_sized_amounts() {
        let renderer = NlheRenderer::new(Some(NlheDemoHand::new(47).snapshot(NlheDemoPhase::Turn)));

        assert_eq!(
            renderer.clarify("raise"),
            Some("raise or bet to how much?".to_string())
        );
        assert_eq!(renderer.clarify("call"), None);
    }

    #[test]
    fn pipe_output_stays_plain_text() {
        let renderer = NlheRenderer::new(Some(NlheDemoHand::new(47).snapshot(NlheDemoPhase::Turn)));
        let output = renderer.pipe_output();

        assert!(output.starts_with("STATE "));
        assert!(!output.contains('\u{1b}'));
        assert!(output.contains("hand=47"));
        assert!(output.contains("actions=fold|call|raise 18"));
    }

    #[test]
    fn inactive_renderer_collapses() {
        let renderer = NlheRenderer::new(None);

        assert_eq!(renderer.desired_height(80), 0);
        assert_eq!(renderer.declaration(), "NO ACTIVE HAND");
        assert_eq!(renderer.completions(), vec!["/quit"]);
        assert_eq!(renderer.pipe_output(), "STATE idle actions=/quit");
    }

    #[test]
    fn demo_renderer_advances_through_streets() {
        let renderer = NlheRenderer::demo();

        assert!(renderer.pipe_output().contains("street=PREFLOP"));
        assert!(renderer.pipe_output().contains("recommend="));
        assert_eq!(renderer.declaration(), "GENERATED ADVICE: CALL");
        assert!(renderer.pipe_output().contains("advisor=generated"));
        assert_eq!(renderer.completions(), vec!["fold", "call", "raise 6"]);

        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert!(renderer.pipe_output().contains("street=FLOP"));

        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert!(renderer.pipe_output().contains("street=TURN"));

        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert!(renderer.pipe_output().contains("street=RIVER"));
    }

    #[test]
    fn demo_renderer_completes_hand() {
        let renderer = NlheRenderer::demo();

        assert_eq!(renderer.parse_input("raise 6"), Some("raise 6".to_string()));
        assert_eq!(
            renderer.parse_input("raise 15"),
            Some("raise 15".to_string())
        );
        assert_eq!(
            renderer.parse_input("raise 18"),
            Some("raise 18".to_string())
        );
        assert_eq!(renderer.parse_input("all-in"), Some("all-in".to_string()));

        assert_eq!(renderer.declaration(), "TYPE NEW FOR NEXT HAND");
        assert!(renderer.pipe_output().contains("STATE complete"));
        assert_eq!(renderer.completions(), vec!["new", "/quit"]);
    }

    #[test]
    fn demo_renderer_updates_declaration_for_river_jam() {
        let renderer = NlheRenderer::demo();

        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));

        assert_eq!(renderer.declaration(), "GENERATED ADVICE: ALL-IN");
        assert!(renderer.pipe_output().contains("recommend="));
    }

    #[test]
    fn demo_renderer_rejects_illegal_sizing_without_progress() {
        let renderer = NlheRenderer::demo();

        assert_eq!(renderer.parse_input("raise 7"), None);
        assert!(renderer.pipe_output().contains("street=PREFLOP"));
    }

    #[test]
    fn demo_renderer_starts_next_hand_after_completion() {
        let renderer = NlheRenderer::demo();

        assert_eq!(renderer.context_label(), "HAND 47");
        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert_eq!(renderer.parse_input("all-in"), Some("all-in".to_string()));

        assert!(renderer.pipe_output().contains("hand=47"));
        assert_eq!(renderer.parse_input("new"), Some("new hand".to_string()));
        assert!(renderer.declaration().starts_with("GENERATED ADVICE:"));
        assert_eq!(renderer.context_label(), "HAND 48");
        assert!(renderer.pipe_output().contains("hand=48"));
        assert!(renderer.pipe_output().contains("street=PREFLOP"));
    }

    #[test]
    fn demo_renderer_uses_query_backed_blueprint() {
        let renderer = NlheRenderer::demo();

        assert!(renderer.declaration().starts_with("GENERATED ADVICE:"));
        assert!(renderer.pipe_output().contains("recommend="));
        assert!(renderer.pipe_output().contains("advisor=generated"));
        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert!(renderer.declaration().starts_with("GENERATED ADVICE:"));
    }

    #[test]
    fn demo_renderer_generates_new_cards_for_next_hand() {
        let renderer = NlheRenderer::demo();
        let first_state = renderer.pipe_output();

        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert_eq!(renderer.parse_input("all-in"), Some("all-in".to_string()));
        assert_eq!(renderer.parse_input("new"), Some("new hand".to_string()));

        let next_state = renderer.pipe_output();
        assert_ne!(first_state, next_state);
        assert!(first_state.contains("hero_cards="));
        assert!(next_state.contains("hero_cards="));
    }

    #[test]
    fn demo_renderer_reports_real_showdown_winner() {
        let renderer = NlheRenderer::demo();

        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert_eq!(renderer.parse_input("call"), Some("call".to_string()));
        assert_eq!(renderer.parse_input("check"), Some("check".to_string()));

        let output = renderer.pipe_output();
        assert!(output.contains("result=showdown"));
        assert!(output.contains("winner="));
        assert!(output.contains("hero_strength="));
        assert!(output.contains("villain_strength="));
        assert!(
            output.contains("winner=hero")
                || output.contains("winner=villain")
                || output.contains("winner=split")
        );
    }

    #[test]
    fn artifact_renderer_labels_advice_source() {
        let info = NlheStrategyRequest::from_snapshot(
            &NlheDemoHand::new(47).snapshot(NlheDemoPhase::Preflop),
            Vec::new(),
            demo_bucket(&NlheDemoHand::new(47).snapshot(NlheDemoPhase::Preflop)),
        )
        .info()
        .expect("request should build info");
        let blueprint = demo_blueprint(
            &NlheDemoHand::new(47).snapshot(NlheDemoPhase::Preflop),
            &info,
        );
        let renderer = NlheRenderer::demo_with_blueprint(blueprint);

        assert!(renderer.declaration().starts_with("ARTIFACT ADVICE:"));
        assert_eq!(renderer.advice_source(), "artifact");
        assert!(renderer.pipe_output().contains("advisor=artifact"));
    }

    #[test]
    fn renderer_formats_injected_live_recommendation() {
        let renderer = NlheRenderer::demo();
        let output = renderer.pipe_output_with_recommendation("raise 6", "live");

        assert!(output.contains("street=PREFLOP"));
        assert!(output.contains("recommend=raise 6"));
        assert!(output.contains("advisor=live"));
    }

    #[test]
    fn renderer_maps_live_edge_to_human_action() {
        let renderer = NlheRenderer::demo();

        assert_eq!(
            renderer.recommendation_action(NlheEdge::from(rbp_gameplay::Edge::Call)),
            Some("call")
        );
        assert_eq!(
            renderer.recommendation_action(NlheEdge::from(rbp_gameplay::Edge::Raise(
                rbp_gameplay::Odds::new(1, 2)
            ))),
            Some("raise 6")
        );
    }

    #[test]
    fn renderer_displays_cached_live_recommendation() {
        let renderer = NlheRenderer::demo();
        renderer.set_live_recommendation("raise 6");

        assert_eq!(renderer.declaration(), "LIVE ADVICE: RAISE 6");
        assert!(renderer.desired_height(80) >= 8);

        let mut buffer = Buffer::empty(Rect::new(0, 0, 60, 10));
        renderer.render_state(Rect::new(0, 0, 60, 10), &mut buffer);

        let last_line: String = (0..32)
            .map(|x| buffer[(x, 7)].symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(last_line.contains("LIVE ADVICE RAISE 6"));
        assert!(last_line.contains("FRESH 0s"));
    }

    #[test]
    fn renderer_marks_live_recommendation_stale_after_failure() {
        let renderer = NlheRenderer::demo();
        renderer.set_live_recommendation("call");
        renderer.mark_live_recommendation_stale_with_age(7);

        assert_eq!(renderer.declaration(), "LIVE ADVICE STALE: CALL");

        let mut buffer = Buffer::empty(Rect::new(0, 0, 60, 10));
        renderer.render_state(Rect::new(0, 0, 60, 10), &mut buffer);
        let last_line: String = (0..32)
            .map(|x| buffer[(x, 7)].symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(last_line.contains("LIVE ADVICE CALL [STALE 7s]"));
    }

    #[test]
    fn renderer_shows_live_advice_offline_without_cached_action() {
        let renderer = NlheRenderer::demo();
        renderer.set_live_advice_offline();

        assert_eq!(renderer.declaration(), "LIVE ADVICE OFFLINE");

        let mut buffer = Buffer::empty(Rect::new(0, 0, 60, 10));
        renderer.render_state(Rect::new(0, 0, 60, 10), &mut buffer);
        let last_line: String = (0..20)
            .map(|x| buffer[(x, 7)].symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(last_line.contains("LIVE ADVICE OFFLINE"));
    }
}
