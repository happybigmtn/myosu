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
  "genesis/plans/002-spec-corpus-normalization.md"
  "genesis/plans/010-ci-proof-gates-expansion.md"
  "genesis/plans/020-second-game-subnet-execution-proof.md"
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
