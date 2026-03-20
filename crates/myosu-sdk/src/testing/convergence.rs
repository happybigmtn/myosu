//! Convergence testing for CFR solvers.
//!
//! These functions validate that a solver converges to a Nash equilibrium
//! on known games with tractable equilibrium strategies.

use rbp_mccfr::Solver;

/// Assert that a solver converges below the requested exploitability target.
///
/// # Arguments
///
/// * `solver` - Solver instance to train
/// * `max_steps` - Maximum number of training steps to run
/// * `target_exploit` - Target exploitability value (lower is better)
///
/// # Panics
///
/// Panics if the solver fails to converge to the target exploitability.
pub fn assert_solver_converges<S>(mut solver: S, max_steps: usize, target_exploit: f64)
where
    S: Solver,
{
    let starting_exploitability = f64::from(solver.exploitability());

    for _ in 0..max_steps {
        solver.step();
    }

    let exploitability = f64::from(solver.exploitability());
    assert!(
        exploitability <= target_exploit,
        "solver exploitability {exploitability:.6} exceeded target {target_exploit:.6} after {max_steps} steps (started at {starting_exploitability:.6})"
    );
}
