use crate::game::{KuhnCard, KuhnEdge, KuhnGame, KuhnHistory, KuhnInfo, KuhnTurn};
use myosu_games::{CfrGame, Probability, StrategyResponse, Utility};
use rbp_mccfr::{CfrInfo, CfrPublic};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

const ONE_THIRD: Probability = 1.0 / 3.0;
const CHECKPOINT_MAGIC: [u8; 4] = *b"MYOK";
const CHECKPOINT_VERSION: u32 = 1;
const CHECKPOINT_HEADER_LEN: usize = 8;

/// Closed-form exact solver for standard two-player Kuhn poker.
#[derive(Clone, Debug, Default)]
pub struct KuhnSolver;

impl KuhnSolver {
    /// Create a new exact Kuhn poker solver.
    pub fn new() -> Self {
        Self
    }

    /// Load an exact solver checkpoint from disk.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, KuhnSolverError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| KuhnSolverError::Read {
            path: path.display().to_string(),
            source,
        })?;

        Self::from_checkpoint_bytes(&bytes)
    }

    /// Decode an exact solver from checkpoint bytes.
    pub fn from_checkpoint_bytes(bytes: &[u8]) -> Result<Self, KuhnSolverError> {
        if bytes.len() < CHECKPOINT_HEADER_LEN {
            return Err(KuhnSolverError::CheckpointTooShort { bytes: bytes.len() });
        }

        let header = bytes
            .get(..CHECKPOINT_HEADER_LEN)
            .ok_or(KuhnSolverError::CheckpointTooShort { bytes: bytes.len() })?;
        let (magic, version) = header.split_at(4);
        let found_magic: [u8; 4] = magic
            .try_into()
            .map_err(|_| KuhnSolverError::CheckpointTooShort { bytes: bytes.len() })?;
        if found_magic != CHECKPOINT_MAGIC {
            return Err(KuhnSolverError::CheckpointMagic {
                found: String::from_utf8_lossy(&found_magic).into_owned(),
            });
        }

        let found_version = u32::from_le_bytes(
            version
                .try_into()
                .map_err(|_| KuhnSolverError::CheckpointTooShort { bytes: bytes.len() })?,
        );
        if found_version != CHECKPOINT_VERSION {
            return Err(KuhnSolverError::CheckpointVersion {
                found: found_version,
                expected: CHECKPOINT_VERSION,
            });
        }

        Ok(Self)
    }

    /// Save the exact solver checkpoint to disk.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), KuhnSolverError> {
        let path = path.as_ref();
        fs::write(path, self.checkpoint_bytes()).map_err(|source| KuhnSolverError::Write {
            path: path.display().to_string(),
            source,
        })
    }

    /// Serialize the exact solver checkpoint to bytes.
    pub fn checkpoint_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(CHECKPOINT_HEADER_LEN);
        bytes.extend_from_slice(&CHECKPOINT_MAGIC);
        bytes.extend_from_slice(&CHECKPOINT_VERSION.to_le_bytes());
        bytes
    }

    /// Return the fixed epoch count for the closed-form solver.
    pub const fn epochs(&self) -> usize {
        0
    }

    /// Accept a training request for CLI symmetry. The closed-form solver is already exact.
    pub const fn train(&mut self, _iterations: usize) {}

    /// Answer a wire-safe Kuhn strategy query.
    pub fn answer(
        &self,
        query: crate::protocol::KuhnStrategyQuery,
    ) -> crate::protocol::KuhnStrategyResponse {
        self.strategy(query.info)
    }

    /// Return the highest-probability action for a wire-safe query.
    pub fn recommend_query(&self, query: crate::protocol::KuhnStrategyQuery) -> Option<KuhnEdge> {
        crate::protocol::recommended_edge(&self.answer(query))
    }

    /// Return the equilibrium strategy for a specific information set.
    pub fn strategy(&self, info: KuhnInfo) -> StrategyResponse<KuhnEdge> {
        use KuhnCard::{Jack, King, Queen};
        use KuhnEdge::{Bet, Call, Check, Fold};

        let actions = match (info.public().history(), info.card()) {
            (KuhnHistory::Opening, Jack) => vec![(Check, 1.0 - ONE_THIRD), (Bet, ONE_THIRD)],
            (KuhnHistory::Opening, Queen) => vec![(Check, 1.0)],
            (KuhnHistory::Opening, King) => vec![(Bet, 1.0)],
            (KuhnHistory::P1Checked, Jack) => vec![(Check, 1.0 - ONE_THIRD), (Bet, ONE_THIRD)],
            (KuhnHistory::P1Checked, Queen) => vec![(Check, 1.0)],
            (KuhnHistory::P1Checked, King) => vec![(Bet, 1.0)],
            (KuhnHistory::P1Bet, Jack) => vec![(Fold, 1.0)],
            (KuhnHistory::P1Bet, Queen) => vec![(Fold, 1.0 - ONE_THIRD), (Call, ONE_THIRD)],
            (KuhnHistory::P1Bet, King) => vec![(Call, 1.0)],
            (KuhnHistory::P1CheckP2Bet, Jack) => vec![(Fold, 1.0)],
            (KuhnHistory::P1CheckP2Bet, Queen) => {
                vec![(Fold, 1.0 - ONE_THIRD), (Call, ONE_THIRD)]
            }
            (KuhnHistory::P1CheckP2Bet, King) => vec![(Call, 1.0)],
            (history, _) => panic!("strategy requested for unsupported Kuhn history: {history:?}"),
        };

        let response = StrategyResponse::new(actions);
        assert!(
            response.is_valid(),
            "kuhn strategy must be a valid distribution"
        );
        response
    }

    /// Return the entire equilibrium profile for the 12 Kuhn information sets.
    pub fn profile(&self) -> BTreeMap<KuhnInfo, StrategyResponse<KuhnEdge>> {
        let mut profile = BTreeMap::new();

        for dealt in opening_deals() {
            collect_profile(self, dealt, &mut profile);
        }

        profile
    }

    /// Return the exact equilibrium value for player one.
    pub fn expected_value(&self) -> Utility {
        self.evaluate(KuhnGame::root())
    }

    fn evaluate(&self, game: KuhnGame) -> Utility {
        match game.turn() {
            KuhnTurn::Chance => {
                let deals = opening_deals();
                let weight = 1.0 / deals.len() as Utility;
                deals
                    .iter()
                    .map(|dealt| weight * self.evaluate(*dealt))
                    .sum()
            }
            KuhnTurn::PlayerOne | KuhnTurn::PlayerTwo => {
                let info = game.info().expect("player turns should expose info");
                self.strategy(info)
                    .actions
                    .into_iter()
                    .map(|(edge, probability)| probability * self.evaluate(game.apply(edge)))
                    .sum()
            }
            KuhnTurn::Terminal => game.payoff(KuhnTurn::PlayerOne),
        }
    }
}

/// Errors returned by the exact Kuhn solver checkpoint surface.
#[derive(Debug, Error)]
pub enum KuhnSolverError {
    #[error("failed to read checkpoint `{path}`: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to write checkpoint `{path}`: {source}")]
    Write {
        path: String,
        source: std::io::Error,
    },
    #[error("checkpoint too short: {bytes} bytes")]
    CheckpointTooShort { bytes: usize },
    #[error("checkpoint magic mismatch: found `{found}`, expected `MYOK`")]
    CheckpointMagic { found: String },
    #[error("checkpoint version {found} does not match expected version {expected}")]
    CheckpointVersion { found: u32, expected: u32 },
}

fn collect_profile(
    solver: &KuhnSolver,
    game: KuhnGame,
    profile: &mut BTreeMap<KuhnInfo, StrategyResponse<KuhnEdge>>,
) {
    if let Some(info) = game.info() {
        profile.entry(info).or_insert_with(|| solver.strategy(info));
    }

    match game.turn() {
        KuhnTurn::Chance => {
            for dealt in opening_deals() {
                collect_profile(solver, dealt, profile);
            }
        }
        KuhnTurn::PlayerOne | KuhnTurn::PlayerTwo => {
            let info = game.info().expect("player turn should expose info");
            for edge in info.public().choices() {
                collect_profile(solver, game.apply(edge), profile);
            }
        }
        KuhnTurn::Terminal => {}
    }
}

fn opening_deals() -> [KuhnGame; 6] {
    let mut deals = [KuhnGame::root(); 6];
    let mut index = 0;

    for p1 in KuhnCard::all() {
        for p2 in KuhnCard::all() {
            if p1 != p2 {
                deals[index] = KuhnGame::root().apply(KuhnEdge::Deal { p1, p2 });
                index += 1;
            }
        }
    }

    deals
}

#[cfg(test)]
mod tests {
    use super::{KuhnSolver, ONE_THIRD};
    use crate::game::{KuhnCard, KuhnEdge, KuhnGame};
    use myosu_games::{CfrGame, Probability};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn opening_info(card: KuhnCard) -> crate::game::KuhnInfo {
        KuhnGame::root()
            .apply(KuhnEdge::Deal {
                p1: card,
                p2: match card {
                    KuhnCard::Jack => KuhnCard::Queen,
                    KuhnCard::Queen => KuhnCard::King,
                    KuhnCard::King => KuhnCard::Jack,
                },
            })
            .info()
            .expect("opening state should expose info")
    }

    fn approx_eq(left: Probability, right: Probability) {
        assert!(
            (left - right).abs() < 1e-6,
            "left={left} right={right} differ by more than tolerance"
        );
    }

    #[test]
    fn profile_covers_all_twelve_information_sets() {
        let solver = KuhnSolver::new();

        assert_eq!(solver.profile().len(), 12);
    }

    #[test]
    fn opening_strategies_match_closed_form_solution() {
        let solver = KuhnSolver::new();

        let jack = solver.strategy(opening_info(KuhnCard::Jack));
        approx_eq(jack.probability_for(&KuhnEdge::Bet), ONE_THIRD);
        approx_eq(jack.probability_for(&KuhnEdge::Check), 1.0 - ONE_THIRD);

        let queen = solver.strategy(opening_info(KuhnCard::Queen));
        approx_eq(queen.probability_for(&KuhnEdge::Check), 1.0);
        approx_eq(queen.probability_for(&KuhnEdge::Bet), 0.0);

        let king = solver.strategy(opening_info(KuhnCard::King));
        approx_eq(king.probability_for(&KuhnEdge::Bet), 1.0);
        approx_eq(king.probability_for(&KuhnEdge::Check), 0.0);
    }

    #[test]
    fn facing_bet_strategies_match_closed_form_solution() {
        let solver = KuhnSolver::new();
        let info = KuhnGame::root()
            .apply(KuhnEdge::Deal {
                p1: KuhnCard::Queen,
                p2: KuhnCard::Jack,
            })
            .apply(KuhnEdge::Check)
            .apply(KuhnEdge::Bet)
            .info()
            .expect("facing a bet should expose info");
        let response = solver.strategy(info);

        approx_eq(response.probability_for(&KuhnEdge::Call), ONE_THIRD);
        approx_eq(response.probability_for(&KuhnEdge::Fold), 1.0 - ONE_THIRD);
    }

    #[test]
    fn expected_value_matches_closed_form() {
        let solver = KuhnSolver::new();
        let expected = -1.0 / 18.0;

        assert!(
            (solver.expected_value() - expected).abs() < 1e-6,
            "kuhn value should match the closed-form equilibrium"
        );
    }

    #[test]
    fn checkpoint_roundtrips_exact_solver() {
        let root = unique_temp_root();
        let checkpoint = root.join("kuhn.bin");
        let solver = KuhnSolver::new();

        std::fs::create_dir_all(&root).expect("temp root should exist");
        solver.save(&checkpoint).expect("checkpoint should save");
        let restored = KuhnSolver::load(&checkpoint).expect("checkpoint should load");

        assert_eq!(restored.epochs(), 0);
        assert_eq!(restored.profile(), solver.profile());

        let _ = std::fs::remove_dir_all(root);
    }

    fn unique_temp_root() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("myosu-kuhn-solver-{nanos}"))
    }
}
