# CI/CD Pipeline Setup

**Plan ID:** 006
**Status:** In Progress
**Priority:** HIGH — no automated quality gates exist

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, every pull request to `trunk` will automatically run:
`cargo check`, `cargo test`, `cargo clippy`, and `cargo fmt --check`. Failed
PRs will block merge. This closes the gap where work can look "done" after
compilation even when the right tests never ran.

---

## Progress

- [x] Create `.github/workflows/ci.yml`
- [x] Add `cargo check` for active workspace members
- [x] Add `cargo test` for active workspace members
- [x] Add `cargo clippy` with strict warnings
- [x] Add crate-scoped formatting check for active workspace members
- [x] Add a runnable smoke proof for `myosu-play`
- [ ] Configure branch protection on `trunk`

---

## Surprises & Discoveries

- `cargo fmt --check` is not a truthful first gate for this repo because unrelated
  chain files already fail formatting. The initial CI pass has to scope formatting
  to the active crates rather than pretending the whole workspace is clean.
- `cargo clippy -p myosu-games -p myosu-tui -- -D warnings` was immediately useful.
  It surfaced a few low-noise cleanup issues in `screens.rs` and `shell.rs`, which
  were fixed before the workflow landed.
- The right first CI surface is not "all workspace members." The active crates are
  the only ones that currently support honest green verification.
- Once `myosu-games-poker` and `myosu-play` became real active crates, the
  original CI surface (`myosu-games` + `myosu-tui`) was no longer enough. The
  honest active-crate set now includes the poker/gameplay crates too.

---

## Decision Log

- Decision: CI runs on PR to `trunk`, not on every push.
  Rationale: The single-author workflow means frequent pushes. CI on PR only reduces noise while maintaining quality gates.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: CI should track the active user-facing crate set, not a frozen
  initial pair.
  Rationale: The first workflow started with `myosu-games` and `myosu-tui`, but
  once `myosu-games-poker` and `myosu-play` became live execution surfaces,
  leaving them out would recreate a false-green path.
  Date/Author: 2026-03-28 / Codex

- Decision: `myosu-play` needs a dedicated smoke command in CI.
  Rationale: A runnable binary should prove behavior, not just compilation or a
  broad unit-test pass. `--smoke-test` gives CI a deterministic Level 4 gate.
  Date/Author: 2026-03-28 / Codex

- Decision: `cargo clippy` runs with `-D warnings` to treat lint warnings as errors.
  Rationale: Current codebase has no lint discipline. Enforcing clippy prevents new warnings from accumulating.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: Format checks are file-scoped to `crates/myosu-games/**/*.rs` and
  `crates/myosu-tui/**/*.rs`, not workspace-wide.
  Rationale: Workspace-wide `cargo fmt --check` currently fails on unrelated
  chain formatting debt, which would make the first CI gate noisy and misleading.
  Date/Author: 2026-03-27 / Codex

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Create `.github/workflows/ci.yml`
A GitHub Actions workflow that triggers on PR to `trunk`.

Proof: `test -f .github/workflows/ci.yml`; workflow syntax is valid YAML.

Current local proof: `.github/workflows/ci.yml` exists and is checked into the repo.

### M2: Add `cargo check` and `cargo test` for active crates
The CI runs `cargo check` and `cargo test` across the current active crate set:
`myosu-games`, `myosu-tui`, `myosu-games-poker`, and `myosu-play`.

Proof: GitHub Actions run shows both check and test steps pass on a PR.

Current local proof:
- `cargo check -p myosu-games -p myosu-tui -p myosu-games-poker -p myosu-play`
- `cargo test -p myosu-games -p myosu-tui -p myosu-games-poker -p myosu-play --quiet`
- focused proof commands:
  `cargo test -p myosu-games --quiet serialization_roundtrip`
  `cargo test -p myosu-tui shell_state`
  `cargo test -p myosu-play smoke_report_proves_preflop_to_flop_progression --quiet`

### M3: Add `cargo clippy` with strict warnings
CI fails if any clippy warning exists (enforced via `-D warnings`).

Proof: A deliberately introduced clippy warning (e.g., `unused variable`) causes CI to fail.

Current local proof: `cargo clippy -p myosu-games -p myosu-tui -p myosu-games-poker -p myosu-play -- -D warnings`
passes after fixing the initial warning set in `screens.rs` and `shell.rs`.

### M4: Add formatting check for active crates
CI fails if the active crates are not formatted according to `rustfmt`.

Proof: `git ls-files 'crates/myosu-tui/**/*.rs' 'crates/myosu-games/**/*.rs' 'crates/myosu-games-poker/**/*.rs' 'crates/myosu-play/**/*.rs' | xargs rustfmt --edition 2024 --check`
fails on misformatted source and passes once those files are formatted.

### M5: Configure branch protection on `trunk`
Require CI to pass before merge.

Proof: GitHub branch protection settings show `trunk` requires passing CI checks.

---

## Context and Orientation

Current CI state: a repo-local workflow now exists, but no remote GitHub run has
been observed yet from this session.

The repo now has `.github/workflows/ci.yml`. No GitLab CI or Jenkins surfaces
are present in the current tree.

Key files to create:
- `.github/workflows/ci.yml` — main CI workflow
- `.github/CODEOWNERS` — code owners (optional)

Key files to modify:
- `.github/workflows/ci.yml` — workflow entrypoint
- `crates/myosu-tui/src/screens.rs` — clippy cleanup for strict linting
- `crates/myosu-tui/src/shell.rs` — clippy cleanup for strict linting
- `crates/myosu-play/src/main.rs` — deterministic runnable smoke path

---

## Plan of Work

1. Create `.github/workflows/ci.yml` with an active-crate verification job
2. Run `cargo check`, focused proof tests, full active-crate tests, clippy, rustfmt, and binary smoke behavior
3. Add clippy with `-D warnings`
4. Add a truthful formatting check scoped to active crates
5. Configure branch protection on GitHub
6. Verify by opening a test PR

---

## Concrete Steps

```bash
# Local validation commands
test -f .github/workflows/ci.yml
cargo check -p myosu-games -p myosu-tui -p myosu-games-poker -p myosu-play
cargo test -p myosu-games --quiet serialization_roundtrip
cargo test -p myosu-tui shell_state
cargo test -p myosu-play smoke_report_proves_preflop_to_flop_progression --quiet
cargo run -p myosu-play --quiet -- --smoke-test
cargo test -p myosu-games -p myosu-tui -p myosu-games-poker -p myosu-play --quiet
cargo clippy -p myosu-games -p myosu-tui -p myosu-games-poker -p myosu-play -- -D warnings
git ls-files 'crates/myosu-tui/**/*.rs' 'crates/myosu-games/**/*.rs' 'crates/myosu-games-poker/**/*.rs' 'crates/myosu-play/**/*.rs' \
  | xargs rustfmt --edition 2024 --check
```

---

## Validation

- `test -f .github/workflows/ci.yml`
- `cargo check -p myosu-games -p myosu-tui -p myosu-games-poker -p myosu-play` passes
- `cargo test -p myosu-games --quiet serialization_roundtrip` passes
- `cargo test -p myosu-tui shell_state` passes
- `cargo test -p myosu-play smoke_report_proves_preflop_to_flop_progression --quiet` passes
- `cargo run -p myosu-play --quiet -- --smoke-test` prints `SMOKE myosu-play ok`
- `cargo test -p myosu-games -p myosu-tui -p myosu-games-poker -p myosu-play --quiet` passes
- `cargo clippy -p myosu-games -p myosu-tui -p myosu-games-poker -p myosu-play -- -D warnings` passes (may require fixing existing warnings first)
- `git ls-files 'crates/myosu-tui/**/*.rs' 'crates/myosu-games/**/*.rs' 'crates/myosu-games-poker/**/*.rs' 'crates/myosu-play/**/*.rs' | xargs rustfmt --edition 2024 --check` passes
- GitHub Actions run on a test PR shows all steps passing
