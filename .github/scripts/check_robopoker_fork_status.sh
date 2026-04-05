#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

changelog_path="docs/robopoker-fork-changelog.md"
upstream_url="${MYOSU_ROBOPOKER_UPSTREAM_URL:-https://github.com/krukah/robopoker}"
fork_url="${MYOSU_ROBOPOKER_FORK_URL:-https://github.com/happybigmtn/robopoker}"

if [[ ! -f "$changelog_path" ]]; then
  echo "missing robopoker changelog: $changelog_path"
  exit 1
fi

mapfile -t pinned_revs < <(
  git grep -n 'happybigmtn/robopoker' -- 'crates/*/Cargo.toml' \
    | sed -n 's/.*rev = "\([0-9a-f]\{40\}\)".*/\1/p' \
    | LC_ALL=C sort -u
)

if [[ "${#pinned_revs[@]}" -eq 0 ]]; then
  echo "no robopoker git pin found in crate Cargo.toml files"
  exit 1
fi

if [[ "${#pinned_revs[@]}" -ne 1 ]]; then
  echo "multiple robopoker pins found in workspace:"
  printf '%s\n' "${pinned_revs[@]}"
  exit 1
fi

pinned_rev="${pinned_revs[0]}"
changelog_pinned_rev="$(sed -n 's/^- pinned fork rev: `\([0-9a-f]\{40\}\)`$/\1/p' "$changelog_path" | head -n1)"
baseline_tag="$(sed -n 's/^- upstream baseline: `\([^`]\+\)`$/\1/p' "$changelog_path" | head -n1)"

if [[ -z "$changelog_pinned_rev" ]]; then
  echo "could not read pinned fork rev from $changelog_path"
  exit 1
fi

if [[ -z "$baseline_tag" ]]; then
  echo "could not read upstream baseline tag from $changelog_path"
  exit 1
fi

changelog_status="ok"
if [[ "$changelog_pinned_rev" != "$pinned_rev" ]]; then
  changelog_status="mismatch"
  echo "::warning title=Robopoker fork coherence::Workspace pin ${pinned_rev:0:12} does not match changelog pin ${changelog_pinned_rev:0:12}."
fi

upstream_head_ref="$(
  git ls-remote --symref "$upstream_url" HEAD \
    | awk '/^ref:/ {sub("refs/heads/", "", $2); print $2; exit}'
)"

if [[ -z "$upstream_head_ref" ]]; then
  echo "could not resolve upstream default branch for $upstream_url"
  exit 1
fi

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

git -C "$tmpdir" init -q
git -C "$tmpdir" remote add upstream "$upstream_url"
git -C "$tmpdir" remote add fork "$fork_url"

git -C "$tmpdir" fetch --quiet upstream \
  "refs/tags/$baseline_tag:refs/tags/$baseline_tag" \
  "refs/heads/$upstream_head_ref:refs/remotes/upstream/$upstream_head_ref"
git -C "$tmpdir" fetch --quiet fork '+refs/heads/*:refs/remotes/fork/*'

if ! git -C "$tmpdir" cat-file -e "${pinned_rev}^{commit}" 2>/dev/null; then
  echo "pinned robopoker rev $pinned_rev is not reachable from $fork_url"
  exit 1
fi

upstream_head_sha="$(git -C "$tmpdir" rev-parse --short=12 "refs/remotes/upstream/$upstream_head_ref")"
fork_ahead_baseline="$(git -C "$tmpdir" rev-list --count "$baseline_tag..$pinned_rev")"
upstream_ahead_baseline="$(git -C "$tmpdir" rev-list --count "$baseline_tag..refs/remotes/upstream/$upstream_head_ref")"
upstream_not_in_pinned="$(git -C "$tmpdir" rev-list --count "$pinned_rev..refs/remotes/upstream/$upstream_head_ref")"
pinned_not_in_upstream="$(git -C "$tmpdir" rev-list --count "refs/remotes/upstream/$upstream_head_ref..$pinned_rev")"

echo "robopoker fork coherence"
echo "  baseline tag: $baseline_tag"
echo "  pinned fork rev: $pinned_rev"
echo "  changelog status: $changelog_status"
echo "  upstream default branch: $upstream_head_ref"
echo "  upstream head: $upstream_head_sha"
echo "  fork commits ahead of baseline: $fork_ahead_baseline"
echo "  upstream commits ahead of baseline: $upstream_ahead_baseline"
echo "  upstream commits not in pinned rev: $upstream_not_in_pinned"
echo "  pinned-only commits not in upstream head: $pinned_not_in_upstream"

if [[ "$upstream_not_in_pinned" -gt 0 ]]; then
  echo "::warning title=Robopoker fork coherence::Upstream $upstream_head_ref is $upstream_not_in_pinned commit(s) ahead of pinned rev ${pinned_rev:0:12}."
else
  echo "::notice title=Robopoker fork coherence::Pinned rev ${pinned_rev:0:12} is caught up with upstream $upstream_head_ref for the compared history."
fi

if [[ "$pinned_not_in_upstream" -gt 0 ]]; then
  echo "::notice title=Robopoker fork coherence::Pinned rev carries $pinned_not_in_upstream downstream-only commit(s) beyond upstream $upstream_head_ref."
fi

if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  {
    echo "## Robopoker Fork Coherence"
    echo
    echo "| Metric | Value |"
    echo "|---|---:|"
    echo "| Baseline tag | \`$baseline_tag\` |"
    echo "| Pinned fork rev | \`${pinned_rev:0:12}\` |"
    echo "| Changelog status | \`$changelog_status\` |"
    echo "| Upstream branch | \`$upstream_head_ref\` |"
    echo "| Upstream head | \`$upstream_head_sha\` |"
    echo "| Fork commits ahead of baseline | $fork_ahead_baseline |"
    echo "| Upstream commits ahead of baseline | $upstream_ahead_baseline |"
    echo "| Upstream commits not in pinned rev | $upstream_not_in_pinned |"
    echo "| Pinned-only commits not in upstream head | $pinned_not_in_upstream |"
  } >> "$GITHUB_STEP_SUMMARY"
fi
