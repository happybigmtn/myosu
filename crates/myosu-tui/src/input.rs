use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

const MAX_HISTORY: usize = 100;

#[derive(Debug, PartialEq, Eq)]
pub enum InputAction {
    Submit(String),
    SlashCommand(String),
    Continue,
}

struct TabState {
    word_start: usize,
    matches: Vec<String>,
    index: usize,
}

pub struct InputLine {
    buf: Vec<char>,
    cursor: usize,
    history: Vec<String>,
    history_pos: Option<usize>,
    saved_buf: String,
    completions: Vec<String>,
    tab_state: Option<TabState>,
}

impl Default for InputLine {
    fn default() -> Self {
        Self::new()
    }
}

impl InputLine {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            cursor: 0,
            history: Vec::new(),
            history_pos: None,
            saved_buf: String::new(),
            completions: Vec::new(),
            tab_state: None,
        }
    }

    pub fn set_completions(&mut self, completions: Vec<String>) {
        self.completions = completions;
    }

    pub fn text(&self) -> String {
        self.buf.iter().collect()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn viewport(&self, width: usize) -> (String, usize) {
        if width == 0 {
            return (String::new(), 0);
        }

        if self.buf.len() <= width {
            return (self.text(), self.cursor);
        }

        let mut budget = width;
        let mut leading = false;
        let mut trailing = false;

        if width > 2 {
            budget -= 1;
            leading = true;
            trailing = true;
            budget -= 1;
        }

        let mut start = self.cursor.saturating_sub(budget / 2);
        if start + budget > self.buf.len() {
            start = self.buf.len() - budget;
        }
        let end = start + budget;

        let mut visible = String::new();
        let mut cursor_offset = self.cursor.saturating_sub(start);

        if leading && start > 0 {
            visible.push('<');
            cursor_offset += 1;
        }

        for ch in &self.buf[start..end] {
            visible.push(*ch);
        }

        if trailing && end < self.buf.len() {
            visible.push('>');
        }

        (visible, cursor_offset)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> InputAction {
        if key.kind != KeyEventKind::Press {
            return InputAction::Continue;
        }

        if key.code != KeyCode::Tab {
            self.tab_state = None;
        }

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('a') if ctrl => {
                self.cursor = 0;
                InputAction::Continue
            }
            KeyCode::Char('e') if ctrl => {
                self.cursor = self.buf.len();
                InputAction::Continue
            }
            KeyCode::Char('w') if ctrl => {
                self.delete_word_backward();
                InputAction::Continue
            }
            KeyCode::Char('u') if ctrl => {
                self.buf.drain(..self.cursor);
                self.cursor = 0;
                InputAction::Continue
            }
            KeyCode::Char('k') if ctrl => {
                self.buf.truncate(self.cursor);
                InputAction::Continue
            }
            KeyCode::Enter => self.submit(),
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.buf.remove(self.cursor);
                }
                InputAction::Continue
            }
            KeyCode::Delete => {
                if self.cursor < self.buf.len() {
                    self.buf.remove(self.cursor);
                }
                InputAction::Continue
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                InputAction::Continue
            }
            KeyCode::Right => {
                if self.cursor < self.buf.len() {
                    self.cursor += 1;
                }
                InputAction::Continue
            }
            KeyCode::Up => {
                self.history_prev();
                InputAction::Continue
            }
            KeyCode::Down => {
                self.history_next();
                InputAction::Continue
            }
            KeyCode::Tab => {
                self.tab_complete();
                InputAction::Continue
            }
            KeyCode::Char(c) if !ctrl => {
                self.buf.insert(self.cursor, c);
                self.cursor += 1;
                InputAction::Continue
            }
            _ => InputAction::Continue,
        }
    }

    fn submit(&mut self) -> InputAction {
        let text: String = self.buf.iter().collect();
        let trimmed = text.trim().to_string();

        if trimmed.is_empty() {
            return InputAction::Continue;
        }

        // Avoid consecutive duplicates in history
        if self.history.last() != Some(&trimmed) {
            self.history.push(trimmed.clone());
            if self.history.len() > MAX_HISTORY {
                self.history.remove(0);
            }
        }

        self.buf.clear();
        self.cursor = 0;
        self.history_pos = None;
        self.saved_buf.clear();

        if trimmed.starts_with('/') {
            InputAction::SlashCommand(trimmed)
        } else {
            InputAction::Submit(trimmed)
        }
    }

    fn delete_word_backward(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let mut pos = self.cursor;

        // Skip trailing spaces
        while pos > 0 && self.buf[pos - 1] == ' ' {
            pos -= 1;
        }
        // Skip non-space characters (the word)
        while pos > 0 && self.buf[pos - 1] != ' ' {
            pos -= 1;
        }

        self.buf.drain(pos..self.cursor);
        self.cursor = pos;
    }

    fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_pos {
            None => {
                self.saved_buf = self.text();
                self.history_pos = Some(self.history.len() - 1);
            }
            Some(pos) if pos > 0 => {
                self.history_pos = Some(pos - 1);
            }
            _ => return,
        }

        let entry = self.history[self.history_pos.unwrap()].clone();
        self.buf = entry.chars().collect();
        self.cursor = self.buf.len();
    }

    fn history_next(&mut self) {
        match self.history_pos {
            None => {}
            Some(pos) if pos + 1 < self.history.len() => {
                self.history_pos = Some(pos + 1);
                let entry = self.history[pos + 1].clone();
                self.buf = entry.chars().collect();
                self.cursor = self.buf.len();
            }
            Some(_) => {
                self.history_pos = None;
                self.buf = self.saved_buf.chars().collect();
                self.cursor = self.buf.len();
                self.saved_buf.clear();
            }
        }
    }

    fn tab_complete(&mut self) {
        if let Some(mut state) = self.tab_state.take() {
            if !state.matches.is_empty() {
                state.index = (state.index + 1) % state.matches.len();
                let replacement: Vec<char> = state.matches[state.index].chars().collect();
                self.buf.drain(state.word_start..self.cursor);
                let len = replacement.len();
                for (i, c) in replacement.into_iter().enumerate() {
                    self.buf.insert(state.word_start + i, c);
                }
                self.cursor = state.word_start + len;
            }
            self.tab_state = Some(state);
        } else {
            let (word_start, prefix) = self.current_word();
            if prefix.is_empty() {
                return;
            }

            let matches: Vec<String> = self
                .completions
                .iter()
                .filter(|c| c.starts_with(&prefix))
                .cloned()
                .collect();

            if matches.is_empty() {
                return;
            }

            let replacement: Vec<char> = matches[0].chars().collect();
            self.buf.drain(word_start..self.cursor);
            let len = replacement.len();
            for (i, c) in replacement.into_iter().enumerate() {
                self.buf.insert(word_start + i, c);
            }
            self.cursor = word_start + len;

            self.tab_state = Some(TabState {
                word_start,
                matches,
                index: 0,
            });
        }
    }

    fn current_word(&self) -> (usize, String) {
        let mut start = self.cursor;
        while start > 0 && self.buf[start - 1] != ' ' {
            start -= 1;
        }
        let word: String = self.buf[start..self.cursor].iter().collect();
        (start, word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventState;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn ctrl(c: char) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn type_str(input: &mut InputLine, s: &str) {
        for c in s.chars() {
            input.handle_key(press(KeyCode::Char(c)));
        }
    }

    #[test]
    fn type_and_submit() {
        let mut input = InputLine::new();
        type_str(&mut input, "raise 15");
        assert_eq!(input.text(), "raise 15");
        assert_eq!(
            input.handle_key(press(KeyCode::Enter)),
            InputAction::Submit("raise 15".into())
        );
        assert_eq!(input.text(), "");
    }

    #[test]
    fn empty_submit_is_noop() {
        let mut input = InputLine::new();
        assert_eq!(
            input.handle_key(press(KeyCode::Enter)),
            InputAction::Continue
        );
    }

    #[test]
    fn history_navigation() {
        let mut input = InputLine::new();

        type_str(&mut input, "fold");
        input.handle_key(press(KeyCode::Enter));

        type_str(&mut input, "call");
        input.handle_key(press(KeyCode::Enter));

        // Up recalls "call"
        input.handle_key(press(KeyCode::Up));
        assert_eq!(input.text(), "call");

        // Up again recalls "fold"
        input.handle_key(press(KeyCode::Up));
        assert_eq!(input.text(), "fold");

        // Down goes back to "call"
        input.handle_key(press(KeyCode::Down));
        assert_eq!(input.text(), "call");

        // Down again restores empty buffer
        input.handle_key(press(KeyCode::Down));
        assert_eq!(input.text(), "");
    }

    #[test]
    fn history_saves_current_buffer() {
        let mut input = InputLine::new();

        type_str(&mut input, "fold");
        input.handle_key(press(KeyCode::Enter));

        // Type partial input, then navigate history
        type_str(&mut input, "ra");
        input.handle_key(press(KeyCode::Up));
        assert_eq!(input.text(), "fold");

        // Down restores the partial input
        input.handle_key(press(KeyCode::Down));
        assert_eq!(input.text(), "ra");
    }

    #[test]
    fn tab_completion() {
        let mut input = InputLine::new();
        input.set_completions(vec![
            "raise".into(),
            "call".into(),
            "fold".into(),
            "check".into(),
        ]);

        type_str(&mut input, "ra");
        input.handle_key(press(KeyCode::Tab));
        assert_eq!(input.text(), "raise");
    }

    #[test]
    fn tab_completion_cycles() {
        let mut input = InputLine::new();
        input.set_completions(vec!["call".into(), "check".into()]);

        type_str(&mut input, "c");
        input.handle_key(press(KeyCode::Tab));
        assert_eq!(input.text(), "call");

        input.handle_key(press(KeyCode::Tab));
        assert_eq!(input.text(), "check");

        // Wraps around
        input.handle_key(press(KeyCode::Tab));
        assert_eq!(input.text(), "call");
    }

    #[test]
    fn tab_completion_no_match() {
        let mut input = InputLine::new();
        input.set_completions(vec!["call".into(), "fold".into()]);

        type_str(&mut input, "xyz");
        input.handle_key(press(KeyCode::Tab));
        assert_eq!(input.text(), "xyz");
    }

    #[test]
    fn ctrl_w_deletes_word() {
        let mut input = InputLine::new();
        type_str(&mut input, "raise 15");
        input.handle_key(ctrl('w'));
        assert_eq!(input.text(), "raise ");
    }

    #[test]
    fn ctrl_w_deletes_word_with_trailing_spaces() {
        let mut input = InputLine::new();
        type_str(&mut input, "raise   ");
        input.handle_key(ctrl('w'));
        assert_eq!(input.text(), "");
    }

    #[test]
    fn ctrl_u_deletes_to_start() {
        let mut input = InputLine::new();
        type_str(&mut input, "raise 15");
        // Move cursor left 2 positions
        input.handle_key(press(KeyCode::Left));
        input.handle_key(press(KeyCode::Left));
        input.handle_key(ctrl('u'));
        assert_eq!(input.text(), "15");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn ctrl_k_deletes_to_end() {
        let mut input = InputLine::new();
        type_str(&mut input, "raise 15");
        // Ctrl-A to start, then right 5 to after "raise"
        input.handle_key(ctrl('a'));
        for _ in 0..5 {
            input.handle_key(press(KeyCode::Right));
        }
        input.handle_key(ctrl('k'));
        assert_eq!(input.text(), "raise");
    }

    #[test]
    fn ctrl_a_and_ctrl_e() {
        let mut input = InputLine::new();
        type_str(&mut input, "hello");
        assert_eq!(input.cursor(), 5);

        input.handle_key(ctrl('a'));
        assert_eq!(input.cursor(), 0);

        input.handle_key(ctrl('e'));
        assert_eq!(input.cursor(), 5);
    }

    #[test]
    fn slash_command_detected() {
        let mut input = InputLine::new();
        type_str(&mut input, "/quit");
        assert_eq!(
            input.handle_key(press(KeyCode::Enter)),
            InputAction::SlashCommand("/quit".into())
        );
    }

    #[test]
    fn slash_command_with_args() {
        let mut input = InputLine::new();
        type_str(&mut input, "/stats session");
        assert_eq!(
            input.handle_key(press(KeyCode::Enter)),
            InputAction::SlashCommand("/stats session".into())
        );
    }

    #[test]
    fn backspace_and_delete() {
        let mut input = InputLine::new();
        type_str(&mut input, "abc");

        input.handle_key(press(KeyCode::Backspace));
        assert_eq!(input.text(), "ab");

        input.handle_key(ctrl('a'));
        input.handle_key(press(KeyCode::Delete));
        assert_eq!(input.text(), "b");
    }

    #[test]
    fn history_no_consecutive_duplicates() {
        let mut input = InputLine::new();

        type_str(&mut input, "call");
        input.handle_key(press(KeyCode::Enter));
        type_str(&mut input, "call");
        input.handle_key(press(KeyCode::Enter));

        // Only one "call" in history
        input.handle_key(press(KeyCode::Up));
        assert_eq!(input.text(), "call");

        // No more entries
        input.handle_key(press(KeyCode::Up));
        assert_eq!(input.text(), "call");
    }

    #[test]
    fn ignores_key_release_events() {
        let mut input = InputLine::new();
        let release = KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
        };
        assert_eq!(input.handle_key(release), InputAction::Continue);
        assert_eq!(input.text(), "");
    }

    #[test]
    fn viewport_keeps_cursor_visible_for_long_input() {
        let mut input = InputLine::new();
        type_str(&mut input, "raise to 123456789 chips");

        let (visible, cursor) = input.viewport(10);

        assert!(visible.starts_with('<'));
        assert!(visible.len() <= 10);
        assert!(cursor <= visible.len());
        assert!(visible.contains("chips"));
    }

    #[test]
    fn viewport_shows_start_without_leading_marker_when_cursor_near_front() {
        let mut input = InputLine::new();
        type_str(&mut input, "raise to 123456789 chips");
        input.handle_key(ctrl('a'));
        for _ in 0..3 {
            input.handle_key(press(KeyCode::Right));
        }

        let (visible, cursor) = input.viewport(10);

        assert!(!visible.starts_with('<'));
        assert!(visible.ends_with('>'));
        assert_eq!(cursor, 3);
    }
}
