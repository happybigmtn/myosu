use crate::game::{KuhnCard, KuhnEdge, KuhnGame, KuhnHistory, KuhnInfo};
use myosu_games::{CfrGame, CfrInfo};
use myosu_tui::GameRenderer;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use rbp_mccfr::CfrPublic;

/// Static renderer snapshot for Kuhn poker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KuhnSnapshot {
    /// Acting player's private card.
    pub your_card: KuhnCard,
    /// Public betting history.
    pub history: KuhnHistory,
    /// Current pot size in chips.
    pub pot: u8,
    /// Textual legal actions for shell completion.
    pub legal_actions: Vec<String>,
    /// Header context text.
    pub context: String,
    /// Optional terminal outcome text.
    pub outcome: Option<String>,
}

impl KuhnSnapshot {
    /// Build a live snapshot from a Kuhn information set.
    pub fn from_info(info: KuhnInfo, context: impl Into<String>) -> Self {
        let public = info.public();

        Self {
            your_card: info.card(),
            history: public.history(),
            pot: pot_for(public.history()),
            legal_actions: public
                .choices()
                .into_iter()
                .filter_map(action_name)
                .map(str::to_string)
                .collect(),
            context: context.into(),
            outcome: None,
        }
    }

    /// Demo snapshot for tests and manual inspection.
    pub fn demo() -> Self {
        let info = KuhnGame::root()
            .apply(KuhnEdge::Deal {
                p1: KuhnCard::Queen,
                p2: KuhnCard::Jack,
            })
            .info()
            .expect("dealt opening state should expose info");

        Self::from_info(info, "HAND 1")
    }

    /// Terminal snapshot for completed-hand rendering.
    pub fn complete(
        your_card: KuhnCard,
        history: KuhnHistory,
        outcome: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            your_card,
            history,
            pot: pot_for(history),
            legal_actions: vec!["new hand".to_string()],
            context: context.into(),
            outcome: Some(outcome.into()),
        }
    }
}

/// Basic TUI renderer for Kuhn poker.
#[derive(Clone, Debug)]
pub struct KuhnRenderer {
    snapshot: Option<KuhnSnapshot>,
}

impl KuhnRenderer {
    /// Create a renderer with an optional active snapshot.
    pub fn new(snapshot: Option<KuhnSnapshot>) -> Self {
        Self { snapshot }
    }

    fn render_line(area: Rect, buf: &mut Buffer, line_index: u16, text: &str) {
        if line_index >= area.height {
            return;
        }

        for (index, ch) in text.chars().enumerate() {
            let x = area.x + index as u16;
            if x >= area.right() {
                break;
            }
            buf[(x, area.y + line_index)].set_char(ch);
        }
    }
}

impl GameRenderer for KuhnRenderer {
    fn render_state(&self, area: Rect, buf: &mut Buffer) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };

        Self::render_line(
            area,
            buf,
            0,
            &format!("your card: {}", card_label(snapshot.your_card)),
        );
        Self::render_line(
            area,
            buf,
            1,
            &format!("history: {}", history_label(snapshot.history)),
        );
        Self::render_line(area, buf, 2, &format!("pot: {} chips", snapshot.pot));

        let footer = match &snapshot.outcome {
            Some(outcome) => format!("result: {outcome}"),
            None => format!("actions: {}", snapshot.legal_actions.join(", ")),
        };
        Self::render_line(area, buf, 3, &footer);
    }

    fn desired_height(&self, _width: u16) -> u16 {
        if self.snapshot.is_some() { 4 } else { 0 }
    }

    fn declaration(&self) -> &str {
        match &self.snapshot {
            Some(snapshot) if snapshot.outcome.is_some() => "THE HAND IS COMPLETE",
            Some(snapshot) if facing_bet(&snapshot.legal_actions) => "THE BET IS TO YOU",
            Some(_) => "THE CARDS ARE DEALT",
            None => "NO ACTIVE HAND",
        }
    }

    fn completions(&self) -> Vec<String> {
        match &self.snapshot {
            Some(snapshot) => snapshot.legal_actions.clone(),
            None => vec!["/quit".to_string()],
        }
    }

    fn parse_input(&self, input: &str) -> Option<String> {
        let normalized = normalize_action(input);

        self.completions()
            .into_iter()
            .find(|candidate| candidate.eq_ignore_ascii_case(&normalized))
    }

    fn clarify(&self, _input: &str) -> Option<String> {
        None
    }

    fn pipe_output(&self) -> String {
        match &self.snapshot {
            Some(snapshot) => {
                let mut parts = vec![
                    "STATE".to_string(),
                    "game=kuhn_poker".to_string(),
                    format!("card={}", card_label(snapshot.your_card)),
                    format!("history={}", history_label(snapshot.history)),
                    format!("pot={}", snapshot.pot),
                    format!("actions={}", snapshot.legal_actions.join("|")),
                ];

                if let Some(outcome) = &snapshot.outcome {
                    parts.push(format!("result={outcome}"));
                }

                parts.join(" ")
            }
            None => "STATE idle".to_string(),
        }
    }

    fn game_label(&self) -> &str {
        "KUHN"
    }

    fn context_label(&self) -> String {
        match &self.snapshot {
            Some(snapshot) => snapshot.context.clone(),
            None => "NO HAND".to_string(),
        }
    }
}

fn action_name(edge: KuhnEdge) -> Option<&'static str> {
    match edge {
        KuhnEdge::Check => Some("check"),
        KuhnEdge::Bet => Some("bet"),
        KuhnEdge::Call => Some("call"),
        KuhnEdge::Fold => Some("fold"),
        KuhnEdge::Deal { .. } => None,
    }
}

fn card_label(card: KuhnCard) -> &'static str {
    match card {
        KuhnCard::Jack => "J",
        KuhnCard::Queen => "Q",
        KuhnCard::King => "K",
    }
}

fn history_label(history: KuhnHistory) -> &'static str {
    match history {
        KuhnHistory::Opening => "opening",
        KuhnHistory::P1Checked => "check",
        KuhnHistory::P1Bet => "bet",
        KuhnHistory::P1CheckP2Bet => "check-bet",
        KuhnHistory::CheckCheck => "check-check",
        KuhnHistory::BetCall => "bet-call",
        KuhnHistory::BetFold => "bet-fold",
        KuhnHistory::CheckBetCall => "check-bet-call",
        KuhnHistory::CheckBetFold => "check-bet-fold",
    }
}

fn pot_for(history: KuhnHistory) -> u8 {
    match history {
        KuhnHistory::Opening | KuhnHistory::P1Checked | KuhnHistory::CheckCheck => 2,
        KuhnHistory::P1Bet | KuhnHistory::P1CheckP2Bet => 3,
        KuhnHistory::BetCall | KuhnHistory::CheckBetCall => 4,
        KuhnHistory::BetFold | KuhnHistory::CheckBetFold => 3,
    }
}

fn facing_bet(legal_actions: &[String]) -> bool {
    legal_actions
        .iter()
        .any(|action| action.eq_ignore_ascii_case("call"))
        && legal_actions
            .iter()
            .any(|action| action.eq_ignore_ascii_case("fold"))
}

fn normalize_action(input: &str) -> String {
    match input.trim().to_ascii_lowercase().as_str() {
        "k" => "check".to_string(),
        "b" => "bet".to_string(),
        "c" => "call".to_string(),
        "f" => "fold".to_string(),
        "n" | "new" => "new hand".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{KuhnCard, KuhnEdge, KuhnGame, KuhnHistory};
    use crate::renderer::{KuhnRenderer, KuhnSnapshot};
    use myosu_games::CfrGame;
    use myosu_tui::GameRenderer;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    #[test]
    fn snapshot_from_info_derives_pot_and_actions() {
        let info = KuhnGame::root()
            .apply(KuhnEdge::Deal {
                p1: KuhnCard::Queen,
                p2: KuhnCard::Jack,
            })
            .info()
            .expect("dealt opening state should expose info");
        let snapshot = KuhnSnapshot::from_info(info, "HAND 1");

        assert_eq!(snapshot.your_card, KuhnCard::Queen);
        assert_eq!(snapshot.history, KuhnHistory::Opening);
        assert_eq!(snapshot.pot, 2);
        assert_eq!(snapshot.legal_actions, vec!["check", "bet"]);
    }

    #[test]
    fn active_renderer_reports_shell_contract() {
        let renderer = KuhnRenderer::new(Some(KuhnSnapshot::demo()));

        assert_eq!(renderer.desired_height(80), 4);
        assert_eq!(renderer.game_label(), "KUHN");
        assert_eq!(renderer.declaration(), "THE CARDS ARE DEALT");
        assert_eq!(renderer.context_label(), "HAND 1");
        assert!(renderer.pipe_output().contains("game=kuhn_poker"));
        assert_eq!(renderer.parse_input("k"), Some("check".to_string()));
        assert_eq!(renderer.parse_input("bet"), Some("bet".to_string()));
        assert_eq!(renderer.clarify("bet"), None);
    }

    #[test]
    fn terminal_renderer_exposes_result_and_new_hand() {
        let renderer = KuhnRenderer::new(Some(KuhnSnapshot::complete(
            KuhnCard::King,
            KuhnHistory::BetCall,
            "hero+2",
            "HAND 1",
        )));

        assert_eq!(renderer.declaration(), "THE HAND IS COMPLETE");
        assert_eq!(renderer.completions(), vec!["new hand".to_string()]);
        assert_eq!(renderer.parse_input("new"), Some("new hand".to_string()));
        assert!(renderer.pipe_output().contains("result=hero+2"));
    }

    #[test]
    fn inactive_renderer_collapses() {
        let renderer = KuhnRenderer::new(None);

        assert_eq!(renderer.desired_height(80), 0);
        assert_eq!(renderer.declaration(), "NO ACTIVE HAND");
        assert_eq!(renderer.pipe_output(), "STATE idle");
        assert_eq!(renderer.completions(), vec!["/quit".to_string()]);
    }

    #[test]
    fn render_state_writes_snapshot_lines() {
        let renderer = KuhnRenderer::new(Some(KuhnSnapshot::demo()));
        let mut buffer = Buffer::empty(Rect::new(0, 0, 60, 4));

        renderer.render_state(Rect::new(0, 0, 60, 4), &mut buffer);

        let first_line: String = (0..12)
            .map(|x| buffer[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();
        assert_eq!(first_line, "your card: Q");
    }
}
