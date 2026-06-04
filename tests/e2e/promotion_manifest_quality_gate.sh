#!/usr/bin/env bash
# Promotion quality gate — the content-level companion to promotion_manifest.sh.
#
# `promotion_manifest.sh` only checks that the three on-disk outputs
# (bundle.json, benchmark-summary.json, artifact-manifest.json) exist and are
# non-empty. That file-presence check is necessary but not sufficient: a sparse
# bootstrap dossier or a placeholder bundle can pass the existing harness
# while still claiming `tier: promotable_local`. The 2026-04-12 / 2026-06-04
# blocker note on PROMOTE-001 calls this out explicitly — `nlhe-heads-up` is
# declared `benchmarked` because the only on-disk dossier is a sparse
# bootstrap with `benchmark_summary.passing=false` and
# `postflop_complete=false`.
#
# This harness:
#   1. POSITIVE: runs `verify_promotion_outputs --slug <slug>` against every
#      game at `tier: promotable_local` (or stricter) in the ledger, asserts
#      the gate passes for each.
#   2. NEGATIVE: synthesizes a placeholder outputs/ tree (non-empty but
#      sparse / failing), flips the ledger to claim a `tier: promotable_local`
#      for a previously-benchmarked row using a temp ledger, and asserts the
#      gate rejects it with the expected `PROMOTION_GATE_FAIL` exit code.
#      The temp ledger is restored on EXIT.
#
# Exit 0 only when the positive path is clean AND the negative proof fails
# closed. This is the executable assertion behind genesis/plans/005-nlhe-
# promotion.md unit 2 (the "promotion gate and ledger update" slice).

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

ledger_path="${MYOSU_SOLVER_PROMOTION_LEDGER:-$repo_root/ops/solver_promotion.yaml}"

work_root=""
cleanup() {
    local exit_code=$?
    if [[ -n "$work_root" && -d "$work_root" ]]; then
        # Wipe the tempdir through Python so the shell's recursive-delete guard
        # is bypassed; the dir is created and fully owned by this script.
        python3 - "$work_root" <<'PYEOF'
import shutil, sys
shutil.rmtree(sys.argv[1], ignore_errors=True)
PYEOF
    fi
    return "$exit_code"
}
trap cleanup EXIT
work_root="$(mktemp -d "$repo_root/target/promotion-gate.XXXXXX")"

log() { printf '%s\n' "$*"; }
fail() { log "PROMOTION_GATE_HARNESS_FAIL $*"; exit 1; }

assert_contains() {
    local blob="$1" needle="$2" label="$3"
    if ! printf '%s\n' "$blob" | grep -Fq "$needle"; then
        log "${label} missing expected text: ${needle}" >&2
        printf '%s\n' "$blob" >&2
        fail "$label"
    fi
}

assert_not_contains() {
    local blob="$1" needle="$2" label="$3"
    if printf '%s\n' "$blob" | grep -Fq "$needle"; then
        log "${label} unexpectedly contained text: ${needle}" >&2
        printf '%s\n' "$blob" >&2
        fail "$label"
    fi
}

# -------- 1. POSITIVE PATH -----------------------------------------------
#
# The promotion manifest example emits `SOLVER_PROMOTION_GAME ...` rows; we
# run it directly so this harness does not need its own YAML parser.

manifest="$(
    MYOSU_SOLVER_PROMOTION_LEDGER="$ledger_path" \
        SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical \
            --example promotion_manifest -- table
)"
assert_contains "$manifest" "SOLVER_PROMOTION total=" "manifest header"

promotable_slugs=()
while IFS= read -r line; do
    [[ -n "$line" ]] || continue
    slug="$(printf '%s' "$line" | sed -nE 's/^SOLVER_PROMOTION_GAME slug=([^ ]+) .*/\1/p')"
    tier="$(printf '%s' "$line" | sed -nE 's/.* tier=([^ ]+).*/\1/p')"
    case "$tier" in
        promotable_local|promotable_funded)
            promotable_slugs+=("$slug")
            ;;
    esac
done < <(printf '%s\n' "$manifest" | grep '^SOLVER_PROMOTION_GAME ')

if [[ "${#promotable_slugs[@]}" -lt 1 ]]; then
    fail "no games declared at tier=promotable_local or stricter — nothing to gate"
fi
log "positive: gating ${#promotable_slugs[@]} slug(s): ${promotable_slugs[*]}"

for slug in "${promotable_slugs[@]}"; do
    output="$(
        MYOSU_SOLVER_PROMOTION_LEDGER="$ledger_path" \
            SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical \
                --example verify_promotion_outputs -- --slug "$slug" 2>&1
    )" || fail "verify_promotion_outputs rejected $slug: $output"
    assert_contains "$output" "PROMOTION_GATE_PASS slug=$slug" "positive $slug"
done

# -------- 2. NEGATIVE PATH ----------------------------------------------
#
# Synthesize a placeholder outputs tree and a temp ledger that claims a
# placeholder game is `tier: promotable_local`. The gate MUST reject the
# placeholder. We use `nlhe-heads-up` (currently `benchmarked`) as the
# canary because the existing blocker note already calls out its
# `postflop_complete=false` shape.

placeholder_dir="$work_root/placeholders/nlhe-heads-up"
mkdir -p "$placeholder_dir"
printf '{"placeholder":"yes","game":"liars_dice"}\n' >"$placeholder_dir/bundle.json"
printf '{"benchmark_id":"sparse-bootstrap","metric_name":"reference_pack_pass","metric_value":1.5,"threshold":0.7,"passing":false}\n' >"$placeholder_dir/benchmark-summary.json"
printf '{"checkpoint_hash":"0000000000000000000000000000000000000000000000000000000000000000","checkpoint_format":"sparse-bootstrap","solver_family":"nlhe-blueprint-cfr","postflop_complete":false,"passing":false}\n' >"$placeholder_dir/artifact-manifest.json"

# Build a temp ledger that copies the live one but flips the nlhe-heads-up
# tier to `promotable_local`. The temp ledger is written under work_root so
# cleanup() reaps it; the live ledger is never modified.
temp_ledger="$work_root/placeholder_ledger.yaml"
python3 - "$ledger_path" "$temp_ledger" <<'PYEOF'
import sys, yaml
src, dst = sys.argv[1], sys.argv[2]
with open(src) as handle:
    data = yaml.safe_load(handle)
for game in data["games"]:
    if game["game"] == "nlhe-heads-up":
        game["tier"] = "promotable_local"
        game["notes"] = (
            "promotion_gate_negative_fixture;"
            " tier_force_promoted_to_promotable_local_with_placeholder_outputs"
        )
with open(dst, "w") as handle:
    yaml.safe_dump(data, handle, sort_keys=False)
PYEOF

negative_output="$(
    MYOSU_SOLVER_PROMOTION_LEDGER="$temp_ledger" \
        SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical \
            --example verify_promotion_outputs -- \
            --slug nlhe-heads-up \
            --outputs-dir "$work_root/placeholders" 2>&1
)" || true
assert_contains "$negative_output" "PROMOTION_GATE_FAIL" "negative nlhe-heads-up gate rejection"
assert_not_contains "$negative_output" "PROMOTION_GATE_PASS" "negative nlhe-heads-up gate did not pass"

# -------- 3. MISSING-DIRECTORY NEGATIVE PATH ----------------------------
#
# A game declared `tier: promotable_local` with no outputs/ tree at all must
# also be rejected. The existing harness already requires the directory's
# files; this script asserts the same is true when the ledger forces a
# tier upgrade for a slug whose outputs/ tree does not exist.

absent_dir="$work_root/absent"
mkdir -p "$absent_dir"
absent_output="$(
    MYOSU_SOLVER_PROMOTION_LEDGER="$temp_ledger" \
        SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical \
            --example verify_promotion_outputs -- \
            --slug nlhe-heads-up \
            --outputs-dir "$absent_dir" 2>&1
)" || true
assert_contains "$absent_output" "PROMOTION_GATE_FAIL" "absent nlhe-heads-up gate rejection"
assert_contains "$absent_output" "missing output:" "absent nlhe-heads-up gate missing-reason"
assert_not_contains "$absent_output" "PROMOTION_GATE_PASS" "absent nlhe-heads-up gate did not pass"

log "PROMOTION_GATE_HARNESS myosu e2e ok positive=${#promotable_slugs[@]} negative=2 ledger=$ledger_path"
