#!/usr/bin/env bash
# W-07 dedicated-solver read surface executable proof.
#
# The `myosu-solver-read-dedicated` binary is the read-only solver surface
# for the two dedicated-solver games (liars-dice, nlhe-heads-up) that the
# W-03 portfolio binary explicitly rejects. It takes a JSON request on stdin
# and emits one grep-friendly `SOLVER_READ` line (or `SOLVER_READ_FAIL
# reason=...` on error). The harness fail-closes on every regression that
# would change the line protocol, drop a dedicated game, or silently accept
# malformed input.
#
# Eight real assertions; failing any one of them fails the gate with a
# concrete error message.
#
#   1. The binary builds with `cargo build -p myosu-solver-read-dedicated
#      --bin myosu-solver-read-dedicated`.
#   2. A valid Liar's Dice JSON request round-trips through the binary and
#      emits `SOLVER_READ game=liars-dice action=... confidence=...
#      engine_tier=dedicated-cfr checkpoint_sha256=64-hex
#      legal_action_count=N elapsed_ms=N`.
#   3. A valid NLHE JSON request round-trips through the binary and emits
#      `SOLVER_READ game=nlhe-heads-up action=... confidence=...
#      engine_tier=dedicated-cfr checkpoint_sha256=64-hex
#      legal_action_count=N elapsed_ms=N`.
#   4. An unknown game slug prints `SOLVER_READ_FAIL reason=unknown_game: ...`
#      and exits non-zero.
#   5. A missing checkpoint field prints `SOLVER_READ_FAIL
#      reason=not_a_directory: ...` and exits non-zero.
#   6. A missing `encoder_dir` for NLHE prints `SOLVER_READ_FAIL
#      reason=missing_encoder_dir` and exits non-zero.
#   7. The checkpoint SHA-256 is byte-stable across two runs of the same
#      checkpoint (determinism contract).
#   8. The full `cargo test -p myosu-solver-read-dedicated` suite stays
#      green.
#
# Usage: bash tests/e2e/solver_read_dedicated.sh

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

pass=0
fail=0
note() { printf 'solver_read_dedicated: %s\n' "$*"; }
ok()   { note "ok $*"; pass=$((pass+1)); }
bad()  { note "FAIL $*"; fail=$((fail+1)); }

# --- 1. binary builds ---
note "building myosu-solver-read-dedicated binary..."
SKIP_WASM_BUILD=1 cargo build --quiet -p myosu-solver-read-dedicated --bin myosu-solver-read-dedicated
binary_path="$repo_root/target/debug/myosu-solver-read-dedicated"
if [[ ! -x "$binary_path" ]]; then
    bad "binary not found at $binary_path after build"
    echo
    echo "solver_read_dedicated: pass=$pass fail=$fail"
    [[ "$fail" -eq 0 ]]
    exit 1
fi
ok "myosu-solver-read-dedicated binary builds"

# Generate test artifacts (Liar's Dice checkpoint + NLHE encoder + checkpoint)
artifact_root="$(mktemp -d /tmp/solver-read-dedicated.XXXXXX)"
trap 'rm -rf "$artifact_root"' EXIT

SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-solver-read-dedicated --example generate_test_artifacts -- "$artifact_root"

ld_checkpoint="$artifact_root/liars-dice/checkpoint.bin"
nlhe_checkpoint="$artifact_root/nlhe/checkpoint.bin"
nlhe_encoder_dir="$artifact_root/nlhe/encoder"

if [[ ! -f "$ld_checkpoint" ]]; then
    bad "Liar's Dice checkpoint not generated at $ld_checkpoint"
    echo
    echo "solver_read_dedicated: pass=$pass fail=$fail"
    [[ "$fail" -eq 0 ]]
    exit 1
fi
if [[ ! -f "$nlhe_checkpoint" ]]; then
    bad "NLHE checkpoint not generated at $nlhe_checkpoint"
    echo
    echo "solver_read_dedicated: pass=$pass fail=$fail"
    [[ "$fail" -eq 0 ]]
    exit 1
fi
if [[ ! -d "$nlhe_encoder_dir" ]]; then
    bad "NLHE encoder dir not generated at $nlhe_encoder_dir"
    echo
    echo "solver_read_dedicated: pass=$pass fail=$fail"
    [[ "$fail" -eq 0 ]]
    exit 1
fi
ok "test artifacts generated"

# --- 2. Liar's Dice happy path ---
ld_request='{"game": "liars-dice", "checkpoint": "'"$ld_checkpoint"'", "query": {"info": {"public": {"actor": "P1", "last_claim_rank": 255}, "secret": 6}}}'
ld_output="$(printf '%s' "$ld_request" | "$binary_path")"
ld_exit=$?
if [[ $ld_exit -ne 0 ]]; then
    bad "liars-dice happy path exited $ld_exit; output: $ld_output"
else
    if printf '%s\n' "$ld_output" | grep -Eq '^SOLVER_READ game=liars-dice action=.* confidence=[0-9.]+ engine_tier=dedicated-cfr checkpoint_sha256=[a-f0-9]{64} legal_action_count=[0-9]+ elapsed_ms=[0-9]+$'; then
        ok "liars-dice happy path returns the documented SOLVER_READ line shape"
    else
        bad "liars-dice happy path line did not match protocol regex: $ld_output"
    fi
fi

# --- 3. NLHE happy path ---
nlhe_request='{"game": "nlhe-heads-up", "checkpoint": "'"$nlhe_checkpoint"'", "encoder_dir": "'"$nlhe_encoder_dir"'", "query": {"info": {"subgame": 179, "bucket": 42, "choices": 3394}}}'
nlhe_output="$(printf '%s' "$nlhe_request" | "$binary_path")"
nlhe_exit=$?
if [[ $nlhe_exit -ne 0 ]]; then
    bad "nlhe-heads-up happy path exited $nlhe_exit; output: $nlhe_output"
else
    if printf '%s\n' "$nlhe_output" | grep -Eq '^SOLVER_READ game=nlhe-heads-up action=.* confidence=[0-9.]+ engine_tier=dedicated-cfr checkpoint_sha256=[a-f0-9]{64} legal_action_count=[0-9]+ elapsed_ms=[0-9]+$'; then
        ok "nlhe-heads-up happy path returns the documented SOLVER_READ line shape"
    else
        bad "nlhe-heads-up happy path line did not match protocol regex: $nlhe_output"
    fi
fi

# --- 4. Unknown game slug fails closed ---
unknown_request='{"game": "not-a-real-game", "checkpoint": "'"$ld_checkpoint"'", "query": {}}'
set +e
unknown_output="$(printf '%s' "$unknown_request" | "$binary_path" 2>&1)"
unknown_exit=$?
set -e
if [[ $unknown_exit -eq 0 ]]; then
    bad "unknown-game slug exited 0, expected non-zero: $unknown_output"
else
    if printf '%s\n' "$unknown_output" | grep -Eq '^SOLVER_READ_FAIL reason=unknown_game:'; then
        ok "unknown-game slug returns SOLVER_READ_FAIL reason=unknown_game + non-zero exit"
    else
        bad "unknown-game slug did not return expected failure line: $unknown_output"
    fi
fi

# --- 5. Missing checkpoint fails closed ---
missing_checkpoint_request='{"game": "liars-dice", "checkpoint": "/tmp/nonexistent-checkpoint.bin", "query": {}}'
set +e
missing_checkpoint_output="$(printf '%s' "$missing_checkpoint_request" | "$binary_path" 2>&1)"
missing_checkpoint_exit=$?
set -e
if [[ $missing_checkpoint_exit -eq 0 ]]; then
    bad "missing-checkpoint exited 0, expected non-zero: $missing_checkpoint_output"
else
    if printf '%s\n' "$missing_checkpoint_output" | grep -Eq '^SOLVER_READ_FAIL reason=not_a_directory:'; then
        ok "missing-checkpoint returns SOLVER_READ_FAIL reason=not_a_directory + non-zero exit"
    else
        bad "missing-checkpoint did not return expected failure line: $missing_checkpoint_output"
    fi
fi

# --- 6. Missing encoder_dir for NLHE fails closed ---
missing_encoder_request='{"game": "nlhe-heads-up", "checkpoint": "'"$nlhe_checkpoint"'", "query": {}}'
set +e
missing_encoder_output="$(printf '%s' "$missing_encoder_request" | "$binary_path" 2>&1)"
missing_encoder_exit=$?
set -e
if [[ $missing_encoder_exit -eq 0 ]]; then
    bad "missing-encoder_dir exited 0, expected non-zero: $missing_encoder_output"
else
    if printf '%s\n' "$missing_encoder_output" | grep -Fxq 'SOLVER_READ_FAIL reason=missing_encoder_dir'; then
        ok "missing-encoder_dir returns SOLVER_READ_FAIL reason=missing_encoder_dir + non-zero exit"
    else
        bad "missing-encoder_dir did not return expected failure line: $missing_encoder_output"
    fi
fi

# --- 7. Checkpoint SHA-256 is byte-stable across two runs ---
ld_output_a="$(printf '%s' "$ld_request" | "$binary_path")"
ld_sha_a="$(printf '%s\n' "$ld_output_a" | grep -oP 'checkpoint_sha256=\K[a-f0-9]{64}')"
ld_output_b="$(printf '%s' "$ld_request" | "$binary_path")"
ld_sha_b="$(printf '%s\n' "$ld_output_b" | grep -oP 'checkpoint_sha256=\K[a-f0-9]{64}')"
if [[ "$ld_sha_a" == "$ld_sha_b" && -n "$ld_sha_a" ]]; then
    ok "checkpoint SHA-256 is byte-stable across two runs ($ld_sha_a)"
else
    bad "checkpoint SHA-256 drifted between runs: a=$ld_sha_a b=$ld_sha_b"
fi

# --- 8. unit tests pass ---
note "running: SKIP_WASM_BUILD=1 cargo test -p myosu-solver-read-dedicated --quiet"
unit_output="$(env SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-solver-read-dedicated 2>&1)"
unit_passed=0
unit_failed=0
while IFS= read -r line; do
    if [[ "$line" =~ ^test\ result:\ ok\.\ ([0-9]+)\ passed ]]; then
        unit_passed=$((unit_passed + BASH_REMATCH[1]))
    elif [[ "$line" =~ ^test\ result:\ FAILED\.\ ([0-9]+)\ failed ]]; then
        unit_failed=$((unit_failed + BASH_REMATCH[1]))
    fi
done <<< "$unit_output"
if [[ "$unit_failed" -gt 0 ]]; then
    bad "unit tests had $unit_failed failures"
    printf '%s\n' "$unit_output" | tail -20
elif [[ "$unit_passed" -ge 8 ]]; then
    ok "unit tests are green: $unit_passed passed (>= 8 expected)"
else
    bad "expected at least 8 unit tests, got $unit_passed"
    printf '%s\n' "$unit_output" | tail -20
fi

echo
echo "solver_read_dedicated: pass=$pass fail=$fail"
[[ "$fail" -eq 0 ]]
