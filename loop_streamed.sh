#!/usr/bin/env bash
set -euo pipefail

# Ralph-style streamed loop for myosu using Codex.
# Usage:
#   ./loop_streamed.sh
#   ./loop_streamed.sh 5
#   MODEL=gpt-5.4 REASONING_EFFORT=xhigh ./loop_streamed.sh

if [[ "${1:-}" =~ ^[0-9]+$ ]]; then
    MAX_ITERATIONS="$1"
else
    MAX_ITERATIONS="${MAX_ITERATIONS:-0}"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROMPT_FILE="${PROMPT_FILE:-$SCRIPT_DIR/PROMPT_build.md}"
MODEL="${MODEL:-gpt-5.4}"
REASONING_EFFORT="${REASONING_EFFORT:-xhigh}"
CURRENT_BRANCH="$(git -C "$SCRIPT_DIR" branch --show-current)"
ITERATION=0
RUN_ROOT="${RUN_ROOT:-$SCRIPT_DIR/.ralph-loop}"
mkdir -p "$RUN_ROOT"
ERROR_LOG="$RUN_ROOT/codex.stderr.log"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Mode:   build"
echo "Prompt: $(basename "$PROMPT_FILE")"
echo "Model:  $MODEL"
echo "Think:  $REASONING_EFFORT"
echo "Branch: $CURRENT_BRANCH"
[[ "$MAX_ITERATIONS" -gt 0 ]] && echo "Max:    $MAX_ITERATIONS iterations"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [[ "$CURRENT_BRANCH" != "trunk" ]]; then
    echo "Error: loop_streamed.sh must run on branch 'trunk' (current: $CURRENT_BRANCH)" >&2
    exit 1
fi

if [[ ! -f "$PROMPT_FILE" ]]; then
    echo "Error: $PROMPT_FILE not found" >&2
    exit 1
fi

while true; do
    if [[ "$MAX_ITERATIONS" -gt 0 ]] && [[ "$ITERATION" -ge "$MAX_ITERATIONS" ]]; then
        echo "Reached max iterations: $MAX_ITERATIONS"
        break
    fi

    FULL_PROMPT="$(cat "$PROMPT_FILE")

Execute the instructions above."

    COMMIT_BEFORE="$(git -C "$SCRIPT_DIR" rev-parse HEAD)"

    echo "⏳ Running Codex..."
    echo ""

    set +e
    printf "%s" "$FULL_PROMPT" | codex exec \
        --json \
        --dangerously-bypass-approvals-and-sandbox \
        --skip-git-repo-check \
        --cd "$SCRIPT_DIR" \
        -m "$MODEL" \
        -c "model_reasoning_effort=\"$REASONING_EFFORT\"" \
        2>>"$ERROR_LOG" \
        | node "$SCRIPT_DIR/parse_stream.js"
    CODEX_EXIT="${PIPESTATUS[0]}"
    set -e

    if [[ "$CODEX_EXIT" -ne 0 ]]; then
        echo "⚠️  Codex exited with code $CODEX_EXIT — check $ERROR_LOG"
        break
    fi

    echo ""
    echo "✅ Codex iteration complete"

    COMMIT_AFTER="$(git -C "$SCRIPT_DIR" rev-parse HEAD)"
    if [[ "$COMMIT_BEFORE" == "$COMMIT_AFTER" ]]; then
        echo "⚠️  No commit produced — all tasks may be complete. Stopping."
        break
    fi

    git -C "$SCRIPT_DIR" push origin trunk

    ITERATION=$((ITERATION + 1))
    echo -e "\n\n======================== LOOP $ITERATION ========================\n"
done
