#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

shopt -s nullglob
mapfile -t plan_files < <(
  find genesis/plans -maxdepth 1 -type f -name '0[0-2][0-9]-*.md' \
    ! -name '001-*' \
    | LC_ALL=C sort
)

if [[ "${#plan_files[@]}" -eq 0 ]]; then
  echo "no numbered genesis plans found in expected range"
  exit 1
fi

for plan in "${plan_files[@]}"; do
  if ! rg -q '^### Milestone ' "$plan"; then
    echo "missing milestone heading: $plan"
    exit 1
  fi

  if ! rg -q '^Proof commands?:' "$plan"; then
    echo "missing proof command: $plan"
    exit 1
  fi
done

echo "genesis plan quality ok"
