#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

index_file="specs/031626-00-master-index.md"

if [[ ! -f "$index_file" ]]; then
  echo "missing master index: $index_file"
  exit 1
fi

actual_file="$(mktemp)"
allowed_file="$(mktemp)"
trap 'rm -f "$actual_file" "$allowed_file"' EXIT

rg --files specs -g '031626-*.md' \
  | xargs -n1 basename \
  | LC_ALL=C sort -u \
  > "$actual_file"

{
  echo "031626-00-master-index.md"
  rg -o '031626-\d{2}[a-z]?-[a-z-]+\.md' "$index_file"
} | LC_ALL=C sort -u > "$allowed_file"

extra="$(comm -23 "$actual_file" "$allowed_file")"
missing="$(comm -13 "$actual_file" "$allowed_file")"

if [[ -n "$extra" ]]; then
  echo "unexpected active specs:"
  echo "$extra"
  exit 1
fi

if [[ -n "$missing" ]]; then
  echo "missing indexed specs:"
  echo "$missing"
  exit 1
fi

while IFS= read -r spec; do
  path="specs/$spec"
  if [[ ! -s "$path" ]]; then
    echo "empty canonical spec: $path"
    exit 1
  fi
done < "$actual_file"

echo "doctrine integrity ok"
