# Specification: Abstraction Pipeline — Self-Contained Hand Clustering for Miners

Source: PE-01 solver wrapper (needs populated NlheEncoder), RF-02 (non-DB constructor)
Status: Draft
Date: 2026-03-30
Depends-on: RF-01..02 (robopoker fork), PE-01 (poker solver wrapper)

## Purpose

Define the remaining abstraction-generation boundary for poker artifacts.
The repo no longer lacks file-based artifact loading: stage-0 poker already
ships manifest-backed encoder directories and checkpoint-backed local advice.
What is still missing is the standalone path that regenerates those artifacts
from clustering logic instead of consuming a prebuilt bundle.

The abstraction table is the bridge between concrete poker hands and abstract
strategy buckets. Without it, the MCCFR encoder maps every hand to a default
bucket and the solver produces random strategies.

## Whole-System Goal

Current state:
- `crates/myosu-games-poker/src/artifacts.rs` already defines a manifest-backed
  encoder artifact format and verified on-disk loader
- `crates/myosu-miner/` and `crates/myosu-validator/` already consume
  `--encoder-dir` as a stage-0 artifact input
- `crates/myosu-games-poker/examples/bootstrap_artifacts.rs` already proves the
  local bootstrap shape for encoder-dir plus query artifacts
- the missing seam is now narrower: generating a full artifact directory from
  clustering logic without relying on a database-heavy robopoker workflow

This spec adds:
- the standalone artifact-generation path that matches the loader contract the
  repo already uses
- a clearer boundary between "artifact consumption is live" and "artifact
  regeneration is still future work"
- the ownership map for the eventual clustering/export surface

If all ACs land:
- A miner can bootstrap with zero infrastructure: download abstraction table,
  load into encoder, start MCCFR training
- A miner with compute can regenerate abstractions from scratch
- Abstraction artifacts are versioned and hash-checked (INV-003: all validators
  must use identical encoder state)

Still not solved here:
- Abstractions for non-poker games (each game defines its own encoder)
- Distributed clustering across multiple machines
- Incremental clustering (re-clustering when new isomorphisms are discovered)

12-month direction:
- Automated clustering pipeline as a Fabro/Raspberry-supervised artifact lane
- Per-game abstraction artifacts published to a registry
- Miners can specialize: run clustering for new games to discover novel buckets

## Why This Spec Exists As One Unit

- The clustering algorithm, file format, download mechanism, and miner
  integration form one operational path: "miner starts → loads encoder →
  trains MCCFR"
- Breaking this into separate specs creates gaps where the file format
  doesn't match the loader or the download hash doesn't match the artifact

## Scope

In scope:
- `myosu-cluster` binary wrapping robopoker's clustering pipeline
- File-based output format for abstraction tables
- `NlheEncoder::from_file()` loader (in robopoker fork)
- Artifact versioning with SHA-256 hash
- Bootstrap download path for miners
- Memory and time estimates for clustering

Out of scope:
- Non-NLHE abstractions (future game specs)
- Distributed clustering
- Clustering as an on-chain incentive (possible future subnet type)
- PostgreSQL integration (robopoker's existing path remains for power users)

## Current State

- `/home/r/coding/myosu/crates/myosu-games-poker/src/artifacts.rs` — live
  manifest-backed encoder artifact loading, hashing, and directory writing
- `/home/r/coding/myosu/crates/myosu-games-poker/examples/bootstrap_artifacts.rs`
  — live bootstrap example that writes a local encoder-dir and matching query
  artifact
- `/home/r/coding/myosu/crates/myosu-miner/src/training.rs` and
  `/home/r/coding/myosu/crates/myosu-miner/src/strategy.rs` — live consumers of
  encoder-dir-backed poker artifacts
- `/home/r/coding/myosu/crates/myosu-validator/src/validation.rs` — live
  validator-side artifact consumer
- `/home/r/coding/robopoker/crates/clustering/` still contains the underlying
  clustering logic, but the self-contained export path remains external to the
  current repo
- Clustering constants (from `rbp-core`):
  - Preflop: 169 isomorphisms → 169 buckets (1:1, no clustering)
  - Flop: ~1.3M isomorphisms → 128 buckets
  - Turn: ~14M isomorphisms → 144 buckets
  - River: ~123M isomorphisms → 101 equity buckets (0%-100%)
- Total abstraction table: ~138M entries, ~500 unique buckets
- Memory for full table: ~3GB (river alone is 3.02GB per COMPLEXITY.md)

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| Artifact manifest and hashing | `myosu-games-poker::artifacts` | reuse | File-backed stage-0 contract already exists here |
| Local bootstrap example | `myosu-games-poker/examples/bootstrap_artifacts.rs` | reuse | Current proof of encoder-dir creation shape |
| Miner artifact loading | `myosu-miner` training and strategy paths | reuse | Stage-0 training already consumes encoder-dir |
| Validator artifact loading | `myosu-validator::validation` | reuse | Stage-0 validation already consumes encoder-dir |
| Clustering logic | `rbp-clustering` crate | extend via wrapper | The missing path is generation/export, not consumption |
| Database-heavy orchestration | `rbp-database` + PostgreSQL | replace where possible | The future generator should not require stage-0 operators to run PostgreSQL |

## Non-goals

- Improving the clustering algorithm — robopoker's k-means with EMD is proven
- Supporting databases — the PostgreSQL path exists in robopoker for power users
- Incremental updates — full re-clustering is the only path for now
- GPU acceleration — CPU clustering is slow but correct

## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| Artifact manifest + loader | Implemented | crates/myosu-games-poker/src/artifacts.rs |
| Bootstrap artifact example | Implemented | crates/myosu-games-poker/examples/bootstrap_artifacts.rs |
| Miner artifact consumption | Implemented | crates/myosu-miner/src/training.rs, crates/myosu-miner/src/strategy.rs |
| Validator artifact consumption | Implemented | crates/myosu-validator/src/validation.rs |
| Standalone clustering/export binary | Planned | no canonical in-repo crate yet |
| Published artifact distribution | Planned | no canonical distribution surface yet |

---

## Acceptance Criteria

### AC-AP-01: Clustering Binary

- Where: `crates/myosu-cluster/src/main.rs (new)`
- How: Binary wrapping robopoker's clustering pipeline:

  ```
  myosu-cluster --output ./abstractions/ [--street preflop|flop|turn|river|all]
  ```

  Runs the full pipeline: enumerate isomorphisms → compute equity
  distributions → k-means clustering → write abstraction table to files.

  Output: three files per street (lookup + metric + future):

  ```
  abstractions/
  ├── preflop.lookup.bin     # (i64, i16) pairs: iso → abs, 4KB
  ├── preflop.metric.bin     # (i32, f32) pairs: triangular EMD, 301KB
  ├── flop.lookup.bin        # iso → abs, 32MB
  ├── flop.metric.bin        # triangular EMD, 175KB
  ├── flop.future.bin        # (i16, i16, f32): centroid distributions
  ├── turn.lookup.bin        # iso → abs, 347MB
  ├── turn.metric.bin        # triangular EMD, 175KB
  ├── turn.future.bin        # centroid distributions
  ├── river.lookup.bin       # iso → abs, ~3GB
  └── manifest.json
  ```

  The lookup files contain `BTreeMap<Isomorphism, Abstraction>` as bincode.
  The metric files contain pairwise EMD distances between abstract buckets
  (needed for hierarchical clustering: flop clustering reads turn metric).
  The future files contain centroid histogram distributions for belief
  propagation between streets.

  Also writes `manifest.json`:
  ```json
  {
    "version": 1,
    "game": "nlhe_hu",
    "streets": {
      "preflop": { "file": "preflop.bin", "entries": 169, "sha256": "..." },
      "flop": { "file": "flop.bin", "entries": 1286792, "sha256": "..." },
      "turn": { "file": "turn.bin", "entries": 13960050, "sha256": "..." },
      "river": { "file": "river.bin", "entries": 123156254, "sha256": "..." }
    },
    "created": "2026-03-16T00:00:00Z",
    "total_sha256": "..."
  }
  ```

  `total_sha256` is the hash of all 4 file hashes concatenated. This is the
  encoder identity used for INV-003 verification (validators compare this hash).

- Whole-system effect: enables miner self-sufficiency. A miner with enough
  compute can generate abstractions from scratch.
- State: isomorphism enumeration state, clustering state.
- Wiring contract:
  - Trigger: `myosu-cluster --output ./abstractions/` CLI
  - Callsite: miner bootstrap script or manual execution
  - State effect: abstraction files written to disk
  - Persistence effect: 4 binary files + manifest.json
  - Observable signal: manifest.json contains SHA-256 hashes
- Required tests:
  - `cargo test -p myosu-cluster cluster::tests::preflop_produces_169_entries`
  - `cargo test -p myosu-cluster cluster::tests::manifest_valid_json`
  - `cargo test -p myosu-cluster cluster::tests::sha256_matches_content`
- Pass/fail:
  - Preflop clustering produces exactly 169 entries (1:1 mapping)
  - All 4 files written and non-empty
  - Manifest SHA-256 matches actual file hashes
  - Re-running with same parameters produces identical output (deterministic)
  - Full pipeline completes (may take hours for river — test with --street preflop)
- Blocking note: without abstraction tables, miners produce random strategies.
- Rollback condition: robopoker clustering API not accessible from outside rbp-autotrain.

### AC-AP-02: File-Based Encoder Loading

- Where: `happybigmtn/robopoker rbp-nlhe/src/encoder.rs (extend)` — part of RF-02
- How: Add `NlheEncoder::from_dir(path)` that:
  1. Reads `manifest.json` from the directory
  2. Validates SHA-256 hashes of all 4 files
  3. Deserializes each file into `BTreeMap<Isomorphism, Abstraction>`
  4. Merges all streets into a single `BTreeMap`
  5. Returns `NlheEncoder(merged_map)`

  Also add `NlheEncoder::hash(&self) -> String` that returns the
  `total_sha256` from the manifest. This is what validators compare for
  INV-003.

- Whole-system effect: miners load encoder without PostgreSQL.
- State: NlheEncoder populated from files.
- Wiring contract:
  - Trigger: miner startup calls `NlheEncoder::from_dir("./abstractions/")`
  - Callsite: PokerSolver::new() in myosu-miner
  - State effect: encoder populated with ~138M entries
  - Persistence effect: N/A (read-only from files)
  - Observable signal: `encoder.hash()` returns consistent SHA-256
- Required tests:
  - `cargo test -p rbp-nlhe encoder::tests::from_dir_loads_all_streets`
  - `cargo test -p rbp-nlhe encoder::tests::hash_is_deterministic`
  - `cargo test -p rbp-nlhe encoder::tests::invalid_hash_rejected`
- Pass/fail:
  - `from_dir` loads all 4 street files successfully
  - `encoder.abstraction(obs)` returns valid abstraction for any observation
  - Tampered file (wrong hash) → error on load
  - `hash()` returns same value as manifest's total_sha256
- Blocking note: this is part of RF-02 in the robopoker fork. Listed here
  for completeness — the implementation lives in the fork.
- Rollback condition: ~138M entries don't fit in memory on target hardware
  (need ~3GB RAM).

### AC-AP-03: Pre-Computed Artifact Distribution

- Where: `artifacts/abstractions/ (new)`, `scripts/download-abstractions.sh (new)`
- How: Publish pre-computed abstraction tables as a versioned artifact:

  1. Run `myosu-cluster --output artifacts/abstractions/`
  2. Upload to a public URL (GitHub release, S3, IPFS)
  3. `download-abstractions.sh` fetches + verifies:

  ```bash
  #!/bin/bash
  ARTIFACT_URL="https://github.com/happybigmtn/myosu/releases/download/abstractions-v1/nlhe-hu-abstractions.tar.gz"
  EXPECTED_HASH="abc123..."

  curl -L "$ARTIFACT_URL" | tar xz -C ./abstractions/
  ACTUAL_HASH=$(cat ./abstractions/manifest.json | jq -r '.total_sha256')

  if [ "$ACTUAL_HASH" != "$EXPECTED_HASH" ]; then
      echo "HASH MISMATCH: expected $EXPECTED_HASH, got $ACTUAL_HASH"
      exit 1
  fi
  echo "abstractions loaded. encoder hash: $ACTUAL_HASH"
  ```

  The miner's startup sequence:
  1. Check if `./abstractions/manifest.json` exists
  2. If not, run `download-abstractions.sh`
  3. Load encoder via `NlheEncoder::from_dir("./abstractions/")`
  4. Log encoder hash for INV-003 auditability

- Whole-system effect: miners bootstrap in minutes (download) instead of
  hours/days (compute). All miners use the same artifact = same encoder hash
  = deterministic scoring (INV-003).
- State: artifact files on disk.
- Wiring contract:
  - Trigger: miner startup detects missing abstractions
  - Callsite: myosu-miner/src/main.rs bootstrap
  - State effect: abstraction files on disk
  - Persistence effect: ~3GB in ./abstractions/
  - Observable signal: encoder hash logged on startup
- Required tests:
  - `scripts/download-abstractions.sh` exits 0 with valid artifact
  - Hash verification catches tampered files
- Pass/fail:
  - Download completes within 5 minutes on 100Mbps connection
  - Hash matches expected value
  - Miner starts successfully after download
  - Tampered artifact → download script exits 1 with clear error
- Blocking note: without pre-computed artifacts, every new miner must spend
  hours/days computing abstractions before producing any strategy.
- Rollback condition: artifact URL is unavailable or file too large for target.

---

## Resource Requirements

| phase | CPU | peak RAM | disk output | time |
|-------|-----|----------|-------------|------|
| preflop | 1 core | 1 MB | 4 KB + 301 KB | <1 second |
| flop k-means | 16 cores | ~1.3 GB | 32 MB + 175 KB | ~10 minutes |
| turn k-means | 16 cores | ~16 GB | 347 MB + 175 KB | ~2 hours |
| river equity | 16 cores | streaming | ~3 GB | ~8 hours |
| **total** | **16 cores** | **~16 GB peak** | **~3.4 GB** | **~10 hours** |

Peak RAM is during turn clustering: 14M histograms × 144 buckets × 4 bytes
× 2 (data + bounds) ≈ 16 GB. River is streamed (no full-memory load).
Source: robopoker clustering architecture analysis.

## Decision Log

- 2026-03-16: File-based (not database) — miners should run with zero
  infrastructure beyond the binary and abstraction files.
- 2026-03-16: Pre-computed artifact as primary path — most miners should
  download, not compute. Computing is for power users or new game variants.
- 2026-03-16: SHA-256 hash as encoder identity — validators compare this
  hash to enforce INV-003 determinism across the network.
- 2026-03-16: Bincode for file format — compact, fast, matches checkpoint format.

## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | `myosu-cluster --street preflop` produces 169-entry file | Clustering binary | AP-01 |
| 2 | `NlheEncoder::from_dir()` loads preflop file | File loading | AP-02 |
| 3 | Download script fetches and verifies artifact | Distribution | AP-03 |
| 4 | Miner starts with downloaded abstractions, trains successfully | End-to-end | All |
| 5 | Two independent encoder loads produce identical hash | INV-003 | AP-02 |
