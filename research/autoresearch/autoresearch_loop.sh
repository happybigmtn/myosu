#!/usr/bin/env bash
set -euo pipefail

BASE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE="$BASE/workspace"
STATE_DIR="$BASE/state"
CONFIG_DIR="$BASE/config"
LOG_DIR="$BASE/logs"
LOCK_DIR="$BASE/.lock"
CANDIDATE_CONFIG="$WORKSPACE/candidate_config.json"
CHAMPION_CONFIG="$CONFIG_DIR/champion_config.json"
CHAMPION_SMOKE="$STATE_DIR/champion_smoke.json"
CHAMPION_FULL="$STATE_DIR/champion_full.json"
RESULTS_JSONL="$STATE_DIR/results.jsonl"
SMOKE_CORPUS="$CONFIG_DIR/corpus_smoke.json"
FULL_CORPUS="$CONFIG_DIR/corpus_full.json"
EVAL_SCRIPT="$BASE/scripts/run_eval.py"
MUTATE_SCRIPT="$BASE/scripts/mutate_candidate.sh"
EVAL_TIMEOUT_SECONDS="${AUTORESEARCH_EVAL_TIMEOUT_SECONDS:-300}"
MUTATOR_TIMEOUT_SECONDS="${AUTORESEARCH_MUTATOR_TIMEOUT_SECONDS:-1200}"
SLEEP_SECONDS="${AUTORESEARCH_SLEEP_SECONDS:-60}"

mkdir -p "$WORKSPACE" "$STATE_DIR" "$LOG_DIR" "$STATE_DIR/archive"
exec > >(tee -a "$LOG_DIR/autoresearch-loop.log") 2>&1

if ! mkdir "$LOCK_DIR" 2>/dev/null; then
  echo "autoresearch lock is already held at $LOCK_DIR"
  exit 1
fi
cleanup() {
  jobs -pr 2>/dev/null | xargs -r kill 2>/dev/null || true
  pkill -P $$ 2>/dev/null || true
  rmdir "$LOCK_DIR" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

timestamp() {
  date -u +"%Y-%m-%dT%H:%M:%SZ"
}

ensure_baseline() {
  cp "$BASE/AUTORESEARCH.md" "$WORKSPACE/AUTORESEARCH.md"
  cp "$CHAMPION_CONFIG" "$CANDIDATE_CONFIG"

  if [ ! -f "$CHAMPION_SMOKE" ]; then
    python3 "$EVAL_SCRIPT" \
      --config "$CHAMPION_CONFIG" \
      --corpus "$SMOKE_CORPUS" \
      --timeout "$EVAL_TIMEOUT_SECONDS" \
      --output "$CHAMPION_SMOKE" >/dev/null
  fi
  if [ ! -f "$CHAMPION_FULL" ]; then
    python3 "$EVAL_SCRIPT" \
      --config "$CHAMPION_CONFIG" \
      --corpus "$FULL_CORPUS" \
      --timeout "$EVAL_TIMEOUT_SECONDS" \
      --output "$CHAMPION_FULL" >/dev/null
  fi
}

render_iteration_context() {
  python3 - "$CHAMPION_SMOKE" "$CHAMPION_FULL" "$RESULTS_JSONL" > "$WORKSPACE/ITERATION_CONTEXT.md" <<'PY'
from pathlib import Path
import json
import sys

smoke_path = Path(sys.argv[1])
full_path = Path(sys.argv[2])
results_path = Path(sys.argv[3])

smoke = json.loads(smoke_path.read_text(encoding="utf-8"))["summary"]
full = json.loads(full_path.read_text(encoding="utf-8"))["summary"]

recent_lines = []
if results_path.exists():
    recent_lines = [line for line in results_path.read_text(encoding="utf-8").splitlines() if line.strip()][-5:]

print("# Iteration Context")
print()
print("Current champion scores:")
print(f"- smoke total_score: {smoke['total_score']}")
print(f"- smoke successful_cases: {smoke['successful_cases']}/{smoke['case_count']}")
print(f"- smoke mean_primary_metric: {smoke['mean_primary_metric']}")
print(f"- full total_score: {full['total_score']}")
print(f"- full successful_cases: {full['successful_cases']}/{full['case_count']}")
print(f"- full mean_primary_metric: {full['mean_primary_metric']}")
print()
print("Corpus mapping:")
print("- smoke = 3 games (NLHE HU, Liar's Dice, Backgammon)")
print("- full = all 20 imperfect-information games from OS.md")
print()
print("Rules:")
print("- Only edit `candidate_config.json` in this workspace.")
print("- Make one small, reversible change.")
print("- Do not change method names, evaluation harness, or corpus files.")
print("- Optimize for architecture ranking accuracy (primary_metric).")
print()
print("Recent iteration ledger:")
if recent_lines:
    for line in recent_lines:
        print(f"- {line}")
else:
    print("- no prior iterations yet")
PY
}

is_better() {
  python3 - "$1" "$2" <<'PY'
from pathlib import Path
import json
import sys

candidate = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))["summary"]
champion = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))["summary"]

candidate_key = (
    candidate["total_score"],
    candidate["successful_cases"],
    -candidate["timed_out_cases"],
)
champion_key = (
    champion["total_score"],
    champion["successful_cases"],
    -champion["timed_out_cases"],
)
print("1" if candidate_key > champion_key else "0")
PY
}

append_result() {
  local status="$1"
  local iteration="$2"
  local note="$3"
  local smoke_score="$4"
  local full_score="$5"
  printf '%s\titeration=%s\tstatus=%s\tsmoke=%s\tfull=%s\tnote=%s\n' \
    "$(timestamp)" "$iteration" "$status" "$smoke_score" "$full_score" "$note" >> "$RESULTS_JSONL"
}

archive_champion() {
  local iteration="$1"
  local archive_path="$STATE_DIR/archive/champion-iter${iteration}-$(date -u +%Y%m%dT%H%M%SZ).json"
  cp "$CHAMPION_CONFIG" "$archive_path"
  echo "archived previous champion to $archive_path"
}

run_iteration() {
  local iteration="$1"
  local before_hash mutator_log smoke_out full_out
  local smoke_score full_score champion_smoke_score champion_full_score
  local quota_backoff_seconds mutator_note

  cp "$CHAMPION_CONFIG" "$CANDIDATE_CONFIG"
  cp "$BASE/AUTORESEARCH.md" "$WORKSPACE/AUTORESEARCH.md"
  render_iteration_context
  before_hash="$(sha256sum "$CANDIDATE_CONFIG" | awk '{print $1}')"
  mutator_log="$LOG_DIR/mutator-iteration-${iteration}.log"

  set +e
  timeout "$MUTATOR_TIMEOUT_SECONDS" "$MUTATE_SCRIPT" "$WORKSPACE" "$iteration" \
    >"$mutator_log" 2>&1
  local mutator_rc=$?
  set -e

  if [ "$mutator_rc" -ne 0 ]; then
    quota_backoff_seconds="${AUTORESEARCH_QUOTA_BACKOFF_SECONDS:-1800}"
    if grep -qiE 'usage limit|rate limit|try again at|too many requests|quota' "$mutator_log"; then
      mutator_note="mutator_usage_limit"
      append_result "mutator_quota" "$iteration" "$mutator_note" "-" "-"
      echo "iteration $iteration: mutator usage limit detected; sleeping ${quota_backoff_seconds}s before retry"
      sleep "$quota_backoff_seconds"
      return 0
    fi
    if grep -qiE 'failed to authenticate|authentication credentials|401' "$mutator_log"; then
      mutator_note="mutator_auth_error"
      append_result "mutator_auth" "$iteration" "$mutator_note" "-" "-"
      echo "iteration $iteration: mutator authentication failed"
      return 0
    fi
    mutator_note="mutator_rc=$mutator_rc"
    append_result "mutator_error" "$iteration" "$mutator_note" "-" "-"
    echo "iteration $iteration: mutator failed rc=$mutator_rc"
    return 0
  fi

  if [ "$before_hash" = "$(sha256sum "$CANDIDATE_CONFIG" | awk '{print $1}')" ]; then
    append_result "no_change" "$iteration" "candidate unchanged" "-" "-"
    echo "iteration $iteration: no candidate change"
    return 0
  fi

  # Validate candidate is parseable JSON
  if ! python3 -c "import json, sys; json.load(open(sys.argv[1]))" "$CANDIDATE_CONFIG" 2>/dev/null; then
    append_result "invalid_json" "$iteration" "candidate not valid JSON" "-" "-"
    echo "iteration $iteration: candidate config is not valid JSON"
    return 0
  fi

  smoke_out="$STATE_DIR/smoke-iteration-${iteration}.json"
  full_out="$STATE_DIR/full-iteration-${iteration}.json"

  # Smoke test: 3 games
  python3 "$EVAL_SCRIPT" \
    --config "$CANDIDATE_CONFIG" \
    --corpus "$SMOKE_CORPUS" \
    --timeout "$EVAL_TIMEOUT_SECONDS" \
    --output "$smoke_out" >/dev/null

  if [ "$(is_better "$smoke_out" "$CHAMPION_SMOKE")" != "1" ]; then
    smoke_score="$(python3 -c "import json; print(json.load(open('$smoke_out'))['summary']['total_score'])")"
    champion_smoke_score="$(python3 -c "import json; print(json.load(open('$CHAMPION_SMOKE'))['summary']['total_score'])")"
    append_result "reject_smoke" "$iteration" "smoke_not_better" "$smoke_score" "-"
    echo "iteration $iteration: smoke $smoke_score did not beat champion $champion_smoke_score"
    return 0
  fi

  # Full eval: 20 games
  python3 "$EVAL_SCRIPT" \
    --config "$CANDIDATE_CONFIG" \
    --corpus "$FULL_CORPUS" \
    --timeout "$EVAL_TIMEOUT_SECONDS" \
    --output "$full_out" >/dev/null

  if [ "$(is_better "$full_out" "$CHAMPION_FULL")" != "1" ]; then
    smoke_score="$(python3 -c "import json; print(json.load(open('$smoke_out'))['summary']['total_score'])")"
    full_score="$(python3 -c "import json; print(json.load(open('$full_out'))['summary']['total_score'])")"
    champion_full_score="$(python3 -c "import json; print(json.load(open('$CHAMPION_FULL'))['summary']['total_score'])")"
    append_result "reject_full" "$iteration" "full_not_better" "$smoke_score" "$full_score"
    echo "iteration $iteration: full $full_score did not beat champion $champion_full_score"
    return 0
  fi

  # Promote: candidate becomes champion
  archive_champion "$iteration"
  cp "$CANDIDATE_CONFIG" "$CHAMPION_CONFIG"
  cp "$smoke_out" "$CHAMPION_SMOKE"
  cp "$full_out" "$CHAMPION_FULL"
  smoke_score="$(python3 -c "import json; print(json.load(open('$CHAMPION_SMOKE'))['summary']['total_score'])")"
  full_score="$(python3 -c "import json; print(json.load(open('$CHAMPION_FULL'))['summary']['total_score'])")"
  append_result "accept" "$iteration" "promoted_to_champion" "$smoke_score" "$full_score"
  echo "iteration $iteration: promoted candidate — smoke=$smoke_score full=$full_score"
}

echo "[$(timestamp)] starting myosu game-solving autoresearch loop"
ensure_baseline

if [ -f "$RESULTS_JSONL" ]; then
  iteration=$(( $(wc -l < "$RESULTS_JSONL") + 1 ))
else
  iteration=1
fi
while true; do
  echo "[$(timestamp)] iteration $iteration begin"
  run_iteration "$iteration"
  echo "[$(timestamp)] iteration $iteration end"
  iteration=$((iteration + 1))
  sleep "$SLEEP_SECONDS"
done
