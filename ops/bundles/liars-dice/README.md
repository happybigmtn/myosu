# `ops/bundles/liars-dice/` — canonical published policy bundle

This directory is the W-02 "Public policy bundles" surface for the
Liar's Dice `promotable_local` game. The triple of files at
`ops/bundles/liars-dice/liars-dice/{bundle,benchmark-summary,artifact-manifest}.json`
is the **byte-stable canonical evidence** an external operator `curl`s
and feeds into `verify_policy_bundle` (the same verifier the
`promotion_manifest_quality_gate` CI job uses). The canonical triple is
identical in shape to the live `outputs/solver-promotion/liars-dice/`
tree; the `ops/bundles/` location is the public-facing surface and
`outputs/solver-promotion/` is the internal ledger surface.

## Layout

```
ops/bundles/liars-dice/
├── README.md                                  # this file
└── liars-dice/                                # <slug>-shaped subdir
    ├── bundle.json                            # CanonicalPolicyBundle (verified)
    ├── benchmark-summary.json                 # benchmark summary
    └── artifact-manifest.json                 # LiarsDiceArtifactDossier
```

The `<slug>-shaped subdir` convention matches the
`verify_promotion_outputs --slug <slug> --outputs-dir <path>` CLI
expectation: the verifier looks for `<outputs-dir>/<slug>/<file>`, so an
operator invoking
`cargo run -p myosu-games-canonical --example verify_promotion_outputs -- --slug liars-dice --outputs-dir ops/bundles/liars-dice`
finds the canonical triple at exactly
`ops/bundles/liars-dice/liars-dice/{bundle,benchmark-summary,artifact-manifest}.json`.

## Reproduction (byte-for-byte)

The canonical triple is reproducible end-to-end from a clean checkout
with the cargo example the Liar's Dice crate already ships. The
`POLICY_BUNDLE bundle_hash=` line printed by the example must match the
`bundle_hash` field embedded in `liars-dice/bundle.json` and the
`bundle_hash` recorded under "Pinned hashes" below; a future change to
the solver, the bundle builder, or the dossier that drifts the hash
fails the `tests/e2e/public_bundles_manifest.sh` sub-check (5).

```bash
SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-liars-dice \
    --example liars_dice_policy_bundle -- \
    --output ops/bundles/liars-dice/liars-dice/bundle.json \
    --iterations 512
```

The example writes the three files into
`ops/bundles/liars-dice/liars-dice/` (the parent of `--output` is the
parent of the bundle file; the bundle/summary/manifest are written next
to `--output`).

## Verification

The canonical triple passes `verify_policy_bundle` end-to-end (this is
proof sub-check 3 in `tests/e2e/public_bundles_manifest.sh`):

```bash
SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical \
    --example verify_promotion_outputs -- \
    --slug liars-dice --outputs-dir ops/bundles/liars-dice
# expected: PROMOTION_GATE_PASS slug=liars-dice tier=promotable_local ...
```

The `promotion_manifest_quality_gate` CI job additionally asserts the
canonical triple is accepted as the live liars-dice evidence (the
gate's positive path is run against the live `ops/solver_promotion.yaml`
ledger, where `liars-dice` is `tier: promotable_local`).

## Pinned hashes (record; do not edit by hand)

These hashes pin the canonical triple at the time of the W-02 ship.
They are checked by `tests/e2e/public_bundles_manifest.sh` sub-check
(2) (the file-content SHA-256) and sub-check (5) (the `bundle_hash`
from the verifier matches the one recorded here).

| Field                          | Value                                                              |
|--------------------------------|--------------------------------------------------------------------|
| `bundle.bundle_hash`           | `b5e0af16218860d698958a292c6a05201082082d18c250158ba4a8a775fb540c` |
| `bundle.provenance.game_slug`  | `liars-dice`                                                       |
| `bundle.provenance.engine_tier`| `promotable_local`                                                 |
| `bundle.provenance.solver_family` | `liars-dice-cfr`                                                |
| `artifact_manifest.checkpoint_hash` | `61707f0e14149fff46af6c8e25ac010ee04f34d4c78b1b7ce23f09201274aeb4` |
| `benchmark_summary.metric_value` | `0.6738117933273315` (below `threshold=0.7`, `passing=true`)    |
| `benchmark_summary.benchmark_id` | `liars-dice-exact-n1024-epochs512`                               |

## Scope boundary

W-02 publishes the Liar's Dice bundle only. Do NOT publish a public
host (operator's job; this row only ships the on-disk surface).
Do NOT change the bundle builder, the verifier, the example, the
liars-dice tier in `ops/solver_promotion.yaml`, or the
F-005 Bitino code. Do NOT attempt to publish a full-encoder NLHE
bundle — `nlhe-heads-up` is blocked on external artifact supply per
the PROMOTE-001 blocker note. Do NOT generalize the bundle publisher
to all `promotable_local` rows; liars-dice is the only current row,
so defer generalization until a second `promotable_local` row lands.
