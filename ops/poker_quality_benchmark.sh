#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF' >&2
usage: bash ops/poker_quality_benchmark.sh \
  --db-url <postgres-url> \
  --robopoker-dir <path> \
  --encoder-dir <path> \
  [--iterations "0 128 256 512"]
EOF
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
db_url=""
robopoker_dir=""
encoder_dir=""
iterations="0 128 256 512"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --db-url)
      db_url="${2:-}"
      shift 2
      ;;
    --robopoker-dir)
      robopoker_dir="${2:-}"
      shift 2
      ;;
    --encoder-dir)
      encoder_dir="${2:-}"
      shift 2
      ;;
    --iterations)
      iterations="${2:-}"
      shift 2
      ;;
    *)
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$db_url" || -z "$robopoker_dir" || -z "$encoder_dir" ]]; then
  usage
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required" >&2
  exit 1
fi
if ! command -v psql >/dev/null 2>&1; then
  echo "psql is required" >&2
  exit 1
fi
if [[ ! -d "$robopoker_dir" ]]; then
  echo "robopoker dir does not exist: $robopoker_dir" >&2
  exit 1
fi

mkdir -p "$encoder_dir"

table_state="$(
  psql "$db_url" -tAqc "
    SELECT CASE
      WHEN to_regclass('public.isomorphism') IS NULL THEN 'missing'
      WHEN EXISTS (SELECT 1 FROM isomorphism LIMIT 1) THEN 'ready'
      ELSE 'empty'
    END;
  "
)"

if [[ "$table_state" != "ready" ]]; then
  echo "clustering full robopoker abstraction into postgres..."
  (
    cd "$robopoker_dir"
    DB_URL="$db_url" cargo run --quiet -p trainer -- --cluster
  )
else
  echo "reusing existing robopoker isomorphism table at $db_url"
fi

echo "exporting postgres lookup to myosu encoder dir..."
psql "$db_url" -At -F $'\t' -c 'SELECT obs, abs FROM isomorphism ORDER BY obs' \
  | (
    cd "$repo_root"
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker \
      --example import_robopoker_lookup -- - "$encoder_dir"
  )

echo "running poker exploitability benchmark..."
read -r -a iteration_args <<<"$iterations"
(
  cd "$repo_root"
  env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker \
    --example quality_benchmark -- "$encoder_dir" "${iteration_args[@]}"
)
