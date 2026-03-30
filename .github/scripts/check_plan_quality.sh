#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

shopt -s nullglob

if command -v rg >/dev/null 2>&1; then
  has_milestone() {
    rg -q '^### Milestone ' "$1"
  }

  has_proof_command() {
    rg -q '^Proof commands?:' "$1"
  }
else
  has_milestone() {
    grep -qE '^### Milestone ' "$1"
  }

  has_proof_command() {
    grep -qE '^Proof commands?:' "$1"
  }
fi

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
  if ! has_milestone "$plan"; then
    echo "missing milestone heading: $plan"
    exit 1
  fi

  if ! has_proof_command "$plan"; then
    echo "missing proof command: $plan"
    exit 1
  fi
done

echo "genesis plan quality ok"
