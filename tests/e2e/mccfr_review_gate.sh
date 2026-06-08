#!/usr/bin/env bash
# MCCFR review gate proof harness (NEM-007 / ADR 015).
#
# This is the executable end-to-end gate for the MCCFR review
# gate documented in `docs/adr/015-mccfr-review-gate.md`. The
# harness enforces the seven assertions listed in the ADR's
# `## Validation / Evidence` section:
#
#   (a) `docs/adr/015-mccfr-review-gate.md` exists and is the
#       ADR that documents this decision.
#   (b) `INVARIANTS.md` INV-006 section ends with a
#       cross-reference to this ADR.
#   (c) `docs/robopoker-fork-changelog.md` has a
#       `Review Checklist Pointer` link to this ADR.
#   (d) All four MCCFR-relevant change criteria phrases
#       appear in the ADR (greppable strings: `Regret update
#       formula`, `Averaging / sampling formula`, `Sampling
#       method`, `Public MCCFR-API signature change`).
#   (e) The review checklist template is present in the ADR
#       with all ten numbered items.
#   (f) The `mccfr-review-gate` CI job is wired in
#       `.github/workflows/ci.yml`.
#   (g) The NEM-007 row is `[x]` in both
#       `IMPLEMENTATION_PLAN.md` and
#       `nemesis/IMPLEMENTATION_PLAN.md` with a
#       cross-reference to this ADR.
#
# The harness is pure bash + grep, no extra tooling. It exits
# non-zero with a concrete error message naming the failing
# assertion if any one of the seven checks fails. The harness
# is wired into a new `mccfr-review-gate` CI job in
# `.github/workflows/ci.yml` so every PR and push to trunk /
# main runs the full set of seven checks.
#
# The harness is a string-presence gate: it asserts the
# documented policy cannot silently drift away from the
# implementation. It does not statically analyze the robopoker
# fork; that is out of scope for NEM-007 (see ADR 015
# `## Alternatives Considered` Option B for the cross-crate
# grep follow-on).

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

adr_path="$repo_root/docs/adr/015-mccfr-review-gate.md"
invariants_path="$repo_root/INVARIANTS.md"
changelog_path="$repo_root/docs/robopoker-fork-changelog.md"
workflow_path="$repo_root/.github/workflows/ci.yml"
main_plan_path="$repo_root/IMPLEMENTATION_PLAN.md"
nemesis_plan_path="$repo_root/nemesis/IMPLEMENTATION_PLAN.md"

fail=0
fail_msg() {
    printf 'mccfr_review_gate: %s\n' "$1" >&2
    fail=1
}

# (a) ADR exists.
if [[ ! -f "$adr_path" ]]; then
    fail_msg "missing ADR at $adr_path"
else
    if ! head -1 "$adr_path" | grep -qE '^# ADR 015: MCCFR review gate'; then
        fail_msg "ADR at $adr_path is not the MCCFR review gate ADR (header mismatch)"
    fi
fi

# (b) INVARIANTS.md INV-006 section references this ADR.
if [[ ! -f "$invariants_path" ]]; then
    fail_msg "missing INVARIANTS.md at $invariants_path"
else
    # INV-006 is the last invariant; everything from "## INV-006:"
    # to the end of the file is the INV-006 section. The cross-reference
    # must appear inside that section. We extract the section body to a
    # temp file first (avoids the SIGPIPE / pipefail interaction that
    # `awk | grep -q` has when the match is found early).
    inv006_body="$(mktemp)"
    trap 'rm -f "$inv006_body"' EXIT
    awk '/^## INV-006: Robopoker Fork Coherence/{flag=1; next} /^## /{flag=0} flag' "$invariants_path" > "$inv006_body" || true
    if ! grep -q 'docs/adr/015-mccfr-review-gate.md' "$inv006_body"; then
        fail_msg "INVARIANTS.md INV-006 section does not cross-reference docs/adr/015-mccfr-review-gate.md"
    fi
    rm -f "$inv006_body"
    trap - EXIT
fi

# (c) robopoker-fork-changelog.md has a "Review Checklist Pointer" link.
if [[ ! -f "$changelog_path" ]]; then
    fail_msg "missing fork changelog at $changelog_path"
else
    if ! grep -qE '^## Review Checklist Pointer$' "$changelog_path"; then
        fail_msg "fork changelog at $changelog_path is missing the '## Review Checklist Pointer' section header"
    fi
    changelog_pointer_body="$(mktemp)"
    trap 'rm -f "$changelog_pointer_body"' EXIT
    awk '/^## Review Checklist Pointer/{flag=1; next} /^## /{flag=0} flag' "$changelog_path" > "$changelog_pointer_body" || true
    if ! grep -q 'docs/adr/015-mccfr-review-gate.md' "$changelog_pointer_body"; then
        fail_msg "fork changelog 'Review Checklist Pointer' section does not link to docs/adr/015-mccfr-review-gate.md"
    fi
    rm -f "$changelog_pointer_body"
    trap - EXIT
fi

# (d) All four MCCFR-relevant change criteria appear in the ADR.
if [[ -f "$adr_path" ]]; then
    for criterion in \
        'Regret update formula' \
        'Averaging / sampling formula' \
        'Sampling method' \
        'Public MCCFR-API signature change'; do
        if ! grep -qF "$criterion" "$adr_path"; then
            fail_msg "ADR at $adr_path is missing the MCCFR-relevant change criterion: '$criterion'"
        fi
    done
fi

# (e) The review checklist template is present in the ADR with all ten
# numbered items. The template uses the literal '[ ] N.' checkbox pattern,
# where N is 1..10, and the ten items are anchored by the unique
# phrases in the ADR text.
if [[ -f "$adr_path" ]]; then
    checklist_phrases=(
        'Identify which of the four MCCFR-relevant criteria'
        'Cite the upstream `v1.0.0` baseline'
        'Summarize the algorithmic change in one paragraph'
        'Cite the proof surface that demonstrates the change'
        'If the commit changes a public MCCFR-API signature'
        'numerical change to a regret'
        'Update `docs/robopoker-fork-changelog.md`'
        'Confirm the workspace pin in'
        'security-driven pin change'
        'Sign off in the PR description'
    )
    for phrase in "${checklist_phrases[@]}"; do
        if ! grep -qF "$phrase" "$adr_path"; then
            fail_msg "ADR review checklist is missing the item: '$phrase'"
        fi
    done
    # All ten numbered items must be present in the checklist code block.
    checklist_block_count="$(awk '/^```text$/{flag=1; next} /^```$/ && flag{flag=0} flag' "$adr_path" | grep -cE '^\[ \] [0-9]+\.')"
    if [[ "$checklist_block_count" -ne 10 ]]; then
        fail_msg "ADR review checklist code block must contain exactly 10 numbered '[ ] N.' items; found $checklist_block_count"
    fi
fi

# (f) The `mccfr-review-gate` CI job is wired in `.github/workflows/ci.yml`.
if [[ ! -f "$workflow_path" ]]; then
    fail_msg "missing workflow at $workflow_path"
else
    if ! grep -qE '^  mccfr-review-gate:' "$workflow_path"; then
        fail_msg "ci.yml is missing the 'mccfr-review-gate' top-level job"
    fi
    workflow_job_body="$(mktemp)"
    trap 'rm -f "$workflow_job_body"' EXIT
    awk '/^  mccfr-review-gate:/{flag=1; next} /^  [a-zA-Z][a-zA-Z0-9_-]*:/{flag=0} flag' "$workflow_path" > "$workflow_job_body" || true
    if ! grep -q 'tests/e2e/mccfr_review_gate.sh' "$workflow_job_body"; then
        fail_msg "ci.yml 'mccfr-review-gate' job does not run 'tests/e2e/mccfr_review_gate.sh'"
    fi
    rm -f "$workflow_job_body"
    trap - EXIT
fi

# (g) The NEM-007 row is `[x]` in both IMPLEMENTATION_PLAN.md and
# nemesis/IMPLEMENTATION_PLAN.md with a cross-reference to this ADR.
if [[ ! -f "$main_plan_path" ]]; then
    fail_msg "missing main plan at $main_plan_path"
else
    if ! grep -qE '^- \[x\] `NEM-007`' "$main_plan_path"; then
        fail_msg "main IMPLEMENTATION_PLAN.md does not have the NEM-007 row marked [x]"
    fi
    # The NEM-007 row body must cross-reference the ADR. We use awk to
    # extract the row body (from the row header to the next top-level
    # `- ` row) and check the body in one grep call (avoiding the
    # SIGPIPE / pipefail interaction that `awk | grep -q` has when
    # the match is found early).
    nem007_body="$(mktemp)"
    trap 'rm -f "$nem007_body"' EXIT
    awk '/^- \[x\] `NEM-007`/{flag=1; next} /^- /{flag=0} flag' "$main_plan_path" > "$nem007_body" || true
    if ! grep -q 'docs/adr/015-mccfr-review-gate.md' "$nem007_body"; then
        fail_msg "main IMPLEMENTATION_PLAN.md NEM-007 row does not cross-reference docs/adr/015-mccfr-review-gate.md"
    fi
    rm -f "$nem007_body"
    trap - EXIT
fi

if [[ ! -f "$nemesis_plan_path" ]]; then
    fail_msg "missing nemesis plan at $nemesis_plan_path"
else
    if ! grep -qE '^### `- \[x\] NEM-007' "$nemesis_plan_path"; then
        fail_msg "nemesis IMPLEMENTATION_PLAN.md does not have the NEM-007 row marked [x]"
    fi
fi

if [[ "$fail" -ne 0 ]]; then
    exit 1
fi

printf 'mccfr_review_gate: ok (ADR exists; INV-006 cross-references ADR; fork changelog has Review Checklist Pointer; 4 MCCFR-relevant change criteria present; review checklist has 10 numbered items; mccfr-review-gate CI job wired; NEM-007 row [x] in both plans)\n'
