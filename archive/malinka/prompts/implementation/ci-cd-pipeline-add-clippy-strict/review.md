# Add cargo clippy with strict warnings Lane — Review

Review only the current slice for `ci-cd-pipeline-add-clippy-strict`.

Current Slice Contract:
Plan file:
- `genesis/plans/006-ci-cd-pipeline.md`

Child work item: `ci-cd-pipeline-add-clippy-strict`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# CI/CD Pipeline Setup

**Plan ID:** 006
**Status:** New
**Priority:** HIGH — no automated quality gates exist

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, every pull request to `trunk` will automatically run: `cargo check`, `cargo test`, `cargo clippy`, and `cargo fmt --check`. Failed PRs will block merge. This closes the gap where Fabro lanes can complete with passing proof commands that don't actually run the right tests.

---

## Progress

- [ ] Create `.github/workflows/ci.yml`
- [ ] Add `cargo check` for all workspace members
- [ ] Add `cargo test` for active workspace members (`myosu-games`, `myosu-tui`)
- [ ] Add `cargo clippy` with strict warnings
- [ ] Add `cargo fmt --check`
- [ ] Configure branch protection on `trunk`

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: CI runs on PR to `trunk`, not on every push.
  Rationale: The single-author workflow means frequent pushes. CI on PR only reduces noise while maintaining quality gates.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: CI tests only `myosu-games` and `myosu-tui` initially — not the chain pallets.
  Rationale: Chain pallets require Substrate's full toolchain (heavy), and they're already well-tested. The user-facing crates need CI coverage most urgently.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: `cargo clippy` runs with `-D warnings` to treat lint warnings as errors.
  Rationale: Current codebase has no lint discipline. Enforcing clippy prevents new warnings from accumulating.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Create `.github/workflows/ci.yml`
A GitHub Actions workflow that triggers on PR to `trunk`.

Proof: `test -f .github/workflows/ci.yml`; workflow syntax is valid YAML.

### M2: Add `cargo check` and `cargo test` for active crates
The CI runs `cargo check -p myosu-games -p myosu-tui` and `cargo test -p myosu-games -p myosu-tui` on every PR.

Proof: GitHub Actions run shows both check and test steps pass on a PR.

### M3: Add `cargo clippy` with strict warnings
CI fails if any clippy warning exists (enforced via `-D warnings`).

Proof: A deliberately introduced clippy warning (e.g., `unused variable`) causes CI to fail.

### M4: Add `cargo fmt --check`
CI fails if code is not formatted according to `rustfmt.toml`.

Proof: `cargo fmt` without `--check` followed by PR shows diff; `cargo fmt --check` passes only after `cargo fmt`.

### M5: Configure branch protection on `trunk`
Require CI to pass before merge.

Proof: GitHub branch protection settings show `trunk` requires passing CI checks.

---

## Context and Orientation

Current CI state: **NONE**.

The `.github/` directory does not exist. No GitHub Actions, no GitLab CI, no Jenkins.

Key files to create:
- `.github/workflows/ci.yml` — main CI workflow
- `.github/CODEOWNERS` — code owners (optional)

Key files to modify:
- `Cargo.toml` — consider adding a `[profile.ci]` profile for faster builds
- (no other files needed — the workspace is already testable)

---

## Plan of Work

1. Create `.github/workflows/ci.yml` with the standard Rust CI template
2. Configure matrix for `myosu-games` and `myosu-tui` crates
3. Add clippy with `-D warnings`
4. Add fmt check
5. Configure branch protection on GitHub
6. Verify by opening a test PR

---

## Concrete Steps

```bash
# Verify no .github directory exists
ls -la .github 2>/dev/null || echo "No .github directory"

# Create directory structure
mkdir -p .github/workflows

# Create the CI workflow
cat > .github/workflows/ci.yml << 'EOF'
name: CI
on:
  pull_request:
    branches: [trunk, main]
  push:
    branches: [trunk, main]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - name: cargo check
        run: cargo check -p myosu-games -p myosu-tui
      - name: cargo test
        run: cargo test -p myosu-games -p myosu-tui
      - name: cargo clippy
        run: cargo clippy -p myosu-games -p myosu-tui -- -D warnings
      - name: cargo fmt
        run: cargo fmt --check
EOF

# Test locally (if act is installed)
# act -P ubuntu-latest=catthehacker/ubuntu:rust-latest
```

---

## Validation

- `test -f .github/workflows/ci.yml`
- `cargo fmt --check` passes (no formatting diff)
- `cargo clippy -p myosu-games -p myosu-tui -- -D warnings` passes (may require fixing existing warnings first)
- `cargo test -p myosu-games -p myosu-tui` passes
- GitHub Actions run on a test PR shows all steps passing


Workflow archetype: implement

Review profile: standard

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: CI workflow step for cargo clippy with -D warnings
- How: Add CI step that fails on any clippy warning via -D warnings flag
- Required tests: cargo clippy -p myosu-games -p myosu-tui -- -D warnings
- Verification plan: Clippy passes with zero warnings; introducing an unused variable causes CI failure
- Rollback condition: Clippy warnings are present in active crates or -D warnings flag is removed

Proof commands:
- `cargo clippy -p myosu-games -p myosu-tui -- -D warnings`

Artifacts to write:
- `spec.md`
- `review.md`


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Nemesis-style security review
- Pass 1 — first-principles challenge: question trust boundaries, authority assumptions, and who can trigger the slice's dangerous actions
- Pass 2 — coupled-state review: identify paired state or protocol surfaces and check that every mutation path keeps them consistent or explains the asymmetry
- check secret handling, capability scoping, pairing/idempotence behavior, and privilege escalation paths

Focus on:
- slice scope discipline
- proof-gate coverage for the active slice
- touched-surface containment
- implementation and verification artifact quality
- remaining blockers before the next slice

Deterministic evidence:
- treat `quality.md` as machine-generated truth about placeholder debt, warning debt, manual follow-up, and artifact mismatch risk
- if `quality.md` says `quality_ready: no`, do not bless the slice as merge-ready


Write `promotion.md` in this exact machine-readable form:

merge_ready: yes|no
manual_proof_pending: yes|no
reason: <one sentence>
next_action: <one sentence>

Only set `merge_ready: yes` when:
- `quality.md` says `quality_ready: yes`
- automated proof is sufficient for this slice
- any required manual proof has actually been performed
- no unresolved warnings or stale failures undermine confidence
- the implementation and verification artifacts match the real code.

Review stage ownership:
- you may write or replace `promotion.md` in this stage
- read `quality.md` before deciding `merge_ready`
- when the slice is security-sensitive, perform a Nemesis-style pass: first-principles assumption challenge plus coupled-state consistency review
- include security findings in the review verdict when the slice touches trust boundaries, keys, funds, auth, control-plane behavior, or external process control
- prefer not to modify source code here unless a tiny correction is required to make the review judgment truthful
