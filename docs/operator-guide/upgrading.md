# Operator Upgrading Guide

## Goal

Give operators one truthful process for evaluating, scheduling, executing, and
rolling back Myosu upgrades during the current stage-0 and early stage-1 era.

This guide covers communication and procedure. It does not pretend the repo
already has automated upgrade tooling, package-manager releases, or on-chain
runtime governance.

## Release Contract

During stage-0 and stage-1, Myosu uses semantic versioning with an explicit
operator contract:

| Version change | Operator meaning | Expected process |
|---|---|---|
| `0.x.y` patch | intended to stay operator-compatible | review changelog, upgrade when convenient |
| `0.x.0` minor | may include breaking operator changes | follow the full upgrade window and migration steps |
| `1.0.0+` major | reserved for a later compatibility contract | not defined yet |

Important current reality:

- `0.0.x` tags on `trunk` are internal checkpoint tags, not supported operator
  releases
- `0.1.0` is the first supported operator-facing baseline
- during `0.x`, a minor release is the breaking-change signal and a patch
  release should remain operator-compatible

## Authoritative Change Surfaces

When deciding whether an upgrade is safe, treat these as the repo-owned truth
surfaces in order:

1. `CHANGELOG.md`
2. the tagged release notes generated from the changelog
3. release-specific migration steps linked from the changelog or release notes
4. operator-guide updates in `quickstart.md`, `troubleshooting.md`, and this
   document

Direct operator messages are useful for scheduling, but they are not the source
of truth unless they link back to one of the repo-owned surfaces above.

## How Breaking Changes Are Announced

Before cutting a breaking `0.x` minor release, the release owner must publish
the change in four places:

1. add a `Breaking Changes` section or explicit migration steps to
   `CHANGELOG.md` under `Unreleased`
2. update the relevant operator docs before tagging the release
3. publish release notes for the tagged version with concrete operator action
   items
4. send a direct notice to operators in the same coordination channel already
   used to distribute RPC endpoints and bundle details, linking back to the
   changelog and tagged release notes

The direct notice must include:

- target version
- whether the release is patch-compatible or breaking
- the planned upgrade window
- the exact docs operators must read before restarting services
- the rollback owner or decision maker for the window

## Upgrade Windows

Use the following minimum windows unless a release-specific note says
otherwise:

| Release type | Minimum notice | Expected operator action |
|---|---|---|
| compatible patch | at release time, or 24 hours if manual action is required | review changelog and roll forward when ready |
| breaking minor | 7 calendar days before the cutover window | schedule maintenance, back up state, follow migration steps |
| urgent security fix | as soon as the fix is ready | follow the release note immediately; some steps may compress the window |

Support posture during stage-0:

- the current tagged operator release is the default supported target
- the previous minor release is only considered supported until the published
  upgrade window closes
- once the window closes, incidents on the older minor should first be treated
  as upgrade debt, not fresh bugs

## Operator Preflight Checklist

Before touching a running miner, validator, or follower node:

1. record the current tag with `git describe --tags --abbrev=0`
2. read the target release entry in `CHANGELOG.md`
3. read any linked migration notes before stopping services
4. back up:
   - `~/.myosu/` or your active config dir
   - the current operator bundle directory
   - local miner artifacts such as checkpoints and encoder data
5. confirm whether the target release changes:
   - chain spec or bootnode metadata
   - required CLI flags
   - config file shape
   - storage or artifact formats

If the release notes describe an irreversible migration, do not start until you
have a tested backup path.

## Upgrade Procedure

### 1. Fetch and pin the target release

```bash
git fetch --tags origin
git checkout vX.Y.Z
```

If the release notes point at a specific commit instead of a tag during a
pre-release window, use that exact commit sha and treat the later tag as the
same rollout target.

### 2. Rebuild the required surfaces

Use the same owned build steps as the quickstart:

```bash
rustup target add wasm32v1-none
rustup target add wasm32-unknown-unknown
cargo build -p myosu-chain-runtime
SKIP_WASM_BUILD=1 cargo build -p myosu-chain --features fast-runtime
SKIP_WASM_BUILD=1 cargo build -p myosu-keys -p myosu-games-poker -p myosu-miner -p myosu-validator
```

If the release notes call out different targets or additional crates, prefer
the release notes over this baseline.

### 3. Regenerate the operator bundle

Do not keep using an old bundle across a breaking release.

```bash
export MYOSU_OPERATOR_CHAIN="${MYOSU_CHAIN:?set MYOSU_CHAIN first}"
bash .github/scripts/prepare_operator_network_bundle.sh ./operator-bundle "$MYOSU_CONFIG_DIR"
./operator-bundle/verify-bundle.sh
```

This step forces the wrapper scripts, chain spec, and bootnode metadata to
match the checked-out release.

### 4. Restart services in dependency order

Upgrade in this order:

1. follower node, if you run one
2. miner
3. validator

For each layer, use the release notes if they provide a more specific command
contract than the quickstart.

### 5. Run post-upgrade probes

At minimum, prove the same surfaces you rely on in normal operation:

```bash
curl -fsS \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}' \
  http://127.0.0.1:9955
./operator-bundle/verify-bundle.sh
./operator-bundle/start-miner.sh --help
./operator-bundle/start-validator.sh --help
```

If the release changes miner or validator behavior, also rerun the bounded
bootstrap probes from the [quickstart](./quickstart.md).

## Rollback Procedure

If the upgrade fails before operators reach a healthy steady state:

1. stop the upgraded services
2. check out the previous known-good tag
3. rebuild the same binaries for that tag
4. restore the previous operator bundle and any backed-up config or artifacts
5. restart in the same dependency order: follower node, miner, validator
6. notify the release owner that rollback was required and include the failing
   command or probe

Do not blindly roll back if the release notes mention an irreversible storage,
artifact, or config migration. In that case, restore from the backups taken in
preflight or wait for an explicit rollback plan from the release owner.

## What Must Appear In A Breaking Release Note

Every breaking `0.x` minor release note should answer these operator questions:

- what changed that breaks the previous minor release
- whether a maintenance window is required
- which commands or config values changed
- whether checkpoints, bundles, or chain specs must be regenerated
- whether rollback is safe after the migration starts
- what probes prove the upgrade succeeded

If any of those answers are missing, treat the release note as incomplete and
do not start the upgrade window yet.

## Current Constraints

- Upgrades are source-based today. Operators move by checking out a tag and
  rebuilding, not by downloading versioned binaries from a release registry.
- The operator bundle is derived from the checked-out repo state, so regenerate
  it for every upgrade that changes chain or service assumptions.
- Runtime-upgrade governance, automated cutovers, and long-lived release
  branches are out of scope for the current stage.

## Related Docs

- [quickstart.md](./quickstart.md) for the zero-to-running operator path
- [troubleshooting.md](./troubleshooting.md) for failure recovery after a bad
  upgrade attempt
- [CHANGELOG.md](../../CHANGELOG.md) for the operator-facing release history
