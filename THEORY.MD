# Theory: Stage-0 Chain Truth Through Node-Owned Proof

## Problem

Myosu had drifted toward describing progress above the actual bottleneck. The
repo had stronger docs, better planning, and a usable local poker wedge, but
stage 0 in `OS.md` is not satisfied by any of those things. Stage 0 ends only
when a stripped chain, a miner, a validator, and gameplay all connect as one
truthful loop.

That makes the real risk very specific: the repo can start sounding like a
distributed-compute product before it actually has one. The only antidote is to
keep cutting inherited chain surface, prove every participant seam on a live
devnet, and update the repo narrative whenever the live chain contradicts the
documentation.

## Operating Theory

The stage-0 path is now clearer than it was at the start of the refocus. The
runtime and pallet reduction work did what it needed to do: remove obviously
wrong inherited surfaces, keep the node compiling, and get the local chain to
author blocks and finalize. That destructive simplification still matters,
because every participant proof depends on the chain being smaller and more
legible than the original subtensor carry.

The newest reduction is subtle but important: the stripped runtime and node no
longer export the old swap runtime API shell at all. Stage 0 still keeps the
noop swap contract internally because the pallet prices stake and emissions
through that seam, but the live node contract no longer advertises
`current_alpha_price` or `sim_swap_*` endpoints that the surviving chain
client, miner, validator, and gameplay loop never call. That is the right
shape for this phase: keep the minimum internal seam that the pallet still
needs, while deleting outward compatibility surfaces that only preserve the
old network story.

The next reduction closed the last real Frontier/EVM chapter in the runtime
and node. The stripped chain no longer carries `pallet_evm_chain_id`, the old
admin-only `sudo_set_evm_chain_id` hook, or the node's empty
`ethereum`/`EthConfiguration` placeholder threading. The remaining EVM-named
symbol in the runtime is now just the pallet's own EVM-key association rate
limit, which is a stage-0 gameplay/account-linking seam rather than leftover
Frontier infrastructure. That is a much healthier boundary: the runtime and
node stop pretending to be an Ethereum hybrid chain, while the pallet keeps
only the narrow account-association behavior it still actually uses.

The next strip was smaller but still useful: the runtime no longer carries the
dead `Registry` or `Contracts` pallets, their config blocks, or the contracts
runtime API. Those surfaces were still consuming code and metadata space, but
they were not part of the live node/client/miner/validator stage-0 proof. The
remaining runtime mismatch was now clearer instead of blurrier, and that made
the next cut obvious. `MevShield` turned out not to be a real stage-0
dependency after all; it was being kept alive only by a coldkey-swap guard
exception for shielded `submit_encrypted` calls plus a node-side no-op
author/revealer scaffold. Once that exception and placeholder plumbing were
removed, the runtime and node both still compiled and the focused guard tests
still passed. That leaves `AdminUtils` as the only intentionally carried extra
runtime seam, and that carry is honest because the bootstrap flow still uses
its sudo subnet-normalization hooks. In other words, the runtime-reduction
lane is no longer "find more hidden baggage"; it is "treat `003` as closed and
keep future chain work honest about the one remaining bootstrap helper it
still needs."

The participant loop is no longer hypothetical. `myosu-chain-client` is a real
shared seam for RPC, storage reads, and signed extrinsics. `myosu-miner` can
train a bounded checkpoint, answer one wire query, register on-chain, and
publish a devnet-safe axon record. `myosu-validator` can score the same saved
artifacts, prove deterministic local scoring, register on-chain, and drive the
first on-chain weight path.

The important update is that the first on-chain proof changed shape once we
stopped trusting subnet `1` as the canonical bootstrap surface. The earlier
self-weight proof on subnet `1` was real, but later inspection showed that the
subnet itself had stale bootstrap state: its owner read back as the zero
account, its subtoken flag was disabled, and owner-side bootstrap actions did
not behave like a clean local subnet should. That means subnet `1` is still
useful as archaeological evidence that commit-reveal once worked, but it is no
longer the right operator truth for current stage-0 bring-up.

The cleaner proof now lives on a fresh subnet. Creating subnet `2` on the live
devnet restored the expected control flow: Alice can own the subnet, `start_call`
can enable subtoken/staking, sudo tempo override works, Bob can register, stake,
acquire validator permit, and commit-reveal a real Bob -> Alice weight vector.
The chain now reads back `bob_weights=[(0, 65535)]` on that clean subnet. This
changes the confidence picture materially. The permit path is no longer an
unproven future seam; it is live on clean subnet state and reaches the actual
on-chain weight store.

The latest shift is about where that proof lives. The repo now has a node-owned
stage-0 local-loop smoke path instead of relying entirely on a handwritten
playbook. `myosu-chain --stage0-local-loop-smoke` boots a dedicated local node
on isolated ports, creates a fresh subnet, generates poker bootstrap artifacts,
runs the miner bootstrap, runs the owner-side subnet normalization, and then
hands off to the Bob validator path. That is the right shape for stage-0 truth:
the chain executable itself owns the proof of the full local loop.

The latest stage-0 blocker turned out not to be a fresh-subnet pallet bug after
all. It was a shared-client truth bug. `myosu-chain-client` was reading an
absent `WeightsSetRateLimit` storage key as `0`, while the pallet's real
default for that map is `100`. On a brand-new smoke subnet, the owner-side
bootstrap therefore concluded "already set to zero" and skipped the sudo write,
then Bob hit the carried default during the first post-registration
commit-reveal submission. Once that default mismatch was fixed in the shared
client, the same node-owned smoke path completed end to end and wrote
`bob_weights=[(0, 65535)]` on the fresh subnet.

The newest improvement is that the local-loop proof no longer stops at weight
storage. The shared client can now read post-epoch incentive, dividend, and
emission vectors directly from chain state, and the node-owned smoke waits for
those values to move. On the fresh subnet, the same Bob -> Alice write now
proves more than acceptance: Alice receives positive miner incentive and
emission, and Bob receives positive validator dividends. That is the first
honest local evidence that the chain is reacting to the participant loop as an
economic system, not just recording it.

The next useful change was to stop treating gameplay as a separate manual
appendix to that proof. `myosu-play` already had a smoke path and artifact
loading, but its proof still allowed a generated fallback and only advanced one
street. The tighter stage-0 version now requires artifact-backed advice when
asked, completes a full demo hand, and gets executed from the same
`myosu-chain --stage0-local-loop-smoke` flow using the checkpoint the miner
just wrote. That means the local chain proof no longer ends at miner/validator
economics. It now reaches a human-facing poker hand driven by trained
artifacts from the same bootstrap loop.

The next refinement was about truth timing, not more surface area. `myosu-play`
can now ask the chain who the best visible miner is on the target subnet, but
the first attempt to require that during gameplay smoke failed honestly: before
the Bob -> Alice weight write had produced an epoch outcome, the clean subnet
still had no nonzero incentive winner. The right answer was not to weaken
discovery. It was to move the gameplay smoke later in the node-owned loop so
miner discovery happens only after the chain has a truthful incentive leader.
Now the same executable proof reports both `gameplay_final_state=complete` and
`gameplay_discovered_miner_uid=0`, which means gameplay is no longer consuming
artifacts in isolation from chain ranking.

The next resolved seam was to stop letting gameplay discovery end at chain
state. `myosu-miner` now has a small checkpoint-backed HTTP axon with
`GET /health` and `POST /strategy`, and `myosu-play` uses the discovered miner
endpoint to execute one real health check plus one real wire query. The local
devnet still cannot advertise `127.0.0.1`, so the chain-visible endpoint stays
`0.0.0.0:8080` while gameplay normalizes that to `127.0.0.1:8080` for the
actual local connection. That keeps INV-004 intact: gameplay still depends on
chain-visible miner state plus a network boundary, not on miner internals.

That seam exposed one more truthful integration bug. The new HTTP axon passed a
TCP readiness check, then died before gameplay queried it because the bare
probe socket triggered a broken-pipe write on the server's 400 response path.
Treating response-write failures as non-fatal fixed that without weakening the
proof. The node-owned smoke now reaches all the way through live miner query,
not just artifact-backed local rendering.

The next small product move is to let that live query affect visible gameplay,
not just smoke reporting. `myosu-play` pipe mode now refreshes the current
state against the discovered miner and overlays the returned recommendation onto
each printed state line. That is intentionally narrower than rewriting the full
TUI render loop: it keeps the network seam honest where the interface is
already line-oriented, while avoiding synchronous miner calls during ratatui
paint until that path can be designed more carefully.

The important verification outcome is that this gameplay-side refinement did
not destabilize the higher-order proof. Re-running
`myosu-chain --stage0-local-loop-smoke` after the pipe-mode change still
completed on a fresh subnet with the same successful miner discovery,
artifact-backed gameplay hand, Bob -> Alice weight vector, and positive
post-epoch incentive, dividend, and emission outputs. That matters because the
repo is now carrying two claims at once: gameplay can consume live miner truth,
and the node-owned stage-0 proof remains the non-negotiable source of truth.

The next gameplay refinement follows the same rule, but for the interactive
path instead of the pipe path. `myosu-play train` now keeps a cached live
recommendation inside `NlheRenderer` and refreshes it from a background task
owned by `myosu-play`, rather than trying to make the ratatui paint loop do
network work. The point is not just responsiveness. It is boundary honesty:
the TUI can show chain-visible live advice, but the renderer and shell still
behave like local synchronous UI components.

That change briefly looked riskier than it was. The first full smoke rerun
timed out waiting for the miner HTTP axon on `127.0.0.1:8080`, which initially
looked like a regression from the gameplay-side refactor. A direct replay of
`myosu-miner --serve-http` against the staged checkpoint and encoder artifacts
showed the server still binding `0.0.0.0:8080` and reporting its HTTP axon
surface normally. The next node-owned smoke rerun then completed cleanly,
which shifts the interpretation away from "the TUI change broke miner serving"
and toward "the harness can occasionally miss a ready localhost axon on the
first attempt."

The next correction is smaller but more user-facing. Cached live advice is only
useful if it tells the truth about its own freshness. `myosu-play train` no
longer silently drops the live hint when background refresh fails. Instead, the
renderer distinguishes three states: fresh advice, stale advice that preserves
the last good action, and explicit offline state before any live recommendation
has landed. That keeps the product claim honest for a human at the keyboard:
the system is not just "showing some action," it is telling you whether that
action still reflects a reachable chain-visible miner.

The next refinement makes that freshness concrete instead of categorical.
Interactive live advice now reports a lightweight age in seconds alongside the
fresh/stale label. A successful refresh shows `FRESH 0s`; a failed refresh
preserves the last good action and shows how long it has been since the miner
last answered successfully. That matters because "stale" by itself is vague.
An operator or player can now tell the difference between a brief hiccup and a
much older recommendation without leaving the game surface.

The next usability correction is about visibility rather than state modeling.
Freshness and age in the side panel are useful, but they are easy to miss if a
player is focused on commands and transcript flow. `myosu-play train` now logs
live-advice connectivity transitions into the shell transcript itself: first
offline state before any successful refresh, stale state after a failed refresh
following success, and recovery when the discovered miner answers again. That
keeps the live-advice claim honest in the place users already watch for game
feedback, while preserving the background-refresh architecture that keeps HTTP
out of the ratatui paint loop.

The next refinement is still about observability, but with a sharper operator
angle. Generic "offline" and "fresh" transcript messages confirm that state
changed, but they do not tell the operator enough to act. The transcript now
includes the discovered miner connect endpoint and recovered action on success,
and it carries the first failure reason that caused an offline or stale
transition. That keeps the no-spam state-transition model intact while making
the transcript actually diagnostic when the live advice seam degrades.

The same truthfulness rule also applies to the agent-facing pipe surface.
Before this slice, pipe mode either overlaid a live recommendation or quietly
fell back to the artifact-backed renderer, but the printed state line did not
say which had just happened. Pipe output now carries live-query metadata when
discovery is active: successful refreshes append the live connect endpoint and
action, while fallback states append the live failure detail. That matters
because agents consume those state lines directly; the system should not force
them to infer whether the last recommendation was live or fallback from
surrounding context.

The final cleanup on that seam is small but worth making explicit. A missing
`live_query` field still leaves one ambiguous case for agents: was live query
disabled, or did the program simply forget to report it? Pipe mode now always
states the truth, including `live_query=not_requested` when discovery/live HTTP
was never in play. That turns the pipe surface into a stable contract instead
of a sparse hint stream, which is the right shape for agent-native consumers.

There was one more mismatch left between the pipe contract and the stronger
smoke/reporting contract. Pipe success states told an agent which action won,
but not how much strategy surface had actually been returned or which edge the
solver preferred before UI mapping. Live-backed pipe states now include the
action count and recommended edge alongside the connect endpoint and final
action. That keeps the pipe surface compact, but it means an agent can reason
from the same core live-query payload the smoke proof already exposes instead
of treating pipe mode as a lower-fidelity shadow.

The next small correction is about endpoint truth. On localhost, the miner has
two relevant addresses: the chain-visible advertised endpoint and the normalized
endpoint that gameplay can actually dial. Pipe success states now expose both.
That matters because an agent should not have to cross-reference discovery
output or separate smoke lines to understand why the miner is published as
`0.0.0.0:8080` but queried at `127.0.0.1:8080`. The pipe surface now carries
that distinction directly.

The next hardening slice comes from the remaining genesis-plan doctrine rather
than from new user-facing functionality. Plan 006 now has its first executable
proofs instead of pure intent. A workspace invariant test shells out to
`cargo tree` and proves the literal INV-004 rule: `myosu-play` does not depend
on `myosu-miner`, and `myosu-miner` does not depend on `myosu-play`. The repo
also now carries a local robopoker fork changelog tied to the exact pinned rev
used in the workspace, which finally makes INV-006 inspectable instead of
implicit.

That slice also exposed an important boundary nuance we should keep saying out
loud. The literal gameplay-miner separation invariant is currently true, but
the stricter "gameplay shares only game crates" design target is not yet
finished. `myosu-play` still depends on `myosu-chain-client`, which drags a
large chain/runtime surface into the gameplay binary. That is not a reason to
block the invariant proof we can already make, but it is a reason not to claim
plan 006 is complete until that seam is narrowed further.

The next small plan-006 step is similarly narrow and factual. The public API
audit shows that the likely growth points are enum variants, not struct
construction. `GameType` and `GameParams` are now marked `#[non_exhaustive]`,
which locks downstream consumers into wildcard-safe matching without inventing
new abstractions. The repo also now has an explicit custom-game extensibility
proof instead of relying on an incidental unknown-bytes test: a named
`custom_game` path in `myosu-games` proves that `GameType::Custom(...)` can be
described, byte-round-tripped, and treated as non-builtin without editing the
registry's built-in table.

That leaves one last honest plan-006 question: whether the wire surface really
has edge coverage, not just roundtrip coverage. It does now. The poker wire
tests explicitly cover empty strategy responses, zero-probability edges,
maximum-width info keys, trailing-byte rejection, and truncated payload
rejection, and the shared game-surface tests now include a unicode custom-game
roundtrip. That is enough to say the boundary plan is complete at its intended
scope. It is not enough to claim the artifact/wire trust boundary is hardened in
full, which is why the next work should move into plan 008 rather than quietly
stretching plan 006 beyond its job.

The first plan-008 correction is already more specific than the old plan text.
Checkpoint versioning was not actually missing anymore; `solver.rs` already had
`MYOS` magic and explicit version rejection. The real missing guardrail was
decode sizing. The trust boundary now uses a 256MB cap on the bincode decode
paths in `wire.rs`, `artifacts.rs`, and `solver.rs`, and each module has a
focused regression test that proves over-budget payloads are rejected by the
decode machinery. That does not finish artifact trust, but it does close the
easiest malformed-input/OOM avenue without conflating it with the still-open
manifest-hash and mmap-bounds work.

The next plan-008 slice made the codexpoker mmap surface more honest too. The
problem was not memory mapping by itself; it was trusting key-record offsets and
lengths to point to valid value payloads. A truncated values file could
previously load and only surface later as a quiet lookup miss. `codexpoker.rs`
now validates those ranges during blueprint load, checks that each pointed-to
record has the right internal byte length for its declared action count, and
fails fast with `MalformedValues` if the file is truncated or internally
inconsistent. That moves the remaining plan-008 truth to artifact-hash audit
cleanup plus broader malformed-input fuzz coverage.

The artifact-hash audit then narrowed further. The runtime-facing crates were
already on the right path: miner, validator, and gameplay all use
`load_encoder_dir()`, which goes through manifest-backed SHA-256 verification.
The stale part of the theory was thinking there was still an active optional
verification callsite to switch on. There was not. The real cleanup was API
shape, so `myosu-games-poker` stopped exporting the unused raw single-file
encoder loader and the manifest parser. That leaves plan 008 with one honest
remainder: property-style malformed-input coverage rather than more boundary
surface tightening.

That remainder is closed now. The final plan-008 slice used property tests in
`wire.rs`, `artifacts.rs`, and `solver.rs`, but the important choice was not
"throw random bytes at everything." The stronger shape was to start from valid
payloads, then truncate or tamper with them and assert that the decode/load
surface returns `Err` without unwinding. That matches the actual failure model
better than raw-noise fuzzing and gives stage-0 the honest claim it needed:
`myosu-games-poker` now has size caps, manifest-backed hash checks, codexpoker
range validation, checkpoint headers, and malformed-input properties across the
remaining decode boundaries. Plan 008 is done at its intended scope.

Plan 013 then started from a similar correction: the retained plan was aiming
at an idealized `TestExternalities` harness, but the strongest real end-to-end
proof surface in the repo today is the node-owned
`myosu-chain --stage0-local-loop-smoke` path. Wrapping that command in an
ignored `myosu-chain` integration test was the right first move because it
turned an executable operator proof into a cargo-managed harness without
pretending we already had a pallet-level deterministic story. More importantly,
it paid for itself immediately. The first run exposed two real integration bugs
that the manual proof had not forced into the open: the smoke owned a global
miner HTTP port, and the serving miner subprocess was accidentally probing the
default chain endpoint instead of the smoke's explicit `ws://127.0.0.1:9955`.
Fixing those inside `command.rs` made the underlying stage-0 proof more honest
for both operators and tests. The current theory is that plan 013 should grow
outward from this node-owned harness, tightening its stdout contract and only
adding a separate pallet-level deterministic layer later if the remaining risks
justify the extra surface.

The next harness improvement made that "tightening the contract" idea real
instead of rhetorical. The important drift to catch every day is not usually
"can this machine boot a full local loop right now?" but "did the stage-0
stdout contract quietly change underneath the harness?" Extracting
`Stage0LoopSummary`, `parse_stage0_summary`, and `assert_stage0_contract` from
the ignored wrapper gave plan 013 a cheap non-ignored layer that parses a
representative fixture and locks the contract shape in normal cargo runs. The
ignored live wrapper still matters because it proves the node/miner/validator/
gameplay path end to end, but now the harness has an honest two-tier shape: a
fast contract gate for routine verification and a heavy executable proof for
full-stage assertions.

That two-tier shape was still slightly too trusting until the cheap gate grew a
broken-sample proof. A positive fixture alone can show what "good" output looks
like, but it does not prove that the parser actually fails loudly when the
contract degrades. Adding an explicit missing-field regression test changed the
theory from "the helpers seem strict" to "the fast harness owns at least one
real contract-break scenario." The practical strategy inside plan 013 is now to
keep pushing cheap negative cases into the fixture layer whenever they defend
the stage-0 contract more directly than another expensive full-loop variant.

The next refinement clarified that "negative case" should not mean only parser
shape failures. Some of the most useful stage-0 regressions are semantic: the
output still parses, but it no longer means the loop is healthy. Pushing a
mismatched advertised/connect miner port case into the fast harness changed the
cheap layer from "strict parser plus one broken field" into "strict parser plus
at least one real invariant." That is the right pattern for the remainder of
plan 013: use the ignored live wrapper to discover important truths, then pull
the highest-value contract and invariant failures down into cheap fixture tests
where they can guard ordinary development.

The same pattern now extends cleanly to chain progress. `best_imported` and
`best_finalized` are not just extra counters in the smoke output; they are part
of the claim that the local loop is actually moving forward coherently. Adding a
cheap `best_imported < best_finalized` failure case sharpened the fixture layer
again: the fast harness is no longer only defending parser strictness and
network-endpoint semantics, but also one of the loop's basic liveness/progress
claims. That suggests the right remaining strategy for plan 013 is to keep
harvesting stage-0 truths from the live wrapper and promote the clearest ones
into cheap fixture regressions until the expensive path is mostly discovery and
final proof, not the only place where meaning is tested.

That promotion path now reaches the economic layer too. The stage-0 loop is not
done when it merely boots, queries, and reports endpoints; it is trying to
prove that validator weights changed emissions in a meaningful way. Pulling a
zero-emission failure case into the cheap harness tightened the theory again:
the fixture layer is becoming a compact executable statement of what stage-0
success means across structure, semantics, progress, and economics. If plan 013
continues cleanly, the ignored live wrapper should increasingly serve as the
place where new truths are discovered, while the fixture layer becomes the
place where those truths stay defended in ordinary development.

The same logic applies to identity. The stage-0 loop is not just proving that a
miner endpoint exists and that emissions are positive; it is proving that the
miner rediscovered through gameplay is the same chain-visible winner the rest of
the loop is talking about. Pulling a wrong-discovered-miner-UID case into the
cheap harness tightened the fixture layer one more step: it now guards not only
the shape and consequences of success, but also who success belongs to. That is
the current operating pattern for plan 013: keep using the live wrapper to
surface the highest-value truths, then ratchet those truths downward into cheap
fixture regressions until everyday development is guarded by a small but
meaningful executable statement of stage-0 success.

One more refinement became necessary as that fixture layer grew: the cheap
contract should fail like a contract, not like an arbitrary unit test. Some of
the remaining economic and identity assertions were still surfacing as generic
equality panics, which made regressions harder to read than they needed to be.
Tightening those messages and then adding a validator-dividend regression moved
the fixture layer closer to its real role: a concise executable explanation of
what stage-0 success means. The live wrapper still discovers the truths, but
the cheap layer is increasingly where those truths become legible and durable.

The economic side of that fixture contract is now splitting the way the actual
stage-0 claim splits: miner reward and validator reward are related, but they
are not the same invariant. Adding a dedicated wrong-miner-incentive regression
completed that distinction inside the cheap layer. The harness is therefore
becoming less like a generic smoke transcript parser and more like a compact
economic and identity model of what the live wrapper is supposed to prove. That
keeps plan 013 honest: the expensive path remains the place where truths are
first discovered, while the fixture layer accumulates those truths in a form
that ordinary development can guard continuously.

Discovery itself also turned out to have two layers: the winning miner's
identity and the winning miner's advertised address. Pulling a wrong
advertised-endpoint case into the cheap harness sharpened that distinction.
Stage-0 success is not just "gameplay found Alice" but "gameplay found Alice
through the same chain-visible endpoint story the rest of the loop is using."
That makes the fixture layer a little less abstract and a little more faithful
to the actual operator proof: who was found, how they are reached, what
economic consequences follow, and whether the chain stayed coherent while doing
it.

There was still one local-loop-specific part of that reachability story left to
make explicit: the difference between the chain-visible advertised endpoint and
the normalized localhost dial target that gameplay actually uses. Adding a
wrong-live-connect-endpoint regression pulled that final piece down into the
cheap layer. The fixture harness is now close to a compact executable version
of the full stage-0 address model: which miner wins, what address the chain
advertises for that miner, and what address the local gameplay surface is
supposed to dial in practice.

The gameplay proof also had one provenance dimension left to pin down. Stage-0
is not just claiming that the hand completes or that a miner can be reached; it
is claiming that the advice shown in the proof is still artifact-backed. Adding
a wrong-advice-source regression pulled that claim into the cheap layer too.
That makes the fixture harness less like a loose transcript checker and more
like a compact statement of stage-0 provenance: who was found, how they are
reached, what economics followed, and where the gameplay advice came from.

Completion itself still needed to be made explicit after that provenance work.
Adding a wrong-final-state regression pulled the last obvious gameplay claim
down into the cheap layer: not just where advice came from, but that the hand
really reached its terminal state. At this point the fixture harness is close to
an executable summary of the local stage-0 promise rather than a partial smoke
proxy. That suggests plan 013 is nearing diminishing returns on more fixture
cases, and the next leverage may be moving outward again rather than continuing
to deepen the same layer forever.

The remaining gap in that harness turned out not to require a second test
family either. The same cheap fixture layer could still absorb the missing
startup, registration, and weight-routing claims if it simply parsed a little
more of the existing stdout contract. `Stage0LoopSummary` now carries `subnet`,
`alice_uid`, and `bob_uid`, and the contract explicitly fails when finalized
height is zero, when the loop falls back to the root subnet, when Alice and Bob
stop occupying distinct registered UIDs, or when Bob's weights stop pointing at
Alice's discovered miner UID. That closes plan 013 on the honest surface: the
ignored live wrapper still proves the full chain/miner/validator/gameplay loop,
while ordinary development gets a fast executable contract covering startup,
registration, weights, economics, discovery, provenance, and gameplay
completion. In this environment the plain node test target is still blocked by
the missing `wasm32-unknown-unknown` target, so the truthful proof path remains
`SKIP_WASM_BUILD=1`, matching the stripped runtime checks elsewhere.

That outward move is now happening through plan 012 instead of another round of
fixture tightening. The important discovery was that the second-game proof did
not need new trait surgery in `myosu-games`: `GameType::LiarsDice` and the
registry entry were already there. The first honest multi-game slice was
therefore additive, not invasive: a new `myosu-games-liars-dice` crate with a
compact state machine, CFR-facing types, and a basic renderer. That is not the
full architecture proof yet because MCCFR convergence and a cleaner
zero-modification baseline still remain, but it changes the theory in an
important way: the repo's game boundary is already strong enough to accept a
second game crate without reopening poker-specific code.

The next discovery was that "add solver glue" and "prove exploitability" were
not the same thing. A minimal `LiarsDiceSolver` could be added cleanly and
trained with MCCFR, but the first attempt to use the generic robopoker
exploitability path panicked inside best-response evaluation when this toy game
returned control to the hero later in the subtree. That forced a more honest
theory boundary: stage 0 can already prove that a second game trains, records
encounters, and yields averaged policy advice through the shared solver engine,
but it cannot yet honestly claim that the current generic exploitability path
is a finished scoring proof for this repeated-turn Liar's Dice variant. The
second-game proof is therefore real at the training/advice layer now, while the
scoring layer remains a follow-on adaptation rather than something to hand-wave
past.

That same "prove it from downstream" principle now applies to registration.
Because `GameType::LiarsDice` and the shared `GameRegistry` entry already
existed, the right proof was not another shared-crate edit. It was a
cross-crate test in `myosu-games-liars-dice` showing that the new engine crate
can resolve `liars_dice` through the shared registry exactly as the platform
claims. This keeps plan 012 aligned with the broader stage-0 strategy: prove
shared seams by exercising them from the product edge, not by repeatedly
touching the seam itself.

The last open plan-012 item needed the same kind of honesty. On this branch,
"verify zero modifications to `myosu-games` or `myosu-games-poker`" was not an
executable truth claim anymore because shared-crate edits already existed
before the additive Liar's Dice slice. The right closure was to prove the
dependency boundary directly instead of pretending the branch was clean.
`myosu-games-liars-dice` now carries a manifest regression that asserts it
depends on `myosu-games` and does not depend on `myosu-games-poker`, while
`cargo tree -p myosu-games-liars-dice` confirms the same shape externally. That
is the actual architecture guarantee stage 0 needs: a second game can land,
train, and score without reaching back into poker code.

With plans 003, 004, 005, 007, 012, and 013 all now completed, the repo's
control-plane truth needed one more correction too. The master plan and the
node/bootstrap plans could no longer keep presenting the core chain stack as
"active" without becoming misleading. They are now synced to a more useful
split: completed core versus open next. That matters operationally because the
remaining meaningful work is no longer "finish the first loop" but "harden the
live loop" through plans 006, 008, and 002 unless the operator deliberately
switches into productization, CI/security, or doctrine cleanup.

The scoring story has now caught up to that same principle. Instead of forcing
plan 012 to detour into `rbp_mccfr` surgery, the tiny state space made it
possible to add a crate-local exact best-response evaluator in
`myosu-games-liars-dice` itself. That changes the theory again: the second-game
proof is no longer only about trait fit, registry fit, and trainability. It now
also has an executable exploitability score that lives with the second game
crate and respects the repeated-turn information structure that broke the
generic path. In other words, stage 0 now has a truthful multi-game proof at
both the architecture seam and the scoring seam, even though the generic shared
exploitability routine still does not cover this toy variant yet.

The same principle is now paying off again in plan 005. The next honest pallet
reduction was not another broad staking rewrite; it was removing the last bit
of valuation theater from the live helper surface. The runtime already defines
stage 0 swap as an identity-price, zero-fee noop, but the pallet's coldkey
stake-summary helpers were still calling `sim_swap` as though an AMM were live.
Collapsing those helpers down to direct alpha-price conversion brings the
surviving staking utility layer into line with the runtime contract without
touching the active add/remove stake flow itself. That is the current pattern
for the remaining genesis work: prefer reductions that make the live path more
truthful to the stage-0 runtime we already have, rather than preserving old
subtensor generality where the product no longer uses it.

The verification loop tightened that theory a bit further. A first attempt to
prove the helper change with "raw stake equals summary value" was wrong because
the pallet test runtime still carries a nontrivial alpha price surface even
while the helper logic no longer routes through swap simulation. Rewriting the
regression around direct alpha-price conversion produced the right executable
claim. That is a useful reminder for the remaining reductions: stage 0 is not
"everything is literally 1:1 everywhere"; it is "the live path should match the
runtime contract that stage 0 actually defines."

That lesson is now reflected in the proof shape itself. The focused stage-0
regression covers both the coldkey and hotkey summary helpers against the same
direct alpha-price expectation, which is stronger and more honest than the
earlier raw-stake assumption. The remaining safe reductions in plan 005 will
likely look similar: small pieces of reporting and helper logic being pulled
toward the runtime's explicit stage-0 contract, with equally narrow executable
proofs added as each piece moves.

The next slice confirms that same pattern on the reporting side, but it now
goes beyond proof-only. The exposed `StakeInfo` runtime payload had still been
wider than stage 0 really needs because it carried inherited `locked`,
`tao_emission`, and `drain` fields even though the live stage-0 path no longer
used swap-era accounting there. The focused pallet regression first proved that
those fields were inert, and that gave enough confidence to remove them for
real. The stage-0 payload is now smaller, and both the focused pallet proof and
the live runtime compile path stay green with that shrink. That is a better
result than merely documenting zero-only baggage forever.

The next correction uses the same honesty rule, but for fees instead of stake
reporting. The real runtime already wires `SwapInterface` to `Stage0NoopSwap`,
so the live stage-0 contract is identity conversion, zero fee, unit price, and
zero protocol TAO. The subtle trap is that the pallet's default test runtime is
broader than that and still uses the full swap pallet, which means a zero-fee
assertion there would prove the wrong thing. The right proof is now direct:
small runtime tests pin the noop swap contract where it actually lives. That
keeps plan 005 grounded in the real runtime rather than treating every inherited
mock surface as if it were already stage-0-identical.

The next step was to keep following that same contract outward. It is not
enough for `Stage0NoopSwap` itself to be truthful if the runtime API wrapper
quietly reintroduces AMM-shaped semantics at the RPC edge. It does not. The
runtime now has a direct proof that `SwapRuntimeApi` preserves the same stage-0
contract: `current_alpha_price` reports a scaled unit price, and both
`sim_swap_*` methods report identity conversions with zero fees. That is the
useful shape for the remaining reductions too: prove the outward surface where
stage-0 consumers actually read it, not only the implementation beneath it.

The same reasoning now covers the fee query surface too. `get_stake_fee` on the
real runtime is still routed through the pallet wrapper, so it would have been
easy to keep treating zero fees as an implication of the noop swap below. The
better move was to pin that outward behavior directly. The runtime now carries
a direct test over representative fee-query shapes, and they all resolve to
zero under the stripped stage-0 runtime. That continues the same pattern:
truthful stage-0 claims should be proven at the surface where callers observe
them, not only deduced from implementation details further down.

The same pattern now reaches `DynamicInfo`. The active stage-0 payload had
still been carrying `emission` and `pending_root_emission` even though the live
getter only populated them as zero. The right follow-on was to remove them for
real, but the first regression attempt overshot by assuming the subnet reserve
fields would all mirror the same setup value. Reading the test helpers made the
actual contract clearer: `setup_reserves` seeds `SubnetTAO` and
`SubnetAlphaIn`, not `SubnetAlphaOut`. The proof is now honest and stronger.
`DynamicInfo` has a smaller encoded shape, the focused regression rejects any
trailing removed fields, and the surviving decoded values are matched against
the live storage they actually report.

That same principle also applies to the larger metagraph surface. The active
`Metagraph` and `SelectiveMetagraph` payloads were still carrying
`subnet_emission` and `pending_root_emission` even though both construction
paths hardcoded them to zero and even labeled one of them deprecated. The next
truthful reduction was to remove those fields instead of continuing to preserve
zero-only placeholders. The selective API had one extra wrinkle: callers refer
to fields by numeric index. The clean stage-0 move was not to renumber the
entire tail of that map. The old slots are now simply retired in place, so
indexes `11` and `19` resolve to nothing while later indexes keep their old
meanings. That keeps the reporting contract smaller without creating a silent
index drift hazard for any existing selective consumers.

The same rule now reaches the smaller subnet-info DTOs too. `SubnetInfo` and
`SubnetInfov2` had been carrying three fields that were even less defensible:
`network_modality`, `network_connect`, and `emission_value[s]`. Unlike the
metagraph slots, these were not just deprecated in spirit. Both builders were
literally hardcoding them as `0`, `[]`, and `0`. That is the cleanest kind of
stage-0 reporting cut: remove the placeholders entirely, then prove the
remaining payloads still tell the truth. The focused pallet regression now
fully decodes both smaller subnet-info shapes, rejects trailing removed fields,
and matches the surviving contents against the live subnet getters.

That cleanup exposed a second truth too: once both subnet-info versions had
been reduced to the same honest stage-0 content, the older v1 path was only
compatibility inertia. The repo now takes the cleaner step. The active runtime
API, runtime implementation, and JSON-RPC surface expose only
`SubnetInfov2`, and the focused pallet proof now validates that single
surviving contract instead of continuing to carry a parallel v1 interface that
stage 0 no longer needs.

That same logic applied immediately to subnet hyperparams. Once the active
chain surface had already chosen `SubnetHyperparamsV2`, carrying the older v1
hyperparams path in the pallet, runtime API, runtime implementation, and RPC
layer was just one more copy of compatibility inertia. The repo now removes
that duplicate too. The live surface is v2-only, and the focused pallet proof
checks the decoded v2 payload against the live getters, including the newer
flags, instead of guessing what their defaults ought to be.

The next cut was not about versioning but about scope truth. `SubnetState`
still carried an `emission_history` field even though the builder filled it by
iterating across every subnet while the rest of the payload described one
queried subnet. That is exactly the kind of inherited reporting shape stage 0
should stop preserving. The field is now removed from the active payload, and
the focused pallet proof decodes the smaller struct and matches its surviving
fields against the same storage and stake-weight sources the builder uses.

The next aggregate cleanup follows the same doctrine from a different angle.
`get_all_dynamic_info` was still returning `Vec<Option<DynamicInfo>>`, not
because live dynamic payloads were optional, but because the helper walked the
raw subnet-key map and preserved dead `NetworksAdded = false` entries as
outward `None` values. That is not useful stage-0 surface. The live path now
returns `Vec<DynamicInfo>` directly and omits dead entries, and the focused
proof seeds a false subnet marker to verify that the outward list keeps only
the live payloads.

That exact cleanup now reaches the metagraph aggregate too.
`get_all_metagraphs` was still returning `Vec<Option<Metagraph<_>>>`, not
because live metagraph payloads were optional, but because the helper walked
the raw subnet-key map and preserved dead `NetworksAdded = false` entries as
outward `None` values. That is not useful stage-0 surface either. The live
path now returns `Vec<Metagraph<_>>` directly and omits dead entries, and the
focused proof seeds a false subnet marker and verifies the aggregate results
match the live per-netuid `get_metagraph(...)` payloads.

The same cleanup now reaches the surviving subnet-info aggregate too.
`get_subnets_info_v2` was still returning `Vec<Option<SubnetInfov2<_>>>`, not
because live subnet-info payloads were optional, but because the helper walked
stored subnet keys through an older aggregate shape and preserved dead
`NetworksAdded = false` entries as outward `None` values. That is not useful
stage-0 surface either. The live path now returns `Vec<SubnetInfov2<_>>`
directly and omits dead entries, and the focused proof seeds a false subnet
marker and verifies the aggregate results match the live per-netuid
`get_subnet_info_v2(...)` payloads.

That same cleanup now reaches the surviving mechagraph aggregate too.
`get_all_mechagraphs` was still returning `Vec<Option<Metagraph<_>>>`, not
because live mechagraph payloads were optional, but because the aggregate had
kept an older compatibility wrapper even though it only iterated valid
subnet/mechanism pairs. That is not useful stage-0 surface either. The live
path now returns `Vec<Metagraph<_>>` directly for the mechanism aggregate, and
the focused proof seeds a live mechanism count plus a false subnet marker and
verifies the aggregate results match the live per-pair
`get_mechagraph(...)` payloads.

The next fix was not about deleting one more wrapper. It was about removing a
semantic lie from a surviving payload. The neuron reporting path documented its
`stake` field as a coldkey-to-stake map including delegations, but the active
builder was really emitting a single owner-coldkey row with the hotkey's total
subnet stake. That is not stage-0 truth. The full and lite neuron builders now
collect the real per-coldkey subnet stake entries, and the focused proof stakes
both the owner and a delegator into one hotkey and verifies that the outward
map matches the live staking getters instead of attributing all stake to the
owner.

The next fix was another truth correction, but this time about sparse keys
rather than field semantics. The delegate reporting path rebuilt nominator stake
from live share pools, but it indexed those pools by raw `netuid` into a dense
vector. That meant a dead lower subnet key could make a live higher-netuid
nomination disappear from the outward delegate report. The builder now keys
share pools by `NetUid` directly, and the focused proof creates a sparse
live-netuid layout and verifies that the live nominator stake still survives in
the outward delegate payload.

Delegate reporting still had one more economic lie after that. The outward
daily return was aggregating emissions across the delegate's registered
subnets, but `return_per_1000` was still dividing by root-only stake. That is
not a harmless approximation in stage 0; a delegate with active stake and
emissions only off root can be reported with the wrong return. The builder now
uses `get_total_stake_for_hotkey(&delegate)` as the denominator, and the
focused proof seeds stake and emissions on a non-root subnet and verifies that
the outward `return_per_1000` matches the pallet helper formula exactly.

There was still one more economic mismatch in the same outward payload.
`total_daily_return` was labeled as delegator return, but the builder was
still exposing raw emission before subtracting delegate take. That is a direct
semantic lie whenever commission is nonzero. The builder now routes both
`total_daily_return` and `return_per_1000` through the same net delegator
return helper, and the focused proof sets a 50% take and verifies that the
outward daily-return field matches the post-commission amount rather than the
raw emission total.

The next reduction moved back out to the RPC boundary. The custom runtime API
was still exporting `get_coldkey_auto_stake_hotkey` and `get_subnet_to_prune`,
but those helpers are not part of the active stage-0 doctrine and had no
remaining consumer beyond the compatibility plumbing itself. Keeping them would
mean advertising outward helper surface that the genesis plan does not depend
on. The right move was to delete the endpoints from the runtime API, runtime
impl, and RPC server, then remove the now-unreferenced pallet getter too. This
does not create a new behavioral proof in `stage_0_flow`; the evidence here is
successful rebuild of the narrowed runtime/RPC seam.

That same reasoning still left one more outward-only inspection seam. The
custom runtime/RPC layer continued to export `get_subnet_state`,
`get_selective_metagraph`, and `get_selective_mechagraph`, but none of those
endpoints had a remaining doctrine, spec, chain-client, or other in-repo
consumer beyond the compatibility wiring itself. Leaving them in place would
keep advertising a broader node RPC contract than stage 0 actually needs. They
are now removed from the runtime API, runtime impl, and RPC server, while the
pallet-side helpers remain available for local tests and internal code. Like
the prior RPC cut, the proof here is successful rebuild and clippy over the
narrowed runtime/RPC seam rather than a new DTO-level regression test.

The same pattern still remained for delegate reporting at the node boundary.
The pallet-level delegate helpers still matter because the stage-0 truth
corrections and local tests exercise them directly, but the custom runtime/RPC
trio around `get_delegates`, `get_delegate`, and `get_delegated` had no
remaining doctrine, spec, chain-client, or other in-repo consumer beyond the
compatibility shell itself. Keeping them would still advertise a broader live
RPC contract than the current genesis plans require. Those outward methods are
now removed from the runtime API, runtime impl, and RPC server. As with the
other RPC cuts, the proof is successful rebuild and clippy over the narrowed
runtime/RPC seam rather than a new DTO-level regression.

That left one final asymmetry: the checked-in doctrine and shared client still
named exactly one custom node RPC, `neuronInfo_getNeuronsLite`, while the
custom runtime/RPC layer still exported a much broader inherited shell of
neuron, subnet-info, metagraph, dynamic-info, mechagraph, stake-info, and
lock-cost methods. Keeping that shell would keep advertising a broader node
contract than the bootstrap plans actually rely on. The right move was to
collapse the custom runtime API, runtime impl, and RPC server down to the
single lite-neuron path, then remove the dead runtime stake-fee helper that was
left behind by the earlier API cuts. As with the other boundary reductions, the
evidence is successful rebuild and clippy over the narrowed seam rather than a
new DTO-level regression test.

That same asymmetry still existed one layer inward. After the runtime/RPC shell
was collapsed, the active pallet still carried full-neuron and single-item
neuron reporting helpers even though the only surviving outward consumer was
the lite list. Keeping those helpers would still leave a broader live pallet
surface than the runtime and shared client actually exercise. The next truthful
cut was to remove the heavier pallet-side neuron helpers too and point the
stage-0 proof at `get_neurons_lite` directly. That keeps the proof on the same
contract the node actually exports instead of relying on a narrower private
getter that stage 0 no longer needs.

The same reasoning applies to aggregate reporting helpers that no longer have
any stage-0 consumer at all. Once the custom delegate RPC trio and stake-info
RPC shell were gone, the pallet still carried `get_delegates`,
`get_delegated`, and `get_stake_info_for_coldkeys` even though no runtime
surface, shared client path, or focused stage-0 proof called them anymore.
Those methods were not preserving an active contract; they were only preserving
shape. Removing them tightens the live pallet API without weakening the proofs,
because the still-relevant single-item delegate and stake paths remain in place
and continue to be exercised directly.

The metagraph seam had one last version of that same pattern. Even after the
deprecated subnet-emission fields were removed, the payloads still carried
`tao_dividends_per_hotkey`, and the builder still synthesized it as zero for
every hotkey because the TAO-side dividend path is gone. That is not useful
stage-0 information; it is just inert shape. The right move was to remove it
and retire selective index `70` in place rather than sliding the later
metagraph indexes underneath callers. That keeps the live reporting contract
smaller while preserving the meaning of the surviving selective fields.

That seam is now a little tighter too. The runtime API wrapper for
`get_stake_fee` no longer duplicates the forwarding logic separately from the
proof; both now use the same tiny helper in the runtime crate. That is a small
change, but it matters for the repo's operating style: if we say a stage-0
surface is pinned, the proof path should share the same implementation path
whenever the seam is this small.

The important pivot was not another hyperparameter guess. It was better
observability. Decoding live block events showed that the failing Bob commit was
being rejected as `CommittingWeightsTooFast`. Fresh subnets still carry a
weights-set rate limit policy that blocks immediate post-registration weighting,
which is reasonable for a real network but wrong for the one-machine bootstrap
loop. Once the subnet was brought into a stage-0-friendly state with
`weights_set_rate_limit=0`, the same Bob -> Alice commit-reveal path succeeded
without needing to disable commit-reveal entirely.

That same lesson now closes one of the remaining plan-005 cleanup items too.
The pallet had already been stripped down to hash-based commit-reveal v2 as the
only surviving weight-hiding mechanism, but the focused stage-0 proof family
was still only exercising the direct `set_weights` lane with commit-reveal
disabled. The honest closure was not another refactor. It was a focused proof
that the surviving live path behaves exactly the way the pallet now claims:
direct `set_weights` is rejected with `CommitRevealEnabled` while the subnet is
in hiding mode, a `commit_weights` hash is accepted, an early reveal is
rejected with `RevealTooEarly`, and the valid reveal window opens the commit
and writes the live weight storage. That proves the stage-0 pallet no longer
has a second hidden compatibility path lurking behind the public API.

The same pass also shaved one more small piece off the remaining swap-era
reporting shell. `SubnetHyperparamsV2` was still carrying
`user_liquidity_enabled`, but under the stripped runtime's stage-0 swap seam
that value can only ever be `false`. Keeping it would mean advertising a
liquidity-era knob that the live runtime can no longer make meaningful. The
smaller hyperparams payload now drops that field entirely, and the focused
stage-0 decode proof still matches the surviving live getters. That does not
finish the broader staking cleanup, but it keeps the remaining work honest by
separating real economic dependence from dead outward baggage.

The next swap-era reduction was similar in spirit but one layer deeper. The
live pallet was still routing coinbase emission and subnet dissolution through
`adjust_protocol_liquidity`, `dissolve_all_liquidity_providers`, and
`clear_protocol_liquidity`, plus a dead `get_protocol_tao` wrapper, even
though the stripped runtime implements those hooks as inert no-ops. Keeping
those calls would make the stage-0 pallet look more liquidity-aware than it
really is. Removing them keeps the behavior unchanged on the live runtime but
makes the remaining staking cleanup more honest: what is left now is actual
swap-shaped economics and limit-price logic, not fake liquidity housekeeping.

That remaining dependence turned out to be narrower than the inherited swap
contract made it look. After the no-op liquidity hooks were gone, the live
pallet only still needed two concrete swap directions, TAO->alpha and
alpha->TAO, plus the surviving price and fee helpers. The honest closeout was
to stop binding the pallet config to the full inherited `SwapHandler` surface
and replace it with a pallet-local `Stage0SwapInterface<Self>` that carries
only those stage-0 operations. The important part is not the trait itself; it
is that the focused proof stack still stays green afterward: `stage_0_flow`
passes at `16/16`, the stripped runtime rebuilds, and both pallet and runtime
clippy gates still pass. That closes plan 005 by making the final surviving
swap-shaped staking seam explicit instead of merely smaller by convention.

## Strategy

The strategy remains chain-first, but the emphasis has shifted from broad
construction to truthful narrowing:

- keep the runtime and pallet stage-0-shaped enough that participant proofs are
  readable and trustworthy
- treat `myosu-chain-client` as the one place where live-chain truth is
  discovered, submitted, and explained to operators
- use fresh local subnets when inherited bootstrap state becomes suspect rather
  than quietly normalizing corrupted state into the playbook
- push the operator story upward only when live storage proves the new claim
- move manual proof steps into node-owned execution surfaces so failures present
  as concrete integration bugs, not as operator folklore
- keep proving post-weight consequences, not just successful submissions, so
  stage-0 claims stay anchored to emission behavior instead of storage writes
- keep gameplay proof in the same executable loop as chain proof, so the
  product claim "a human can play a hand against the trained bot" stays tied to
  the same artifacts and same bootstrap sequence
- only let miner discovery claim a "best" solver after the subnet has produced
  a nonzero incentive vector, so gameplay discovery tracks the same economic
  truth the chain is actually emitting
- keep the gameplay-to-miner seam network-shaped, even on localhost, so the
  stage-0 product proof does not regress into direct crate coupling

That means the next slices are not about inventing more participant features.
They are about holding the green node-owned smoke as a non-negotiable proof and
then moving upward into gameplay proof. The immediate bootstrap bottleneck is
no longer "can we wire miner and validator at all?" and no longer "why does a
fresh subnet still throw `CommittingWeightsTooFast`?" The local loop now proves
that weights drive positive miner/validator outputs. The next question is how
to carry that chain-truth seam into human play without re-growing the chain
surface or letting client-side truth drift again.

## Key Discoveries

- The chain reduction work was enough to get to a real local devnet, but not
  enough by itself to justify stage-0 product claims. The participant loop had
  to become real before the repo could honestly say it was building the actual
  network.
- The first local loop failure was small and useful: `rpc_methods.version` was
  optional on the live node. Fixing that confirmed the node/miner/validator path
  was real and moved the work from speculation to integration.
- The pallet rejects `127.0.0.1` as an axon address, but accepts `ip = 0` for
  testing. That gave the local devnet an honest way to prove on-chain serving
  state without pretending a single-machine loop already has routable network
  endpoints.
- The first on-chain weight proof on subnet `1` was not the final operator
  truth. Later inspection showed subnet `1` had stale owner/bootstrap state, so
  it should not be the canonical playbook surface for current bring-up.
- A fresh subnet fixes the operator story dramatically. On subnet `2`, Alice can
  enable subtoken and set tempo, Bob can register, Bob can add stake, Bob can
  acquire validator permit, and Bob can now write a real Bob -> Alice weight
  vector through commit-reveal.
- The decoded live failure mattered more than another blind retry. The first
  Bob -> Alice failure was `CommittingWeightsTooFast`, which narrowed the issue
  to subnet rate-limit truth instead of generic validator bring-up.
- Commit-reveal disablement is not a stage-0 blocker. The clean path keeps
  commit-reveal enabled and succeeds by making the subnet's weights rate limit
  compatible with the bootstrap loop instead of trying to route around the
  reveal mechanism.
- A node-owned smoke path is worth more than another manual proof. Once the
  chain executable owned the subnet creation, miner bootstrap, and validator
  bootstrap flow, the remaining failure surface got much smaller and more honest.
- The actual bug was in the shared client, not the pallet. Fresh subnets were
  only "reporting" `weights_set_rate_limit=0` because the client treated an
  absent storage key as zero instead of honoring the pallet default of `100`.
- Fixing that default read was enough to make the owner-side subnet bootstrap
  write the zero rate-limit override for real, after which the full
  `--stage0-local-loop-smoke` path completed successfully.
- A stored weight vector is not a strong enough stage-0 proof on its own. The
  better proof is post-epoch state: positive miner incentive/emission and
  positive validator dividends on the clean subnet after the Bob -> Alice
  weight write.
- Gameplay proof is only honest when it uses trained artifacts. A fallback
  generated advisor is fine for local crate sanity, but it cannot count toward
  stage-0 exit.
- Miner discovery is only honest when incentive state is populated. Before the
  first scored epoch, "best miner" is a premature claim even if the axon state
  is already present on-chain.
- A live gameplay query can still fail after a readiness probe succeeds if the
  server treats probe-socket write failures as fatal. Readiness should prove
  the process can survive malformed local probes, not just that a port opened
  once.
- The right place to introduce live miner advice into gameplay is the
  line-oriented pipe seam first, not the ratatui paint path. That keeps the
  network boundary honest while preserving the stricter node-owned smoke proof
  as the gate on whether the product story is still true.
- The same boundary rule also works for the interactive path, but only when
  the live query is cached outside the paint loop. The TUI can consume
  chain-visible advice without becoming the place where async network work
  lives.
- Cached live advice needs freshness semantics, not just presence semantics.
  A stale recommendation is still useful if it is clearly marked stale; a
  silently disappearing recommendation is harder for an operator or player to
  interpret.
- Freshness semantics are even better when they are measurable. A tiny age
  signal is enough to turn "stale" from a vague warning into an interpretable
  operational cue.
- The first honest `002` slice was not about writing more doctrine from
  scratch. It was about making the canonical spec surface stop lying: promote
  the real TUI spec into `031626-07`, fill `031626-10` from the live pipe and
  schema surfaces, and break the fake `031626-11`/`031626-12` duplication.
- Closing `002` required one more mundane but important step: archive the
  mirror namespace entirely and rewrite the stale lane reviews so they stop
  describing completed code as "ready to build." That cleanup matters because
  control-plane truth drifts just as easily in docs and reviews as it does in
  code.
- The first plan-009 correction was similar: the old productization draft was
  right about needing decomposition, but wrong about the scale of the problem.
  `myosu-play/src/main.rs` was no longer a 31K-line monster. The honest first
  move was still to extract a real seam, so the CLI and discovery-request
  parsing now live in `crates/myosu-play/src/cli.rs` and the smoke path stayed
  green.
- The next honest plan-009 move was not to invent new UI states in the middle
  of a monolith. Artifact and blueprint resolution were already cohesive, so
  pulling them into `crates/myosu-play/src/blueprint.rs` was the cleaner
  prerequisite. That cut reduced `main.rs` again, kept the smoke path green,
  and made the remaining product gap easier to state precisely: startup and
  onboarding already have diagnostics, but those diagnostics still are not
  first-class user-visible states.
- The first user-visible correction after that extraction was to stop hiding
  startup state behind implementation detail fields alone. `myosu-play` now
  maps artifact resolution into explicit `success` / `empty` / `partial`
  startup states, logs actionable guidance in training mode, and emits a
  structured startup status line in pipe mode. That still is not the full
  interaction-state taxonomy, but it is the first honest step toward it.
- The next correction belonged in the shared TUI layer, not in one binary.
  Declaration text and operator feedback were still implicit because the shell
  had no first-class interaction-state model. Giving `myosu-tui` explicit
  `success` / `partial` / `error` state handling made the startup-state work
  reusable and testable, and it turned the remaining `009` work into a much
  narrower problem: finish the loading/empty/partial/error taxonomy and refine
  how those states appear across layouts.
- The first responsive-layout cut confirmed that state handling and layout
  handling are really the same productization problem. Narrow terminals needed
  transcript-tail priority and state-panel collapse, desktop terminals needed a
  side-by-side reading mode, and the loading state only became honest once
  `train` rendered a real pre-resolution frame instead of merely setting a flag
  before hidden work.
- The next correction was to stop treating the transcript as a dumb append-only
  log. Once the shell knew about loading, empty, partial, and error states, the
  transcript had to surface those states directly or the interface still read
  like a pile of unrelated messages. Making the transcript state-aware turned
  first-run guidance and failure detail into part of the shell contract, not
  just side-channel log text.
- The same lesson held for live miner connectivity. Mid-hand network health is
  not "just another message"; it changes what the player or operator can rely
  on. Emitting structured shell status updates alongside the human-readable
  transcript message turned fresh/stale/offline advice transitions into a real
  UI contract instead of a best-effort log convention.
- The keyboard and onboarding polish work landed in the same spirit: usability
  problems were not about adding more commands or more docs, they were about
  making the existing surface legible under stress. Windowing long input around
  the cursor, showing a visible focus marker, and turning empty-artifact copy
  into a clearer “you can still use the demo, here is how to recover saved
  advice” flow all reduced confusion without changing the underlying stage-0
  contract.
- The onboarding path was worth making real instead of leaving as copy only.
  Once empty-artifact startup became a genuine onboarding screen that could
  complete into the lobby or route directly to the demo hand, several plan-009
  checklist items stopped being aspirational and became honestly complete. That
  leaves the productization lane in a much narrower state: mostly one last
  edge-case bucket instead of broad UX architecture work.
- The last plan-009 item turned out to be exactly the kind of thing that
  should close a productization lane: not a new surface, just the last sharp
  corners. Zero-result miner discovery and failed live query now promote
  startup into explicit partial state instead of hiding in summary prose, and
  overlong shell headers now truncate intentionally on narrow terminals instead
  of relying on whatever clipping the buffer happened to do. With that, `009`
  is no longer "mostly done"; it is actually closed, and the next honest phase
  is `010`/`011` rather than more poker-surface invention.
- The first real `010` move followed the same rule: gate what the repo can
  actually prove today, not what an ideal CI machine might prove later. The
  stripped chain now has explicit CI jobs for runtime/node/pallet compile,
  doctrine integrity, plan quality, and chain clippy, but the chain jobs carry
  the honest `SKIP_WASM_BUILD=1` constraint because the current proof posture
  still assumes no installed `wasm32-unknown-unknown` target. That is a real
  hardening gain even though the timing question remains open.
- The next useful `010` correction was that the wasm-target truth did not stop
  at the new chain jobs. The gameplay lane also needed `SKIP_WASM_BUILD=1`
  once clippy reached through `myosu-play` into the chain graph. After that
  fix, the expanded workflow looked operationally sane under warm local cache:
  doctrine and plan-quality were negligible, chain-core and chain-clippy were
  only a couple of seconds each, and the long pole remained the gameplay test
  bundle at under a minute. That does not close the GitHub-hosted timing proof,
  but it does mean the remaining `010` risk is verification-in-Actions, not
  obvious workflow bloat.
- The first real `011` move confirmed that the biggest remaining stage-0 risk
  is not "missing security work" in the abstract; it is uneven operator truth.
  A grounded `ops/security-audit-stage0.md` and `ops/release-gate-stage0.md`
  now make the repo say something concrete: the boundary hardening and CI work
  are real, key management is still an accepted stage-1 risk, and a final
  stage-0 completion claim would still be blocked by release-governance items
  around INV-001, INV-005, and INV-006. On the code side, miner and validator
  observability is no longer only a top-level "probing chain" log; the service
  actions that operators actually care about now emit timing/result fields for
  training, bounded query serving, scoring, and weight submission.
- Closing the remaining `011` slice required being honest about where timing
  can and cannot live. The node can measure real wall-clock startup and RPC
  assembly time, so `myosu-chain/node` now logs `node_service_start`,
  `node_rpc_ready`, and `node_service_ready` with elapsed milliseconds. The
  runtime pallet cannot honestly pretend to export wall-clock metrics from
  inside `on_initialize`, so the chain-side observability finish was to add
  concise `block_step_summary` and `epoch_mechanism_summary` lines instead:
  enough operator truth to see when epochs drain, how much alpha moves, and
  how many validators/neurons are active, without burying signal in trace-only
  math dumps. That closes `011` at the stage-0 bar even though `010` still has
  one remote-only GitHub timing proof left open.
- The next local move became clear once the GitHub side was checked: there is
  no Actions history at all on `happybigmtn/myosu`, so the remaining `010`
  proof is not a code problem but a future push/observe problem. The right
  response was not to stall the repo behind an external blocker; it was to
  open the real next implementation lane explicitly as plan `020`.
- The first honest `020` slice was not miner CLI flags or chain-level game
  selection. Those would have been interfaces in search of a protocol. The
  actual blocker was lower and narrower: poker already had a wire-safe
  query/response seam, while Liar's Dice only had an internal solver and
  renderer. Giving `myosu-games-liars-dice` bounded bincode transport helpers,
  shared `StrategyQuery` / `StrategyResponse` aliases, and solver
  `query` / `answer` / `recommend` helpers turns the second game into
  something miner and validator can really consume next, rather than a crate
  that only proves additive architecture in isolation.
- The next `020` discovery was that transport was necessary but still not
  sufficient. Miner and validator could not actually share second-game truth
  until Liar's Dice had the same kind of durable local state boundary poker
  already had. Adding checkpoint save/load plus bounded `train()` / `step()`
  wrappers to `LiarsDiceSolver` was therefore the real enabling move for the
  next layer, because it let both services consume the same saved second-game
  profile instead of depending on ad hoc in-memory setup.
- That made the honest service-side generalization much simpler than a
  speculative "generic game engine" pass. `myosu-miner` and
  `myosu-validator` now branch explicitly on executable game choice for their
  bounded local paths: poker keeps encoder-backed checkpoints, Liar's Dice
  uses checkpoint-only local training and scoring, and the operator-facing
  reports say which game actually ran. This is the right shape for stage 0:
  reuse the shared query/response contract, but keep the game-specific truth
  visible instead of hiding it behind premature abstractions.
- The important restraint is what did *not* get generalized yet. Once
  `--game` existed in the miner, it would have been easy to let
  `--serve-http --game liars-dice` silently fall through a poker-shaped path
  or to invent a fake generic axon surface. The better move was an explicit
  unsupported error. That keeps the repo honest about the current state of
  plan `020`: local bounded miner and validator execution are now real for the
  second game, but only the surfaces that actually exist should be generalized.
- The next honest step was therefore not a fresh protocol layer for gameplay
  but an explicit branch in `myosu-play`. The shared shell and pipe surfaces
  were already generic enough to host a second renderer; what they lacked was
  a truthful way to say "this game has a local consumable surface, but it does
  not yet participate in poker's discovery/live-advice loop." Adding
  `--game liars-dice` to `myosu-play`, wiring it to the built-in and
  checkpoint-backed Liar's Dice renderers, and keeping miner discovery plus
  live HTTP advice behind poker-only checks proved that the consumption layer
  can scale by game without faking a shared network contract.
- That changes the remaining `020` thesis again. The second game is now real at
  four layers: shared game seam, durable local artifacts, bounded
  miner/validator execution, and human/agent local consumption. The only
  unproven part left is the one that actually matters for subnet scaling:
  coexistence of poker and Liar's Dice as distinct live subnets in the owned
  chain harness.
- The next narrowing proved that the remaining contradiction was not a pallet
  impossibility but a layered truth problem in the local proof surface. Making
  `register_network` inclusion-aware showed the second-subnet failure was real
  enough to inspect, but the next discovery was that the native node binary and
  the embedded runtime wasm had diverged. The binary contained the newest
  `register_network_check` marker while the cached wasm did not, which meant the
  local smoke could no longer be trusted until the runtime artifact itself was
  refreshed.
- Refreshing that embedded runtime wasm required a more exact environment model
  than the repo had before. The stable toolchain already had
  `wasm32-unknown-unknown`, but Substrate's wasm builder was shedding the outer
  `RUSTC` selection and falling back to the wrong `rustc` from `PATH`. Once the
  build was rerun with the stable toolchain's `bin/` directory ahead of the
  system path, the cached wasm updated, the runtime marker appeared in the wasm
  too, and the second-subnet registration contradiction disappeared.
- That did not mean the owned two-subnet proof was suddenly done. It meant the
  repo could finally see the next real blockers. The shared chain client was
  still reading default-backed storage items such as `NetworkRateLimit` and
  `Tempo` as zero when the keys were absent, even though the pallet uses
  `ValueQuery` defaults. Fixing those reads to use the runtime's actual default
  constants closed two more places where the operator truth could quietly drift
  from live chain behavior.
- The final owned-harness blocker turned out to be permission shape, not subnet
  shape. Liar's Dice owner-side configuration was still trying to run
  root-only sudo normalization under the subnet owner's `//Charlie` key. Once
  that path was split so `--enable-subtoken` stayed under the owner key and the
  sudo tempo/weights-rate-limit calls moved back to `//Alice`, the owned local
  loop completed.
- That changes the `020` thesis one more time, and this time to closure. The
  second-game proof is no longer just local transport, local artifacts, bounded
  execution, and local rendering. The owned chain harness now proves poker
  subnet `2` and Liar's Dice subnet `3` coexist on the same local chain, both
  register miners and validators, both produce Bob -> Alice weights, both yield
  positive miner incentive / validator dividend / miner emission, and both
  reach a consumable gameplay surface. The only major plan left ahead of the
  doctrine/governance stack is the still-remote `010` GitHub timing check.
- With that execution proof closed, the next leverage point shifted away from
  protocol feasibility and back toward doctrine honesty. The old `OS.md` was
  trying to be a market memo, product roadmap, and operator handbook all at
  once, which made it bad at the one job the repo actually needed from it:
  state the current stage-0 doctrine and the current truthful operator loop.
  Rewriting `OS.md`, slimming `README.md`, and adding a dedicated local-advisor
  playbook changed the documentation surface from "everything we might become"
  to "what is live, what is proved, and where an operator actually starts."
- That same cleanup immediately surfaced the next doctrine cutover seam. Once
  the active docs were narrowed, the remaining root-level legacy executor
  footprint became much easier to see: the `malinka/` tree itself plus a small
  set of active-doc references in `fabro/` and `genesis/ASSESSMENT.md`. That is
  a healthier problem than vague repo-wide drift, because it means the next
  doctrine work is not "scrub history from every file." It is "archive the
  remaining root legacy tree cleanly and keep the active control plane free of
  those assumptions."
- The next cleanup proved that this narrower framing was right. The root
  `malinka/` tree is now archived under `archive/malinka/`, the active-doc
  references in `fabro/` and `genesis/ASSESSMENT.md` are scrubbed, and the
  active surface sweep over `README.md`, `OS.md`, `AGENTS.md`,
  `genesis/ASSESSMENT.md`, `fabro/`, and `outputs/` comes back empty for
  `malinka|autodev`. That is the doctrine win that matters: the repo can still
  preserve provenance, but the live root no longer advertises a legacy executor
  path as if it were current.
- The next doctrine slice stayed true to that same principle: do not invent a
  new control plane when the real problem is labeling. `myosu-bootstrap` was
  already the live entrypoint, but the surrounding program and output docs still
  made the broader manifest set and the entire `outputs/` tree look equally
  current. Marking the non-bootstrap manifests as secondary and explicitly
  calling out the four bootstrap output roots closed that ambiguity without
  changing the manifests themselves. The control plane is now easier to explain
  because the docs match the actual promotion policy instead of forcing readers
  to infer it.
- The next simplification confirmed the same pattern again inside `fabro/`
  itself. The workflows, run-configs, and programs were not a junk pile that
  needed deletion; they were a legitimate but uneven execution substrate. Once
  all three README surfaces used the same active/secondary classification
  language, the checked-in Fabro surface became much easier to supervise and
  explain. That matters because the next Genesis cleanup should not be solving
  a naming problem by deleting useful structure.
- Closing `018` proved that the same principle applies to the Genesis corpus
  itself. The right move was not to invent a fresh doctrine document or treat
  the synth output as disposable. It was to make the existing corpus carry
  explicit truth about status: `010` is still the only active next-step plan,
  `019` became the last queued doctrine slice and is now complete locally, and
  the rest of the selected doctrine/execution work is
  either completed locally or preserved as historical/reference context. Once
  the report and assessment were rewritten to match that status model, Genesis
  stopped sounding like an older repo snapshot and started behaving like a
  trustworthy control-plane artifact again.
- Closing `019` follows the same rule one level higher. The synth-governance
  question did not need a fresh policy document; it needed a rule in the file
  future Genesis contributors already must read. Writing the run procedure,
  provider order, fallback posture, live-repo versus worktree rule, and
  adjudication-before-merge discipline into `genesis/PLANS.md` means a future
  synth pass now has one authoritative place to look before it rewrites
  anything. That is a better shape than scattering the policy across ad hoc
  notes, because the next corpus refresh now inherits the same "explicit truth
  in the smallest durable surface" pattern as the rest of the repo.
- The same pattern clarified the next `010` seam in two steps. First, the repo
  had to stop pretending the remaining CI proof was merely "observe Actions
  timing" when GitHub had no workflows or runs at all. Publishing the workflow
  fixed that. But the first real hosted runs on draft PR `#1` immediately
  exposed the deeper truth: the workflow is now proving against an older remote
  repository shape, not the current local stage-0 workspace. Doctrine failed on
  legacy `031626-*` spec surfaces, the gameplay lane failed because
  `myosu-games-poker` does not exist remotely, and the chain lanes failed
  because `myosu-chain-runtime` is outside the remote workspace. So the last
  `010` blocker is no longer publication and not yet runner speed; it is remote
  repo drift.
- That also means the next useful change inside `010` is not relaxing the gate
  or arguing with cargo error text. The workflow should say the truthful thing
  first. Adding a fast `repo-shape` preflight makes GitHub fail on "remote repo
  is behind the current stage-0 workspace contract" before doctrine, gameplay,
  or chain jobs each rediscover the same mismatch differently. That does not
  close `010`, but it turns the remaining blocker into one actionable sentence.
- Publishing that preflight proved the point immediately. Hosted run
  `23729944496` now fails in four seconds on `Stage-0 Repo Shape` and lists the
  missing current-stage files directly, while every other job skips. That is a
  better state than the earlier noisy failures because the repo can now point
  to one exact remote-side gap: GitHub does not yet contain the current
  workspace and Genesis control-plane files that local stage-0 work depends on.
- Widening the draft PR to the actual CI-critical stage-0 surface then cleared
  that blocker exactly as hoped. Hosted run `23730119476` got real current-code
  proof immediately: `Stage-0 Repo Shape`, `Plan Quality`, and `Doctrine
  Integrity` all passed on GitHub before the longer cargo lanes began. So `010`
  is no longer stuck in "is the remote repo even the right shape?" territory;
  it is now in the honest final phase of hosted compile/test/clippy timing.
- The follow-up cleanup work improved the quality of that proof surface without
  changing its substance. Moving to `actions/checkout@v6` removed the Node 20
  deprecation warning from the observed early output, and adding workflow
  concurrency means future pushes on the same PR should stop burning runner
  time on stale in-progress runs. Hosted run `23730306070` preserved the same
  fast early passes (`Stage-0 Repo Shape`, `Doctrine Integrity`, and `Plan
  Quality`) under that cleaner workflow shape.
- Letting that run settle replaced "pending cargo lanes" with specific fixable
  blockers. `Active Crates` failed because two `myosu-play` startup-state tests
  were accidentally keyed to ambient local poker artifact discovery: they
  passed on this machine but failed on the clean GitHub runner when advice
  startup stayed `Empty`. `Chain Core` and `Chain Clippy` both failed much
  earlier in `litep2p` because the hosted runner did not have `protoc`.
- The right `010` response is therefore not another round of abstract workflow
  tuning. The repo needs one narrow fix bundle: make those `myosu-play` tests
  deterministic by constructing an explicit non-empty advice fixture, install
  `protobuf-compiler` in the chain jobs, and keep the node lane deny-Clippy
  clean once `protoc` allows the hosted runner to reach the binary. Local
  verification already shows that combination is coherent again.

## Open Questions

- How should the repo make the stable-toolchain wasm build path reproducible
  for other operators, so future runtime refreshes do not depend on
  rediscovering the `PATH` precedence issue between the stable toolchain and
  the system rustc?
- What evidence should count as sufficient to close the remaining remote-only
  `010` GitHub-hosted timing proof: one green hosted run, a short series, or a
  captured timing envelope checked into the repo?
- Why does `sudo_set_commit_reveal_weights_enabled` still fail to converge on a
  fresh local subnet when the owner key and other owner-side knobs do converge?
- What is the cleanest second-game consumption surface for Liar's Dice:
  now that `myosu-play` hosts it locally, what is the smallest honest upgrade
  path from this local shell/pipe surface to a real second-game live network
  path?
- How should a future live HTTP axon for Liar's Dice line up with the current
  poker axon shape without smuggling poker assumptions into a supposedly
  shared route contract?
- Which other default-backed storage reads in `myosu-chain-client` still need
  explicit pallet-default handling so stage-0 operator truth cannot drift the
  same way again?
