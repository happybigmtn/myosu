use myosu_tui::GameRenderer;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::game::ResearchGame;
use crate::protocol::{PortfolioAction, recommended_action};
use crate::solver::PortfolioSolver;

/// Static renderer snapshot for a portfolio-routed research game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortfolioSnapshot {
    pub game: ResearchGame,
    pub decision: String,
    pub solver_family: String,
    pub recommendation: String,
    pub legal_actions: Vec<String>,
    pub context: String,
    pub engine_tier: String,
    pub challenge_id: String,
    pub quality_score: String,
}

impl PortfolioSnapshot {
    /// Build a demo snapshot from the rule-aware portfolio solver.
    pub fn demo(game: ResearchGame) -> Self {
        Self::from_solver(game, &PortfolioSolver::new())
    }

    /// Build a snapshot from a loaded portfolio solver.
    pub fn from_solver(game: ResearchGame, solver: &PortfolioSolver) -> Self {
        if let Ok(snapshot) = Self::typed_from_solver(game, solver) {
            return snapshot;
        }

        Self::legacy_from_solver(game, solver)
    }

    fn typed_from_solver(
        game: ResearchGame,
        solver: &PortfolioSolver,
    ) -> Result<Self, crate::solver::PortfolioSolverError> {
        let query = PortfolioSolver::strength_query(game)?;
        let spot = query.info.challenge.spot().clone();
        let response = solver.answer_strength_checked(query.clone())?;
        let quality = solver.strength_quality(query)?;
        let recommendation = recommended_action(&response)
            .map(PortfolioAction::label)
            .unwrap_or("none")
            .to_string();
        let legal_actions = response
            .actions
            .iter()
            .map(|(action, _)| action.label().to_string())
            .collect();

        Ok(Self {
            game,
            decision: spot.decision,
            solver_family: quality.engine_family,
            recommendation,
            legal_actions,
            context: "RULE-AWARE ENGINE".to_string(),
            engine_tier: quality.engine_tier.as_str().to_string(),
            challenge_id: quality.challenge_id,
            quality_score: format!("{:.6}", quality.score),
        })
    }

    fn legacy_from_solver(game: ResearchGame, solver: &PortfolioSolver) -> Self {
        let query = PortfolioSolver::bootstrap_query(game);
        let response = solver.answer(query.clone());
        let recommendation = recommended_action(&response)
            .map(PortfolioAction::label)
            .unwrap_or("none")
            .to_string();
        let legal_actions = response
            .actions
            .iter()
            .map(|(action, _)| action.label().to_string())
            .collect();

        Self {
            game,
            decision: query.info.decision,
            solver_family: query.info.solver_family,
            recommendation,
            legal_actions,
            context: "LEGACY PORTFOLIO QUERY".to_string(),
            engine_tier: "static-baseline".to_string(),
            challenge_id: format!("{}:bootstrap-v1", game.chain_id()),
            quality_score: "1.000000".to_string(),
        }
    }
}

/// Basic TUI renderer for portfolio-routed research games.
#[derive(Clone, Debug)]
pub struct PortfolioRenderer {
    snapshot: Option<PortfolioSnapshot>,
}

impl PortfolioRenderer {
    /// Create a renderer with an optional active snapshot.
    pub fn new(snapshot: Option<PortfolioSnapshot>) -> Self {
        Self { snapshot }
    }

    /// Create a demo renderer for a portfolio-routed research game.
    pub fn demo(game: ResearchGame) -> Self {
        Self::new(Some(PortfolioSnapshot::demo(game)))
    }

    /// Create an artifact-backed renderer from a loaded solver.
    pub fn from_solver(game: ResearchGame, solver: &PortfolioSolver) -> Self {
        Self::new(Some(PortfolioSnapshot::from_solver(game, solver)))
    }

    fn render_line(area: Rect, buf: &mut Buffer, line_index: u16, text: &str) {
        if line_index >= area.height {
            return;
        }

        let Some(y) = area.y.checked_add(line_index) else {
            return;
        };

        for (index, ch) in text.chars().enumerate() {
            let Ok(offset) = u16::try_from(index) else {
                break;
            };
            let Some(x) = area.x.checked_add(offset) else {
                break;
            };
            if x >= area.right() {
                break;
            }
            buf[(x, y)].set_char(ch);
        }
    }
}

impl GameRenderer for PortfolioRenderer {
    fn render_state(&self, area: Rect, buf: &mut Buffer) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };

        Self::render_line(area, buf, 0, snapshot.game.display_name());
        Self::render_line(area, buf, 1, &format!("spot: {}", snapshot.decision));
        Self::render_line(
            area,
            buf,
            2,
            &format!("advice: {}", snapshot.recommendation),
        );
        Self::render_line(
            area,
            buf,
            3,
            &format!("actions: {}", snapshot.legal_actions.join(", ")),
        );
    }

    fn desired_height(&self, _width: u16) -> u16 {
        if self.snapshot.is_some() { 5 } else { 0 }
    }

    fn declaration(&self) -> &str {
        if self
            .snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.engine_tier == "rule-aware")
        {
            "THE RULE-AWARE ENGINE HAS A LINE"
        } else if self.snapshot.is_some() {
            "THE LEGACY PORTFOLIO SOLVER HAS A LINE"
        } else {
            "NO ACTIVE RESEARCH SPOT"
        }
    }

    fn completions(&self) -> Vec<String> {
        match &self.snapshot {
            Some(snapshot) => snapshot.legal_actions.clone(),
            None => vec!["/quit".to_string()],
        }
    }

    fn parse_input(&self, input: &str) -> Option<String> {
        let normalized = input.trim().to_ascii_lowercase();
        self.completions()
            .into_iter()
            .find(|candidate| candidate.eq_ignore_ascii_case(&normalized))
    }

    fn clarify(&self, _input: &str) -> Option<String> {
        None
    }

    fn pipe_output(&self) -> String {
        match &self.snapshot {
            Some(snapshot) => format!(
                "STATE game={} slug={} decision={} recommend={} engine_tier={} challenge_id={} quality_score={} solver_family={} actions={}",
                snapshot.game.chain_id(),
                snapshot.game.slug(),
                pipe_token(&snapshot.decision),
                snapshot.recommendation,
                snapshot.engine_tier,
                snapshot.challenge_id,
                snapshot.quality_score,
                pipe_token(&snapshot.solver_family),
                snapshot.legal_actions.join("|"),
            ),
            None => "STATE idle".to_string(),
        }
    }

    fn game_label(&self) -> &str {
        match &self.snapshot {
            Some(snapshot) => snapshot.game.slug(),
            None => "PORTFOLIO",
        }
    }

    fn context_label(&self) -> String {
        match &self.snapshot {
            Some(snapshot) => snapshot.context.clone(),
            None => "NO SPOT".to_string(),
        }
    }
}

fn pipe_token(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_whitespace() { '_' } else { ch })
        .collect()
}

#[cfg(test)]
mod tests {
    use myosu_tui::GameRenderer;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    use crate::game::ResearchGame;
    use crate::renderer::{PortfolioRenderer, PortfolioSnapshot};
    use crate::solver::PortfolioSolver;

    #[test]
    fn demo_snapshot_uses_solver_policy() {
        let snapshot = PortfolioSnapshot::demo(ResearchGame::Bridge);

        assert_eq!(snapshot.game, ResearchGame::Bridge);
        assert_eq!(snapshot.recommendation, "double-dummy-play");
        assert_eq!(snapshot.engine_tier, "rule-aware");
        assert!(snapshot.legal_actions.contains(&"bid-contract".to_string()));
    }

    #[test]
    fn artifact_snapshot_tracks_solver_epoch_independently_of_policy() {
        let mut solver = PortfolioSolver::new();
        solver.train(3);
        let snapshot = PortfolioSnapshot::from_solver(ResearchGame::Cribbage, &solver);

        assert_eq!(solver.epochs(), 3);
        assert_eq!(snapshot.recommendation, "peg-run");
        assert_eq!(snapshot.context, "RULE-AWARE ENGINE");
        assert!(snapshot.legal_actions.contains(&"keep-crib".to_string()));
    }

    #[test]
    fn active_renderer_reports_shell_contract() {
        let renderer = PortfolioRenderer::demo(ResearchGame::RiichiMahjong);

        assert_eq!(renderer.desired_height(80), 5);
        assert_eq!(renderer.game_label(), "riichi-mahjong");
        assert_eq!(renderer.declaration(), "THE RULE-AWARE ENGINE HAS A LINE");
        assert_eq!(renderer.context_label(), "RULE-AWARE ENGINE");
        assert!(renderer.pipe_output().contains("game=riichi_mahjong"));
        assert!(renderer.pipe_output().contains("engine_tier=rule-aware"));
        assert_eq!(
            renderer.parse_input("declare-riichi"),
            Some("declare-riichi".to_string())
        );
        assert_eq!(renderer.clarify("declare"), None);
    }

    #[test]
    fn inactive_renderer_collapses() {
        let renderer = PortfolioRenderer::new(None);

        assert_eq!(renderer.desired_height(80), 0);
        assert_eq!(renderer.declaration(), "NO ACTIVE RESEARCH SPOT");
        assert_eq!(renderer.pipe_output(), "STATE idle");
        assert_eq!(renderer.completions(), vec!["/quit".to_string()]);
    }

    #[test]
    fn render_state_writes_snapshot_lines() {
        let renderer = PortfolioRenderer::demo(ResearchGame::Backgammon);
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 5));

        renderer.render_state(Rect::new(0, 0, 80, 5), &mut buffer);

        let first_line: String = (0..10)
            .map(|x| buffer[(x, 0)].symbol().chars().next().unwrap_or(' '))
            .collect();
        assert_eq!(first_line, "Backgammon");
    }
}
