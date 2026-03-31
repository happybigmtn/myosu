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

use crate::events::{Event, EventLoop, InteractionState, UpdateEvent};
use crate::input::{InputAction, InputLine};
use crate::renderer::GameRenderer;
use crate::screens::{Screen, ScreenManager};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};
use std::collections::VecDeque;
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;

/// Maximum number of lines to keep in the transcript buffer.
const MAX_TRANSCRIPT_LINES: usize = 1000;

/// Minimum terminal dimensions for usable layout.
const MIN_WIDTH: u16 = 40;
const MIN_HEIGHT: u16 = 12;
const COMPACT_WIDTH: u16 = 80;
const DESKTOP_WIDTH: u16 = 120;
const NARROW_TRANSCRIPT_LINES: usize = 3;

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
    transcript: VecDeque<String>,
    /// Screen navigation manager
    screens: ScreenManager,
    /// Whether the shell is running
    running: bool,
    /// Whether to show help overlay
    show_help: bool,
    /// Terminal size (width, height)
    terminal_size: (u16, u16),
    /// Current operator-facing interaction state for the declaration panel.
    interaction_state: InteractionState,
    /// Optional detail line paired with the current interaction state.
    interaction_detail: Option<String>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LayoutTier {
    Narrow,
    Compact,
    Desktop,
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
            transcript: VecDeque::new(),
            screens: ScreenManager::new(),
            running: false,
            show_help: false,
            terminal_size: (80, 24),
            interaction_state: InteractionState::Neutral,
            interaction_detail: None,
        }
    }

    /// Create a shell with a specific starting screen (for testing).
    #[must_use]
    pub fn with_screen(screen: Screen) -> Self {
        Self {
            theme: Theme::default(),
            input: InputLine::new(),
            transcript: VecDeque::new(),
            screens: ScreenManager::with_screen(screen),
            running: false,
            show_help: false,
            terminal_size: (80, 24),
            interaction_state: InteractionState::Neutral,
            interaction_detail: None,
        }
    }

    fn layout_tier(&self, width: u16) -> LayoutTier {
        if width < COMPACT_WIDTH {
            LayoutTier::Narrow
        } else if width < DESKTOP_WIDTH {
            LayoutTier::Compact
        } else {
            LayoutTier::Desktop
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

    /// Run the shell while owning terminal redraws.
    ///
    /// This is the normal application entrypoint for interactive mode.
    pub async fn run_terminal(
        &mut self,
        terminal: &mut DefaultTerminal,
        renderer: &dyn GameRenderer,
        tick_rate: Duration,
    ) -> io::Result<()> {
        self.run_terminal_with_updates(terminal, renderer, tick_rate, |_| {})
            .await
    }

    /// Run the shell while owning terminal redraws and exposing the update sender.
    pub async fn run_terminal_with_updates<F>(
        &mut self,
        terminal: &mut DefaultTerminal,
        renderer: &dyn GameRenderer,
        tick_rate: Duration,
        setup_updates: F,
    ) -> io::Result<()>
    where
        F: FnOnce(mpsc::UnboundedSender<UpdateEvent>),
    {
        let mut event_loop = EventLoop::new(tick_rate);
        setup_updates(event_loop.update_sender());
        self.running = true;
        self.update_completions(renderer);

        while self.running {
            terminal.draw(|frame| {
                let area = frame.area();
                let buf = frame.buffer_mut();
                self.draw(area, buf, renderer);
            })?;

            match event_loop.next().await {
                Some(Event::Tick) => {}
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

        // Help toggle with '?' only when input buffer is empty
        if key.code == KeyCode::Char('?')
            && key.modifiers.is_empty()
            && self.input.text().is_empty()
        {
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

        if self.screens.current() == Screen::Onboarding {
            self.screens.complete_onboarding();
            self.set_status(
                InteractionState::Success,
                Some("Onboarding complete.".to_string()),
            );

            if self.screens.apply_command(&text) {
                self.set_status(
                    InteractionState::Success,
                    Some("Onboarding complete. Lobby selection accepted.".to_string()),
                );
                return;
            }

            self.log("Onboarding complete. Type 1 to enter the demo hand.".to_string());
            return;
        }

        // In Lobby, bare text (including numbers) routes to screen manager
        if self.screens.current() == Screen::Lobby && self.screens.apply_command(&text) {
            self.set_status(
                InteractionState::Success,
                Some("Lobby selection accepted.".to_string()),
            );
            return;
        }

        // Parse through game renderer
        match renderer.parse_input(&text) {
            Some(action) => {
                self.set_status(
                    InteractionState::Success,
                    Some(format!("Accepted action `{action}`.")),
                );
                self.log(format!("Action: {action}"));
                self.update_completions(renderer);
            }
            None => {
                if let Some(clarify) = renderer.clarify(&text) {
                    self.set_status(InteractionState::Partial, Some(clarify.clone()));
                    self.log(clarify);
                } else {
                    self.set_status(
                        InteractionState::Error,
                        Some("Input was not recognized.".to_string()),
                    );
                    self.log("Invalid input. Type /help for commands.".to_string());
                }
            }
        }
    }

    /// Handle slash commands.
    fn handle_slash_command(&mut self, cmd: String, _renderer: &dyn GameRenderer) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
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
                self.set_status(
                    InteractionState::Success,
                    Some("Transcript cleared.".to_string()),
                );
            }
            _ => {
                // Try screen manager for navigation commands
                if self.screens.apply_command(&cmd) {
                    self.set_status(
                        InteractionState::Success,
                        Some(format!("Navigated with `{}`.", parts[0])),
                    );
                    // Screen changed
                } else {
                    // Unknown command
                    self.set_status(
                        InteractionState::Error,
                        Some(format!("Unknown command `{}`.", parts[0])),
                    );
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
            UpdateEvent::Status { state, detail } => {
                self.set_status(state, detail);
            }
            UpdateEvent::Message(msg) => {
                self.log(msg);
            }
        }
    }

    /// Add a message to the transcript.
    pub fn log(&mut self, message: String) {
        self.transcript.push_back(message);
        if self.transcript.len() > MAX_TRANSCRIPT_LINES {
            self.transcript.pop_front();
        }
    }

    /// Get a reference to the transcript.
    pub fn transcript(&self) -> &VecDeque<String> {
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

    /// Set the current declaration/interation state.
    pub fn set_status(&mut self, state: InteractionState, detail: Option<String>) {
        self.interaction_state = state;
        self.interaction_detail = detail;
    }

    /// Get the current interaction state.
    pub const fn interaction_state(&self) -> InteractionState {
        self.interaction_state
    }

    /// Get the current screen.
    pub fn current_screen(&self) -> Screen {
        self.screens.current()
    }

    /// Calculate the panel layout for the given area.
    fn calculate_layout(&self, area: Rect, renderer: &dyn GameRenderer) -> PanelLayout {
        let tier = self.layout_tier(area.width);
        let header_and_footer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);
        let header = header_and_footer[0];
        let body = header_and_footer[1];
        let declaration = header_and_footer[2];
        let input = header_and_footer[3];

        let state_height = if self.screens.current() == Screen::Game && tier != LayoutTier::Narrow {
            renderer.desired_height(area.width)
        } else {
            0
        };

        let (transcript, state) = match tier {
            LayoutTier::Desktop if state_height > 0 => {
                let columns = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(46), Constraint::Percentage(54)])
                    .split(body);
                (columns[0], columns[1])
            }
            _ => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(3), Constraint::Length(state_height)])
                    .split(body);
                (chunks[0], chunks[1])
            }
        };

        PanelLayout {
            header,
            transcript,
            state,
            declaration,
            input,
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
        if area.width == 0 {
            return;
        }

        let (game_label, context_label) = if self.screens.current() == Screen::Game {
            (renderer.game_label().to_string(), renderer.context_label())
        } else {
            (
                "MYOSU".to_string(),
                self.screens.current().header_context().to_string(),
            )
        };

        let header_text =
            truncate_single_line(&format!("{game_label}  {context_label}"), area.width);
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

        let visible_lines = self.transcript_lines(area);

        let paragraph = Paragraph::new(visible_lines)
            .block(block)
            .wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }

    /// Render the state panel (delegated to GameRenderer).
    fn render_state(&self, area: Rect, buf: &mut Buffer, renderer: &dyn GameRenderer) {
        if area.height == 0 || area.width == 0 {
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
        let (declaration, style) = if self.interaction_state == InteractionState::Neutral {
            let declaration = if self.screens.current() == Screen::Game {
                renderer.declaration()
            } else {
                self.screens.current().default_declaration()
            };
            (
                declaration.to_string(),
                Style::default()
                    .fg(self.theme.fg_bright)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            let declaration = interaction_state_banner(
                self.interaction_state,
                self.interaction_detail.as_deref(),
            );
            (
                declaration,
                Style::default()
                    .fg(interaction_state_color(self.interaction_state, &self.theme))
                    .add_modifier(Modifier::BOLD),
            )
        };

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
        if area.width == 0 {
            return;
        }

        let prompt = "> ";
        let prompt_width = prompt.chars().count() as u16;
        let available_width = area.width.saturating_sub(prompt_width) as usize;
        let (visible_input, cursor_offset) = self.input.viewport(available_width);

        let line = Line::from(vec![
            Span::styled(
                prompt.to_string(),
                Style::default()
                    .fg(self.theme.focus)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(visible_input.clone(), Style::default().fg(self.theme.fg)),
        ]);
        let paragraph = Paragraph::new(vec![line]);

        paragraph.render(area, buf);

        let cursor_x = area.x + prompt_width + cursor_offset as u16;
        if cursor_x >= area.right() {
            return;
        }

        if let Some(cell) = buf.cell_mut((cursor_x, area.y)) {
            if cursor_offset == visible_input.chars().count() {
                cell.set_symbol(" ");
            }
            cell.set_style(
                Style::default()
                    .fg(self.theme.fg_bright)
                    .bg(self.theme.focus)
                    .add_modifier(Modifier::BOLD),
            );
        }
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

    fn transcript_lines(&self, area: Rect) -> Vec<Line<'static>> {
        let tier = self.layout_tier(area.width);
        let max_lines = match tier {
            LayoutTier::Narrow => NARROW_TRANSCRIPT_LINES.min(area.height as usize),
            LayoutTier::Compact | LayoutTier::Desktop => area.height as usize,
        };
        let mut lines = self.interaction_transcript_lines(max_lines);
        let remaining = max_lines.saturating_sub(lines.len());

        if self.transcript.is_empty() {
            if lines.is_empty() {
                return self.transcript_placeholder_lines();
            }
            return lines;
        }

        let transcript_lines = self
            .transcript
            .iter()
            .rev()
            .take(remaining)
            .rev()
            .map(|text| {
                Line::from(Span::styled(
                    text.clone(),
                    Style::default().fg(self.theme.fg),
                ))
            });
        lines.extend(transcript_lines);
        lines
    }

    fn transcript_placeholder_lines(&self) -> Vec<Line<'static>> {
        let (message, style) = match self.interaction_state {
            InteractionState::Loading => (
                "Loading play surface...",
                Style::default().fg(self.theme.focus),
            ),
            InteractionState::Empty => (
                "No local artifacts yet. Follow the startup guidance below.",
                Style::default().fg(self.theme.unstable),
            ),
            InteractionState::Partial => (
                "Running with partial support. See declaration for detail.",
                Style::default().fg(self.theme.unstable),
            ),
            InteractionState::Error => (
                "Last action failed. See declaration for detail.",
                Style::default().fg(self.theme.diverge),
            ),
            InteractionState::Success => ("Ready.", Style::default().fg(self.theme.converge)),
            InteractionState::Neutral => (
                "Transcript is empty.",
                Style::default().fg(self.theme.fg_dim),
            ),
        };

        vec![Line::from(Span::styled(message.to_string(), style))]
    }

    fn interaction_transcript_lines(&self, max_lines: usize) -> Vec<Line<'static>> {
        if max_lines == 0 {
            return Vec::new();
        }

        let detail = self.interaction_detail.as_deref();
        let mut lines = Vec::new();

        match self.interaction_state {
            InteractionState::Neutral | InteractionState::Success => {}
            InteractionState::Loading => {
                lines.push(Line::from(Span::styled(
                    "STATUS loading startup context".to_string(),
                    Style::default().fg(self.theme.focus),
                )));
                if let Some(detail) = detail {
                    lines.push(Line::from(Span::styled(
                        detail.to_string(),
                        Style::default().fg(self.theme.fg_dim),
                    )));
                }
            }
            InteractionState::Empty => {
                lines.push(Line::from(Span::styled(
                    "STATUS empty local artifact cache".to_string(),
                    Style::default().fg(self.theme.unstable),
                )));
                lines.push(Line::from(Span::styled(
                    "Set MYOSU_BLUEPRINT_DIR or pass --checkpoint and --encoder-dir.".to_string(),
                    Style::default().fg(self.theme.fg_dim),
                )));
            }
            InteractionState::Partial => {
                lines.push(Line::from(Span::styled(
                    "STATUS partial mode".to_string(),
                    Style::default().fg(self.theme.unstable),
                )));
                if let Some(detail) = detail {
                    lines.push(Line::from(Span::styled(
                        detail.to_string(),
                        Style::default().fg(self.theme.fg_dim),
                    )));
                }
            }
            InteractionState::Error => {
                lines.push(Line::from(Span::styled(
                    "STATUS error".to_string(),
                    Style::default().fg(self.theme.diverge),
                )));
                if let Some(detail) = detail {
                    lines.push(Line::from(Span::styled(
                        detail.to_string(),
                        Style::default().fg(self.theme.fg_dim),
                    )));
                }
            }
        }

        lines.truncate(max_lines);
        lines
    }
}

fn truncate_single_line(text: &str, width: u16) -> String {
    let width = width as usize;
    let char_count = text.chars().count();
    if char_count <= width {
        return text.to_string();
    }
    if width == 0 {
        return String::new();
    }
    if width <= 3 {
        return ".".repeat(width);
    }

    let visible = width - 3;
    let prefix: String = text.chars().take(visible).collect();
    format!("{prefix}...")
}

fn interaction_state_color(state: InteractionState, theme: &Theme) -> ratatui::style::Color {
    match state {
        InteractionState::Neutral | InteractionState::Success => theme.fg_bright,
        InteractionState::Loading => theme.focus,
        InteractionState::Empty => theme.unstable,
        InteractionState::Partial => theme.unstable,
        InteractionState::Error => theme.diverge,
    }
}

fn interaction_state_banner(state: InteractionState, detail: Option<&str>) -> String {
    let base = match state {
        InteractionState::Neutral => "THE SYSTEM AWAITS YOUR DECISION",
        InteractionState::Loading => "LOADING",
        InteractionState::Empty => "NO LOCAL ARTIFACTS",
        InteractionState::Partial => "PARTIAL MODE",
        InteractionState::Error => "INPUT ERROR",
        InteractionState::Success => "READY",
    };

    match detail {
        Some(detail) if !detail.is_empty() => format!("{base}: {}", detail.to_ascii_uppercase()),
        _ => base.to_string(),
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
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

    struct LongHeaderRenderer;

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

        fn context_label(&self) -> String {
            "HAND 1".to_string()
        }
    }

    impl GameRenderer for LongHeaderRenderer {
        fn render_state(&self, _area: Rect, _buf: &mut Buffer) {}

        fn desired_height(&self, _width: u16) -> u16 {
            0
        }

        fn declaration(&self) -> &str {
            "THE SYSTEM AWAITS YOUR DECISION"
        }

        fn completions(&self) -> Vec<String> {
            vec!["call".into()]
        }

        fn parse_input(&self, _input: &str) -> Option<String> {
            None
        }

        fn clarify(&self, _input: &str) -> Option<String> {
            None
        }

        fn pipe_output(&self) -> String {
            "STATE mock".into()
        }

        fn game_label(&self) -> &str {
            "ULTRA-LONG-NO-LIMIT-HOLDEM-LABEL"
        }

        fn context_label(&self) -> String {
            "TABLE 9999 WITH EXTREMELY LONG CONTEXT".to_string()
        }
    }

    fn buffer_text(buf: &Buffer) -> String {
        buf.content.iter().map(|cell| cell.symbol()).collect()
    }

    fn buffer_lines(buf: &Buffer, area: Rect) -> Vec<String> {
        let width = area.width as usize;

        buf.content
            .chunks(width)
            .map(|row| row.iter().map(|cell| cell.symbol()).collect())
            .collect()
    }

    #[test]
    fn shell_state_new() {
        let shell = Shell::new();
        assert!(!shell.is_running());
        assert_eq!(shell.transcript().len(), 0);
    }

    #[test]
    fn shell_state_log() {
        let mut shell = Shell::new();
        shell.log("Test message".to_string());
        assert_eq!(shell.transcript().len(), 1);
        assert_eq!(shell.transcript()[0], "Test message");
    }

    #[test]
    fn shell_state_log_limit() {
        let mut shell = Shell::new();
        for i in 0..MAX_TRANSCRIPT_LINES + 100 {
            shell.log(format!("Message {i}"));
        }
        assert_eq!(shell.transcript().len(), MAX_TRANSCRIPT_LINES);
    }

    #[test]
    fn shell_state_stop() {
        let mut shell = Shell::new();
        shell.stop();
        assert!(!shell.is_running());
    }

    #[test]
    fn shell_state_set_status_tracks_interaction_state() {
        let mut shell = Shell::new();

        shell.set_status(
            InteractionState::Partial,
            Some("waiting for live advice".to_string()),
        );

        assert_eq!(shell.interaction_state(), InteractionState::Partial);
    }

    #[test]
    fn shell_state_draw_basic() {
        let shell = Shell::with_screen(Screen::Game);
        let renderer = MockRenderer::active();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));

        shell.draw(Rect::new(0, 0, 80, 24), &mut buf, &renderer);

        // Should have rendered something (not empty buffer)
        let content = buffer_text(&buf);
        assert!(!content.trim().is_empty());
    }

    #[test]
    fn shell_state_draw_too_small() {
        let shell = Shell::new();
        let renderer = MockRenderer::active();
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 5));

        shell.draw(Rect::new(0, 0, 10, 5), &mut buf, &renderer);

        let content = buffer_text(&buf);
        // Check for partial match since text wrapping may break up words in small buffer
        assert!(
            content.contains("small") || content.contains("Terminal"),
            "expected 'small' or 'Terminal' in content: {:?}",
            content
        );
    }

    #[test]
    fn shell_state_layout_calculates_correctly() {
        let shell = Shell::with_screen(Screen::Game);
        let renderer = MockRenderer::active();
        let area = Rect::new(0, 0, 100, 24);
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
        let used = layout.header.height
            + layout.declaration.height
            + layout.input.height
            + layout.state.height;
        assert_eq!(layout.transcript.height, area.height - used);
    }

    #[test]
    fn shell_state_layout_collapses_state_when_inactive() {
        let shell = Shell::with_screen(Screen::Game);
        let renderer = MockRenderer::inactive();
        let area = Rect::new(0, 0, 80, 24);
        let layout = shell.calculate_layout(area, &renderer);

        // State should collapse to 0 when inactive
        assert_eq!(layout.state.height, 0);
    }

    #[test]
    fn shell_state_layout_narrow_collapses_state_panel() {
        let shell = Shell::with_screen(Screen::Game);
        let renderer = MockRenderer::active();
        let area = Rect::new(0, 0, 70, 24);
        let layout = shell.calculate_layout(area, &renderer);

        assert_eq!(shell.layout_tier(area.width), LayoutTier::Narrow);
        assert_eq!(layout.state.height, 0);
        assert_eq!(layout.transcript.width, area.width);
    }

    #[test]
    fn shell_state_layout_desktop_places_transcript_beside_state() {
        let shell = Shell::with_screen(Screen::Game);
        let renderer = MockRenderer::active();
        let area = Rect::new(0, 0, 140, 24);
        let layout = shell.calculate_layout(area, &renderer);

        assert_eq!(shell.layout_tier(area.width), LayoutTier::Desktop);
        assert_eq!(layout.transcript.y, layout.state.y);
        assert_eq!(layout.transcript.height, layout.state.height);
        assert!(layout.state.x > layout.transcript.x);
        assert!(layout.state.width > 0);
    }

    #[test]
    fn shell_state_layout_compact_stacks_transcript_above_state() {
        let shell = Shell::with_screen(Screen::Game);
        let renderer = MockRenderer::active();
        let area = Rect::new(0, 0, 100, 24);
        let layout = shell.calculate_layout(area, &renderer);

        assert_eq!(shell.layout_tier(area.width), LayoutTier::Compact);
        assert_eq!(layout.transcript.width, area.width);
        assert_eq!(layout.state.width, area.width);
        assert!(layout.state.y > layout.transcript.y);
        assert_eq!(layout.state.height, 4);
    }

    #[test]
    fn shell_state_update_completions() {
        let mut shell = Shell::new();
        let renderer = MockRenderer::active();
        shell.update_completions(&renderer);
        // Completions are set internally, just verify it doesn't panic
    }

    #[test]
    fn shell_state_handle_slash_clear() {
        let mut shell = Shell::new();
        shell.log("Test".to_string());
        assert_eq!(shell.transcript().len(), 1);

        let renderer = MockRenderer::active();
        shell.handle_slash_command("/clear".to_string(), &renderer);
        assert_eq!(shell.transcript().len(), 0);
    }

    #[test]
    fn shell_state_handle_key_lobby_submit_routes_to_game() {
        let mut shell = Shell::with_screen(Screen::Lobby);
        let renderer = MockRenderer::inactive();

        shell.handle_key(
            KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE),
            &renderer,
        );
        shell.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), &renderer);

        assert_eq!(shell.current_screen(), Screen::Game);
        assert_eq!(shell.transcript().back(), Some(&"> 1".to_string()));
    }

    #[test]
    fn shell_state_handle_submit_onboarding_completes_to_lobby() {
        let renderer = MockRenderer::inactive();
        let mut shell = Shell::with_screen(Screen::Onboarding);

        shell.handle_submit("start".to_string(), &renderer);

        assert_eq!(shell.current_screen(), Screen::Lobby);
        assert_eq!(shell.interaction_state(), InteractionState::Success);
        assert!(
            shell.transcript().contains(&"> start".to_string()),
            "transcript should include onboarding input"
        );
        assert!(
            shell
                .transcript()
                .contains(&"Onboarding complete. Type 1 to enter the demo hand.".to_string())
        );
    }

    #[test]
    fn shell_state_handle_submit_onboarding_can_route_directly_to_game() {
        let renderer = MockRenderer::inactive();
        let mut shell = Shell::with_screen(Screen::Onboarding);

        shell.handle_submit("1".to_string(), &renderer);

        assert_eq!(shell.current_screen(), Screen::Game);
        assert_eq!(shell.interaction_state(), InteractionState::Success);
    }

    #[test]
    fn shell_state_handle_key_help_toggle_only_when_input_is_empty() {
        let mut shell = Shell::new();
        let renderer = MockRenderer::inactive();

        shell.handle_key(
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
            &renderer,
        );
        assert!(shell.show_help);

        shell.handle_key(
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
            &renderer,
        );
        assert!(!shell.show_help);

        shell.handle_key(
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
            &renderer,
        );
        shell.handle_key(
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
            &renderer,
        );

        assert_eq!(shell.input_text(), "x?".to_string());
        assert!(!shell.show_help);
    }

    #[test]
    fn shell_state_draw_game_screen_renders_all_panels() {
        let area = Rect::new(0, 0, 80, 24);
        let renderer = MockRenderer::active();
        let mut shell = Shell::with_screen(Screen::Game);
        let mut buf = Buffer::empty(area);

        shell.log("Villain raises to 6".to_string());
        shell.handle_key(
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE),
            &renderer,
        );
        shell.handle_key(
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
            &renderer,
        );

        shell.draw(area, &mut buf, &renderer);

        let content = buffer_text(&buf);
        assert!(content.contains("TEST  HAND 1"));
        assert!(content.contains("Villain raises to 6"));
        assert!(content.contains("Mock Game State"));
        assert!(content.contains("THE SYSTEM AWAITS YOUR DECISION"));
        assert!(content.contains("> ca"));
    }

    #[test]
    fn shell_state_render_input_windows_long_commands() {
        let area = Rect::new(0, 0, 40, 12);
        let renderer = MockRenderer::inactive();
        let mut shell = Shell::with_screen(Screen::Lobby);
        let mut buf = Buffer::empty(area);

        for ch in "very long command that should keep the cursor visible".chars() {
            shell.handle_key(
                KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
                &renderer,
            );
        }

        shell.draw(area, &mut buf, &renderer);

        let content = buffer_text(&buf);
        assert!(content.contains("<"));
        assert!(content.contains("cursor visible"));
    }

    #[test]
    fn shell_state_draw_non_game_screens_skip_state_panel() {
        let renderer = MockRenderer::active();
        let area = Rect::new(0, 0, 80, 24);
        let screens = [
            Screen::Onboarding,
            Screen::Lobby,
            Screen::Stats,
            Screen::Coaching,
            Screen::History,
            Screen::Wallet,
            Screen::Spectate,
        ];

        for screen in screens {
            let shell = Shell::with_screen(screen);
            let mut buf = Buffer::empty(area);

            shell.draw(area, &mut buf, &renderer);

            let content = buffer_text(&buf);
            assert!(
                !content.contains("Mock Game State"),
                "screen {screen:?} unexpectedly rendered the game state",
            );
        }
    }

    #[test]
    fn shell_state_transcript_placeholder_reflects_loading() {
        let renderer = MockRenderer::inactive();
        let area = Rect::new(0, 0, 80, 24);
        let mut shell = Shell::with_screen(Screen::Lobby);
        let mut buf = Buffer::empty(area);

        shell.set_status(
            InteractionState::Loading,
            Some("Resolving startup context.".to_string()),
        );
        shell.draw(area, &mut buf, &renderer);

        let content = buffer_text(&buf);
        assert!(content.contains("STATUS loading startup context"));
        assert!(content.contains("Resolving startup context."));
    }

    #[test]
    fn shell_state_narrow_transcript_shows_tail_only() {
        let renderer = MockRenderer::inactive();
        let area = Rect::new(0, 0, 70, 24);
        let mut shell = Shell::with_screen(Screen::Lobby);
        let mut buf = Buffer::empty(area);

        for index in 0..5 {
            shell.log(format!("Message {index}"));
        }

        shell.draw(area, &mut buf, &renderer);

        let content = buffer_text(&buf);
        assert!(!content.contains("Message 0"));
        assert!(!content.contains("Message 1"));
        assert!(content.contains("Message 2"));
        assert!(content.contains("Message 3"));
        assert!(content.contains("Message 4"));
    }

    #[test]
    fn shell_state_transcript_includes_error_status_and_detail() {
        let renderer = MockRenderer::inactive();
        let area = Rect::new(0, 0, 100, 24);
        let mut shell = Shell::with_screen(Screen::Lobby);
        let mut buf = Buffer::empty(area);

        shell.log("Existing transcript line".to_string());
        shell.set_status(
            InteractionState::Error,
            Some("input was not recognized".to_string()),
        );
        shell.draw(area, &mut buf, &renderer);

        let content = buffer_text(&buf);
        assert!(content.contains("STATUS error"));
        assert!(content.contains("input was not recognized"));
        assert!(content.contains("Existing transcript line"));
    }

    #[test]
    fn shell_state_empty_transcript_shows_onboarding_hint() {
        let renderer = MockRenderer::inactive();
        let area = Rect::new(0, 0, 100, 24);
        let mut shell = Shell::with_screen(Screen::Lobby);
        let mut buf = Buffer::empty(area);

        shell.set_status(InteractionState::Empty, None);
        shell.draw(area, &mut buf, &renderer);

        let content = buffer_text(&buf);
        assert!(content.contains("STATUS empty local artifact cache"));
        assert!(content.contains("Set MYOSU_BLUEPRINT_DIR"));
    }

    #[test]
    fn shell_state_draw_lobby_uses_screen_header_and_declaration() {
        let renderer = MockRenderer::active();
        let area = Rect::new(0, 0, 80, 24);
        let shell = Shell::with_screen(Screen::Lobby);
        let mut buf = Buffer::empty(area);

        shell.draw(area, &mut buf, &renderer);

        let content = buffer_text(&buf);
        assert!(content.contains("MYOSU  LOBBY"));
        assert!(content.contains("SELECT A GAME"));
        assert!(!content.contains("THE SYSTEM AWAITS YOUR DECISION"));
    }

    #[test]
    fn shell_state_header_truncates_long_labels() {
        let shell = Shell::with_screen(Screen::Game);
        let renderer = LongHeaderRenderer;
        let area = Rect::new(0, 0, 24, 1);
        let mut buf = Buffer::empty(area);

        shell.render_header(area, &mut buf, &renderer);

        let lines = buffer_lines(&buf, area);
        assert!(lines[0].contains("..."));
        assert!(!lines[0].contains("EXTREMELY LONG CONTEXT"));
    }

    #[test]
    fn shell_state_draw_explicit_error_declaration() {
        let renderer = MockRenderer::active();
        let area = Rect::new(0, 0, 80, 24);
        let mut shell = Shell::with_screen(Screen::Game);
        let mut buf = Buffer::empty(area);

        shell.set_status(
            InteractionState::Error,
            Some("input was not recognized".to_string()),
        );
        shell.draw(area, &mut buf, &renderer);

        let content = buffer_text(&buf);
        assert!(content.contains("INPUT ERROR: INPUT WAS NOT RECOGNIZED"));
        assert!(!content.contains("THE SYSTEM AWAITS YOUR DECISION"));
    }

    #[test]
    fn shell_state_handle_submit_sets_partial_and_error_status() {
        let renderer = MockRenderer::active();
        let mut shell = Shell::with_screen(Screen::Game);

        shell.handle_submit("rx".to_string(), &renderer);
        assert_eq!(shell.interaction_state(), InteractionState::Partial);

        shell.handle_submit("banana".to_string(), &renderer);
        assert_eq!(shell.interaction_state(), InteractionState::Error);
    }

    #[test]
    fn shell_state_handle_submit_sets_success_status() {
        let renderer = MockRenderer::active();
        let mut shell = Shell::with_screen(Screen::Game);

        shell.handle_submit("call".to_string(), &renderer);

        assert_eq!(shell.interaction_state(), InteractionState::Success);
    }

    #[test]
    fn shell_state_handle_update_status_sets_interaction_state() {
        let renderer = MockRenderer::active();
        let mut shell = Shell::with_screen(Screen::Game);

        shell.handle_update(
            UpdateEvent::Status {
                state: InteractionState::Error,
                detail: Some("live advice offline".to_string()),
            },
            &renderer,
        );

        assert_eq!(shell.interaction_state(), InteractionState::Error);
    }

    #[test]
    fn shell_state_render_help_overlay_within_bounds() {
        let area = Rect::new(0, 0, 80, 24);
        let renderer = MockRenderer::inactive();
        let mut shell = Shell::new();
        let mut buf = Buffer::empty(area);

        shell.show_help = true;
        shell.draw(area, &mut buf, &renderer);

        let lines = buffer_lines(&buf, area);
        let title_row = lines
            .iter()
            .position(|line| line.contains("MYOSU TUI HELP"))
            .expect("help title should be rendered");
        let title_col = lines[title_row]
            .find("MYOSU TUI HELP")
            .expect("title should be in line");

        assert!(lines.iter().any(|line| line.contains("Commands:")));
        assert!(lines.iter().any(|line| line.contains("/quit, /q")));
        assert!(title_row > 0);
        assert!(title_row < area.height as usize - 1);
        assert!(title_col > 0);
        assert!(title_col < area.width as usize - "MYOSU TUI HELP".len());
    }

    #[test]
    fn shell_state_default_trait() {
        let shell: Shell = Default::default();
        assert!(!shell.is_running());
    }
}
