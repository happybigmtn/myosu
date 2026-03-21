# `agent-integration` Review

**Lane**: `agent-integration`
**Date**: 2026-03-21
**Bootstrap status**: Complete — reviewed integration decision written

---

## 1. Keep / Reopen / Reset Judgment

**Judgment: KEEP**

The product frontier should move into an implementation-family workflow next.
It does **not** need another upstream review-only unblock before product
execution resumes.

### Rationale for KEEP

- `outputs/play/tui/review.md` already says `play:tui` is ready for an
  implementation-family workflow immediately.
- `outputs/agent/experience/review.md` already says `agent:experience` should
  proceed to implementation-family work next, with clear slice ordering.
- The actual repository tree still matches those reviews: the product code is
  genuinely absent rather than partially implemented in a contradictory way.
- `fabro/programs/myosu-product.yaml` already provides the reviewed product
  frontier shape, so the decision now is execution sequencing, not more
  bootstrap decomposition.

### Rationale against REOPEN

There is no new contradiction between doctrine and repository state. The lane
reviews remain current and their blockers are already explicit.

### Rationale against RESET

Resetting would throw away truthful reviewed product artifacts without exposing
new information. The open question is now which implementation family to start,
not whether the product frontier needs to be redesigned.

---

## 2. Current Integration State

| Surface | State | Why it matters |
|---|---|---|
| `play:tui` | Reviewed and implementation-ready | Owns `myosu-play`, the first executable product seam |
| `agent:experience` | Reviewed and partially implementation-ready | Owns context, journal, narration, and spectator extensions on top of play |
| `crates/myosu-play/` | Missing | Confirms `play:tui` Slice 1 is still the next real seam |
| `crates/myosu-games-poker/` | Missing | Confirms poker renderer delivery has not started |
| `crates/myosu-tui/src/agent_context.rs` | Missing | Confirms agent persistence remains Slice 1 work |
| `crates/myosu-tui/src/journal.rs` | Missing | Confirms journal work remains Slice 2 work |
| `crates/myosu-tui/src/narration.rs` | Missing | Confirms narration is still a later agent slice |

---

## 3. Product-Level Decision

**Decision: open a product implementation family next, led by `play:tui`.**

This is the honest sequencing implied by the reviewed artifacts:

1. `play:tui` must move first because it creates the `myosu-play` binary and
   concrete `--pipe` entrypoint that `agent:experience` extends.
2. `agent:experience` should not be treated as blocked in the large. Its first
   slices remain valid and can follow as soon as product execution opens.
3. Product does not need to wait for chain-backed play, miner discovery, or
   spectator networking before starting implementation.

**Recommended control-plane shape**:

- first, add a lane-scoped `implement/` family for `play:tui`
- then open a second lane-scoped implementation family for `agent:experience`
  once `play:tui` Slice 1 establishes the binary seam

This keeps ownership clean and matches the existing pattern used for
`games:traits`.

---

## 4. Remaining Blockers and Risks

### 4.1 `robopoker` absolute-path coupling remains real

This is still the main upstream risk, but it is not strong enough to justify
another review-only frontier before product implementation begins.

**Operational consequence**:

- do not let `play:tui` advance past the first robopoker-dependent slice
  without checking the `games:traits` dependency fix
- do not duplicate the absolute-path coupling into new product crates

### 4.2 `agent:experience` depends on `play:tui` Slice 1 for CLI wiring

This is the most important product-local gate.

**Operational consequence**:

- start `play:tui` first
- treat `agent:experience` Slices 3+ as waiting on that seam, not on a new
  upstream review

### 4.3 No product implementation manifest exists yet

Unlike `games:traits`, product does not yet have an implementation-family
program manifest or run config.

**Operational consequence**:

- the next slice should create that implementation-family control-plane
  surface rather than authoring more bootstrap review artifacts

---

## 5. Review Verdict

The reviewed product frontier is strong enough to leave bootstrap mode.

**Next honest move**:

- start `play:tui` implementation-family work
- keep `agent:experience` as the immediately adjacent downstream lane
- revisit only if the first `play:tui` slice proves unexpectedly blocked by the
  unresolved `robopoker` dependency

That means the answer to the frontier question is:

**product needs an implementation family next, not another upstream unblock.**
