//! `myosu-play` — Myosu gameplay TUI binary.
//!
//! Interactive NLHE poker training with blueprint and heuristic bots.
//! Run with `--train` for local training mode.

use anyhow::Context;
use clap::Parser;
use myosu_tui::shell::Shell;
use myosu_tui::{renderer::GameRenderer, screens::Screen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use tracing_subscriber::EnvFilter;

/// Command-line arguments for myosu-play.
#[derive(Parser, Debug)]
#[command(name = "myosu-play")]
#[command(about = "Myosu NLHE poker training TUI", long_about = None)]
struct Args {
    /// Local training mode (no chain required).
    #[arg(long)]
    train: bool,

    /// Chain websocket URL for miner-connected play.
    #[arg(long)]
    chain: Option<String>,

    /// Pipe mode for agent protocol (non-interactive).
    #[arg(long)]
    pipe: bool,
}

/// Stub NLHE renderer for Slice 1 — hardcoded state, proves the render loop.
///
/// Full NLHE rendering with real game states comes in Slice 2.
struct NlheRenderer {
    hand_active: bool,
}

impl NlheRenderer {
    fn new() -> Self {
        Self { hand_active: true }
    }
}

impl GameRenderer for NlheRenderer {
    fn render_state(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        if !self.hand_active || area.width < 2 || area.height < 1 {
            return;
        }

        let line = "pot: 12bb  hero: A♠ K♥  board: T♠ 7♥ 2♣";
        let x = area.x;
        let y = area.y;

        for (i, ch) in line.chars().enumerate() {
            if x + i as u16 >= area.right() {
                break;
            }
            buf[(x + i as u16, y)].set_char(ch);
        }

        let line2 = "bot: 47bb  action: hero's turn";
        for (i, ch) in line2.chars().enumerate() {
            if x + i as u16 >= area.right() {
                break;
            }
            if y + 1 < area.bottom() {
                buf[(x + i as u16, y + 1)].set_char(ch);
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
            vec!["deal".into(), "quit".into()]
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
            "STATE hand=1 pot=12 hero=AK board=T72".into()
        } else {
            "STATE idle".into()
        }
    }

    fn game_label(&self) -> &str {
        "NLHE-HU"
    }

    fn context_label(&self) -> &str {
        "HAND 1"
    }
}

fn run_train_mode() -> anyhow::Result<()> {
    let renderer = NlheRenderer::new();
    let shell = Shell::with_screen(Screen::Game);

    let backend = CrosstermBackend::new(io::stderr());
    let mut terminal = Terminal::new(backend).context("failed to create terminal")?;

    // Initial render to prove the render loop works
    terminal
        .draw(|f| {
            let area = f.area();
            let mut buf = ratatui::buffer::Buffer::empty(area);
            shell.draw(area, &mut buf, &renderer);
            f.buffer_mut().clone_from(&buf);
        })
        .context("render")?;

    // For Slice 1, we prove the render loop works.
    // Full async event loop integration comes in later slices.
    eprintln!("[Slice 1] Render loop proven. Shell draws 5-panel layout without panic.");

    Ok(())
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    if args.train {
        run_train_mode()?;
    } else if args.pipe {
        anyhow::bail!("--pipe mode not yet implemented");
    } else if args.chain.is_some() {
        anyhow::bail!("--chain mode not yet implemented (depends on chain:runtime lane)");
    } else {
        eprintln!("myosu-play 0.1.0");
        eprintln!("Usage: myosu-play --train    Local training mode");
        eprintln!("       myosu-play --chain WS  Miner-connected play");
        eprintln!("       myosu-play --pipe      Agent protocol mode");
        std::process::exit(1);
    }

    Ok(())
}
