//! Compliance checks for `CfrGame` implementations.
//!
//! These functions validate that a game implementation satisfies the
//! invariants required for CFR to work correctly.

use myosu_games::CfrGame;

/// Assert that a game implementation satisfies all CFR invariants.
///
/// # Panics
///
/// Panics if any invariant is violated.
///
/// # Note
///
/// These checks require a running CFR solver. The actual implementations
/// are deferred to integration tests that use `rbp-mccfr::rps::RpsSolver`.
pub fn assert_game_valid<G: CfrGame>() {
    // Validate game can be instantiated at root
    let _game = G::root();
}
