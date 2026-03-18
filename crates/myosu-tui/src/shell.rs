//! Five-panel shell layout for the myosu TUI.
//!
//! The shell divides the terminal into five vertical panels:
//! 1. Header — game label and context (e.g., "NLHE-HU  HAND 47")
//! 2. Transcript — scrollable game history and messages
//! 3. State — game-specific visualization (delegated to GameRenderer)
//! 4. Declaration — status line (ALLCAPS, centered)
//! 5. Input — command line with prompt
//!
//! The shell handles all cross-cutting concerns: layout, input, history,
//! logging, and mode switching. Games implement `GameRenderer` to draw
//! only their state panel.

use crate::events::{Event, EventLoop, UpdateEvent};
use crate::input::{InputAction, InputLine};
use crate::renderer::{GameRenderer, Renderable};
use crate::screens::{Screen, ScreenManager};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};
use std::io;
use std::time::Duration;

/// Maximum number of lines to keep in the transcript buffer.
const MAX_TRANSCRIPT_LINES: usize = 1000;

/// Minimum terminal dimensions for usable layout.
const MIN_WIDTH: u16 = 40;
const MIN_HEIGHT: u16 = 12;

/// Shell layout manager and event loop coordinator.
///
/// The shell owns the terminal, event loop, input buffer, and transcript
/// history. It delegates game-specific rendering to the `GameRenderer`.
pub struct Shell {
    /// Theme for styling all UI elements
    theme: Theme,
    /// Input line buffer with history and completion
    input: InputLine,
    /// Transcript/history of game events
    transcript: Vec<String>,
    /// Screen navigation manager
    screens: ScreenManager,
    /// Whether the shell is running
    running: bool,
    /// Whether to show help overlay
    show_help: bool,
    /// Terminal size (width, height)
    terminal_size: (u16, u16),
}

/// Layout constraints for the five panels.
///
/// The layout uses flex constraints to distribute available space:
/// - Header: fixed 1 line
/// - Transcript: takes remaining space (min 3 lines)
/// - State: dynamic based on game renderer (min 0, can collapse)
/// - Declaration: fixed 1 line
/// - Input: fixed 1 line
struct PanelLayout {
    header: Rect,
    transcript: Rect,
    state: Rect,
    declaration: Rect,
    input: Rect,
}

impl Shell {
    /// Create a new shell with default theme.
    pub fn new() -> Self {
        Self::with_theme(Theme::default())
    }

    /// Create a new shell with the given theme.
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            theme,
            input: InputLine::new(),
            transcript: Vec::new(),
            screens: ScreenManager::new(),
            running: false,
            show_help: false,
            terminal_size: (80, 24),
        }
    }

    /// Create a shell with a specific starting screen (for testing).
    #[must_use]
    pub fn with_screen(screen: Screen) -> Self {
        Self {
            theme: Theme::default(),
            input: InputLine::new(),
            transcript: Vec::new(),
            screens: ScreenManager::with_screen(screen),
            running: false,
            show_help: false,
            terminal_size: (80, 24),
        }
    }

    /// Run the main event loop.
    ///
    /// This method takes ownership of the terminal and runs until:
    /// - User quits with /quit or Ctrl-C
    /// - An error occurs
    /// - The game signals completion
    ///
    /// # Arguments
    /// * `renderer` - Game-specific renderer for the state panel
    /// * `tick_rate` - Duration between UI update ticks
    pub async fn run(
        &mut self,
        renderer: &dyn GameRenderer,
        tick_rate: Duration,
    ) -> io::Result<()> {
        let mut event_loop = EventLoop::new(tick_rate);
        self.running = true;

        while self.running {
            match event_loop.next().await {
                Some(Event::Tick) => {
                    // UI refresh tick - re-render happens implicitly
                    // when draw is called by the application layer
                }
                Some(Event::Key(key)) => self.handle_key(key, renderer),
                Some(Event::Resize(w, h)) => self.terminal_size = (w, h),
                Some(Event::Update(update)) => self.handle_update(update, renderer),
                Some(Event::Quit) => self.running = false,
                None => break,
            }
        }

        Ok(())
    }

    /// Handle a key event.
    fn handle_key(&mut self, key: KeyEvent, renderer: &dyn GameRenderer) {
        // Global shortcuts
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.running = false;
            return;
        }

        // Help toggle with '?' (but not when typing in input)
        if key.code == KeyCode::Char('?') && key.modifiers.is_empty() {
            self.show_help = !self.show_help;
            return;
        }

        // Handle overlay screens (Coaching, History)
        if self.screens.current().is_game_overlay() {
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                self.screens.handle_overlay_key();
            }
            return;
        }

        // Handle normal input
        match self.input.handle_key(key) {
            InputAction::Submit(text) => self.handle_submit(text, renderer),
            InputAction::SlashCommand(cmd) => self.handle_slash_command(cmd, renderer),
            InputAction::Continue => {}
        }
    }

    /// Handle text submission (non-command input).
    fn handle_submit(&mut self, text: String, renderer: &dyn GameRenderer) {
        // Log the input to transcript
        self.log(format!("> {text}"));

        // Parse through game renderer
        match renderer.parse_input(&text) {
            Some(action) => {
                self.log(format!("Action: {action}"));
            }
            None => {
                // Check if there's a clarification prompt
                if let Some(clarify) = renderer.clarify(&text) {
                    self.log(clarify);
                } else {
                    self.log("Invalid input. Type /help for commands.".to_string());
                }
            }
        }
    }

    /// Handle slash commands.
    fn handle_slash_command(&mut self, cmd: String, _renderer: &dyn GameRenderer) {
        let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "/quit" | "/q" => {
                if self.screens.current() == Screen::Game {
                    self.screens.apply_command("/quit");
                } else {
                    self.running = false;
                }
            }
            "/help" | "/h" => {
                self.show_help = true;
            }
            "/clear" => {
                self.transcript.clear();
            }
            _ => {
                // Try screen manager for navigation commands
                if self.screens.apply_command(&cmd) {
                    // Screen changed
                } else {
                    // Unknown command
                    self.log(format!("Unknown command: {}", parts[0]));
                }
            }
        }
    }

    /// Handle async update events.
    fn handle_update(&mut self, update: UpdateEvent, _renderer: &dyn GameRenderer) {
        match update {
            UpdateEvent::SolverAdvice { actions } => {
                let advice: Vec<String> = actions
                    .iter()
                    .map(|(a, p)| format!("{a}: {:.1}%", p * 100.0))
                    .collect();
                self.log(format!("Solver: {}", advice.join(", ")));
            }
            UpdateEvent::StateChanged { state } => {
                self.log(format!("State: {state}"));
            }
            UpdateEvent::TrainingProgress {
                iteration,
                exploitability,
            } => {
                self.log(format!(
                    "Training iteration {iteration}: exploitability = {exploitability:.4}"
                ));
            }
            UpdateEvent::Message(msg) => {
                self.log(msg);
            }
        }
    }

    /// Add a message to the transcript.
    pub fn log(&mut self, message: String) {
        self.transcript.push(message);
        if self.transcript.len() > MAX_TRANSCRIPT_LINES {
            self.transcript.remove(0);
        }
    }

    /// Get a reference to the transcript.
    pub fn transcript(&self) -> &[String] {
        &self.transcript
    }

    /// Check if the shell is still running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Stop the shell.
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Get the current screen.
    pub fn current_screen(&self) -> Screen {
        self.screens.current()
    }

    /// Calculate the panel layout for the given area.
    fn calculate_layout(&self, area: Rect, renderer: &dyn GameRenderer) -> PanelLayout {
        // Calculate state panel height (collapses to 0 if no active game)
        let state_height = if self.screens.current() == Screen::Game {
            renderer.desired_height(area.width)
        } else {
            0
        };

        // Constraints for vertical layout
        let constraints = [
            Constraint::Length(1),              // header
            Constraint::Min(3),                 // transcript (flexible)
            Constraint::Length(state_height),   // state (dynamic, can be 0)
            Constraint::Length(1),              // declaration
            Constraint::Length(1),              // input
        ];

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        PanelLayout {
            header: chunks[0],
            transcript: chunks[1],
            state: chunks[2],
            declaration: chunks[3],
            input: chunks[4],
        }
    }

    /// Render the shell into the given buffer.
    ///
    /// This is the main entry point for drawing the TUI. The application
    /// should call this on every frame refresh.
    pub fn draw(&self, area: Rect, buf: &mut Buffer, renderer: &dyn GameRenderer) {
        // Check minimum dimensions
        if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
            self.render_too_small(area, buf);
            return;
        }

        // Check for help overlay
        if self.show_help {
            self.render_help(area, buf);
            return;
        }

        let layout = self.calculate_layout(area, renderer);

        self.render_header(layout.header, buf, renderer);
        self.render_transcript(layout.transcript, buf);
        self.render_state(layout.state, buf, renderer);
        self.render_declaration(layout.declaration, buf, renderer);
        self.render_input(layout.input, buf);
    }

    /// Render the header panel.
    fn render_header(&self, area: Rect, buf: &mut Buffer, renderer: &dyn GameRenderer) {
        let game_label = renderer.game_label();
        let context_label = renderer.context_label();

        let header_text = format!("{game_label}  {context_label}");
        let style = Style::default()
            .fg(self.theme.fg_bright)
            .add_modifier(Modifier::BOLD);

        let line = Line::from(vec![Span::styled(header_text, style)]);
        let paragraph = Paragraph::new(vec![line]);

        paragraph.render(area, buf);
    }

    /// Render the transcript panel.
    fn render_transcript(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().fg(self.theme.fg));

        // Show last N lines that fit in the area
        let visible_lines: Vec<Line> = self
            .transcript
            .iter()
            .rev()
            .take(area.height as usize)
            .rev()
            .map(|text| Line::from(Span::styled(text.clone(), Style::default().fg(self.theme.fg))))
            .collect();

        let paragraph = Paragraph::new(visible_lines)
            .block(block)
            .wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }

    /// Render the state panel (delegated to GameRenderer).
    fn render_state(&self, area: Rect, buf: &mut Buffer, renderer: &dyn GameRenderer) {
        if area.height == 0 {
            return;
        }

        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(self.theme.fg_dim))
            .style(Style::default().fg(self.theme.fg));

        let inner = block.inner(area);
        block.render(area, buf);

        renderer.render_state(inner, buf);
    }

    /// Render the declaration panel.
    fn render_declaration(&self, area: Rect, buf: &mut Buffer, renderer: &dyn GameRenderer) {
        let declaration = renderer.declaration();
        let style = Style::default()
            .fg(self.theme.fg_bright)
            .add_modifier(Modifier::BOLD);

        // Center the text
        let available_width = area.width as usize;
        let text_len = declaration.len();
        let padding = if text_len < available_width {
            (available_width - text_len) / 2
        } else {
            0
        };

        let padded_text = format!("{}{}", " ".repeat(padding), declaration);
        let line = Line::from(vec![Span::styled(padded_text, style)]);
        let paragraph = Paragraph::new(vec![line]);

        paragraph.render(area, buf);
    }

    /// Render the input panel.
    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        let prompt = "> ";
        let input_text = self.input.text();
        let full_text = format!("{}{}", prompt, input_text);

        let style = Style::default().fg(self.theme.fg);
        let line = Line::from(vec![Span::styled(full_text, style)]);
        let paragraph = Paragraph::new(vec![line]);

        paragraph.render(area, buf);
    }

    /// Render the "terminal too small" message.
    fn render_too_small(&self, area: Rect, buf: &mut Buffer) {
        let message = format!(
            "Terminal too small. Minimum: {}x{}, Current: {}x{}",
            MIN_WIDTH, MIN_HEIGHT, area.width, area.height
        );
        let style = Style::default().fg(self.theme.diverge);
        let line = Line::from(vec![Span::styled(message, style)]);
        let paragraph = Paragraph::new(vec![line]);

        paragraph.render(area, buf);
    }

    /// Render the help overlay.
    fn render_help(&self, area: Rect, buf: &mut Buffer) {
        let help_text = vec![
            "MYOSU TUI HELP",
            "",
            "Commands:",
            "  /quit, /q     - Quit current game or application",
            "  /help, /h     - Show this help",
            "  /clear        - Clear transcript",
            "  /stats        - Show session statistics",
            "  /analyze      - Analyze current position",
            "  /history      - Show hand history",
            "  /wallet       - Account and staking",
            "  /spectate     - Watch agent vs agent",
            "  /back         - Go back (in wallet)",
            "",
            "Navigation:",
            "  ?             - Toggle this help",
            "  Ctrl-C        - Quit",
            "  q/ESC         - Close overlay screens",
            "",
            "Input:",
            "  Tab           - Auto-complete commands",
            "  Up/Down       - Navigate history",
            "  Ctrl-A        - Move to start of line",
            "  Ctrl-E        - Move to end of line",
            "  Ctrl-W        - Delete word backward",
            "  Ctrl-U        - Delete to start of line",
            "  Ctrl-K        - Delete to end of line",
            "",
            "Press any key to close...",
        ];

        let lines: Vec<Line> = help_text
            .into_iter()
            .map(|text| {
                let style = if text.starts_with("MYOSU") || text.ends_with(':') {
                    Style::default()
                        .fg(self.theme.fg_bright)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.fg)
                };
                Line::from(vec![Span::styled(text.to_string(), style)])
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.focus))
            .style(Style::default().fg(self.theme.fg));

        let paragraph = Paragraph::new(lines).block(block);

        // Center the help box
        let help_width = 50.min(area.width.saturating_sub(4));
        let help_height = 25.min(area.height.saturating_sub(4));
        let help_x = (area.width - help_width) / 2;
        let help_y = (area.height - help_height) / 2;
        let help_area = Rect::new(area.x + help_x, area.y + help_y, help_width, help_height);

        // Clear background
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_symbol(" ");
                    cell.set_style(Style::default());
                }
            }
        }

        paragraph.render(help_area, buf);
    }

    /// Update completions from the game renderer.
    pub fn update_completions(&mut self, renderer: &dyn GameRenderer) {
        let completions = renderer.completions();
        self.input.set_completions(completions);
    }

    /// Get the current input text.
    pub fn input_text(&self) -> String {
        self.input.text()
    }

    /// Get the current cursor position in the input.
    pub fn input_cursor(&self) -> usize {
        self.input.cursor()
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderable for Shell {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // This implementation is a placeholder - Shell requires a GameRenderer
        // to render properly. Use Shell::draw() instead.
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Shell (no renderer)");
        block.render(area, buf);
    }

    fn desired_height(&self, _width: u16) -> u16 {
        10
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

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
                let text = "Mock Game State";
                for (i, ch) in text.chars().enumerate() {
                    if area.x + i as u16 >= area.right() {
                        break;
                    }
                    buf[(area.x + i as u16, area.y)].set_char(ch);
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
                vec!["fold".into(), "call".into(), "raise".into()]
            } else {
                vec!["new".into(), "quit".into()]
            }
        }

        fn parse_input(&self, input: &str) -> Option<String> {
            match input.trim().to_lowercase().as_str() {
                "f" | "fold" => Some("fold".into()),
                "c" | "call" => Some("call".into()),
                "r" | "raise" => Some("raise".into()),
                _ => None,
            }
        }

        fn clarify(&self, input: &str) -> Option<String> {
            if input.starts_with('r') && input != "raise" {
                Some("raise to how much?".into())
            } else {
                None
            }
        }

        fn pipe_output(&self) -> String {
            "STATE mock".into()
        }

        fn game_label(&self) -> &str {
            "TEST"
        }

        fn context_label(&self) -> &str {
            "HAND 1"
        }
    }

    #[test]
    fn shell_new() {
        let shell = Shell::new();
        assert!(!shell.is_running());
        assert_eq!(shell.transcript().len(), 0);
    }

    #[test]
    fn shell_log() {
        let mut shell = Shell::new();
        shell.log("Test message".to_string());
        assert_eq!(shell.transcript().len(), 1);
        assert_eq!(shell.transcript()[0], "Test message");
    }

    #[test]
    fn shell_log_limit() {
        let mut shell = Shell::new();
        for i in 0..MAX_TRANSCRIPT_LINES + 100 {
            shell.log(format!("Message {i}"));
        }
        assert_eq!(shell.transcript().len(), MAX_TRANSCRIPT_LINES);
    }

    #[test]
    fn shell_stop() {
        let mut shell = Shell::new();
        shell.stop();
        assert!(!shell.is_running());
    }

    #[test]
    fn shell_draw_basic() {
        let shell = Shell::with_screen(Screen::Game);
        let renderer = MockRenderer::active();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));

        shell.draw(Rect::new(0, 0, 80, 24), &mut buf, &renderer);

        // Should have rendered something (not empty buffer)
        let content: String = buf
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(!content.trim().is_empty());
    }

    #[test]
    fn shell_draw_too_small() {
        let shell = Shell::new();
        let renderer = MockRenderer::active();
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 5));

        shell.draw(Rect::new(0, 0, 10, 5), &mut buf, &renderer);

        let content: String = buf
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(content.contains("too small"));
    }

    #[test]
    fn layout_calculates_correctly() {
        let shell = Shell::with_screen(Screen::Game);
        let renderer = MockRenderer::active();
        let area = Rect::new(0, 0, 80, 24);
        let layout = shell.calculate_layout(area, &renderer);

        // Header should be 1 line
        assert_eq!(layout.header.height, 1);

        // Declaration should be 1 line
        assert_eq!(layout.declaration.height, 1);

        // Input should be 1 line
        assert_eq!(layout.input.height, 1);

        // State should match renderer's desired height
        assert_eq!(layout.state.height, 4);

        // Transcript takes remaining space
        let used = layout.header.height + layout.declaration.height + layout.input.height + layout.state.height;
        assert_eq!(layout.transcript.height, area.height - used);
    }

    #[test]
    fn layout_collapses_state_when_inactive() {
        let shell = Shell::with_screen(Screen::Game);
        let renderer = MockRenderer::inactive();
        let area = Rect::new(0, 0, 80, 24);
        let layout = shell.calculate_layout(area, &renderer);

        // State should collapse to 0 when inactive
        assert_eq!(layout.state.height, 0);
    }

    #[test]
    fn update_completions() {
        let mut shell = Shell::new();
        let renderer = MockRenderer::active();
        shell.update_completions(&renderer);
        // Completions are set internally, just verify it doesn't panic
    }

    #[test]
    fn handle_slash_clear() {
        let mut shell = Shell::new();
        shell.log("Test".to_string());
        assert_eq!(shell.transcript().len(), 1);

        let renderer = MockRenderer::active();
        shell.handle_slash_command("/clear".to_string(), &renderer);
        assert_eq!(shell.transcript().len(), 0);
    }

    #[test]
    fn shell_default_trait() {
        let shell: Shell = Default::default();
        assert!(!shell.is_running());
    }

    #[test]
    fn shell_renderable_impl() {
        let shell = Shell::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        shell.render(Rect::new(0, 0, 80, 24), &mut buf);

        let content: String = buf
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(content.contains("Shell"));
    }
}
