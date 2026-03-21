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
test -f ./outputs/chain/pallet/implementation.md && test -f ./outputs/chain/pallet/verification.md && test -f ./outputs/chain/pallet/quality.md && test -f ./outputs/chain/pallet/promotion.md && test -f ./outputs/chain/pallet/integration.md
true`
  - Stdout: (empty)
  - Stderr: (empty)
- **implement**: success
  - Model: gpt-5.4, 3.1m tokens in / 42.9k out
  - Files: Cargo.lock, crates/myosu-chain/pallets/game-solver/Cargo.toml, crates/myosu-chain/pallets/game-solver/src/epoch/math.rs, crates/myosu-chain/pallets/game-solver/src/epoch/mod.rs, crates/myosu-chain/pallets/game-solver/src/guards/check_coldkey_swap.rs, crates/myosu-chain/pallets/game-solver/src/lib.rs, crates/myosu-chain/pallets/game-solver/src/macros/config.rs, crates/myosu-chain/pallets/game-solver/src/macros/mod.rs, crates/myosu-chain/pallets/game-solver/src/staking/mod.rs, crates/myosu-chain/pallets/game-solver/src/subnets/mod.rs, crates/myosu-chain/pallets/game-solver/src/utils/mod.rs, crates/myosu-chain/pallets/game-solver/src/utils/rate_limiting.rs, outputs/chain/pallet/implementation.md, outputs/chain/pallet/integration.md, outputs/chain/pallet/verification.md
- **verify**: success
  - Script: `test -f ./outputs/chain/pallet/implementation.md && test -f ./outputs/chain/pallet/verification.md && test -f ./outputs/chain/pallet/quality.md && test -f ./outputs/chain/pallet/promotion.md && test -f ./outputs/chain/pallet/integration.md`
  - Stdout: (empty)
  - Stderr: (empty)
- **fixup**: success
  - Model: gpt-5.4, 1.3m tokens in / 16.3k out
  - Files: outputs/chain/pallet/implementation.md, outputs/chain/pallet/promotion.md, outputs/chain/pallet/quality.md, outputs/chain/pallet/verification.md
- **verify**: success
  - Script: `test -f ./outputs/chain/pallet/implementation.md && test -f ./outputs/chain/pallet/verification.md && test -f ./outputs/chain/pallet/quality.md && test -f ./outputs/chain/pallet/promotion.md && test -f ./outputs/chain/pallet/integration.md`
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


# Pallet Restart Implementation Lane — Review

Review only the current slice for `pallet-implement`.

Current Slice Contract:
Inspect the relevant repo surfaces, preserve existing doctrine, and produce the lane artifacts honestly.


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Focus on:
- slice scope discipline
- proof-gate coverage for the active slice
- touched-surface containment
- implementation and verification artifact quality
- remaining blockers before the next slice

Deterministic evidence:
- treat `quality.md` as machine-generated truth about placeholder debt, warning debt, manual follow-up, and artifact mismatch risk
- if `quality.md` says `quality_ready: no`, do not bless the slice as merge-ready


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

Review stage ownership:
- you may write or replace `promotion.md` in this stage
- read `quality.md` before deciding `merge_ready`
- when the slice is security-sensitive, perform a Nemesis-style pass: first-principles assumption challenge plus coupled-state consistency review
- include security findings in the review verdict when the slice touches trust boundaries, keys, funds, auth, control-plane behavior, or external process control
- prefer not to modify source code here unless a tiny correction is required to make the review judgment truthful
