# ADR 012: Retain Bincode 1.3.3 With Hardened Decode Budgets

- Status: Accepted
- Date: 2026-06-03
- Deciders: Myosu maintainers (SEC-002 owner)
- Consulted: `IMPLEMENTATION_PLAN.md` (SEC-002), `SECURITY.md` (RUSTSEC-2025-0141 row), `genesis/plans/008-security-debt-triage.md`, `ops/decision_log.md`, ADR 007 (checkpoint versioning)
- Informed: solver, wire, artifacts, miner, validator, operator, and portfolio contributors
- Related: `crates/myosu-games-poker/src/wire.rs`, `crates/myosu-games-poker/src/solver.rs`, `crates/myosu-games-poker/src/artifacts.rs`, `crates/myosu-games-kuhn/src/wire.rs`, `crates/myosu-games-liars-dice/src/wire.rs`, `crates/myosu-games-liars-dice/src/solver.rs`, `crates/myosu-games-liars-dice/src/dossier.rs`, `crates/myosu-games-portfolio/src/wire.rs`, `crates/myosu-games-portfolio/src/solver.rs`, `.github/workflows/ci.yml` (audit allowlist)

## Context

`RUSTSEC-2025-0141` ("`bincode 1.3.3` is unmaintained") is the only directly
owned advisory in Myosu's CI allowlist. The other 37 entries are inherited
through the opentensor `polkadot-sdk` fork or workspace-level transitive
dependencies that have no direct Myosu usage. SEC-001 (shipped in commit
`2dca9a3`) classified all 38 suppressions, but the bincode decision itself was
explicitly reserved for SEC-002 because it is the one Myosu-owned crate that
touches the wire, the solver checkpoints, the poker artifacts, and the
promotion dossier formats.

The risk profile of `RUSTSEC-2025-0141` is unfixed-bug and unmaintained, not
known-exploitable. The advisory does not call out a specific vulnerability; it
documents that upstream is no longer accepting fixes and that 1.x has
known-but-undescribed decode hazards that a 2.x rewrite does not retroactively
fix. Any replacement crate would carry its own decode surface, its own
maintenance posture, and its own upgrade story. The actual concern for Myosu
is what happens to a payload-bearing decode site that is reached with a
crafted or truncated input.

Myosu's Myosu-owned bincode usage is concentrated and well-isolated:

| Surface | Crate | Role | Current `MAX_DECODE_BYTES` |
|---|---|---|---|
| Wire encode/decode (info, query, response) | `myosu-games-poker/src/wire.rs` | Operator HTTP axon (poker only) | 1 MiB |
| Wire encode/decode | `myosu-games-kuhn/src/wire.rs` | Operator HTTP axon (kuhn) | 256 MiB |
| Wire encode/decode | `myosu-games-liars-dice/src/wire.rs` | File-based query/response | 1 MiB |
| Wire encode/decode | `myosu-games-portfolio/src/wire.rs` | Portfolio wire (kuhn cribbage etc.) | 1 MiB |
| Checkpoint encode/decode (`MYOS` + version + payload) | `myosu-games-poker/src/solver.rs` | Solver checkpoint persistence | 1 MiB |
| Checkpoint encode/decode | `myosu-games-liars-dice/src/solver.rs` | Solver checkpoint persistence | 1 MiB |
| Encoder / dossier / lookup artifact encode/decode | `myosu-games-poker/src/artifacts.rs` | Local artifact bundle, dossier lookup | 16 GiB |
| Solver checkpoint clone helper | `myosu-games-poker/src/solver.rs` (`clone_with_bincode`) | In-process defensive copy | 1 MiB |
| Solver encode/decode (portfolio) | `myosu-games-portfolio/src/solver.rs` | Portfolio profile/encoder save | 1 MiB |
| Dossier `checkpoint_format` string | `myosu-games-liars-dice/src/dossier.rs` | Audit tag = `"myos-v1-bincode"` | n/a |

All call sites share three properties: they use `bincode::DefaultOptions`
with `.with_fixint_encoding()`, `.reject_trailing_bytes()`, and
`.with_limit(MAX_DECODE_BYTES)`. There is no streaming bincode usage and no
`deserialize_from_reader` usage. Kuhn's exact-solver checkpoint uses a
`"MYOK"` + version header with no bincode payload and is therefore outside
this decision.

`rbp-nlhe`, `rbp-mccfr`, `rbp-cards`, `rbp-gameplay`, `rbp-core`, and
`rbp-transport` (the robopoker fork pinned at
`04716310143094ab41ec7172e6cea5a2a66744ef`) do not depend on bincode. Every
bincode call site in Myosu-owned code is owned by Myosu.

The CI allowlist already contains
`--ignore RUSTSEC-2025-0141 --ignore RUSTSEC-2024-0436` (the latter covers
`paste`, a bincode *transitive* from `serde` derive macros, not a direct
bincode concern). The bincode row in `SECURITY.md` was already classified as
"accept, pending SEC-002" so this ADR closes that loop.

## Decision

Retain `bincode = "1.3"` as the Myosu-owned serialization crate for wire
codecs, solver checkpoints, encoder/dossier artifacts, and the in-process
checkpoint clone helper. The acceptance rests on four hardened-decode
invariants and a one-axis change to a known-wrong knob.

1. **Hard decode budget at every read site.** Every `decode_codec` already
   passes `.with_limit(MAX_DECODE_BYTES)`. The check above lists the live
   values. The 1 MiB figure on wire, checkpoint, and portfolio solver sites
   is sized to comfortably exceed any legitimate payload (a worst-case
   strategy response is well under 100 KiB) while staying cheap to reject.
   The 16 GiB figure on `artifacts.rs` is sized to admit the full
   `EncoderLookupArtifact` in a single decode; it is a different threat
   model (operator-local file, not network input) and is treated separately
   in follow-up.

2. **Tighten kuhn wire to 1 MiB.** The current kuhn wire decode budget is
   256 MiB, an order of magnitude larger than any other wire site. The
   legitimate strategy-query and strategy-response payloads for Kuhn poker
   are sub-KiB, and the 256 MiB knob was inherited from a pre-axon
   measurement, not from a current payload measurement. SEC-002 ships a
   one-line reduction to 1 MiB so that the wire site has the same
   reject-cheaply posture as the rest of the codebase. This is the only
   behavioral code change that ships with this ADR.

3. **Keep the explicit `MYOS` + version header.** ADR 007 already requires
   a four-byte magic, a little-endian version, and a load-time validation
   before any payload is handed to bincode. A corrupted or hostile
   payload cannot reach `decode_bincode` without first passing the magic
   and version check. The 1.x `bincode` payload behind that header is
   versioned; the format string `myos-v1-bincode` in
   `myosu-games-liars-dice/src/dossier.rs:11` is the audit tag that ties
   dossiers to this exact payload.

4. **Accept and document the maintenance posture.** The upstream `bincode`
   crate is unmaintained, not actively broken. Myosu only uses a small,
   well-understood surface (default options, fixint, reject-trailing-bytes,
   bounded length-prefixed decode). Myosu does not depend on
   `bincode::deserialize_from_*` APIs, on `bincode::config::serialize_into`,
   or on any feature flags beyond defaults. The risk of a new
   `RUSTSEC-2025-0141` class of issue materializing inside that surface
   is low and is bounded by invariant (1).

The ADR explicitly does **not** migrate to `bincode 2.x` or to `postcard`
in this cycle. It does **not** ship a `bincode-suspend` or `vintage`
shim. It does **not** remove the `--ignore RUSTSEC-2025-0141` allowlist
entry. SEC-001's "accept, pending SEC-002" classification on the
`SECURITY.md` row moves to "accept (ADR 012)" so the audit trail is
self-consistent.

## Alternatives Considered

### Option A: Retain bincode 1.3.3 with hardened decode budgets (this ADR)

Why this option won.

- Myosu-owned usage is concentrated in 9 files across 4 crates, all using
  the same three-option `DefaultOptions` builder. The migration blast
  radius is small enough to study, but the upside of migrating is also
  small: `bincode 2.x` has its own `bincode::error::DecodeError` surface
  and its own `serde::Serialize` / `Deserialize` expectations, neither
  of which fixes the maintenance question. `postcard` is a reasonable
  choice for new code, but it is a breaking change to every on-disk
  checkpoint and on-wire type for a stage-0 network that already has
  pinned operator artifacts in the field.
- The 1.x vulnerability is "unmaintained, may have unfixed bugs," not
  "actively exploitable." With explicit decode budgets, the realistic
  attacker model is bounded by `MAX_DECODE_BYTES` and by the
  `MYOS` + version gate. That attacker model is already mitigated.
- Tightening the kuhn wire to 1 MiB in the same change means the
  acceptance rationale is not just "the knob already existed." It is
  the knob, the audit, and the cost-of-an-attack together.

### Option B: Migrate to `bincode 2.x`

Why this option was not chosen.

- `bincode 2.x` is a breaking API change. Every `bincode::Error` match
  arm in `WireCodecError` and `PokerSolverError` would have to migrate
  to `bincode::error::EncodeError` / `DecodeError`. Every
  `bincode::DefaultOptions` builder call would have to migrate to
  `bincode::config::standard()` / `bincode::config::legacy()` and the
  payload bytes themselves are not bit-compatible. The Myosu operator
  network already has `MYOS` + version + bincode-1.x payload artifacts
  in the field; a stage-0 mainnet candidate cannot break those without
  a migration plan, and the migration plan is out of scope for
  SEC-002.
- The upstream RUSTSEC row is still open against `bincode 1.3.3` even
  after `bincode 2.x` shipped. Moving to 2.x removes the "1.x is
  unmaintained" framing but does not by itself close the audit
  allowlist row, because `bincode 2.x` is a different package version
  and the `--ignore` is keyed on advisory id. A follow-up audit pass
  would still be required to confirm 2.x is advisory-clean.
- 2.x does not by itself provide a hardened-decode invariant. The same
  payload-size and `reject_trailing_bytes` discipline would still need
  to be re-applied at the call sites.

### Option C: Migrate to `postcard`

Why this option was not chosen.

- `postcard` is a sensible long-term choice: it has a smaller
  decode surface, it has a maintained upstream, and it has
  `std::no_std` parity that aligns with the chain-runtime direction
  recorded in `docs/adr/stage-2-roadmap.md`. But it is also a
  breaking on-disk format change, and the same `MYOS` + version
  audit-trail concern applies. Migrating to `postcard` is the right
  move **at the next checkpoint-format bump** (currently version 1),
  not in this cycle. ADR 007's follow-up section already calls for
  checkpoint versions to be bumped deliberately and only with a
  matching migration plan; SEC-002 should not pre-empt that decision.

### Option D: Wrap bincode behind a Myosu-owned codec trait

Why this option was not chosen.

- An internal trait that hides `bincode::DefaultOptions` would reduce
  the surface area visible to Myosu code and would make a future
  migration mechanical. But the trait is itself new code, new
  tests, and a new maintenance surface; the savings come only when
  the migration actually happens, and the migration is currently
  out of scope. Adding the trait without the migration is a
  speculative refactor that the hard rules in the SEC-002 plan
  definition would reject as "code that does not land the item."

## Consequences

### Positive

- The CI allowlist for `RUSTSEC-2025-0141` is now backed by an
  explicit, dated, reviewable decision instead of a `pending SEC-002`
  placeholder. SEC-001 and SEC-002 are both landed and cross-linked.
- The decode budget for kuhn wire matches the rest of the wire sites.
  Any future wire fuzzing runs against a uniform 1 MiB ceiling and
  a uniform "fixint + reject-trailing + bounded-length" recipe.
- The dossier `checkpoint_format` string (`"myos-v1-bincode"`) is
  the explicit on-disk and audit-trail anchor. A future bump to
  `myos-v2-postcard` (or `myos-v2-bincode-2`) is a deliberately
  versioned event, not a silent drift.
- The audit row in `SECURITY.md` is consistent with this ADR: the
  classification moves from "accept, pending SEC-002" to "accept
  (ADR 012)" so a future reader does not have to chase the
  cross-reference.

### Negative

- `bincode 1.3.3` remains a direct dependency in 4 owned crates. The
  `--ignore RUSTSEC-2025-0141` row stays in the CI allowlist. The
  `SECURITY.md` row stays in the "accept" bucket.
- `myosu-games-poker/src/artifacts.rs` continues to use a 16 GiB
  decode budget. The encoder lookup is operator-local and the
  decode is bounded by the surrounding on-disk file size, but a
  future hardening pass may want to move that to a streaming or
  length-prefixed-read form. That follow-up is logged below.
- Any future `bincode 1.x` advisory that is more severe than the
  current "unmaintained" framing will require re-opening this
  decision. Until then, Myosu owns the 1.x surface.

### Follow-up

- Move the `SECURITY.md` row for `RUSTSEC-2025-0141` from "accept,
  pending SEC-002" to "accept (ADR 012)" so the audit trail is
  self-consistent.
- Document the next checkpoint-format bump as the right moment to
  re-evaluate `postcard` (Option C) or `bincode 2.x` (Option B),
  with a version-2 reader + version-1 fallback migration plan.
- Audit the `16 GiB` decode limit in
  `myosu-games-poker/src/artifacts.rs` against a streaming or
  size-checked read in a follow-up security pass. This is a
  hardening opportunity, not a blocker.
- Keep the audit allowlist row in `.github/workflows/ci.yml` with
  a comment that points at this ADR.

## Reversibility

Moderate.

The decision is reversible: every wire, checkpoint, artifact, and
dossier call site already lives behind a `MYOS` + version header
(per ADR 007), so a future migration to a different crate can
read the existing `myos-v1-bincode` payload with the current
bincode 1.x crate while emitting a `myos-v2-...` payload. The
costs of reversal are:

- the migration code for wire, checkpoint, artifacts, and dossier
  formats,
- a versioned rollout of new producers and a one-way upgrade of
  consumers (a v1 reader can be kept alongside a v2 writer),
- an update to `ops/decision_log.md` and to the CI allowlist.

The decision should be reopened if:

- a new `RUSTSEC-2025-0141`-class advisory is published against
  `bincode 1.3.3` that is actively exploitable through a payload
  that fits inside `MAX_DECODE_BYTES` (1 MiB on wire, 1 MiB on
  checkpoint, 16 GiB on artifacts),
- `bincode 2.x` is audited clean and a stage-2 checkpoint format
  bump becomes the natural moment to migrate, or
- `postcard` is audited clean and a stage-2 checkpoint format
  bump becomes the natural moment to migrate.

## Validation / Evidence

- `crates/myosu-games-poker/src/wire.rs` — uses
  `DefaultOptions::new().with_fixint_encoding().reject_trailing_bytes()`
  on encode and the same plus `.with_limit(1 MiB)` on decode. Tests:
  `info_key_roundtrips_through_bincode`, `strategy_query_roundtrips_through_bincode`,
  `strategy_response_roundtrips_through_bincode`, plus
  `fuzz_strategy_query_roundtrips_through_bincode` and
  `fuzz_strategy_response_roundtrips_through_bincode`.
- `crates/myosu-games-poker/src/solver.rs` — `MYOS` + version 1 +
  bincode 1.x payload. Tests: oversized-payload rejection at
  `MAX_DECODE_BYTES - 1` (line 552-553).
- `crates/myosu-games-liars-dice/src/wire.rs` — same recipe with
  1 MiB. Tests: `info_roundtrips_through_bincode`, plus
  `fuzz_strategy_query_roundtrips_through_bincode` and
  `fuzz_strategy_response_roundtrips_through_bincode`.
- `crates/myosu-games-liars-dice/src/solver.rs` — `MYOS` + version 1
  + bincode 1.x payload. Tests: oversized-payload rejection at
  `MAX_DECODE_BYTES - 1` (line 843-844).
- `crates/myosu-games-liars-dice/src/dossier.rs:11` —
  `CHECKPOINT_FORMAT: &str = "myos-v1-bincode"`.
- `crates/myosu-games-kuhn/src/wire.rs` — same recipe. `MAX_DECODE_BYTES`
  reduced from 256 MiB to 1 MiB by the code change that ships with
  this ADR. Test: `info_roundtrips_through_bincode`,
  `strategy_query_roundtrips_through_bincode`,
  `strategy_response_roundtrips_through_bincode`.
- `crates/myosu-games-portfolio/src/wire.rs` and
  `crates/myosu-games-portfolio/src/solver.rs` — same recipe with
  1 MiB. Tests: `strategy_query_roundtrips_through_bincode`,
  `strength_query_roundtrips_through_bincode`,
  `strategy_response_roundtrips_through_bincode`.
- `crates/myosu-games-poker/src/artifacts.rs` — same recipe with
  16 GiB. Tests: `encoder_roundtrips_through_bincode`, plus the
  constant assertion that `MAX_DECODE_BYTES >= 4 * 1024 * 1024 * 1024`.
- `.github/workflows/ci.yml` — `dependency-audit` job retains
  `--ignore RUSTSEC-2025-0141` with a comment pointing at this ADR.
- `SECURITY.md` — row for `RUSTSEC-2025-0141` is updated to
  "accept (ADR 012)" by the code change that ships with this ADR.
