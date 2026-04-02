# Specification: Release Governance

Source: Genesis Plan 010 (Release Governance and Versioning)
Status: Draft
Depends-on: 006-multi-node-devnet

## Purpose

The project has no release process, no version numbers beyond Cargo defaults, no
changelog, and no mechanism for communicating breaking changes to operators.
Before stage-1 (public devnet with external operators), the project needs a way
to version releases, document changes, and give operators advance notice of
breaking changes. Without release governance, operators cannot pin to known-good
versions, cannot assess upgrade risk, and cannot plan maintenance windows.

## Whole-System Goal

Current state: No git tags, releases, or changelog exist. Workspace crates use
default version numbers. Fabro execution branches serve as de facto "runs" but
are not releases. No process coordinates breaking changes with operators. The
workspace `Cargo.toml` uses `workspace.package` but does not set a
workspace-level version.

This spec adds: Semantic versioning across workspace crates, a changelog, a
release script that tags and builds an operator bundle, and a documented process
for communicating breaking changes to operators.

If all ACs land: Operators can pin to a specific tagged version, review a
changelog for what changed, and follow an upgrade guide when breaking changes
occur.

Still not solved here: Automated release pipelines (CD), on-chain runtime
upgrade governance, backwards compatibility guarantees, and release signing
or attestation.

## Scope

In scope:
- Applying semantic versioning to all workspace crates
- Creating and maintaining a changelog
- A release script that tags, builds the operator bundle, and generates release
  notes
- A documented process for communicating breaking changes to operators

Out of scope:
- Continuous deployment or automated release triggers
- On-chain runtime upgrade mechanisms
- Backwards compatibility guarantees across major versions
- Release binary signing or notarization
- Package registry publishing (crates.io)

## Current State

The workspace `Cargo.toml` at the repository root defines `workspace.package`
metadata but does not set explicit version numbers for individual crates. Cargo
defaults are in use.

No git tags exist in the repository. The 50 trunk commits have no associated
releases. The 232 unmerged remote branches are Fabro execution artifacts, not
release branches.

No CHANGELOG.md exists. Change history is only available through git log.

The operator bundle build script at
`.github/scripts/prepare_operator_network_bundle.sh` produces a distributable
bundle but is not integrated into a release workflow.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Workspace metadata | Root `Cargo.toml` `workspace.package` | Extend | Add explicit versions |
| Bundle build script | `.github/scripts/prepare_operator_network_bundle.sh` | Reuse | Already produces distributable bundle |
| Git history | 50 trunk commits | Reference | Source for initial changelog |
| CI pipeline | `.github/workflows/ci.yml` | Extend | Release workflow trigger |
| Bundle manifest | `bundle-manifest.toml` | Extend | Add version field |

## Non-goals

- Publishing crates to crates.io or any package registry.
- Supporting multiple concurrent release branches.
- Automating rollback procedures for failed releases.
- Providing binary downloads (operators build from source in stage-1).
- Maintaining backwards compatibility across breaking changes — the goal is
  communication, not avoidance.

## Behaviors

All workspace crates carry explicit semantic version numbers. Version numbers
are updated as part of the release process. All crates in the workspace share
the same version during stage-0 and stage-1, diverging only when crate stability
levels differ.

A changelog at the repository root documents notable changes for each release,
organized by version. Entries describe what changed from an operator's
perspective: new capabilities, breaking changes, required migration steps, and
notable fixes.

A release script automates the mechanical steps of releasing: verifying CI
passes, tagging the commit, building the operator bundle, and generating release
notes from the changelog. The script supports a dry-run mode that shows what
would happen without making changes.

When a release includes breaking changes, the release notes include specific
migration steps operators must follow. An upgrade guide describes the process
for operators to move from one version to the next, including any configuration
changes, data migrations, or manual steps required.

## Acceptance Criteria

- All workspace crates have explicit semantic version numbers in their
  Cargo.toml files.
- A CHANGELOG.md exists at the repository root with at least one version entry.
- A release script tags, builds the operator bundle, and generates release notes
  in dry-run mode without errors.
- An upgrade guide documents the process for operators to move between versions,
  including how breaking changes are communicated.
