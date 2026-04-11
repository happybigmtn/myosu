#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

seed="${MYOSU_PLAYTRACE_SEED:-42}"
max_steps="${MYOSU_PLAYTRACE_MAX_STEPS:-8}"

manifest="$(
  SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical --example canonical_manifest
)"
mapfile -t games < <(
  printf '%s\n' "$manifest" \
    | awk '/^CANONICAL_GAME / {
        for (i = 1; i <= NF; i++) {
          if ($i ~ /^slug=/) {
            sub(/^slug=/, "", $i);
            print $i;
          }
        }
      }'
)

if [[ "${#games[@]}" -ne 10 ]]; then
  printf 'expected 10 canonical manifest games, found %s\n' "${#games[@]}" >&2
  exit 1
fi

run_playtrace() {
  SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical \
    --example play_against_strategy -- \
    --all-canonical-ten \
    --seed "$seed" \
    --max-steps "$max_steps"
}

output="$(run_playtrace)"
repeat_output="$(run_playtrace)"

if [[ "$output" != "$repeat_output" ]]; then
  printf 'playtrace output was not deterministic for seed %s\n' "$seed" >&2
  diff -u <(printf '%s\n' "$output") <(printf '%s\n' "$repeat_output") >&2 || true
  exit 1
fi

line_count="$(printf '%s\n' "$output" | grep -c '^PLAYTRACE ')"
if [[ "$line_count" -ne 10 ]]; then
  printf 'expected 10 PLAYTRACE lines, found %s\n' "$line_count" >&2
  printf '%s\n' "$output" >&2
  exit 1
fi

for game in "${games[@]}"; do
  game_count="$(printf '%s\n' "$output" | grep -c "^PLAYTRACE game=${game} ")"
  if [[ "$game_count" -ne 1 ]]; then
    printf 'expected one PLAYTRACE line for %s, found %s\n' "$game" "$game_count" >&2
    printf '%s\n' "$output" >&2
    exit 1
  fi
done

if printf '%s\n' "$output" | grep -qE 'illegal_action|trace_hash_mismatch|panic'; then
  printf 'playtrace output contained a forbidden failure marker\n' >&2
  printf '%s\n' "$output" >&2
  exit 1
fi

while IFS= read -r line; do
  [[ -n "$line" ]] || continue
  if [[ ! "$line" =~ strategy_source=(best-local|best-local\+legal-continuation) ]]; then
    printf 'unexpected strategy source in line: %s\n' "$line" >&2
    exit 1
  fi
  if [[ "$line" =~ status=terminal ]] && [[ "$line" =~ payoff=none ]]; then
    printf 'terminal trace is missing payoff: %s\n' "$line" >&2
    exit 1
  fi
  if [[ "$line" =~ status=bounded ]] && [[ ! "$line" =~ terminal=false ]]; then
    printf 'bounded trace must report terminal=false: %s\n' "$line" >&2
    exit 1
  fi
done < <(printf '%s\n' "$output")

printf '%s\n' "$output"
printf 'CANONICAL_TEN_PLAY_HARNESS myosu e2e ok seed=%s max_steps=%s games=%s\n' \
  "$seed" \
  "$max_steps" \
  "${#games[@]}"
