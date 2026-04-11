# Specification: Security Posture and Advisory Debt

## Objective

Define the current security posture of the myosu project: the advisory inventory, supply chain risks, codec trust boundaries, and the intended remediation path. This spec provides the evidence base for plan 008 (SEC-001 triage).

## Evidence Status

### Verified facts (code-grounded)

#### Advisory inventory (19 suppressed in CI)

The following RUStsec advisories are explicitly ignored in `cargo audit` via `.github/workflows/ci.yml:358-376`:

| Advisory | Description Category |
|----------|---------------------|
| RUSTSEC-2025-0009 | Suppressed advisory |
| RUSTSEC-2025-0055 | Suppressed advisory |
| RUSTSEC-2023-0091 | Suppressed advisory |
| RUSTSEC-2024-0438 | Suppressed advisory |
| RUSTSEC-2025-0118 | Suppressed advisory |
| RUSTSEC-2026-0020 | Suppressed advisory |
| RUSTSEC-2026-0021 | Suppressed advisory |
| RUSTSEC-2025-0141 | **bincode 1.3.3** — direct Myosu ownership |
| RUSTSEC-2024-0388 | Suppressed advisory |
| RUSTSEC-2025-0057 | Suppressed advisory |
| RUSTSEC-2024-0384 | Suppressed advisory |
| RUSTSEC-2020-0168 | Suppressed advisory |
| RUSTSEC-2022-0061 | Suppressed advisory |
| RUSTSEC-2024-0436 | Suppressed advisory |
| RUSTSEC-2024-0370 | Suppressed advisory |
| RUSTSEC-2025-0010 | Suppressed advisory |
| RUSTSEC-2021-0127 | Suppressed advisory |
| RUSTSEC-2026-0002 | Suppressed advisory |
| RUSTSEC-2024-0442 | Suppressed advisory |

- Allowlist comment states alignment with WORKLIST.md SEC-001 — `ci.yml:350-355`
- The CI workflow does not yet encode per-advisory classifications; it only carries the shared allowlist comment and 19 ignored IDs — `.github/workflows/ci.yml:350-376`

#### Highest-signal item: bincode 1.3.3 (RUSTSEC-2025-0141)

- Direct dependencies on `bincode = "1.3"` exist in `myosu-games-poker`, `myosu-games-kuhn`, `myosu-games-liars-dice`, and `myosu-games-portfolio` — crate `Cargo.toml` files
- Direct usage appears in game crate wire/checkpoint paths: poker solver/wire/artifacts, Kuhn wire, Liar's Dice solver/wire — `crates/myosu-games-poker/src/solver.rs:7`, `crates/myosu-games-poker/src/wire.rs:1`, `crates/myosu-games-poker/src/artifacts.rs:5`, `crates/myosu-games-kuhn/src/wire.rs:1`, `crates/myosu-games-liars-dice/src/solver.rs:6`, `crates/myosu-games-liars-dice/src/wire.rs:1`
- Checkpoint format in poker and Liar's Dice: 4-byte magic + 4-byte version + **bincode 1.3.3** payload; Kuhn's exact-solver checkpoint uses `"MYOK"` + version `1` without a bincode payload — `crates/myosu-games-poker/src/solver.rs:20-21`, `crates/myosu-games-liars-dice/src/solver.rs:21-22`, `crates/myosu-games-kuhn/src/solver.rs:10-12`
- Also used by upstream robopoker (rbp-mccfr) via `serde` feature — workspace dependency chain

#### Inherited chain advisories

- Source: opentensor polkadot-sdk fork at rev `71629fd93b6c12a362a5cfb6331accef9b2b2b61` — `Cargo.toml`
- Which suppressed advisories are inherited-only versus directly reachable remains a SEC-001 triage output, not an established code fact

#### Encryption security (myosu-keys)

- Algorithm: crypto_secretbox (XSalsa20-Poly1305) — `crates/myosu-keys/`
- KDF: scrypt with `Params::recommended()` at keyfile creation time; chosen parameters are stored in the encrypted keyfile JSON — `crates/myosu-keys/src/storage.rs:347-374`
- Generated mnemonics are printed once and explicitly not stored on disk; imported mnemonics are read from an environment variable and converted into encrypted seed material — `crates/myosu-keys/src/main.rs:26-31`, `crates/myosu-keys/src/main.rs:512-518`, `crates/myosu-keys/src/storage.rs:75-83`
- Exporting an active keyfile copies the encrypted Myosu keyfile JSON and does not decrypt plaintext seed material — `crates/myosu-keys/src/storage.rs:264-279`
- Key password from environment variable only — miner/validator CLI

#### Actions security

- All GitHub Actions pinned to full SHA hashes — `ci.yml` throughout
- `persist-credentials: false` on all checkouts — `ci.yml` throughout
- Permissions scoped to `contents: read` — `ci.yml:18-19`

#### Workspace lint configuration

- Clippy pedantic with deny on: `arithmetic-side-effects`, `expect-used`, `indexing-slicing`, `unwrap-used` — `Cargo.toml` workspace lints
- `-D warnings` applied to all clippy runs — `ci.yml`

### Recommendations (intended system, from plan 008)

- Classify all 19 advisories into three buckets: remediate, accept, defer
- Update CI allowlist to reflect triage decisions with per-advisory justification comments
- Bincode 1.3.3 requires explicit migration decision or documented acceptance
- Inherited chain advisories should be documented as "no direct Myosu usage" with rationale
- Operator key documentation should make the one-time mnemonic backup contract explicit
- Run plan 008 in parallel with promotion work (does not block core product loop)
- Target: `cargo audit -D warnings` passes with reduced allowlist

### Hypotheses / unresolved questions

- Whether bincode migration to 2.x or postcard is feasible without breaking checkpoint compatibility
- Whether inherited polkadot-sdk advisories pose actual risk to myosu or are false positives
- Whether scrypt parameters in myosu-keys are sufficient for modern attack resistance
- Whether `zizmor` GitHub Actions security scanner has been run against the workflow

## Acceptance Criteria

- Every advisory in the CI allowlist has a documented classification (remediate/accept/defer) with justification
- Bincode 1.3.3 usage has an explicit decision: migrate or accept with documented rationale
- `cargo audit -D warnings` passes with the active allowlist (no new untracked advisories)
- Inherited chain advisories are documented with "no direct Myosu usage" rationale per advisory
- Workspace advisories (paste, lru) are documented with version check and upgrade feasibility
- GitHub Actions remain pinned to full SHA with `persist-credentials: false`
- No secrets, API keys, or credentials committed to the repository
- Clippy runs with `-D warnings` on all active crates and chain crates
- Mnemonic backup guidance documents that generated mnemonics are shown once and are not stored by the keystore

## Verification

```bash
# Dependency audit with current allowlist
cargo audit -D warnings \
  --ignore RUSTSEC-2025-0009 --ignore RUSTSEC-2025-0055 \
  --ignore RUSTSEC-2023-0091 --ignore RUSTSEC-2024-0438 \
  --ignore RUSTSEC-2025-0118 --ignore RUSTSEC-2026-0020 \
  --ignore RUSTSEC-2026-0021 --ignore RUSTSEC-2025-0141 \
  --ignore RUSTSEC-2024-0388 --ignore RUSTSEC-2025-0057 \
  --ignore RUSTSEC-2024-0384 --ignore RUSTSEC-2020-0168 \
  --ignore RUSTSEC-2022-0061 --ignore RUSTSEC-2024-0436 \
  --ignore RUSTSEC-2024-0370 --ignore RUSTSEC-2025-0010 \
  --ignore RUSTSEC-2021-0127 --ignore RUSTSEC-2026-0002 \
  --ignore RUSTSEC-2024-0442

# Check for committed secrets (basic scan)
git log --all --diff-filter=A -- '*.env' '*.key' '*.pem' 'credentials*'

# Verify Actions SHA pinning
grep -E 'uses:.*@[a-f0-9]{40}' .github/workflows/ci.yml | wc -l  # should match total uses count

# Verify persist-credentials
grep -c 'persist-credentials: false' .github/workflows/ci.yml  # should match checkout count

# Clippy (all active crates)
SKIP_WASM_BUILD=1 cargo clippy \
  -p myosu-games -p myosu-games-kuhn -p myosu-games-poker \
  -p myosu-games-liars-dice -p myosu-games-portfolio \
  -p myosu-games-canonical -p myosu-tui -p myosu-play \
  -p myosu-chain-client -p myosu-miner -p myosu-validator \
  -- -D warnings
```

## Open Questions

1. **Bincode migration complexity:** Poker and Liar's Dice checkpoints embed bincode 1.3.3 payloads, while Kuhn uses bincode in wire paths but not in its exact-solver checkpoint. A migration to bincode 2.x or postcard would require: (a) a versioned reader for old payload-bearing checkpoints, (b) robopoker fork changes if rbp-mccfr uses bincode directly, (c) potential checkpoint incompatibility. What's the blast radius?
2. **Inherited advisory triage criteria:** For advisories inherited from the polkadot-sdk fork, what constitutes "no direct Myosu usage"? If the advisory affects a crate used only in the chain runtime (not game logic), is that sufficient for an "accept" classification?
3. **Scrypt parameter audit:** New myosu-keys keyfiles use `Params::recommended()`. Should those exact parameters be audited and pinned against current OWASP guidance?
4. **Mnemonic backup guidance:** Operators generate mnemonics that are printed once and not stored on disk. Should the creation flow require backup verification rather than only printing a note?
5. **`zizmor` integration:** The global CLAUDE.md recommends `zizmor` for GitHub Actions security audit. Has this been run? Should it become a CI job?
6. **Supply chain depth:** `pip-audit` is recommended for Python dependencies but not run in CI. Python research files install `numpy`, `pytest`, `ruff` without version pinning or hash verification. Is this an acceptable risk for research-only code?
