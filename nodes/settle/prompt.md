Goal: Implement the next approved `chain:pallet` slice.

Inputs:
- `pallet/spec.md`
- `pallet/review.md`

Scope:
- work only inside the smallest next approved implementation slice
- treat the reviewed lane artifacts as the source of truth
- keep changes aligned with the owned surfaces for `chain:pallet`

Required curated artifacts:
- `pallet/implementation.md`
- `pallet/verification.md`
- `pallet/quality.md`
- `pallet/promotion.md`
- `pallet/integration.md`


## Completed stages
- **preflight**: success
  - Script: `set +e
true
true`
  - Stdout: (empty)
  - Stderr: (empty)
- **implement**: success
  - Model: MiniMax-M2.7-highspeed, 145 tokens in / 41 out
  - Files: Cargo.lock, crates/myosu-chain/pallets/game-solver/Cargo.toml, crates/myosu-chain/pallets/game-solver/src/coinbase/block_emission.rs, crates/myosu-chain/pallets/game-solver/src/coinbase/block_step.rs, crates/myosu-chain/pallets/game-solver/src/coinbase/mod.rs, crates/myosu-chain/pallets/game-solver/src/coinbase/root.rs, crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs, crates/myosu-chain/pallets/game-solver/src/coinbase/subnet_emissions.rs, crates/myosu-chain/pallets/game-solver/src/epoch/math.rs, crates/myosu-chain/pallets/game-solver/src/epoch/mod.rs, crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs, crates/myosu-chain/pallets/game-solver/src/extensions/mod.rs, crates/myosu-chain/pallets/game-solver/src/extensions/subtensor.rs, crates/myosu-chain/pallets/game-solver/src/guards/check_coldkey_swap.rs, crates/myosu-chain/pallets/game-solver/src/guards/mod.rs, crates/myosu-chain/pallets/game-solver/src/lib.rs, crates/myosu-chain/pallets/game-solver/src/macros/config.rs, crates/myosu-chain/pallets/game-solver/src/macros/dispatches.rs, crates/myosu-chain/pallets/game-solver/src/macros/errors.rs, crates/myosu-chain/pallets/game-solver/src/macros/events.rs, crates/myosu-chain/pallets/game-solver/src/macros/genesis.rs, crates/myosu-chain/pallets/game-solver/src/macros/hooks.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_auto_stake_destination.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_clear_rank_trust_pruning_maps.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_coldkey_swap_scheduled.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_coldkey_swap_scheduled_to_announcements.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_commit_reveal_settings.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_commit_reveal_v2.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_create_root_network.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_crv3_commits_add_block.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_crv3_v2_to_timelocked.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_delete_subnet_21.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_delete_subnet_3.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_disable_commit_reveal.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_fix_childkeys.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_fix_is_network_member.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_fix_root_subnet_tao.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_fix_root_tao_and_alpha_in.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_fix_staking_hot_keys.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_init_tao_flow.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_init_total_issuance.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_kappa_map_to_default.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_network_immunity_period.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_network_lock_cost_2500.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_network_lock_reduction_interval.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_orphaned_storage_items.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_pending_emissions.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_populate_owned_hotkeys.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_rao.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_rate_limit_keys.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_rate_limiting_last_blocks.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_remove_commitments_rate_limit.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_remove_network_modality.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_remove_old_identity_maps.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_remove_stake_map.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_remove_tao_dividends.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_remove_total_hotkey_coldkey_stakes_this_interval.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_remove_unknown_neuron_axon_cert_prom.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_remove_unused_maps_and_values.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_remove_zero_total_hotkey_alpha.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_reset_bonds_moving_average.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_reset_max_burn.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_reset_unactive_sn.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_set_first_emission_block_number.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_set_min_burn.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_set_min_difficulty.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_set_nominator_min_stake.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_set_registration_enable.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_set_subtoken_enabled.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_stake_threshold.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_subnet_limit_to_default.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_subnet_locked.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_subnet_symbols.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_subnet_volume.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_to_v1_separate_emission.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_to_v2_fixed_total_stake.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_total_issuance.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_transfer_ownership_to_foundation.rs, crates/myosu-chain/pallets/game-solver/src/migrations/migrate_upgrade_revealed_commitments.rs, crates/myosu-chain/pallets/game-solver/src/migrations/mod.rs, crates/myosu-chain/pallets/game-solver/src/rpc_info/delegate_info.rs, crates/myosu-chain/pallets/game-solver/src/rpc_info/dynamic_info.rs, crates/myosu-chain/pallets/game-solver/src/rpc_info/metagraph.rs, crates/myosu-chain/pallets/game-solver/src/rpc_info/mod.rs, crates/myosu-chain/pallets/game-solver/src/rpc_info/neuron_info.rs, crates/myosu-chain/pallets/game-solver/src/rpc_info/show_subnet.rs, crates/myosu-chain/pallets/game-solver/src/rpc_info/stake_info.rs, crates/myosu-chain/pallets/game-solver/src/rpc_info/subnet_info.rs, crates/myosu-chain/pallets/game-solver/src/staking/account.rs, crates/myosu-chain/pallets/game-solver/src/staking/add_stake.rs, crates/myosu-chain/pallets/game-solver/src/staking/claim_root.rs, crates/myosu-chain/pallets/game-solver/src/staking/decrease_take.rs, crates/myosu-chain/pallets/game-solver/src/staking/helpers.rs, crates/myosu-chain/pallets/game-solver/src/staking/increase_take.rs, crates/myosu-chain/pallets/game-solver/src/staking/mod.rs, crates/myosu-chain/pallets/game-solver/src/staking/move_stake.rs, crates/myosu-chain/pallets/game-solver/src/staking/recycle_alpha.rs, crates/myosu-chain/pallets/game-solver/src/staking/remove_stake.rs, crates/myosu-chain/pallets/game-solver/src/staking/set_children.rs, crates/myosu-chain/pallets/game-solver/src/staking/stake_utils.rs, crates/myosu-chain/pallets/game-solver/src/subnets/leasing.rs, crates/myosu-chain/pallets/game-solver/src/subnets/mechanism.rs, crates/myosu-chain/pallets/game-solver/src/subnets/mod.rs, crates/myosu-chain/pallets/game-solver/src/subnets/registration.rs, crates/myosu-chain/pallets/game-solver/src/subnets/serving.rs, crates/myosu-chain/pallets/game-solver/src/subnets/subnet.rs, crates/myosu-chain/pallets/game-solver/src/subnets/symbols.rs, crates/myosu-chain/pallets/game-solver/src/subnets/uids.rs, crates/myosu-chain/pallets/game-solver/src/subnets/weights.rs, crates/myosu-chain/pallets/game-solver/src/swap/mod.rs, crates/myosu-chain/pallets/game-solver/src/swap/swap_coldkey.rs, crates/myosu-chain/pallets/game-solver/src/swap/swap_hotkey.rs, crates/myosu-chain/pallets/game-solver/src/utils/evm.rs, crates/myosu-chain/pallets/game-solver/src/utils/identity.rs, crates/myosu-chain/pallets/game-solver/src/utils/misc.rs, crates/myosu-chain/pallets/game-solver/src/utils/mod.rs, crates/myosu-chain/pallets/game-solver/src/utils/rate_limiting.rs, crates/myosu-chain/pallets/game-solver/src/utils/try_state.rs, crates/myosu-chain/pallets/game-solver/src/utils/voting_power.rs
- **verify**: success
  - Script: `true`
  - Stdout: (empty)
  - Stderr: (empty)
- **quality**: success
  - Script: `set -e
QUALITY_PATH='outputs/chain/pallet/quality.md'
IMPLEMENTATION_PATH='outputs/chain/pallet/implementation.md'
VERIFICATION_PATH='outputs/chain/pallet/verification.md'
placeholder_hits=""
scan_placeholder() {
  surface="$1"
  if [ ! -e "$surface" ]; then
    return 0
  fi
  if [ -f "$surface" ]; then
    surface="$(dirname "$surface")"
  fi
  hits="$(rg -n -i -g '*.rs' -g '*.py' -g '*.js' -g '*.ts' -g '*.tsx' -g '*.md' -g 'Cargo.toml' -g '*.toml' 'TODO|stub|placeholder|not yet implemented|compile-only|for now|will implement|todo!|unimplemented!' "$surface" || true)"
  if [ -n "$hits" ]; then
    if [ -n "$placeholder_hits" ]; then
      placeholder_hits="$(printf '%s\n%s' "$placeholder_hits" "$hits")"
    else
      placeholder_hits="$hits"
    fi
  fi
}
true
artifact_hits="$(rg -n -i 'manual proof still required|placeholder|stub implementation|not yet fully implemented|todo!|unimplemented!' "$IMPLEMENTATION_PATH" "$VERIFICATION_PATH" 2>/dev/null || true)"
warning_hits="$(rg -n 'warning:' "$IMPLEMENTATION_PATH" "$VERIFICATION_PATH" 2>/dev/null || true)"
manual_hits="$(rg -n -i 'manual proof still required|manual;' "$VERIFICATION_PATH" 2>/dev/null || true)"
placeholder_debt=no
warning_debt=no
artifact_mismatch_risk=no
manual_followup_required=no
[ -n "$placeholder_hits" ] && placeholder_debt=yes
[ -n "$warning_hits" ] && warning_debt=yes
[ -n "$artifact_hits" ] && artifact_mismatch_risk=yes
[ -n "$manual_hits" ] && manual_followup_required=yes
quality_ready=yes
if [ "$placeholder_debt" = yes ] || [ "$warning_debt" = yes ] || [ "$artifact_mismatch_risk" = yes ] || [ "$manual_followup_required" = yes ]; then
  quality_ready=no
fi
mkdir -p "$(dirname "$QUALITY_PATH")"
cat > "$QUALITY_PATH" <<EOF
quality_ready: $quality_ready
placeholder_debt: $placeholder_debt
warning_debt: $warning_debt
artifact_mismatch_risk: $artifact_mismatch_risk
manual_followup_required: $manual_followup_required

## Touched Surfaces
- (none declared)

## Placeholder Hits
$placeholder_hits

## Artifact Consistency Hits
$artifact_hits

## Warning Hits
$warning_hits

## Manual Followup Hits
$manual_hits
EOF
test "$quality_ready" = yes`
  - Stdout: (empty)
  - Stderr: (empty)


# Pallet Restart Implementation Lane — Promotion

Decide whether `pallet-implement` is truly merge-ready.


Write `promotion.md` in this exact machine-readable form:

merge_ready: yes|no
manual_proof_pending: yes|no
reason: <one sentence>
next_action: <one sentence>

Only set `merge_ready: yes` when:
- `quality.md` says `quality_ready: yes`
- automated proof is sufficient for this slice
- any required manual proof has actually been performed
- no unresolved warnings or stale failures undermine confidence
- the implementation and verification artifacts match the real code.

Settle stage ownership:
- you may write or replace `promotion.md` in this stage
- read `quality.md` before deciding `merge_ready`
- prefer not to modify source code here unless a tiny correction is required to make the settlement judgment truthful

Current Slice Contract:
Use `pallet/spec.md` and `pallet/review.md` as the approved contract. Implement only the smallest honest next slice, write what changed to `pallet/implementation.md`, write proof results plus remaining risk to `pallet/verification.md`, rely on the machine-generated quality evidence in `pallet/quality.md`, write the merge/promotion verdict to `pallet/promotion.md`, and ensure the required integration artifact exists at `pallet/integration.md` before the lane is considered complete.
