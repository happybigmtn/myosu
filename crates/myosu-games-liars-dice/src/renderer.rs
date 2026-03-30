use crate::game::LiarsDiceClaim;
use myosu_tui::GameRenderer;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// Static renderer snapshot for minimal Liar's Dice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiarsDiceSnapshot {
    /// Acting player's die value.
    pub your_die: u8,
    /// Most recent public claim.
    pub last_claim: Option<LiarsDiceClaim>,
    /// Textual legal actions for shell completion.
    pub legal_actions: Vec<String>,
    /// Header context text.
    pub context: String,
}

impl LiarsDiceSnapshot {
    /// Demo snapshot for tests and manual inspection.
    pub fn demo() -> Self {
        Self {
            your_die: 4,
            last_claim: LiarsDiceClaim::new(1, 3),
            legal_actions: vec![
                "bid 1x4".to_string(),
                "bid 2x1".to_string(),
                "liar".to_string(),
            ],
            context: "ROUND 1".to_string(),
        }
    }
}

/// Basic TUI renderer for minimal Liar's Dice.
#[derive(Clone, Debug)]
pub struct LiarsDiceRenderer {
    snapshot: Option<LiarsDiceSnapshot>,
}

impl LiarsDiceRenderer {
    /// Create a renderer with an optional active snapshot.
    pub fn new(snapshot: Option<LiarsDiceSnapshot>) -> Self {
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

impl GameRenderer for LiarsDiceRenderer {
    fn render_state(&self, area: Rect, buf: &mut Buffer) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let claim = match snapshot.last_claim {
            Some(claim) => format!("last claim: {}x{}", claim.count, claim.face),
            None => "last claim: none".to_string(),
        };
        let actions = format!("actions: {}", snapshot.legal_actions.join(", "));
        Self::render_line(area, buf, 0, &format!("your die: {}", snapshot.your_die));
        Self::render_line(area, buf, 1, &claim);
        Self::render_line(area, buf, 2, &actions);
    }

    fn desired_height(&self, _width: u16) -> u16 {
        if self.snapshot.is_some() { 4 } else { 0 }
    }

    fn declaration(&self) -> &str {
        if self.snapshot.is_some() {
            "THE TABLE WAITS FOR YOUR CALL"
        } else {
            "NO ACTIVE ROUND"
        }
    }

    fn completions(&self) -> Vec<String> {
        match &self.snapshot {
            Some(snapshot) => snapshot.legal_actions.clone(),
            None => vec!["/quit".to_string()],
        }
    }

    fn parse_input(&self, input: &str) -> Option<String> {
        let normalized = input.trim().to_lowercase();
        self.completions()
            .into_iter()
            .find(|candidate| candidate.to_lowercase() == normalized)
    }

    fn clarify(&self, input: &str) -> Option<String> {
        if input.trim().eq_ignore_ascii_case("bid") {
            Some("bid <count>x<face>".to_string())
        } else {
            None
        }
    }

    fn pipe_output(&self) -> String {
        match &self.snapshot {
            Some(snapshot) => {
                let claim = match snapshot.last_claim {
                    Some(claim) => format!("{}x{}", claim.count, claim.face),
                    None => "none".to_string(),
                };
                format!(
                    "STATE game=liars_dice die={} last_claim={} actions={}",
                    snapshot.your_die,
                    claim,
                    snapshot.legal_actions.join("|"),
                )
            }
            None => "STATE idle".to_string(),
        }
    }

    fn game_label(&self) -> &str {
        "LIARS-DICE"
    }

    fn context_label(&self) -> String {
        match &self.snapshot {
            Some(snapshot) => snapshot.context.clone(),
            None => "NO ROUND".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::renderer::{LiarsDiceRenderer, LiarsDiceSnapshot};
    use myosu_tui::GameRenderer;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    #[test]
    fn active_renderer_reports_shell_contract() {
        let renderer = LiarsDiceRenderer::new(Some(LiarsDiceSnapshot::demo()));

        assert_eq!(renderer.desired_height(80), 4);
        assert_eq!(renderer.game_label(), "LIARS-DICE");
        assert_eq!(renderer.declaration(), "THE TABLE WAITS FOR YOUR CALL");
        assert_eq!(renderer.context_label(), "ROUND 1");
        assert!(renderer.pipe_output().contains("game=liars_dice"));
        assert_eq!(renderer.parse_input("liar"), Some("liar".to_string()));
        assert_eq!(
            renderer.clarify("bid"),
            Some("bid <count>x<face>".to_string())
        );
    }

    #[test]
    fn inactive_renderer_collapses() {
        let renderer = LiarsDiceRenderer::new(None);

        assert_eq!(renderer.desired_height(80), 0);
        assert_eq!(renderer.declaration(), "NO ACTIVE ROUND");
        assert_eq!(renderer.pipe_output(), "STATE idle");
        assert_eq!(renderer.completions(), vec!["/quit".to_string()]);
    }

    #[test]
    fn render_state_writes_snapshot_lines() {
        let renderer = LiarsDiceRenderer::new(Some(LiarsDiceSnapshot::demo()));
        let mut buffer = Buffer::empty(Rect::new(0, 0, 60, 4));

        renderer.render_state(Rect::new(0, 0, 60, 4), &mut buffer);

        let first_line: String = (0..11)
            .map(|x| buffer[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();
        assert_eq!(first_line, "your die: 4");
    }
}
