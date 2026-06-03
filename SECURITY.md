# Security Policy

Myosu is still a stage-0 project. Security reports are welcome, but the only
supported remediation targets are the `trunk` branch and the most recent git
tag. Older tags, archived plans, and historical bootstrap artifacts may not
receive fixes.

## Reporting a Vulnerability

Please do not open a public GitHub issue, pull request, or discussion for a
suspected vulnerability.

Preferred private channel:

- Use GitHub's private vulnerability reporting flow for
  `happybigmtn/myosu` if it is available in the repository UI.

Fallback private channel:

- Contact repository owner `@happybigmtn` privately on GitHub and request a
  security disclosure thread before sharing details publicly.

Include as much of the following as you can:

- affected component, crate, command, or workflow
- impact and realistic attacker model
- exact commit, tag, or branch tested
- reproduction steps or proof-of-concept
- any suggested mitigation or patch direction

## Response Expectations

- Acknowledgement target: within 72 hours
- Initial triage target: within 7 calendar days
- Status updates: at least weekly while the report is active
- Disclosure target: coordinated disclosure after a fix ships or a mitigation
  is documented

If a report cannot be resolved quickly because it depends on inherited
Substrate, Bittensor, or robopoker fork behavior, maintainers will document the
risk, the upstream dependency, and the temporary operator guidance before
closing the report.

## Scope

In scope:

- active crates and binaries used by the stage-0 local loop
- chain runtime, node, and `pallet-game-solver`
- miner, validator, gameplay, key-management, and operator-bundle flows
- artifact, checkpoint, wire-format, and blueprint loading boundaries
- CI and release-gate automation that can change shipped artifacts or operator
  instructions

Out of scope:

- public internet infrastructure not operated from this repository
- third-party services and GitHub platform bugs
- denial-of-service from unrealistic resource exhaustion without a plausible
  operator impact path
- historical or archived planning documents unless they create a live exploit
  path in current code

## Safe Harbor

Myosu supports good-faith security research conducted to improve the project.
Maintainers will not pursue action for research that:

- avoids privacy violations, data destruction, and service interruption
- uses test accounts, local devnets, or self-controlled environments whenever
  possible
- limits proof-of-concept activity to the minimum needed to demonstrate impact
- gives maintainers a reasonable opportunity to investigate and fix the issue
  before public disclosure

Do not exfiltrate secrets, modify other users' data, or run destructive load or
consensus attacks against systems you do not own or operate.

## Current Security Context

The current stage-0 audit snapshot lives in
[`ops/security-audit-stage0.md`](ops/security-audit-stage0.md). It is a release
gate input, not a substitute for reporting a vulnerability privately.

The upstream dependency review and cve-tracking process lives in
[`ops/cve-tracking-process.md`](ops/cve-tracking-process.md). Keep it aligned
with the current `cargo audit` ignore list and any upstream pin changes.

This repository does not currently run a bug bounty program.

## SEC-001 Advisory Triage Table

All 38 RUSTSEC advisories currently suppressed in
[`.github/workflows/ci.yml`](.github/workflows/ci.yml) and
[`.cargo/audit.toml`](.cargo/audit.toml) are triaged below per the
[security-posture spec](specs/110426-security-posture.md) (plan 008).
The table is the source of truth for the per-advisory rationale; the
two configuration files mirror it. The consistency between the three
files is enforced by
[`tests/e2e/sec001_allowlist_consistency.sh`](tests/e2e/sec001_allowlist_consistency.sh),
which is wired into the `dependency-audit` CI job and runs as part of
pre-push hygiene.

### Classification buckets

| Bucket | Count | Definition |
|--------|------:|-----------|
| `direct-owned`     |  1 | Myosu crates depend on the affected crate directly. We own the decision to remediate, accept, or defer. |
| `inherited-chain`  | 12 | Affected crate is reachable only through the opentensor `polkadot-sdk` fork. No Myosu game, miner, validator, or operator code path reaches the affected component. The fork is pinned to rev `71629fd93b6c12a362a5cfb6331accef9b2b2b61` and not rebased in this repository. |
| `inherited-wasm`   | 17 | `wasmtime 8.0.1` advisories on the chain runtime's prepare / PVF (parachain validation function) path. Live mainnet operations are blocked; only the dev / local loop exercises this code. |
| `inherited-misc`   |  8 | One-off `libp2p` / `rand` / `libsecp256k1` / `rustls-webpki` / `hickory-proto` / `core2` surface in the chain runtime; no direct Myosu game or operator code path reaches them. |

The `-D warnings` deny policy in
[`.github/workflows/ci.yml`](.github/workflows/ci.yml) is preserved
(security-posture spec acceptance criterion #2). Adding a `--ignore`
line to either configuration file without a corresponding row below
breaks the consistency test, which is the intended safety net.

### `direct-owned` (1)

| Advisory | Crate | Title | Decision | Justification |
|----------|-------|-------|----------|---------------|
| RUSTSEC-2025-0141 | `bincode 1.3.3` | Bincode is unmaintained | **accept (ADR 012)** | `bincode = "1.3"` is a direct dependency in `crates/myosu-games-poker/Cargo.toml`, `crates/myosu-games-kuhn/Cargo.toml`, `crates/myosu-games-liars-dice/Cargo.toml`, and `crates/myosu-games-portfolio/Cargo.toml`. Used for wire serialization in `myosu-games-poker/src/wire.rs`, `myosu-games-kuhn/src/wire.rs`, `myosu-games-liars-dice/src/wire.rs`, `myosu-games-liars-dice/src/solver.rs`, and `myosu-games-poker/src/solver.rs`; payload-bearing checkpoints in `myosu-games-poker/src/solver.rs:20-21` and `myosu-games-liars-dice/src/solver.rs:21-22` (4-byte magic + version + bincode payload); poker artifact dossiers in `myosu-games-poker/src/artifacts.rs`. Kuhn's exact-solver checkpoint uses `"MYOK"` + version `1` without a bincode payload (`myosu-games-kuhn/src/solver.rs:10-12`). Migration decision (bincode 2.x vs postcard vs accept) is owned by **SEC-002** and recorded in [`docs/adr/012-bincode-1.3.3-decision.md`](/srv/dev/repos/myosu/docs/adr/012-bincode-1.3.3-decision.md). Bounded 1 MiB decode budget at every wire, checkpoint, and portfolio solver site mitigates the documented unfixed issue; the kuhn wire site was reduced from 256 MiB to 1 MiB in the same change (`myosu-games-kuhn/src/wire.rs:7-13`). The `16 GiB` budget on `myosu-games-poker/src/artifacts.rs` is operator-local file decode, sized to admit the full `EncoderLookupArtifact`, and is documented as a follow-up hardening pass. No fix in the upstream crate. |

### `inherited-chain` (12)

All twelve advisories are reachable only through the opentensor
`polkadot-sdk` fork pinned at rev `71629fd93b6c12a362a5cfb6331accef9b2b2b61`.
Myosu inherits the runtime, executor, and pallet crates from that fork
but does not exercise the affected code paths from any game, miner,
validator, or operator binary. The fork is not rebased in this
repository (SEC-001 scope boundary).

| Advisory | Crate | Title | Justification |
|----------|-------|-------|---------------|
| RUSTSEC-2020-0168 | `mach 0.3.2` | `mach` is unmaintained | macOS-only IOKit bindings reached through `polkadot-sdk` fork. Operator hosts run on Linux x86_64 per the operator guide; macOS targets are not built. |
| RUSTSEC-2021-0127 | `serde_cbor 0.11.2` | `serde_cbor` is unmaintained | Reached through `parity-wasm` / polkadot fork dependency chain. Not in any direct Myosu crate `Cargo.toml`. |
| RUSTSEC-2022-0061 | `parity-wasm 0.45.0` | `parity-wasm` deprecated by author | Reached only via the polkadot-sdk fork's WASM utilities. No direct Myosu code path reaches it. |
| RUSTSEC-2024-0370 | `proc-macro-error 1.0.4` | `proc-macro-error` is unmaintained | Build-time proc-macro reached via polkadot-sdk fork. Not linked into any Myosu runtime binary. |
| RUSTSEC-2024-0384 | `instant 0.1.13` | `instant` is unmaintained | Reached via polkadot-sdk fork. No direct Myosu game or operator code path uses `instant`. |
| RUSTSEC-2024-0388 | `derivative 2.2.0` | `derivative` is unmaintained | Build-time / `#[derive]` macros reached via polkadot-sdk fork. No direct Myosu code path uses `derivative`. |
| RUSTSEC-2024-0436 | `paste 1.0.15` | `paste` is no longer maintained | Build-time proc-macro reached transitively via polkadot-sdk fork. Workspace lints already disallow it for new code; removing the inherited instance requires a fork rebase which is out of scope. |
| RUSTSEC-2025-0009 | `ring 0.16.20` | AES functions may panic when overflow checking is enabled | Reached via polkadot-sdk fork. `ring` 0.16 is pinned upstream; no direct Myosu code path uses `ring`. |
| RUSTSEC-2025-0010 | `ring 0.16.20` | `ring` versions prior to 0.17 are unmaintained | Same reach as RUSTSEC-2025-0009. `ring 0.17` requires a fork rebase. |
| RUSTSEC-2025-0055 | `tracing-subscriber 0.2.25` | Logging user input may result in poisoning logs with ANSI escape sequences | `tracing-subscriber = "0.3"` is used directly by `myosu-miner`, `myosu-validator`, and `myosu-play` (`Cargo.toml:30`). The vulnerable 0.2.25 instance is reached only through a polkadot-sdk fork transitive (via `sc-tracing` / `tracing-log`). Myosu writes operator-controlled strings into spans but not into the chain-runtime tracing path that contains the unpatched 0.2.25. The fork cannot be rebased in this repo (SEC-001 scope boundary). |
| RUSTSEC-2025-0057 | `fxhash 0.2.1` | `fxhash` is no longer maintained | Reached via polkadot-sdk fork. No direct Myosu code path uses `fxhash`. |
| RUSTSEC-2026-0002 | `lru 0.11.1` / `0.12.5` | `IterMut` violates Stacked Borrows by invalidating internal pointer | `lru` is reached only via polkadot-sdk fork (no direct Myosu `Cargo.toml` dependency). The fork is pinned; remediation requires a fork rebase which is out of scope for SEC-001. |

### `inherited-wasm` (17)

All 17 advisories affect `wasmtime 8.0.1` (and one `wasmtime-jit-debug`
advisory) on the chain runtime's prepare / PVF path. The PVF (parachain
validation function) executor is the `sc-executor-wasmtime` host called
by the `polkadot-node-core-pvf` subsystem to validate parachain
candidates. Myosu's stage-0 local loop is devnet-only; mainnet
operations are blocked at the chain-runtimes level. The polkadot-sdk
fork is pinned and not rebased here.

| Advisory | Crate | Title | Justification |
|----------|-------|-------|---------------|
| RUSTSEC-2023-0091 | `wasmtime 8.0.1` | Miscompilation of `i64x2.shr_s` on x86_64 with constant input | Cranelift backend on x86_64 PVF path. Operator hosts run on Linux x86_64 but the local loop does not execute untrusted PVF code. |
| RUSTSEC-2024-0438 | `wasmtime 8.0.1` | Wasmtime does not fully sandbox all Windows device filenames | Windows-only surface. Operator hosts are Linux x86_64. |
| RUSTSEC-2024-0442 | `wasmtime-jit-debug 8.0.1` | Dump Undefined Memory by `JitDumpFile` | JIT debug dump in PVF executor. Not exposed to Myosu game, miner, validator, or operator code paths. |
| RUSTSEC-2025-0118 | `wasmtime 8.0.1` | Unsound API access to a WebAssembly shared linear memory | Reached via the chain executor's PVF host. Live mainnet operations are blocked. |
| RUSTSEC-2026-0020 | `wasmtime 8.0.1` | Guest-controlled resource exhaustion in WASI implementations | Reached via the chain executor's PVF host. |
| RUSTSEC-2026-0021 | `wasmtime 8.0.1` | Panic adding excessive fields to `wasi:http/types.fields` | Reached via the chain executor's PVF host. |
| RUSTSEC-2026-0085 | `wasmtime 8.0.1` | Panic when lifting `flags` component value | Reached via the chain executor's PVF host. |
| RUSTSEC-2026-0086 | `wasmtime 8.0.1` | Host data leakage with 64-bit tables and Winch | Reached via the chain executor's PVF host. |
| RUSTSEC-2026-0087 | `wasmtime 8.0.1` | Segfault / unused out-of-sandbox load with `f64x2.splat` on Cranelift x86-64 | Reached via the chain executor's PVF host. |
| RUSTSEC-2026-0088 | `wasmtime 8.0.1` | Data leakage between pooling allocator instances | Reached via the chain executor's PVF host. |
| RUSTSEC-2026-0089 | `wasmtime 8.0.1` | Host panic when Winch compiler executes `table.fill` | Reached via the chain executor's PVF host. |
| RUSTSEC-2026-0091 | `wasmtime 8.0.1` | Out-of-bounds write or crash when transcoding component model strings | Reached via the chain executor's PVF host. |
| RUSTSEC-2026-0092 | `wasmtime 8.0.1` | Panic when transcoding misaligned component model UTF-16 strings | Reached via the chain executor's PVF host. |
| RUSTSEC-2026-0093 | `wasmtime 8.0.1` | Heap OOB read in component model UTF-16 to latin1+utf16 string transcoding | Reached via the chain executor's PVF host. |
| RUSTSEC-2026-0094 | `wasmtime 8.0.1` | Improperly masked return value from `table.grow` with Winch compiler backend | Reached via the chain executor's PVF host. |
| RUSTSEC-2026-0095 | `wasmtime 8.0.1` | Wasmtime with Winch compiler backend may allow a sandbox-escaping memory access | Reached via the chain executor's PVF host. |
| RUSTSEC-2026-0096 | `wasmtime 8.0.1` | Miscompiled guest heap access enables sandbox escape on aarch64 Cranelift | Reached via the chain executor's PVF host. |

### `inherited-misc` (8)

These advisories affect one-off transitive surfaces in the chain
runtime (`hickory-proto` DNS, `libsecp256k1`, `rand`, `rustls-webpki`,
yanked `core2`) that are reachable only through the polkadot-sdk fork
and not from any Myosu game, miner, validator, or operator code path.

| Advisory | Crate | Title | Justification |
|----------|-------|-------|---------------|
| RUSTSEC-2025-0161 | `libsecp256k1 0.7.2` | `libsecp256k1` is unmaintained | Reached via polkadot-sdk fork. Not in any direct Myosu `Cargo.toml`. |
| RUSTSEC-2026-0097 | `rand 0.8.5` / `0.9.2` | `rand` is unsound with a custom logger using `rand::rng()` | The vulnerable code path requires a user-installed `log` implementation forwarding into `rand::rng()`. Myosu uses `tracing-subscriber`'s default formatter, not a custom `log` -> `rand` shim. The vulnerable instance is reached only via the polkadot-sdk fork. |
| RUSTSEC-2026-0098 | `rustls-webpki 0.101.7` / `0.103.10` | Name constraints for URI names were incorrectly accepted | Reached via `sc-network` / `rustls` chain. Operator hosts do not terminate inbound TLS to Myosu services. |
| RUSTSEC-2026-0099 | `rustls-webpki 0.101.7` / `0.103.10` | Name constraints were accepted for certificates asserting a wildcard name | Reached via `sc-network` / `rustls` chain. Same as RUSTSEC-2026-0098. |
| RUSTSEC-2026-0104 | `rustls-webpki 0.101.7` / `0.103.10` | Reachable panic in certificate revocation list parsing | Reached via `sc-network` / `rustls` chain. Same as RUSTSEC-2026-0098. |
| RUSTSEC-2026-0105 | `core2 0.4.0` | `core2` is unmaintained; all versions yanked | Reached via `multihash` -> `libp2p` -> `polkadot-sdk` 2506.0.0. No direct Myosu crate depends on `core2`. Disabled the standalone `[yanked]` warning in `.cargo/audit.toml` to prevent double-counting with this advisory. |
| RUSTSEC-2026-0118 | `hickory-proto 0.25.2` | NSEC3 closest-encloser proof validation enters unbounded loop on cross-zone responses | Reached via `litep2p` -> `sc-network-types` -> `sc-service` -> polkadot-sdk fork. DNS validation is not exercised in the dev / local loop. |
| RUSTSEC-2026-0119 | `hickory-proto 0.24.4` / `0.25.2` | CPU exhaustion during message encoding due to O(n²) name compression | Reached via `libp2p-dns` -> `libp2p` -> polkadot-sdk fork. Same as RUSTSEC-2026-0118. |

### Update protocol

- Adding an advisory: add the `--ignore` line under the appropriate
  category in `.github/workflows/ci.yml`, add the quoted id to
  `.cargo/audit.toml` `[advisories] ignore = [...]`, and add a row
  above in the matching category. The consistency test will fail
  otherwise.
- Removing an advisory: do all three in reverse and re-run
  `bash tests/e2e/sec001_allowlist_consistency.sh` to confirm.
- The plan 008 acceptance criterion #3 ("Any advisory whose upstream
  crate has been patched is removed from the allowlist") is met by
  re-running `cargo audit --no-fetch` after every dependency update
  and pruning entries whose upstream fix has landed in the lockfile.

