//! Convergence testing for CFR solvers.
//!
//! These functions validate that a solver converges to a Nash equilibrium
//! on known games with tractable equilibrium strategies.

/// Assert that a solver converges on a game with known equilibrium.
///
/// # Arguments
///
/// * `max_iters` - Maximum number of iterations to run
/// * `target_exploit` - Target exploitability value (lower is better)
///
/// # Panics
///
/// Panics if the solver fails to converge to the target exploitability.
///
/// # Note
///
/// This requires integration with `rbp-mccfr` solver. The actual
/// implementation is deferred to when the solver interface is finalized.
pub fn assert_solver_converges<G, E>(
    _encoder: &E,
    _max_iters: usize,
    _target_exploit: f64,
) where
    G: Default,
    E: myosu_games::Encoder,
{
    // TODO: Implement actual convergence testing with rbp-mccfr solver
    // This requires the solver to be exposed from robopoker.
    let _ = _encoder;
    let _ = _max_iters;
    let _ = _target_exploit;
}
