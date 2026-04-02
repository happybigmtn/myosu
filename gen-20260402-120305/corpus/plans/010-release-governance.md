# 010 - Release Governance and Versioning

## Purpose / Big Picture

No release process, no version numbers, no changelog. Before stage-1 (public
devnet), there must be a way to communicate changes to operators. This plan
establishes release governance.

## Context and Orientation

Current state:
- No git tags, releases, or changelog
- No version numbers in Cargo.toml beyond defaults
- No process for coordinating breaking changes
- Fabro branches are de facto "runs" but not releases

## Architecture

```
Release process:
    1. Feature branch -> trunk (via PR)
    2. Tag trunk as vX.Y.Z
    3. Generate changelog from commits
    4. Build operator bundle
    5. Publish release notes with operator action items
```

## Progress

### Milestone 1: Version numbering scheme

- [ ] M1. Apply semver versioning to workspace crates
  - Surfaces: `Cargo.toml`, all crate `Cargo.toml` files
  - What exists after: All crates have explicit version (e.g. `0.1.0`).
  - Why now: Cannot release without version numbers.
Proof command: `rg '^version' crates/*/Cargo.toml crates/myosu-chain/*/Cargo.toml`
  - Tests: All crates have explicit version

### Milestone 2: Changelog infrastructure

- [ ] M2. Add CHANGELOG.md with initial entry
  - Surfaces: `CHANGELOG.md` (new)
  - What exists after: CHANGELOG.md with process for maintaining it.
  - Why now: Operators need to know what changed.
Proof command: `test -s CHANGELOG.md`
  - Tests: Has at least one entry

### Milestone 3: Release script

- [ ] M3. Script that tags, builds bundle, and generates release notes
  - Surfaces: `ops/release.sh` (new)
  - What exists after: `bash ops/release.sh v0.1.0` creates tag, bundle, notes.
  - Why now: Manual releases are error-prone.
Proof command: `bash ops/release.sh --dry-run v0.1.0`
  - Tests: Dry-run completes

### Milestone 4: Breaking change communication

- [ ] M4. Document how to communicate breaking changes
  - Surfaces: `docs/operator-guide/upgrading.md` (new)
  - What exists after: Process for migration guides and upgrade windows.
  - Why now: Stage-1 will have breaking changes.
Proof command: `test -s docs/operator-guide/upgrading.md`
  - Tests: Document covers migration path

## Surprises & Discoveries

- Workspace Cargo.toml uses `workspace.package` but does not set workspace-level
  version. Crates may have default versions.

## Decision Log

- Decision: Semver (not calver).
  - Why: Communicates breaking changes in version number.
  - Failure mode: Harder pre-1.0.
  - Mitigation: 0.x.y where minor = breaking, patch = compatible.
  - Reversible: yes

## Validation and Acceptance

1. All crates have explicit versions.
2. CHANGELOG.md exists.
3. Release script works in dry-run.
4. Upgrade guide exists.

## Outcomes & Retrospective
_Updated after milestones complete._
