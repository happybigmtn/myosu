use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// Base rendering contract for all shell components.
///
/// Every visual component (header, transcript, gameboard, composer) implements
/// this trait. Components report their desired height, and the flex layout
/// allocates vertical space accordingly.
pub trait Renderable {
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn desired_height(&self, width: u16) -> u16;
    fn cursor_pos(&self, _area: Rect) -> Option<(u16, u16)> {
        None
    }
}

/// Game-specific rendering extension point.
///
/// The shell handles header, declaration, log, and input. Each game only draws
/// its state panel via this trait. Adding a new game means implementing
/// `GameRenderer` — no changes to the shell, event loop, or input handling.
///
/// Object-safe: the shell holds `Box<dyn GameRenderer>`.
pub trait GameRenderer: Send {
    /// Render the game-specific state panel into the given area.
    fn render_state(&self, area: Rect, buf: &mut Buffer);

    /// Desired state panel height for the given terminal width.
    /// Returns 0 when no active hand (panel collapses).
    fn desired_height(&self, width: u16) -> u16;

    /// Current declaration text (ALLCAPS).
    fn declaration(&self) -> &str;

    /// Available actions for tab completion.
    fn completions(&self) -> Vec<String>;

    /// Parse user input into a game action. Returns None if invalid.
    fn parse_input(&self, input: &str) -> Option<String>;

    /// Clarification prompt for ambiguous input.
    fn clarify(&self, input: &str) -> Option<String>;

    /// Game state as plain text for --pipe mode.
    fn pipe_output(&self) -> String;

    /// Header path component (e.g., "NLHE-HU", "RIICHI", "HWATU").
    fn game_label(&self) -> &str;

    /// Header context (e.g., "HAND 47", "EAST 1 ROUND 3", "ROUND 5").
    fn context_label(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRenderer {
        hand_active: bool,
    }

    impl MockRenderer {
        fn active() -> Self {
            Self { hand_active: true }
        }

        fn inactive() -> Self {
            Self { hand_active: false }
        }
    }

    impl GameRenderer for MockRenderer {
        fn render_state(&self, area: Rect, buf: &mut Buffer) {
            if self.hand_active && area.width >= 2 && area.height >= 1 {
                let line = "pot: 12bb";
                let x = area.x;
                let y = area.y;
                for (i, ch) in line.chars().enumerate() {
                    if x + i as u16 >= area.right() {
                        break;
                    }
                    buf[(x + i as u16, y)].set_char(ch);
                }
            }
        }

        fn desired_height(&self, _width: u16) -> u16 {
            if self.hand_active { 4 } else { 0 }
        }

        fn declaration(&self) -> &str {
            if self.hand_active {
                "THE SYSTEM AWAITS YOUR DECISION"
            } else {
                "NO ACTIVE HAND"
            }
        }

        fn completions(&self) -> Vec<String> {
            if self.hand_active {
                vec!["fold".into(), "call".into(), "raise".into(), "check".into()]
            } else {
                vec!["new".into(), "quit".into()]
            }
        }

        fn parse_input(&self, input: &str) -> Option<String> {
            match input.trim().to_lowercase().as_str() {
                "f" | "fold" => Some("fold".into()),
                "c" | "call" => Some("call".into()),
                "r" | "raise" => Some("raise".into()),
                "k" | "check" => Some("check".into()),
                _ => None,
            }
        }

        fn clarify(&self, input: &str) -> Option<String> {
            if input.starts_with('r') && input != "raise" {
                Some("raise to how much? (e.g., raise 15)".into())
            } else {
                None
            }
        }

        fn pipe_output(&self) -> String {
            if self.hand_active {
                "STATE hand=47 pot=12 hero=AcKh board=Ts7h2c".into()
            } else {
                "STATE idle".into()
            }
        }

        fn game_label(&self) -> &str {
            "NLHE-HU"
        }

        fn context_label(&self) -> String {
            "HAND 47".to_string()
        }
    }

    #[test]
    fn trait_is_object_safe() {
        let renderer: Box<dyn GameRenderer> = Box::new(MockRenderer::active());
        assert_eq!(renderer.game_label(), "NLHE-HU");
        assert_eq!(renderer.context_label(), "HAND 47");
    }

    #[test]
    fn mock_renderer_works() {
        let renderer = MockRenderer::active();

        assert_eq!(renderer.desired_height(80), 4);
        assert_eq!(renderer.declaration(), "THE SYSTEM AWAITS YOUR DECISION");
        assert_eq!(renderer.game_label(), "NLHE-HU");
        assert_eq!(renderer.context_label(), "HAND 47");

        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 4));
        renderer.render_state(Rect::new(0, 0, 40, 4), &mut buf);
        let line: String = (0..9)
            .map(|x| buf[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();
        assert_eq!(line, "pot: 12bb");
    }

    #[test]
    fn pipe_output_returns_structured_text() {
        let active = MockRenderer::active();
        let output = active.pipe_output();
        assert!(output.starts_with("STATE "));
        assert!(output.contains("hand=47"));

        let idle = MockRenderer::inactive();
        assert_eq!(idle.pipe_output(), "STATE idle");
    }

    #[test]
    fn completions_non_empty_when_active() {
        let renderer = MockRenderer::active();
        let completions = renderer.completions();
        assert!(!completions.is_empty());
        assert!(completions.contains(&"fold".to_string()));
        assert!(completions.contains(&"call".to_string()));
    }

    #[test]
    fn parse_input_accepts_shorthands() {
        let renderer = MockRenderer::active();
        assert_eq!(renderer.parse_input("f"), Some("fold".into()));
        assert_eq!(renderer.parse_input("c"), Some("call".into()));
        assert_eq!(renderer.parse_input("raise"), Some("raise".into()));
        assert_eq!(renderer.parse_input("xyz"), None);
    }

    #[test]
    fn clarify_returns_prompt_for_ambiguous() {
        let renderer = MockRenderer::active();
        assert!(renderer.clarify("r").is_some());
        assert!(renderer.clarify("fold").is_none());
    }

    #[test]
    fn inactive_renderer_collapses() {
        let renderer = MockRenderer::inactive();
        assert_eq!(renderer.desired_height(80), 0);
        assert_eq!(renderer.declaration(), "NO ACTIVE HAND");
        assert!(renderer.completions().contains(&"quit".to_string()));
    }

    #[test]
    fn renderable_default_cursor_is_none() {
        struct Stub;
        impl Renderable for Stub {
            fn render(&self, _area: Rect, _buf: &mut Buffer) {}
            fn desired_height(&self, _width: u16) -> u16 {
                1
            }
        }
        let s = Stub;
        assert!(s.cursor_pos(Rect::new(0, 0, 80, 24)).is_none());
    }
}
