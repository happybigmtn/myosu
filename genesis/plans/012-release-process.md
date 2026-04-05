# 012: Release Process

## Objective

Define and automate the release process for myosu versioned releases,
including operator bundle generation, changelog management, and release
gate enforcement.

## Context

`ops/release.sh --dry-run v0.1.0` already exists and produces a versioned
operator bundle. What is missing:
- Automated release gate checks (CI must pass, all invariants green)
- Changelog generation from git history
- GitHub release creation with artifacts
- Documented rollback procedure

## Acceptance Criteria

- `ops/release.sh` is extended to:
  - Run all CI checks locally before proceeding
  - Generate `CHANGELOG.md` entry from git log since last tag
  - Create git tag
  - Build operator bundle with version metadata
  - Create GitHub release via `gh release create` (with `--dry-run` support)
- A CI workflow `.github/workflows/release.yml` that:
  - Triggers on tag push (`v*`)
  - Builds release binaries for linux-amd64
  - Builds container images
  - Creates GitHub release with binaries and container image references
  - Runs full test suite as release gate
- `docs/operator-guide/upgrading.md` is updated with:
  - Versioning policy (semver for stage-0)
  - Upgrade procedure for each binary
  - Rollback procedure
  - Breaking change communication contract

## Verification

```bash
# Dry-run release
bash ops/release.sh --dry-run v0.1.1

# Verify bundle contents
ls target/releases/v0.1.1/
test -f target/releases/v0.1.1/bundle-manifest.toml
test -f target/releases/v0.1.1/release-notes.md
```

## Dependencies

- 010 (container packaging) -- release includes container images
- 011 (operator documentation) -- release notes reference upgrade guide
