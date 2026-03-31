# End-to-End Integration Test Harness

Status: Completed 2026-03-29.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

Provenance: New plan. No prior genesis plan covered end-to-end integration testing across chain, miner, validator, and gameplay.

## Purpose / Big Picture

Individual components may pass unit tests but fail when integrated. Stage-0 exit requires the full loop: chain produces blocks -> miner registers and trains -> validator queries and scores -> weights are submitted -> emissions are distributed -> human plays against the best strategy. This plan creates an integration test harness that exercises this loop.

After this plan, the repo has a cargo-owned end-to-end harness around the real
stage-0 loop rather than only a manual smoke command. The first honest layer is
an ignored `myosu-chain` integration test that runs
`--stage0-local-loop-smoke` and asserts the emitted full-loop contract. The
next honest layer is a fast non-ignored parser/fixture test that locks the
stdout contract shape without paying the full local-node cost on every run.

## Progress

- [x] (2026-03-28) Confirmed no integration tests exist for the chain + service loop.
- [x] (2026-03-29) Added the first node-owned integration test target at
  `crates/myosu-chain/node/tests/stage0_local_loop.rs`, wrapping the real
  `--stage0-local-loop-smoke` flow and asserting its stdout contract.
- [x] (2026-03-29) Refactored the wrapper around a reusable parsed
  `Stage0LoopSummary`, added a fast fixture-backed contract test, and kept the
  ignored live full-loop run as the expensive proof path.
- [x] (2026-03-29) Added a negative fixture regression test so the fast
  contract gate fails loudly when a required stage-0 stdout field disappears.
- [x] (2026-03-29) Added a semantic fixture regression test so the fast
  contract gate fails when the advertised miner port and the live connect port
  diverge.
- [x] (2026-03-29) Added a chain-progress fixture regression test so the fast
  contract gate fails when imported height lags finalized height.
- [x] (2026-03-29) Added an economic fixture regression test so the fast
  contract gate fails when miner emission drops to zero.
- [x] (2026-03-29) Added a discovery-identity fixture regression test so the
  fast contract gate fails when the discovered miner UID is no longer Alice's.
- [x] (2026-03-29) Added a validator-dividend fixture regression test and
  tightened contract assertion messages so economic failures read as stage-0
  contract breaks instead of generic equality panics.
- [x] (2026-03-29) Added a miner-incentive fixture regression test so the fast
  contract gate fails when Alice's incentive is no longer saturated.
- [x] (2026-03-29) Added an advertised-endpoint fixture regression test so the
  fast contract gate fails when the discovered miner endpoint stops advertising
  the wildcard bind address.
- [x] (2026-03-29) Added a live-connect-endpoint fixture regression test so the
  fast contract gate fails when gameplay stops normalizing the connect address
  to localhost.
- [x] (2026-03-29) Added an advice-source fixture regression test so the fast
  contract gate fails when gameplay stops reporting artifact-backed advice.
- [x] (2026-03-29) Added a gameplay-final-state fixture regression test so the
  fast contract gate fails when the reported hand state is no longer complete.
- [x] (2026-03-29) Added an explicit chain-startup fixture regression so the
  harness now fails if finalized height is zero instead of only checking that
  imported height does not lag finalized height.
- [x] (2026-03-29) Added explicit registration-flow fixture regressions so the
  harness now fails if the smoke falls back to the root subnet or if Alice and
  Bob stop converging on distinct registered UIDs.
- [x] (2026-03-29) Added an explicit weight-submission fixture regression so
  the harness now fails if Bob's submitted weights stop targeting Alice's
  discovered miner UID.
- [x] (2026-03-29) Closed the gameplay item on the existing harness surface:
  the fixture and ignored live wrapper already prove artifact-backed advice,
  completed gameplay state, chain-visible miner discovery, and live HTTP miner
  querying from the registered strategy path.

## Surprises & Discoveries

- Observation: the retained downstream plan was stale about the right harness
  layer. The strongest executable proof in the repo today is the node-owned
  `--stage0-local-loop-smoke` path, not a pallet-only `TestExternalities`
  harness.
  Evidence: `crates/myosu-chain/node/src/command.rs` already owns the miner,
  validator, gameplay, and post-epoch proof loop.
- Observation: the new integration wrapper immediately exposed two real
  stage-0 bugs in the smoke command: miner HTTP binding assumed a globally free
  port, and the serving miner subprocess was not being passed the explicit
  `--chain ws://127.0.0.1:9955` endpoint.
  Evidence: `cargo test -p myosu-chain --test stage0_local_loop -- --ignored --nocapture`
  first failed in the `serving_miner_http` step, and the fix landed in
  `crates/myosu-chain/node/src/command.rs`.
- Observation: the best next harness gain was not another live-loop variant,
  but a cheap contract layer that parses smoke stdout into a typed summary and
  lets cargo run one fast assertion target on every ordinary test pass.
  Evidence: `crates/myosu-chain/node/tests/stage0_local_loop.rs` now exposes
  `parse_stage0_summary`, `assert_stage0_contract`, a fixture-backed fast test,
  and the ignored live wrapper.
- Observation: the cheap gate also needed one explicit broken-sample proof so
  missing fields fail as a harness regression rather than remaining an implicit
  helper behavior.
  Evidence: `fixture_stage0_output_missing_required_field_panics` now removes
  `alice_miner_emission` from the sample and asserts the parser panics with the
  missing-key message.
- Observation: semantic contract drift also belongs in the cheap harness, not
  only parser-shape drift.
  Evidence: `fixture_stage0_output_mismatched_miner_ports_panics` now proves
  that a split between the advertised endpoint port and the live connect port
  fails the stage-0 contract without needing the full live loop.
- Observation: chain-progress invariants are another good fit for the cheap
  fixture layer because they are meaningful stage-0 health signals that parse
  cleanly even when broken.
  Evidence: `fixture_stage0_output_imported_below_finalized_panics` now proves
  that `best_imported < best_finalized` fails the contract without needing to
  boot the local devnet.
- Observation: economic invariants fit the same pattern because stage-0 is also
  claiming that submitted weights changed emissions on-chain, not only that the
  loop ran.
  Evidence: `fixture_stage0_output_zero_emission_panics` now proves that a
  parseable zero-emission sample fails the contract without needing a live run.
- Observation: discovery identity is another cheap semantic seam because the
  stage-0 proof is specifically claiming that gameplay rediscovers Alice as the
  rewarded miner, not merely some miner-shaped record.
  Evidence: `fixture_stage0_output_wrong_discovered_miner_uid_panics` now
  proves that a parseable wrong-UID sample fails the contract without a live
  loop.
- Observation: some cheap regression tests were still failing with generic
  `assert_eq!` output, which made the fixture layer less useful as a contract
  surface than it should be.
  Evidence: `assert_stage0_contract` now carries explicit messages for advice
  source, final state, permit, discovered miner UID, weights, incentive, and
  dividend, and `fixture_stage0_output_wrong_validator_dividend_panics` proves
  the more specific dividend failure.
- Observation: miner-side economics deserve their own cheap invariant too, not
  only the validator dividend and aggregate emission surfaces.
  Evidence: `fixture_stage0_output_wrong_miner_incentive_panics` now proves
  that a non-saturated Alice incentive fails the stage-0 contract without a
  live loop.
- Observation: the discovery story is also making a bind-address claim, not
  only a miner-identity claim.
  Evidence: `fixture_stage0_output_wrong_advertised_endpoint_panics` now proves
  that replacing the advertised wildcard bind address with localhost fails the
  contract without a live loop.
- Observation: discovery coherence also includes the normalized dial target,
  not just the chain-visible advertised address.
  Evidence: `fixture_stage0_output_wrong_live_connect_endpoint_panics` now
  proves that replacing the localhost dial target with a wildcard address fails
  the contract without a live loop.
- Observation: gameplay proof is also making a provenance claim about where the
  displayed advice came from, not just whether gameplay completed.
  Evidence: `fixture_stage0_output_wrong_advice_source_panics` now proves that
  replacing the artifact-backed advice source with `live_http` fails the
  contract without a live loop.
- Observation: gameplay completion itself still deserves an explicit cheap
  regression even after advice provenance is covered.
  Evidence: `fixture_stage0_output_wrong_final_state_panics` now proves that
  replacing `complete` with `in_progress` fails the contract without a live
  loop.
- Observation: the same cheap fixture layer can own the remaining startup,
  registration, and weight-routing claims without a second integration stack.
  Evidence: `stage0_local_loop.rs` now parses `subnet`, `alice_uid`, and
  `bob_uid`, asserts positive finalized height plus distinct registered UIDs on
  a non-root subnet, and fails loudly when Bob's weight target drifts away from
  Alice.

## Decision Log

- Decision: Start plan 013 by wrapping the existing node-owned stage-0 smoke in
  an ignored `myosu-chain` integration test instead of jumping straight to a
  new `TestExternalities` harness.
  Rationale: That is the strongest real cross-crate proof surface already
  present, and it catches integration-only drift immediately.
  Inversion: A node-owned harness is slower than pallet-level tests, so it is a
  first slice, not the whole end state.
  Date/Author: 2026-03-29 / Genesis

- Decision: The first harness test lives under `crates/myosu-chain/node/tests/`
  because the workspace is a virtual manifest and the executable proof already
  belongs to the node package.
  Rationale: This keeps the binary-under-test and the integration wrapper in
  the same cargo target while still exercising the full cross-crate loop.
  Date/Author: 2026-03-29 / Genesis

- Decision: Split the stage-0 harness into two layers inside the same test
  target: a fast fixture-backed stdout-contract test and an ignored full live
  run that proves the contract against the actual node/miner/validator/gameplay
  loop.
  Rationale: Most harness drift is contract-shape drift, not node startup
  drift. The typed parser gives cheap coverage while the ignored test preserves
  the full end-to-end proof.
  Date/Author: 2026-03-29 / Genesis

- Decision: Add at least one broken-sample regression test to the cheap
  contract layer instead of relying only on the positive fixture.
  Rationale: The parser is supposed to fail loudly on missing stage-0 keys, and
  that failure mode deserves an explicit harness check.
  Date/Author: 2026-03-29 / Genesis

- Decision: Cover at least one semantic invariant in the cheap fixture layer,
  not just missing or malformed parser fields.
  Rationale: The stage-0 contract includes meaning, not only shape. Endpoint
  port agreement is a real invariant that can regress independently of parser
  strictness.
  Date/Author: 2026-03-29 / Genesis

- Decision: Reuse the cheap fixture layer for chain-progress invariants instead
  of reserving it only for gameplay and endpoint assertions.
  Rationale: The stage-0 stdout contract is also making claims about block
  progress, and those claims can regress independently of the transport and
  parsing surfaces.
  Date/Author: 2026-03-29 / Genesis

- Decision: Reuse the cheap fixture layer for economic invariants as well,
  rather than treating emissions as something only the live wrapper should
  verify.
  Rationale: Stage-0 truth includes economic consequences. Zero emission is a
  meaningful contract break even when the output remains well-formed.
  Date/Author: 2026-03-29 / Genesis

- Decision: Reuse the cheap fixture layer for discovery-identity invariants,
  not only transport, progress, and economics.
  Rationale: A stage-0 success report is making an identity claim about which
  miner gameplay rediscovered, and that claim can drift independently of the
  other fields.
  Date/Author: 2026-03-29 / Genesis

- Decision: Prefer explicit contract-failure messages inside the cheap fixture
  layer over generic equality panics when the invariant is part of the stage-0
  story.
  Rationale: The harness is becoming a human-readable contract, not just a bag
  of assertions. Failures should explain which stage-0 claim regressed.
  Date/Author: 2026-03-29 / Genesis

- Decision: Cover miner-side saturated incentive separately from validator-side
  saturated dividend.
  Rationale: The stage-0 economic story has two distinct winners, and either
  side can regress independently.
  Date/Author: 2026-03-29 / Genesis

- Decision: Cover advertised-endpoint semantics separately from discovered UID.
  Rationale: The stage-0 discovery story includes both "who was found" and
  "what chain-visible endpoint they advertise," and those can drift
  independently.
  Date/Author: 2026-03-29 / Genesis

- Decision: Cover live-connect endpoint semantics separately from the
  advertised endpoint semantics.
  Rationale: The stage-0 proof is also claiming that gameplay normalizes the
  dial target to localhost for the local loop, and that can regress
  independently of the advertised chain-visible address.
  Date/Author: 2026-03-29 / Genesis

- Decision: Cover gameplay advice provenance separately from gameplay
  completion.
  Rationale: Stage-0 is claiming not just that a hand completes, but that the
  advice shown in the proof comes from the artifact-backed path.
  Date/Author: 2026-03-29 / Genesis

- Decision: Keep gameplay completion state as its own cheap invariant, even
  after adding advice provenance.
  Rationale: The proof needs both "where advice came from" and "the hand
  actually completed," and either one can drift independently.
  Date/Author: 2026-03-29 / Genesis

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Fast contract layer | Smoke stdout shape drifts silently | Parse required keys into `Stage0LoopSummary` and fail on missing/invalid fields |
| Full-node test | Node fails to start in CI or on a busy operator machine | Keep full-node test as `#[ignore]`; use dynamic local miner HTTP port |
| Miner subprocess | Serving miner dials the wrong chain endpoint | Pass explicit `--chain ws://127.0.0.1:9955` from the smoke owner |
| Emission verification | Floating-point comparison fails | Use approx::assert_relative_eq with epsilon from INV-003 |
| Port contract | Live query endpoint hardcodes a global port | Assert advertised and connect endpoints agree on port, not a literal `8080` |

## Outcomes & Retrospective

Plan 013 is now in motion on the right surface instead of the idealized one.
The first harness slice did not just add a test file; it turned the existing
stage-0 smoke into an integration target and immediately found two real issues
in the owned proof path. After fixing the dynamic miner HTTP port selection and
the missing explicit chain endpoint for the serving miner subprocess, the
ignored integration wrapper passed end to end. The second slice then extracted
the emitted stdout contract into a reusable typed parser plus a fast
fixture-backed test, so ordinary cargo verification now checks the contract
shape without requiring a full local node run every time. The next slice made
that cheap gate less trusting by adding an explicit broken-sample test for a
missing required field, so contract regression is now part of the fast harness
rather than only implied by helper internals. The next step after that pushed a
semantic invariant into the same cheap layer: mismatched advertised/connect
miner ports now fail the fixture-backed contract gate without needing to start
the local devnet. The next slice extended the same pattern to chain progress:
`best_imported < best_finalized` is now also a cheap regression instead of a
truth that only the ignored live wrapper is expected to protect. The next slice
did the same for economics: zero miner emission now fails the cheap fixture
contract, so the fast harness is no longer only about structure, transport, and
progress, but also about one of the core post-weight outcomes. The next slice
added discovery identity to the same fast surface, so the fixture layer now
guards the specific claim that gameplay rediscovers Alice as the winning miner
instead of only guarding transport and economics around that discovery. The next
slice then tightened the readability of that fixture contract by replacing some
generic equality failures with explicit stage-0 messages and adding a saturated
validator-dividend regression to the cheap economic layer. The next slice
completed the other side of that economic picture by adding a saturated
miner-incentive regression, so the cheap fixture layer now guards both the
miner and validator winners the stage-0 loop is supposed to produce. The next
slice returned to discovery coherence and added an advertised-endpoint
regression, so the cheap fixture layer now guards not only which miner
gameplay rediscovers, but also what address that rediscovered miner is allowed
to claim on-chain. The next slice completed the other half of that address
story by adding a live-connect-endpoint regression, so the cheap layer now
guards both the chain-visible advertised address and the normalized localhost
dial address that gameplay is supposed to use in the local loop. The next slice
then tightened gameplay provenance by adding an advice-source regression, so
the cheap layer now guards not only that gameplay completed, but that the proof
is still artifact-backed. The next slice made gameplay completion explicit
again with a wrong-final-state regression, so the cheap layer now guards both
the provenance and the terminal state of the gameplay proof.

The final slice closed the remaining plan-shaped gap without adding a new
harness family. The same node-owned contract test now carries explicit startup,
registration, weight-submission, and gameplay invariants on the cheap fixture
layer, while preserving the ignored live smoke wrapper for the full
chain/miner/validator/gameplay proof. On this machine the plain node test path
is still gated by the missing `wasm32-unknown-unknown` target, but the honest
repo proof path is green under `SKIP_WASM_BUILD=1`, matching the stripped
runtime verification route already used elsewhere in stage-0 execution.

## Context and Orientation

```text
INTEGRATION TEST FLOW

  fixture stdout
       |
       v
  parse_stage0_summary()
       |
       v
  assert_stage0_contract()
       |
       v
  cheap cargo test gate

  ignored live wrapper
       |
       v
  myosu-chain --stage0-local-loop-smoke
       |
       v
  node -> miner -> gameplay -> validator -> epoch outputs
       |
       v
  parse_stage0_summary()
       |
       v
  assert_stage0_contract()
```

Current harness file:
- `crates/myosu-chain/node/tests/stage0_local_loop.rs` -- ignored node-owned
  integration wrapper plus fast positive, missing-field, mismatched-port,
  imported-below-finalized, zero-emission, and wrong-discovered-miner-uid
  stage-0 contract tests, plus validator-dividend, miner-incentive, and
  advertised-endpoint, live-connect-endpoint, advice-source, and final-state
  regressions and clearer contract failure messages

## Milestones

### Milestone 1: Test infrastructure

Create the first integration wrapper around the existing node-owned
`--stage0-local-loop-smoke` command so the full stage-0 contract is executable
 from `cargo test`.

Proof command:

    cargo test -p myosu-chain --test stage0_local_loop --no-run

### Milestone 2: Node-owned full-loop wrapper

Run the ignored integration wrapper and assert the stage-0 stdout contract:
successful miner discovery, completed gameplay hand, validator permit/weights,
and positive post-epoch economics.

Proof command:

    cargo test -p myosu-chain --test stage0_local_loop -- --ignored --nocapture

### Milestone 3: Cheap contract gate

Keep a fast non-ignored test that parses a representative stage-0 stdout sample
into `Stage0LoopSummary` and asserts the contract without starting the local
devnet.

Proof command:

    cargo test -p myosu-chain --test stage0_local_loop --quiet

### Milestone 4: Cheap regression failure

Keep at least one non-ignored broken-sample test that removes a required field
from the stage-0 stdout fixture and proves the parser fails loudly.

Proof command:

    cargo test -p myosu-chain --test stage0_local_loop --quiet

### Milestone 5: Cheap semantic invariant

Keep at least one non-ignored fixture test that preserves parseability but
breaks a real stage-0 invariant, such as advertised/connect miner port
agreement.

Proof command:

    cargo test -p myosu-chain --test stage0_local_loop --quiet

### Milestone 6: Cheap chain-progress invariant

Keep at least one non-ignored fixture test that preserves parseability but
breaks a chain-progress claim, such as `best_imported < best_finalized`.

Proof command:

    cargo test -p myosu-chain --test stage0_local_loop --quiet

### Milestone 7: Cheap economic invariant

Keep at least one non-ignored fixture test that preserves parseability but
breaks an economic claim, such as zero miner emission after the loop reports
success.

Proof command:

    cargo test -p myosu-chain --test stage0_local_loop --quiet

### Milestone 8: Cheap discovery identity

Keep at least one non-ignored fixture test that preserves parseability but
breaks a discovery-identity claim, such as returning the wrong discovered miner
UID.

Proof command:

    cargo test -p myosu-chain --test stage0_local_loop --quiet

### Milestone 9: Cheap validator economics

Keep at least one non-ignored fixture test that preserves parseability but
breaks validator-side economics, such as a non-saturated dividend after the
loop reports success.

Proof command:

    cargo test -p myosu-chain --test stage0_local_loop --quiet

### Milestone 10: Cheap miner economics

Keep at least one non-ignored fixture test that preserves parseability but
breaks miner-side economics, such as a non-saturated miner incentive after the
loop reports success.

Proof command:

    cargo test -p myosu-chain --test stage0_local_loop --quiet

### Milestone 11: Cheap advertised endpoint

Keep at least one non-ignored fixture test that preserves parseability but
breaks the advertised-endpoint claim, such as replacing the wildcard bind
address with localhost.

Proof command:

    cargo test -p myosu-chain --test stage0_local_loop --quiet

### Milestone 12: Cheap live connect endpoint

Keep at least one non-ignored fixture test that preserves parseability but
breaks the normalized localhost dial target, such as replacing it with the
wildcard address.

Proof command:

    cargo test -p myosu-chain --test stage0_local_loop --quiet

### Milestone 13: Cheap advice provenance

Keep at least one non-ignored fixture test that preserves parseability but
breaks the advice provenance claim, such as changing the gameplay advice source
from `artifact` to `live_http`.

Proof command:

    cargo test -p myosu-chain --test stage0_local_loop --quiet

### Milestone 14: Cheap gameplay final state

Keep at least one non-ignored fixture test that preserves parseability but
breaks the gameplay terminal-state claim, such as changing `complete` to
`in_progress`.

Proof command:

    cargo test -p myosu-chain --test stage0_local_loop --quiet

## Plan of Work

1. Wrap the existing node-owned stage-0 smoke in an integration test target.
2. Extract the stdout contract into a typed parser with a cheap fixture-backed gate.
3. Add broken-sample regression coverage to that cheap gate.
4. Add semantic invariant coverage to the cheap fixture layer.
5. Add chain-progress invariant coverage to the cheap fixture layer.
6. Add economic invariant coverage to the cheap fixture layer.
7. Add discovery-identity coverage to the cheap fixture layer.
8. Add validator-economic coverage and clearer contract messages to the cheap
   fixture layer.
9. Add miner-economic coverage to the cheap fixture layer.
10. Add advertised-endpoint coverage to the cheap fixture layer.
11. Add live-connect-endpoint coverage to the cheap fixture layer.
12. Add advice-provenance coverage to the cheap fixture layer.
13. Add gameplay-final-state coverage to the cheap fixture layer.
14. Keep tightening the stdout contract so the harness asserts real behavior.
15. Decide later whether a pallet-level deterministic layer is still needed
    after the node-owned harness is stable.

## Concrete Steps

From `/home/r/coding/myosu`:

    cargo test -p myosu-chain --test stage0_local_loop --no-run
    cargo test -p myosu-chain --test stage0_local_loop --quiet
    cargo test -p myosu-chain --test stage0_local_loop -- --ignored --nocapture

## Validation and Acceptance

Accepted when:
- Integration test compiles and passes
- Full loop harness exercises: node bootstrap -> subnet registration -> miner
  -> validator -> gameplay -> post-epoch economics
- Fast contract test catches stdout-contract drift without starting the node
- Fast regression test proves missing required fields fail loudly
- Fast semantic regression test proves advertised/connect miner port drift fails
- Fast chain-progress regression test proves imported/finalized drift fails
- Fast economic regression test proves zero miner emission fails
- Fast discovery-identity regression test proves wrong miner UID fails
- Fast validator-economic regression test proves wrong dividend fails
- Fast miner-economic regression test proves wrong incentive fails
- Fast advertised-endpoint regression test proves wrong bind address fails
- Fast live-connect-endpoint regression test proves wrong dial target fails
- Fast advice-provenance regression test proves wrong advice source fails
- Fast gameplay-state regression test proves wrong final state fails
- CI includes integration test job

## Idempotence and Recovery

The fast contract layer is deterministic by construction. The ignored live
wrapper owns its own local processes and temporary state; re-run is safe, but
it is intentionally heavier than the fixture-backed gate.

## Interfaces and Dependencies

Depends on: 004 (devnet chain spec provides genesis template), 005 (pallet surface is stable), 007 (miner/validator crates exist for API verification).
Blocks: 011 (security audit needs passing integration tests).

```text
stage0_local_loop.rs
  ├── fixture stdout -> parse -> contract assert
  └── ignored live smoke -> parse -> contract assert
```
