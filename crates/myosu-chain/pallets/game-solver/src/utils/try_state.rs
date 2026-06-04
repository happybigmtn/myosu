use frame_support::traits::fungible::Inspect;

use super::*;

impl<T: Config> Pallet<T> {
    /// Stable, grep-friendly error code emitted by
    /// [`Pallet::check_total_issuance`] when the live `TotalIssuance` differs
    /// from the expected `currency_issuance + total_stake` by more than
    /// [`TOTAL_ISSUANCE_TRY_STATE_ALERT_DELTA`] rao. The actual magnitude,
    /// expected total, and live total are recorded via `log::error!` (and
    /// re-emitted as a debug record on the same target for tailing
    /// operators), but the returned `TryRuntimeError` itself is a static
    /// `&'static str` so a wrapper script can `grep try_state failure` and
    /// still correlate the call site to the NEM-006 plan row. The full
    /// numeric triple lives in the runtime log under the
    /// `runtime::game_solver` target — operators can recover it from there
    /// without re-running the on-chain state through an indexer.
    #[allow(dead_code)]
    pub(crate) const TOTAL_ISSUANCE_TRY_STATE_FAILURE: &'static str =
        "TotalIssuance try_state failure: live != expected (diff > delta); \
         see runtime::game_solver log for live/expected/diff/delta";

    /// Checks [`TotalIssuance`] equals the sum of currency issuance, total stake, and total subnet
    /// locked.
    ///
    /// Diagnostic surface (NEM-006): on a non-zero diff, the assertion logs the
    /// actual magnitude, the expected total, and the live total via
    /// `log::error!`/`log::warn!` so an operator tailing the runtime log sees
    /// enough information to triage without re-running the on-chain state
    /// through an indexer. The healthy path (diff == 0) is logged at debug
    /// only, so steady-state log volume is unchanged.
    #[allow(dead_code, clippy::expect_used)]
    pub(crate) fn check_total_issuance() -> Result<(), sp_runtime::TryRuntimeError> {
        // Get the total currency issuance
        let currency_issuance = <T as Config>::Currency::total_issuance();

        // Calculate the expected total issuance
        let expected_total_issuance =
            currency_issuance.saturating_add(TotalStake::<T>::get().into());

        // Verify the diff between calculated TI and actual TI is less than the
        // current stage-0 alert threshold.
        //
        // EMIT-001 closes the coinbase split remainder explicitly, so try-state
        // now treats any diff above a single rao as a real accounting alert
        // instead of tolerating a large dust envelope.
        let delta = TOTAL_ISSUANCE_TRY_STATE_ALERT_DELTA;
        let total_issuance = TotalIssuance::<T>::get().to_u64();

        let diff = if total_issuance > expected_total_issuance {
            total_issuance.checked_sub(expected_total_issuance)
        } else {
            expected_total_issuance.checked_sub(total_issuance)
        }
        .expect("LHS > RHS");

        if diff == 0 {
            // Healthy path: live and expected match exactly. Log at debug
            // (not info/error) so the steady-state log volume is unchanged —
            // the existing `cargo test` runs and CI logs are not noisier.
            log::debug!(
                target: "runtime::game_solver",
                "TotalIssuance try_state ok: live={} expected={} diff=0",
                total_issuance,
                expected_total_issuance,
            );
            return Ok(());
        }

        if diff <= delta {
            // Within the stage-0 alert envelope (≤ TOTAL_ISSUANCE_TRY_STATE_ALERT_DELTA
            // rao). Still log the magnitude so a future operator who later
            // tightens the delta can see the historical drift distribution.
            log::warn!(
                target: "runtime::game_solver",
                "TotalIssuance try_state within alert envelope: live={} expected={} diff={} delta={}",
                total_issuance,
                expected_total_issuance,
                diff,
                delta,
            );
            return Ok(());
        }

        // Hard accounting failure: diff exceeds the stage-0 alert threshold.
        // Log the full diagnostic triple (live, expected, diff) at error
        // severity so a tailing operator sees the magnitude, the
        // expected total, and the live total without needing to re-derive
        // them from on-chain state.
        log::error!(
            target: "runtime::game_solver",
            "TotalIssuance try_state failure: live={} expected={} diff={} delta={} (diff exceeds TOTAL_ISSUANCE_TRY_STATE_ALERT_DELTA; this is a real accounting drift, not dust)",
            total_issuance,
            expected_total_issuance,
            diff,
            delta,
        );

        // Return a static `&'static str` error code so a wrapper script can
        // `grep try_state failure` and recover the call site; the
        // operator-facing magnitude/expected/live triple lives in the
        // `runtime::game_solver` log target above (and is the only path
        // substrate exposes for owned-string error data — the
        // `TryRuntimeError` alias is `DispatchError`, which is a
        // `&'static str` newtype plus enum variants).
        Err(sp_runtime::TryRuntimeError::Other(
            Self::TOTAL_ISSUANCE_TRY_STATE_FAILURE,
        ))
    }

    /// Checks the sum of all stakes matches the [`TotalStake`].
    #[allow(dead_code)]
    pub(crate) fn check_total_stake() -> Result<(), sp_runtime::TryRuntimeError> {
        // Calculate the total staked amount
        let total_staked =
            SubnetTAO::<T>::iter().fold(TaoCurrency::ZERO, |acc, (netuid, stake)| {
                let acc = acc.saturating_add(stake);

                if netuid.is_root() {
                    // root network doesn't have initial pool TAO
                    acc
                } else {
                    acc.saturating_sub(Self::get_network_min_lock())
                }
            });

        log::warn!(
            "total_staked: {}, TotalStake: {}",
            total_staked,
            TotalStake::<T>::get()
        );

        // Verify that the calculated total stake matches the stored TotalStake
        ensure!(
            total_staked == TotalStake::<T>::get(),
            "TotalStake does not match total staked",
        );

        Ok(())
    }
}
