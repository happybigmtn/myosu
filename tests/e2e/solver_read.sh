#!/usr/bin/env bash
# W-03 / genesis/plans/000-ceo-testnet-roadmap.md (the agent-readability
# half) executable proof.
#
# The `myosu-solver-read` binary is the read-only solver surface for
# external agents and humans: it takes a game slug + a JSON scenario on
# stdin and emits one grep-friendly `SOLVER_READ` line (or a
# `SOLVER_READ_FAIL reason=...` line on any error). The harness fail-closes
# on every regression that would change the line protocol, route a
# non-portfolio-routed game through the rule-aware engine, or silently
# accept malformed input.
#
# Seven real assertions; failing any one of them fails the gate with a
# concrete error message.
#
#   1. The binary builds with `cargo build -p myosu-games-portfolio
#      --bin myosu_solver_read`.
#   2. A valid Cribbage challenge round-trip emits a `SOLVER_READ
#      game=cribbage action=<one of peg-run|keep-crib|discard-deadwood>`
#      line and exits 0.
#   3. A valid Hearts (trick-taking family) challenge round-trip emits a
#      `SOLVER_READ game=hearts action=<one of avoid-penalty|follow-suit|
#      shoot-moon>` line and exits 0.
#   4. A valid Dou-Di-Zhu (shedding family) challenge round-trip emits a
#      `SOLVER_READ game=dou-di-zhu action=<one of shed-lowest|preserve-bomb
#      |lead-control|pass-control>` line and exits 0 (proves the typed
#      PortfolioChallenge dispatch covers all three engine families).
#   5. An unknown game slug (`game=not-a-real-game`) prints
#      `SOLVER_READ_FAIL reason=unknown_game` and exits non-zero.
#   6. A malformed JSON input prints `SOLVER_READ_FAIL reason=invalid_json`
#      and exits non-zero.
#   7. An empty stdin prints `SOLVER_READ_FAIL reason=empty_stdin` and
#      exits non-zero.
#
# The full `cargo test -p myosu-games-portfolio` suite is also exercised
# (assertion 8) so the binary's own `myosu_solver_read` unit tests stay
# wired.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

if ! command -v python3 >/dev/null 2>&1; then
    printf 'solver_read: python3 is required for JSON shape checks\n' >&2
    exit 1
fi

# -- 1. The binary builds.
echo "solver_read: building myosu_solver_read binary..." >&2
SKIP_WASM_BUILD=1 cargo build --quiet -p myosu-games-portfolio --bin myosu_solver_read
binary_path="$repo_root/target/debug/myosu_solver_read"
if [[ ! -x "$binary_path" ]]; then
    printf 'solver_read: binary not found at %s after build\n' "$binary_path" >&2
    exit 1
fi

work_root="$(mktemp -d /tmp/solver-read.XXXXXX)"
trap 'rm -rf "$work_root"' EXIT

# -- 2. Cribbage happy path.
cribbage_request="$work_root/cribbage.json"
cat >"$cribbage_request" <<'JSON'
{
  "game": "cribbage",
  "challenge": {
    "cribbage": {
      "spot": {
        "challenge_id": "discard-pressure",
        "decision": "Cribbage discard pressure spot",
        "rule_file": "20-cribbage.md",
        "solver_family": "rule-aware"
      },
      "pegging_count": 4,
      "run_potential": 2,
      "crib_edge": 0,
      "pair_trap": false,
      "go_window": false,
      "fifteen_outs": 1,
      "max_immediate_points": 2
    }
  }
}
JSON
cribbage_output="$("$binary_path" <"$cribbage_request")"
cribbage_exit=$?
if [[ $cribbage_exit -ne 0 ]]; then
    printf 'solver_read: cribbage happy path exited non-zero (%d)\n%s\n' \
        "$cribbage_exit" "$cribbage_output" >&2
    exit 1
fi
if ! printf '%s\n' "$cribbage_output" | grep -Eq '^SOLVER_READ game=cribbage action=(peg-run|keep-crib|discard-deadwood) '; then
    printf 'solver_read: cribbage line did not match protocol\n%s\n' "$cribbage_output" >&2
    exit 1
fi
if ! printf '%s\n' "$cribbage_output" | grep -Fq 'engine_tier=rule-aware'; then
    printf 'solver_read: cribbage line missing engine_tier=rule-aware\n%s\n' "$cribbage_output" >&2
    exit 1
fi
if ! printf '%s\n' "$cribbage_output" | grep -Eq 'confidence=[0-9.]+'; then
    printf 'solver_read: cribbage line missing confidence= field\n%s\n' "$cribbage_output" >&2
    exit 1
fi
if ! printf '%s\n' "$cribbage_output" | grep -Eq 'elapsed_ms=[0-9]+'; then
    printf 'solver_read: cribbage line missing elapsed_ms= field\n%s\n' "$cribbage_output" >&2
    exit 1
fi
cribbage_summary="$(printf '%s\n' "$cribbage_output" | head -n 1)"
printf 'SOLVER_READ_HARNESS happy_path=cribbage line=%s\n' "$cribbage_summary"

# -- 3. Hearts (trick-taking family) happy path.
hearts_request="$work_root/hearts.json"
cat >"$hearts_request" <<'JSON'
{
  "game": "hearts",
  "challenge": {
    "hearts": {
      "spot": {
        "challenge_id": "avoid-penalty",
        "decision": "Hearts avoid-penalty spot",
        "rule_file": "21-hearts-cribbage.md",
        "solver_family": "rule-aware"
      },
      "trump_count": 1,
      "winners": 0,
      "void_suits": 0,
      "contract_pressure": 0,
      "penalty_pressure": 1,
      "cards_in_trick": 1,
      "follow_suit_forced": false,
      "nil_viable": false,
      "moon_shot_viable": false
    }
  }
}
JSON
hearts_output="$("$binary_path" <"$hearts_request")"
hearts_exit=$?
if [[ $hearts_exit -ne 0 ]]; then
    printf 'solver_read: hearts happy path exited non-zero (%d)\n%s\n' \
        "$hearts_exit" "$hearts_output" >&2
    exit 1
fi
if ! printf '%s\n' "$hearts_output" | grep -Eq '^SOLVER_READ game=hearts action=(avoid-penalty|follow-suit|shoot-moon) '; then
    printf 'solver_read: hearts line did not match protocol\n%s\n' "$hearts_output" >&2
    exit 1
fi
hearts_summary="$(printf '%s\n' "$hearts_output" | head -n 1)"
printf 'SOLVER_READ_HARNESS happy_path=hearts line=%s\n' "$hearts_summary"

# -- 4. Dou-Di-Zhu (shedding family) happy path.
ddz_request="$work_root/dou_di_zhu.json"
cat >"$ddz_request" <<'JSON'
{
  "game": "dou-di-zhu",
  "challenge": {
    "dou_di_zhu": {
      "spot": {
        "challenge_id": "landlord-on-lead",
        "decision": "Dou-Di-Zhu landlord on-lead",
        "rule_file": "16-dou-di-zhu.md",
        "solver_family": "rule-aware"
      },
      "bomb_count": 0,
      "control_combos": 1,
      "low_singles": 4,
      "opponents_min_cards": 8,
      "danger_opponents": 0,
      "next_actor_cards": 8,
      "on_lead": true,
      "play_options": 4,
      "finishing_plays": 0,
      "bomb_only_escape": false,
      "forced_pass": false,
      "lead_rank_pressure": 1
    }
  }
}
JSON
ddz_output="$("$binary_path" <"$ddz_request")"
ddz_exit=$?
if [[ $ddz_exit -ne 0 ]]; then
    printf 'solver_read: dou-di-zhu happy path exited non-zero (%d)\n%s\n' \
        "$ddz_exit" "$ddz_output" >&2
    exit 1
fi
if ! printf '%s\n' "$ddz_output" | grep -Eq '^SOLVER_READ game=dou-di-zhu action=(shed-lowest|preserve-bomb|lead-control|pass-control) '; then
    printf 'solver_read: dou-di-zhu line did not match protocol\n%s\n' "$ddz_output" >&2
    exit 1
fi
ddz_summary="$(printf '%s\n' "$ddz_output" | head -n 1)"
printf 'SOLVER_READ_HARNESS happy_path=dou-di-zhu line=%s\n' "$ddz_summary"

# -- 5. Unknown game slug fails closed. The challenge must deserialize as a
# valid PortfolioChallenge variant (so the binary can read the outer
# `game` field) — we use a Hearts challenge variant with a slug that does
# not exist in ResearchGame::from_slug's table. The binary is expected
# to exit non-zero, so we wrap each failing-exit capture in
# `(set +e; ...)` so `set -e` does not abort the harness before the
# exit-code check runs.
unknown_request="$work_root/unknown.json"
cat >"$unknown_request" <<'JSON'
{
  "game": "not-a-real-game",
  "challenge": {
    "hearts": {
      "spot": {
        "challenge_id": "unknown-slug",
        "decision": "unknown slug regression test",
        "rule_file": "21-hearts-cribbage.md",
        "solver_family": "rule-aware"
      },
      "trump_count": 0,
      "winners": 0,
      "void_suits": 0,
      "contract_pressure": 0,
      "penalty_pressure": 0,
      "cards_in_trick": 1,
      "follow_suit_forced": false,
      "nil_viable": false,
      "moon_shot_viable": false
    }
  }
}
JSON
# Run the failing-exit cases under `set +e` so the harness itself does
# not abort on the expected non-zero exit codes.
set +e
unknown_output="$("$binary_path" <"$unknown_request")"
unknown_exit=$?
malformed_output="$("$binary_path" <<<'this is not json')"
malformed_exit=$?
empty_output="$("$binary_path" </dev/null)"
empty_exit=$?
set -e

if [[ $unknown_exit -eq 0 ]]; then
    printf 'solver_read: unknown-game exit was 0, expected non-zero\n%s\n' \
        "$unknown_output" >&2
    exit 1
fi
if ! printf '%s\n' "$unknown_output" | grep -Fxq 'SOLVER_READ_FAIL reason=unknown_game: not-a-real-game'; then
    printf 'solver_read: unknown-game line did not match protocol\n%s\n' "$unknown_output" >&2
    exit 1
fi
printf 'SOLVER_READ_HARNESS unknown_slug=ok line=%s\n' "$unknown_output"

# -- 6. Malformed JSON fails closed.
if [[ $malformed_exit -eq 0 ]]; then
    printf 'solver_read: malformed-json exit was 0, expected non-zero\n%s\n' \
        "$malformed_output" >&2
    exit 1
fi
if ! printf '%s\n' "$malformed_output" | grep -Eq '^SOLVER_READ_FAIL reason=invalid_json:'; then
    printf 'solver_read: malformed-json line did not match protocol\n%s\n' "$malformed_output" >&2
    exit 1
fi
printf 'SOLVER_READ_HARNESS malformed_json=ok line=%s\n' "$malformed_output"

# -- 7. Empty stdin fails closed.
if [[ $empty_exit -eq 0 ]]; then
    printf 'solver_read: empty-stdin exit was 0, expected non-zero\n%s\n' \
        "$empty_output" >&2
    exit 1
fi
if ! printf '%s\n' "$empty_output" | grep -Fxq 'SOLVER_READ_FAIL reason=empty_stdin'; then
    printf 'solver_read: empty-stdin line did not match protocol\n%s\n' "$empty_output" >&2
    exit 1
fi
printf 'SOLVER_READ_HARNESS empty_stdin=ok line=%s\n' "$empty_output"

# -- 8. The portfolio crate's myosu_solver_read bin unit tests pass.
# Run the bin's own unit-test target (the 7 in `mod tests` inside
# `src/bin/myosu_solver_read.rs`) and assert all of them pass. Using
# the bin target (not `-- myosu_solver_read`, which is not a test name
# in the lib) avoids the false-positive of "0 passed; 0 failed".
unit_output="$(
    env SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-games-portfolio --bin myosu_solver_read
)"
if ! printf '%s\n' "$unit_output" | grep -Eq '^test result: ok\. 7 passed'; then
    printf 'solver_read: myosu_solver_read unit tests did not all pass (expected 7 passed)\n%s\n' \
        "$unit_output" >&2
    exit 1
fi
unit_pass_line="$(
    printf '%s\n' "$unit_output" \
        | grep -E '^test result: ok\.' \
        | head -n 1
)"
printf 'SOLVER_READ_HARNESS myosu_solver_read unit tests: %s\n' "$unit_pass_line"

# -- 9. The full portfolio crate test suite stays green (no engine touched,
# no dossier touched, no YAML touched — the only change is the new binary
# and its tests).
full_unit_output="$(
    env SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-games-portfolio
)"
if ! printf '%s\n' "$full_unit_output" | grep -Eq '^test result: ok\. [0-9]+ passed'; then
    printf 'solver_read: full portfolio test suite did not all pass\n%s\n' \
        "$full_unit_output" >&2
    exit 1
fi
full_pass_line="$(
    printf '%s\n' "$full_unit_output" \
        | grep -E '^test result: ok\.' \
        | head -n 1
)"
printf 'SOLVER_READ_HARNESS full portfolio suite: %s\n' "$full_pass_line"

# -- 10. The dedicated-solver pointer is present in the W-03 binary's
# `not_portfolio_routed` source path so an operator running the W-03
# binary discovers the W-07 `myosu-solver-read-dedicated` binary when
# they accidentally route `liars-dice` or `nlhe-heads-up` through the
# portfolio surface. The runtime path to this message requires a
# portfolio-challenge shape that matches the dedicated slug; that
# shape does not exist (NlheHeadsUp and LiarsDice have no
# PortfolioChallenge variants by design), so the source-path grep is
# the regression guard instead.
if ! grep -Fq 'use myosu-solver-read-dedicated' \
    crates/myosu-games-portfolio/src/bin/myosu_solver_read.rs; then
    printf 'solver_read: W-03 not_portfolio_routed reason does not reference myosu-solver-read-dedicated\n' >&2
    exit 1
fi
printf 'SOLVER_READ_HARNESS dedicated_solver_pointer=ok source=myosu_solver_read.rs\n'

printf 'SOLVER_READ_HARNESS myosu e2e ok binary=%s\n' "$binary_path"
