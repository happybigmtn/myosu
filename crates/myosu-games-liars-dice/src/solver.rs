use crate::game::{
    LiarsDiceEdge, LiarsDiceGame, LiarsDiceInfo, LiarsDicePublic, LiarsDiceSecret, LiarsDiceTurn,
    NO_CLAIM,
};
use crate::protocol::{LiarsDiceStrategyQuery, LiarsDiceStrategyResponse, recommended_edge};
use bincode::Options;
use myosu_games::{Encoder, Probability, Profile, StrategyResponse, Utility};
use rbp_mccfr::{
    Branch, CfrPublic, ExternalSampling, FlooredRegret, LinearWeight, Policy, Solver, Tree,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use thiserror::Error;

const LIARS_DICE_BATCH_SIZE: usize = 16;
const DIE_FACES: usize = 6;
const MAX_DECODE_BYTES: u64 = 1_048_576;
const CHECKPOINT_MAGIC: [u8; 4] = *b"MYOS";
const CHECKPOINT_VERSION: u32 = 1;
const CHECKPOINT_HEADER_LEN: usize = 8;

type Encounter = (Probability, Utility, Utility, u32);
type EncounterMap = BTreeMap<LiarsDiceEdge, Encounter>;

/// Fixed-configuration solver for the minimal Liar's Dice proof game.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiarsDiceSolver<const N: usize> {
    epochs: usize,
    encounters: BTreeMap<LiarsDiceInfo, EncounterMap>,
}

impl<const N: usize> Default for LiarsDiceSolver<N> {
    fn default() -> Self {
        Self {
            epochs: 0,
            encounters: BTreeMap::new(),
        }
    }
}

impl<const N: usize> Encoder for LiarsDiceSolver<N> {
    type T = LiarsDiceTurn;
    type E = LiarsDiceEdge;
    type G = LiarsDiceGame;
    type I = LiarsDiceInfo;

    fn seed(&self, game: &Self::G) -> Self::I {
        game.encoded_info()
    }

    fn info(
        &self,
        _tree: &Tree<Self::T, Self::E, Self::G, Self::I>,
        (_edge, game, _index): Branch<Self::E, Self::G>,
    ) -> Self::I {
        game.encoded_info()
    }

    fn resume(&self, _past: &[Self::E], game: &Self::G) -> Self::I {
        game.encoded_info()
    }
}

impl<const N: usize> Profile for LiarsDiceSolver<N> {
    type T = LiarsDiceTurn;
    type E = LiarsDiceEdge;
    type G = LiarsDiceGame;
    type I = LiarsDiceInfo;

    fn increment(&mut self) {
        self.epochs += 1;
    }

    fn walker(&self) -> Self::T {
        match self.epochs % 2 {
            0 => LiarsDiceTurn::P1,
            _ => LiarsDiceTurn::P2,
        }
    }

    fn epochs(&self) -> usize {
        self.epochs
    }

    fn cum_weight(&self, info: &Self::I, edge: &Self::E) -> Probability {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(weight, _, _, _)| *weight)
            .unwrap_or_default()
    }

    fn cum_regret(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(_, regret, _, _)| *regret)
            .unwrap_or_default()
    }

    fn cum_evalue(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(_, _, evalue, _)| *evalue)
            .unwrap_or_default()
    }

    fn cum_counts(&self, info: &Self::I, edge: &Self::E) -> u32 {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(_, _, _, counts)| *counts)
            .unwrap_or_default()
    }
}

impl<const N: usize> Solver for LiarsDiceSolver<N> {
    type T = LiarsDiceTurn;
    type E = LiarsDiceEdge;
    type X = crate::game::LiarsDicePublic;
    type Y = crate::game::LiarsDiceSecret;
    type I = LiarsDiceInfo;
    type G = LiarsDiceGame;
    type P = Self;
    type N = Self;
    type R = FlooredRegret;
    type W = LinearWeight;
    type S = ExternalSampling;

    fn batch_size() -> usize {
        LIARS_DICE_BATCH_SIZE
    }

    fn tree_count() -> usize {
        N
    }

    fn encoder(&self) -> &Self::N {
        self
    }

    fn profile(&self) -> &Self::P {
        self
    }

    fn advance(&mut self) {
        Profile::increment(self);
    }

    fn mut_weight(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self
            .encounters
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert((0.0, 0.0, 0.0, 0))
            .0
    }

    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self
            .encounters
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert((0.0, 0.0, 0.0, 0))
            .1
    }

    fn mut_evalue(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self
            .encounters
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert((0.0, 0.0, 0.0, 0))
            .2
    }

    fn mut_counts(&mut self, info: &Self::I, edge: &Self::E) -> &mut u32 {
        &mut self
            .encounters
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert((0.0, 0.0, 0.0, 0))
            .3
    }
}

impl<const N: usize> LiarsDiceSolver<N> {
    /// Create a solver with an empty strategy profile.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a solver checkpoint from disk.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LiarsDiceSolverError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| LiarsDiceSolverError::Read {
            path: path.display().to_string(),
            source,
        })?;

        Self::from_checkpoint_bytes(&bytes)
    }

    /// Decode a solver from checkpoint bytes.
    pub fn from_checkpoint_bytes(bytes: &[u8]) -> Result<Self, LiarsDiceSolverError> {
        if bytes.len() < CHECKPOINT_HEADER_LEN {
            return Err(LiarsDiceSolverError::CheckpointTooShort { bytes: bytes.len() });
        }

        let found_magic: [u8; 4] = bytes[..4]
            .try_into()
            .expect("checkpoint header should include magic bytes");
        if found_magic != CHECKPOINT_MAGIC {
            return Err(LiarsDiceSolverError::CheckpointMagic {
                found: String::from_utf8_lossy(&found_magic).into_owned(),
            });
        }

        let found_version = u32::from_le_bytes(
            bytes[4..8]
                .try_into()
                .expect("checkpoint header should include version bytes"),
        );
        if found_version != CHECKPOINT_VERSION {
            return Err(LiarsDiceSolverError::CheckpointVersion {
                found: found_version,
                expected: CHECKPOINT_VERSION,
            });
        }

        decode_bincode(&bytes[CHECKPOINT_HEADER_LEN..], "liar's dice profile")
    }

    /// Save the current profile checkpoint to disk.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), LiarsDiceSolverError> {
        let path = path.as_ref();
        let bytes = self.checkpoint_bytes()?;

        fs::write(path, bytes).map_err(|source| LiarsDiceSolverError::Write {
            path: path.display().to_string(),
            source,
        })
    }

    /// Serialize the current solver checkpoint to bytes.
    pub fn checkpoint_bytes(&self) -> Result<Vec<u8>, LiarsDiceSolverError> {
        let encoded = encode_bincode(self, "liar's dice profile")?;
        let mut bytes = Vec::with_capacity(CHECKPOINT_HEADER_LEN + encoded.len());

        bytes.extend_from_slice(&CHECKPOINT_MAGIC);
        bytes.extend_from_slice(&CHECKPOINT_VERSION.to_le_bytes());
        bytes.extend_from_slice(&encoded);

        Ok(bytes)
    }

    /// Return the current training epoch count.
    pub const fn epochs(&self) -> usize {
        self.epochs
    }

    /// Run one MCCFR iteration.
    pub fn step(&mut self) -> Result<(), LiarsDiceSolverError> {
        catch_unwind(AssertUnwindSafe(|| Solver::step(self))).map_err(|payload| {
            LiarsDiceSolverError::UpstreamPanic {
                operation: "solver step",
                message: panic_message(payload.as_ref()),
            }
        })
    }

    /// Run a fixed number of MCCFR iterations.
    pub fn train(&mut self, iterations: usize) -> Result<(), LiarsDiceSolverError> {
        for _ in 0..iterations {
            self.step()?;
        }

        Ok(())
    }

    /// Query the current average strategy for a Liar's Dice information set.
    pub fn query(&self, info: LiarsDiceInfo) -> LiarsDiceStrategyResponse {
        StrategyResponse::new(self.averaged_policy(&info))
    }

    /// Answer a wire-safe Liar's Dice strategy query.
    pub fn answer(&self, query: LiarsDiceStrategyQuery) -> LiarsDiceStrategyResponse {
        self.query(query.info)
    }

    /// Return the highest-probability action for the supplied information set.
    pub fn recommend(&self, info: LiarsDiceInfo) -> Option<LiarsDiceEdge> {
        recommended_edge(&self.query(info))
    }

    /// Return the highest-probability action for a wire-safe query.
    pub fn recommend_query(&self, query: LiarsDiceStrategyQuery) -> Option<LiarsDiceEdge> {
        recommended_edge(&self.answer(query))
    }

    /// Return the averaged distribution for a specific information set.
    pub fn averaged_policy(&self, info: &LiarsDiceInfo) -> Policy<LiarsDiceEdge> {
        Profile::averaged_distribution(self, info)
    }

    /// Compute the exact best-response value against the averaged policy.
    pub fn exact_best_response_value(&self, hero: LiarsDiceTurn) -> Utility {
        let uniform = [1.0 / DIE_FACES as Probability; DIE_FACES];
        let mut total = 0.0;

        for hero_die in 1..=DIE_FACES as u8 {
            total += self.best_response_from_public(
                LiarsDiceTurn::P1,
                NO_CLAIM,
                hero,
                hero_die,
                uniform,
            );
        }

        total / DIE_FACES as Utility
    }

    /// Compute exact exploitability for the minimal proof game.
    pub fn exact_exploitability(&self) -> Utility {
        let p1 = self.exact_best_response_value(LiarsDiceTurn::P1);
        let p2 = self.exact_best_response_value(LiarsDiceTurn::P2);

        (p1 + p2) / 2.0
    }

    fn best_response_from_public(
        &self,
        actor: LiarsDiceTurn,
        last_claim_rank: u8,
        hero: LiarsDiceTurn,
        hero_die: u8,
        opp_probs: [Probability; DIE_FACES],
    ) -> Utility {
        match actor {
            LiarsDiceTurn::P1 | LiarsDiceTurn::P2 if actor == hero => {
                self.hero_value(actor, last_claim_rank, hero, hero_die, opp_probs)
            }
            LiarsDiceTurn::P1 | LiarsDiceTurn::P2 => {
                self.opponent_value(actor, last_claim_rank, hero, hero_die, opp_probs)
            }
            LiarsDiceTurn::Chance | LiarsDiceTurn::Terminal => {
                panic!("exact scoring expects a non-terminal public state")
            }
        }
    }

    fn hero_value(
        &self,
        actor: LiarsDiceTurn,
        last_claim_rank: u8,
        hero: LiarsDiceTurn,
        hero_die: u8,
        opp_probs: [Probability; DIE_FACES],
    ) -> Utility {
        let public = LiarsDicePublic::new(actor, last_claim_rank);
        let mut best = Utility::NEG_INFINITY;

        for edge in public.choices() {
            let value =
                self.follow_public_action(actor, last_claim_rank, hero, hero_die, opp_probs, edge);
            if value > best {
                best = value;
            }
        }

        best
    }

    fn opponent_value(
        &self,
        actor: LiarsDiceTurn,
        last_claim_rank: u8,
        hero: LiarsDiceTurn,
        hero_die: u8,
        opp_probs: [Probability; DIE_FACES],
    ) -> Utility {
        let public = LiarsDicePublic::new(actor, last_claim_rank);
        let mut total = 0.0;

        for edge in public.choices() {
            let mut branch_probs = [0.0; DIE_FACES];
            let mut branch_total = 0.0;

            for opponent_die in 1..=DIE_FACES as u8 {
                let index = Self::die_index(opponent_die);
                let reach = opp_probs[index];
                if reach == 0.0 {
                    continue;
                }

                let info = LiarsDiceInfo::new(public, LiarsDiceSecret(opponent_die));
                let action_prob = self.policy_probability(&info, edge);
                if action_prob == 0.0 {
                    continue;
                }

                let weighted = reach * action_prob;
                branch_probs[index] = weighted;
                branch_total += weighted;
            }

            if branch_total == 0.0 {
                continue;
            }

            for probability in &mut branch_probs {
                *probability /= branch_total;
            }

            let value = self.follow_public_action(
                actor,
                last_claim_rank,
                hero,
                hero_die,
                branch_probs,
                edge,
            );
            total += branch_total * value;
        }

        total
    }

    fn follow_public_action(
        &self,
        actor: LiarsDiceTurn,
        last_claim_rank: u8,
        hero: LiarsDiceTurn,
        hero_die: u8,
        opp_probs: [Probability; DIE_FACES],
        edge: LiarsDiceEdge,
    ) -> Utility {
        match edge {
            LiarsDiceEdge::Bid(claim) => self.best_response_from_public(
                Self::other_player(actor),
                claim.rank(),
                hero,
                hero_die,
                opp_probs,
            ),
            LiarsDiceEdge::Challenge => {
                let claim = LiarsDicePublic::new(actor, last_claim_rank)
                    .last_claim()
                    .expect("challenge requires an existing claim");
                let mut total = 0.0;

                for opponent_die in 1..=DIE_FACES as u8 {
                    let probability = opp_probs[Self::die_index(opponent_die)];
                    if probability == 0.0 {
                        continue;
                    }
                    total += probability
                        * self.challenge_payoff(hero, actor, claim, hero_die, opponent_die);
                }

                total
            }
            LiarsDiceEdge::Roll { .. } => panic!("exact scoring never follows roll edges directly"),
        }
    }

    fn challenge_payoff(
        &self,
        hero: LiarsDiceTurn,
        challenger: LiarsDiceTurn,
        claim: crate::game::LiarsDiceClaim,
        hero_die: u8,
        opponent_die: u8,
    ) -> Utility {
        let actual_count = u8::from(hero_die == claim.face) + u8::from(opponent_die == claim.face);
        let challenger_index = Self::player_index(challenger);
        let claimant_index = 1_u8.saturating_sub(challenger_index);
        let winner = if actual_count >= claim.count {
            claimant_index
        } else {
            challenger_index
        };

        if winner == Self::player_index(hero) {
            1.0
        } else {
            -1.0
        }
    }

    fn policy_probability(&self, info: &LiarsDiceInfo, edge: LiarsDiceEdge) -> Probability {
        self.averaged_policy(info)
            .into_iter()
            .find_map(|(candidate, probability)| (candidate == edge).then_some(probability))
            .unwrap_or_default()
    }

    fn die_index(die: u8) -> usize {
        usize::from(die - 1)
    }

    fn other_player(actor: LiarsDiceTurn) -> LiarsDiceTurn {
        match actor {
            LiarsDiceTurn::P1 => LiarsDiceTurn::P2,
            LiarsDiceTurn::P2 => LiarsDiceTurn::P1,
            LiarsDiceTurn::Chance | LiarsDiceTurn::Terminal => {
                panic!("only player turns have an opponent")
            }
        }
    }

    fn player_index(player: LiarsDiceTurn) -> u8 {
        match player {
            LiarsDiceTurn::P1 => 0,
            LiarsDiceTurn::P2 => 1,
            LiarsDiceTurn::Chance | LiarsDiceTurn::Terminal => {
                panic!("only players have an index")
            }
        }
    }
}

/// Errors returned by the Liar's Dice solver wrapper.
#[derive(Debug, Error)]
pub enum LiarsDiceSolverError {
    #[error("failed to encode {context}: {source}")]
    Encode {
        context: &'static str,
        source: bincode::Error,
    },
    #[error("failed to decode {context}: {source}")]
    Decode {
        context: &'static str,
        source: bincode::Error,
    },
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
    #[error("checkpoint magic mismatch: found `{found}`, expected `MYOS`")]
    CheckpointMagic { found: String },
    #[error("checkpoint version {found} does not match expected version {expected}")]
    CheckpointVersion { found: u32, expected: u32 },
    #[error("{operation} failed upstream: {message}")]
    UpstreamPanic {
        operation: &'static str,
        message: String,
    },
}

fn encode_bincode<T>(value: &T, context: &'static str) -> Result<Vec<u8>, LiarsDiceSolverError>
where
    T: Serialize,
{
    encode_codec()
        .serialize(value)
        .map_err(|source| LiarsDiceSolverError::Encode { context, source })
}

fn decode_bincode<T>(bytes: &[u8], context: &'static str) -> Result<T, LiarsDiceSolverError>
where
    T: for<'de> Deserialize<'de>,
{
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| LiarsDiceSolverError::Decode { context, source })
}

fn encode_codec() -> impl Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .reject_trailing_bytes()
}

fn decode_codec(limit: u64) -> impl Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_limit(limit)
        .reject_trailing_bytes()
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return (*message).to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }

    "panic payload was not a string".to_string()
}

#[cfg(test)]
mod tests {
    use crate::game::{LiarsDiceClaim, LiarsDiceEdge, LiarsDiceGame, LiarsDiceTurn};
    use crate::protocol::LiarsDiceStrategyQuery;
    use crate::solver::{LiarsDiceSolver, LiarsDiceSolverError};
    use bincode::Options;
    use myosu_games::{CfrGame, CfrInfo, Profile, StrategyResponse};
    use rbp_mccfr::{CfrPublic, Solver};
    use std::time::{SystemTime, UNIX_EPOCH};

    const TRAINING_TREES: usize = 1 << 10;

    #[test]
    fn solver_runs_and_tracks_epochs() {
        let solver = LiarsDiceSolver::<TRAINING_TREES>::default().solve();
        let opening = LiarsDiceGame::root().apply(LiarsDiceEdge::Roll { p1: 2, p2: 5 });
        let info = opening
            .info()
            .expect("opening player turn should expose info");

        assert_eq!(solver.epochs(), TRAINING_TREES / 16);
        assert!(!solver.encounters.is_empty());
        assert!(solver.encounters.contains_key(&info));
    }

    #[test]
    fn training_populates_opening_statistics() {
        let solver = LiarsDiceSolver::<TRAINING_TREES>::default().solve();
        let opening = LiarsDiceGame::root().apply(LiarsDiceEdge::Roll { p1: 2, p2: 5 });
        let info = opening
            .info()
            .expect("opening player turn should expose info");
        let choices = info.public().choices();

        assert!(!choices.is_empty());
        assert!(
            choices
                .iter()
                .all(|edge| solver.cum_counts(&info, edge) > 0)
        );
        assert!(
            choices
                .iter()
                .any(|edge| solver.cum_weight(&info, edge) > 0.0)
        );
    }

    #[test]
    fn trained_policy_is_valid_for_opening_claims() {
        let solver = LiarsDiceSolver::<TRAINING_TREES>::default().solve();
        let opening = LiarsDiceGame::root().apply(LiarsDiceEdge::Roll { p1: 2, p2: 5 });
        let info = opening
            .info()
            .expect("opening player turn should expose info");
        let policy = solver.averaged_policy(&info);
        let total: f32 = policy.iter().map(|(_, probability)| *probability).sum();

        assert!((total - 1.0).abs() < 0.001);
        assert!(policy.iter().any(|(edge, _)| {
            *edge == LiarsDiceEdge::Bid(LiarsDiceClaim::new(1, 1).expect("claim should be valid"))
        }));
    }

    #[test]
    fn exact_best_response_values_are_bounded() {
        let solver = LiarsDiceSolver::<TRAINING_TREES>::default().solve();
        let p1 = solver.exact_best_response_value(LiarsDiceTurn::P1);
        let p2 = solver.exact_best_response_value(LiarsDiceTurn::P2);

        assert!(p1.is_finite());
        assert!(p2.is_finite());
        assert!((-1.0..=1.0).contains(&p1));
        assert!((-1.0..=1.0).contains(&p2));
    }

    #[test]
    fn exact_exploitability_is_finite() {
        let solver = LiarsDiceSolver::<TRAINING_TREES>::default().solve();
        let exploitability = solver.exact_exploitability();

        assert!(exploitability.is_finite());
        assert!((0.0..=1.0).contains(&exploitability));
    }

    #[test]
    fn solver_answers_wire_safe_queries() {
        let solver = LiarsDiceSolver::<TRAINING_TREES>::default().solve();
        let opening = LiarsDiceGame::root().apply(LiarsDiceEdge::Roll { p1: 2, p2: 5 });
        let info = opening
            .info()
            .expect("opening player turn should expose info");
        let query = LiarsDiceStrategyQuery::new(info);
        let response = solver.answer(query);

        assert!(response.is_valid());
        assert!(
            StrategyResponse::probability_for(
                &response,
                &LiarsDiceEdge::Bid(LiarsDiceClaim::new(1, 1).expect("claim should be valid")),
            ) >= 0.0
        );
        assert!(solver.recommend(info).is_some());
    }

    #[test]
    fn checkpoint_roundtrip_preserves_epoch_and_strategy() {
        let path = unique_checkpoint_path();
        let mut solver = LiarsDiceSolver::<TRAINING_TREES>::new();
        solver.train(64).expect("training should succeed");
        solver.save(&path).expect("checkpoint should save");

        let restored =
            LiarsDiceSolver::<TRAINING_TREES>::load(&path).expect("checkpoint should load");
        let info = LiarsDiceGame::root()
            .apply(LiarsDiceEdge::Roll { p1: 2, p2: 5 })
            .info()
            .expect("opening player turn should expose info");

        assert_eq!(restored.epochs(), solver.epochs());
        assert_eq!(restored.query(info), solver.query(info));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn checkpoint_rejects_wrong_magic() {
        let error = LiarsDiceSolver::<TRAINING_TREES>::from_checkpoint_bytes(b"NOPE\x01\0\0\0")
            .expect_err("bad checkpoint magic should fail");

        assert!(matches!(
            error,
            LiarsDiceSolverError::CheckpointMagic { .. }
        ));
    }

    #[test]
    fn checkpoint_decode_rejects_oversized_payload() {
        let oversized = vec![0_u8; super::MAX_DECODE_BYTES as usize + 1];
        let result = super::decode_codec(super::MAX_DECODE_BYTES - 1).serialized_size(&oversized);

        assert!(
            result.is_err(),
            "oversized payload should exceed decode budget"
        );
    }

    fn unique_checkpoint_path() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("myosu-liars-dice-checkpoint-{nanos}.bin"))
    }
}
