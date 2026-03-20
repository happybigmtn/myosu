use std::{any::Any, panic::catch_unwind};

use myosu_games::{CfrGame, Utility};
use rbp_mccfr::{
    ExternalSampling, FlooredRegret, LinearWeight, RPS, RpsEdge, RpsGame, RpsTurn, Solver,
};

use super::{assert_game_valid, assert_solver_converges};

type ConvergingRpsSolver = RPS<FlooredRegret, LinearWeight, ExternalSampling, { 1 << 16 }>;
type UndertrainedRpsSolver = RPS<FlooredRegret, LinearWeight, ExternalSampling, { 1 << 14 }>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BrokenZeroSumGame(RpsGame);

impl CfrGame for BrokenZeroSumGame {
    type E = RpsEdge;
    type T = RpsTurn;

    fn root() -> Self {
        Self(RpsGame::root())
    }

    fn turn(&self) -> Self::T {
        self.0.turn()
    }

    fn apply(&self, edge: Self::E) -> Self {
        Self(self.0.apply(edge))
    }

    fn payoff(&self, turn: Self::T) -> Utility {
        match (self.turn(), turn) {
            (RpsTurn::Terminal, RpsTurn::P1 | RpsTurn::P2) => 1.0,
            (RpsTurn::Terminal, RpsTurn::Terminal) => 0.0,
            _ => unreachable!("payoff is only valid at terminal states"),
        }
    }
}

#[test]
fn rps_passes_all_compliance_checks() {
    assert_game_valid::<RpsGame>();
    assert_solver_converges(
        ConvergingRpsSolver::default(),
        <ConvergingRpsSolver as Solver>::iterations(),
        0.03,
    );
}

#[test]
fn broken_game_fails_zero_sum_check() {
    let panic = catch_unwind(assert_game_valid::<BrokenZeroSumGame>)
        .expect_err("broken zero-sum game should fail validation");

    assert!(
        panic_message(panic).contains("zero-sum"),
        "expected zero-sum failure"
    );
}

#[test]
fn convergence_test_detects_non_convergence() {
    let panic = catch_unwind(|| {
        assert_solver_converges(UndertrainedRpsSolver::default(), 0, 0.03);
    })
    .expect_err("undertrained solver should fail convergence target");

    assert!(
        panic_message(panic).contains("exceeded target"),
        "expected convergence failure"
    );
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return (*message).to_string();
    }
    "<non-string panic payload>".to_string()
}
