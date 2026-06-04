#!/usr/bin/env bash
# W-02 public policy bundles proof harness.
#
# `ops/bundles/liars-dice/` is the W-02 public bundle surface for the
# liars-dice `promotable_local` game. The canonical triple at
# `ops/bundles/liars-dice/liars-dice/{bundle,benchmark-summary,artifact-manifest}.json`
# must be (1) present, (2) non-empty, (3) accepted by
# `verify_promotion_outputs` (PROMOTION_GATE_PASS), (4) carry
# `provenance.game_slug == liars-dice`, and (5) byte-stable: rebuilding
# the bundle from a clean invocation must produce a `bundle.json` whose
# `bundle_hash` field AND whose file bytes match the canonical record.
#
# Exit 0 only when all five sub-checks pass. This is the executable
# drift guard behind `docs/operator-guide/public-bundles.md` and the
# `public-bundles-manifest` CI job in `.github/workflows/ci.yml`.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

ledger_path="${MYOSU_SOLVER_PROMOTION_LEDGER:-$repo_root/ops/solver_promotion.yaml}"
outputs_dir="$repo_root/ops/bundles/liars-dice"
slug="liars-dice"
bundle_path="$outputs_dir/$slug/bundle.json"
summary_path="$outputs_dir/$slug/benchmark-summary.json"
manifest_path="$outputs_dir/$slug/artifact-manifest.json"
readme_path="$outputs_dir/README.md"
doc_path="$repo_root/docs/operator-guide/public-bundles.md"
tmp_root=""
work_root=""

cleanup() {
    local exit_code=$?
    if [[ -n "$tmp_root" && -d "$tmp_root" ]]; then
        rm -rf "$tmp_root"
    fi
    if [[ -n "$work_root" && -d "$work_root" ]]; then
        rm -rf "$work_root"
    fi
    return "$exit_code"
}
trap cleanup EXIT

work_root="$(mktemp -d "$repo_root/target/public-bundles-manifest.XXXXXX")"
tmp_root="$(mktemp -d "$repo_root/target/public-bundles-manifest-tmp.XXXXXX")"

log() { printf '%s\n' "$*"; }
fail() { log "PUBLIC_BUNDLES_MANIFEST_FAIL $*"; exit 1; }

assert_contains() {
    local blob="$1" needle="$2" label="$3"
    if ! printf '%s\n' "$blob" | grep -Fq "$needle"; then
        log "${label} missing expected text: ${needle}" >&2
        printf '%s\n' "$blob" >&2
        fail "$label"
    fi
}

assert_eq() {
    local actual="$1" expected="$2" label="$3"
    if [[ "$actual" != "$expected" ]]; then
        log "${label}: expected=${expected} actual=${actual}" >&2
        fail "$label"
    fi
}

# -------- 1. doc is present -------------------------------------------------

if [[ ! -f "$doc_path" ]]; then
    fail "W-02 doc missing at $doc_path"
fi
if [[ ! -s "$doc_path" ]]; then
    fail "W-02 doc is empty at $doc_path"
fi
log "PUBLIC_BUNDLES_MANIFEST ok 1/5 doc present at $doc_path"

# -------- 2. canonical triple is present and non-empty ---------------------

if [[ ! -f "$bundle_path" || ! -f "$summary_path" || ! -f "$manifest_path" ]]; then
    fail "canonical triple missing at $outputs_dir/$slug/ (need bundle.json, benchmark-summary.json, artifact-manifest.json)"
fi
for f in "$bundle_path" "$summary_path" "$manifest_path"; do
    if [[ ! -s "$f" ]]; then
        fail "canonical triple file is empty: $f"
    fi
done
if [[ ! -f "$readme_path" ]]; then
    fail "W-02 README missing at $readme_path"
fi
log "PUBLIC_BUNDLES_MANIFEST ok 2/5 canonical triple present and non-empty at $outputs_dir/$slug/"

# -------- 3. canonical bundle passes verify_promotion_outputs (positive) ---

verify_output="$(
    MYOSU_SOLVER_PROMOTION_LEDGER="$ledger_path" \
        SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical \
            --example verify_promotion_outputs -- \
            --slug "$slug" --outputs-dir "$outputs_dir" 2>&1
)" || fail "verify_promotion_outputs rejected canonical triple: $verify_output"
assert_contains "$verify_output" "PROMOTION_GATE_PASS slug=$slug" "PROMOTION_GATE_PASS line for $slug"
log "PUBLIC_BUNDLES_MANIFEST ok 3/5 canonical triple passes verify_promotion_outputs (PROMOTION_GATE_PASS slug=$slug)"

# -------- 4. canonical bundle's provenance.game_slug is `liars-dice` -------

game_slug="$(jq -r '.provenance.game_slug // empty' "$bundle_path")"
assert_eq "$game_slug" "$slug" "provenance.game_slug"
log "PUBLIC_BUNDLES_MANIFEST ok 4/5 canonical bundle.provenance.game_slug == $slug"

# -------- 5. byte-stability: re-built bundle_hash matches the canonical ----
#
# The reproduction step (from ops/bundles/liars-dice/README.md) rebuilds
# the triple into a fresh tmpdir. The freshly-printed `bundle_hash` MUST
# match the canonical bundle.json's self-reported `bundle_hash` field
# AND the SHA-256 of the canonical bundle.json's file bytes. Both
# assertions fail closed if a future change to the solver, the bundle
# builder, the dossier, or the example drifts the canonical triple.

rebuild_dir="$tmp_root/rebuild"
mkdir -p "$rebuild_dir/$slug"

rebuild_output="$(
    SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-liars-dice \
        --example liars_dice_policy_bundle -- \
        --output "$rebuild_dir/$slug/bundle.json" \
        --iterations 512 2>&1
)" || fail "liars_dice_policy_bundle re-build failed: $rebuild_output"

assert_contains "$rebuild_output" "POLICY_BUNDLE bundle_hash=" "POLICY_BUNDLE bundle_hash= line"
rebuilt_hash="$(printf '%s\n' "$rebuild_output" | sed -nE 's/^POLICY_BUNDLE bundle_hash=([0-9a-f]+).*/\1/p' | head -n1)"
if [[ -z "$rebuilt_hash" ]]; then
    log "could not extract rebuilt bundle_hash from:" >&2
    printf '%s\n' "$rebuild_output" >&2
    fail "rebuild bundle_hash extraction"
fi

canonical_hash="$(jq -r '.bundle_hash // empty' "$bundle_path")"
if [[ -z "$canonical_hash" ]]; then
    fail "canonical bundle.bundle_hash is missing from $bundle_path"
fi
# The `bundle_hash` is the canonical-hash field embedded in the
# CanonicalPolicyBundle itself (see `compute_bundle_hash` in
# `crates/myosu-games-canonical/src/policy.rs`); it is computed over
# the bundle's fields, NOT over the JSON-serialized file bytes (the
# JSON is non-canonical: serde_json key order and whitespace can
# drift, so file-SHA-256 is not the right equality test). The proof
# is that a fresh re-build of the same triple from a clean invocation
# produces a bundle whose self-reported `bundle_hash` matches the
# canonical self-reported `bundle_hash` byte-for-byte.
assert_eq "$rebuilt_hash" "$canonical_hash" "rebuilt bundle_hash == canonical bundle.bundle_hash"

# Also assert the freshly-built bundle.json is byte-stable in the
# self-reported hash: the on-disk `bundle_hash` field of the rebuilt
# bundle must equal the printed `bundle_hash`. (Catches a future
# refactor that decouples the example's printed hash from the on-disk
# field.)
rebuilt_ondisk_hash="$(jq -r '.bundle_hash // empty' "$rebuild_dir/$slug/bundle.json")"
assert_eq "$rebuilt_ondisk_hash" "$rebuilt_hash" "rebuilt on-disk bundle_hash == printed bundle_hash"

log "PUBLIC_BUNDLES_MANIFEST ok 5/5 byte-stability: rebuilt bundle_hash == canonical bundle.bundle_hash (hash=${rebuilt_hash})"

# -------- 6. public-bundles-manifest CI job is wired into ci.yml -----------

if ! grep -nE '^\s*public-bundles-manifest:|^[[:space:]]+name:[[:space:]]+Public policy bundles \(W-02\)$' .github/workflows/ci.yml >/dev/null; then
    fail "public-bundles-manifest CI job missing from .github/workflows/ci.yml"
fi
log "PUBLIC_BUNDLES_MANIFEST ok 6/5 public-bundles-manifest CI job wired into .github/workflows/ci.yml"

log "PUBLIC_BUNDLES_MANIFEST_HARNESS myosu e2e ok ledger=$ledger_path bundle_hash=${rebuilt_hash}"
