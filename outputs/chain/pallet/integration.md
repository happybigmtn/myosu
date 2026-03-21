slice: phase_1_fix_deps_and_strip_non_myosu_modules

## Integration State
- `chain:runtime` can now depend on a buildable `pallet-game-solver` crate again, which clears the reviewed restart gate that required `cargo check -p pallet-game-solver` to exit 0.
- Downstream code may rely on the retained Phase 1 exports: `AxonInfo`, `PrometheusInfo`, `NeuronCertificate`, `RateLimitKey`, `NoOpSwap`, and the local proxy, commitment, authorship, and coldkey-swap interfaces.
- Runtime-facing behavior is still intentionally absent in this slice: there are no restored storage items, registration calls, serving calls, staking calls, or weight submission calls yet.

## Next Integration Handoff
- Phase 2 should restore the minimal `Config` and storage set (`Keys`, `Axons`, `Owner`, `Delegates`, `SubnetOwner`) on top of this compile-clean base.
