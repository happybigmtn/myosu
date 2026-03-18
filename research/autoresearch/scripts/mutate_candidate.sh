#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: mutate_candidate.sh <workspace> <iteration>" >&2
  exit 2
fi

workspace="$1"
iteration="$2"

prompt='Read AUTORESEARCH.md and ITERATION_CONTEXT.md. Edit only candidate_config.json. Make one small, reversible change aimed at improving architecture ranking accuracy on the CIFAR-100 proxy benchmark. Stop after the edit and briefly summarize the change.'

cd "$workspace"

opencode run "$prompt"
