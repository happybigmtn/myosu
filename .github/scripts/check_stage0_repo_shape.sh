#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

required_files=(
  "Cargo.toml"
  "crates/myosu-games-poker/Cargo.toml"
  "crates/myosu-games-liars-dice/Cargo.toml"
  "crates/myosu-play/Cargo.toml"
  "crates/myosu-miner/Cargo.toml"
  "crates/myosu-validator/Cargo.toml"
  "crates/myosu-chain-client/Cargo.toml"
  "crates/myosu-chain/runtime/Cargo.toml"
  "crates/myosu-chain/node/Cargo.toml"
  "genesis/plans/001-master-plan.md"
  "genesis/plans/002-promotion-ledger.md"
  "genesis/plans/003-checkpoint-policy-promotion.md"
  "genesis/plans/004-nlhe-benchmark-unblock.md"
  "genesis/plans/005-nlhe-promotion.md"
  "genesis/plans/006-liars-dice-promotion.md"
  "genesis/plans/007-checkpoint-dedicated-promotion.md"
  "genesis/plans/008-security-debt-triage.md"
  "genesis/plans/009-cribbage-deepening.md"
  "genesis/plans/010-bitino-local-adapter.md"
  "genesis/plans/011-checkpoint-bitino-pilot.md"
  "specs/031626-00-master-index.md"
)

required_members=(
  'crates/myosu-games-poker'
  'crates/myosu-games-liars-dice'
  'crates/myosu-play'
  'crates/myosu-chain-client'
  'crates/myosu-miner'
  'crates/myosu-validator'
  'crates/myosu-chain/runtime'
  'crates/myosu-chain/node'
)

missing_files=()
for path in "${required_files[@]}"; do
  if [[ ! -f "$path" ]]; then
    missing_files+=("$path")
  fi
done

if [[ "${#missing_files[@]}" -gt 0 ]]; then
  echo "stage-0 repo shape mismatch: missing required files"
  printf '%s\n' "${missing_files[@]}"
  exit 1
fi

missing_members=()
for member in "${required_members[@]}"; do
  if ! grep -Fq "\"$member\"" Cargo.toml; then
    missing_members+=("$member")
  fi
done

if [[ "${#missing_members[@]}" -gt 0 ]]; then
  echo "stage-0 repo shape mismatch: missing workspace members in Cargo.toml"
  printf '%s\n' "${missing_members[@]}"
  exit 1
fi

echo "stage-0 repo shape ok"
