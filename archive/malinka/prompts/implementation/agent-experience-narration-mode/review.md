# Implement rich narration mode Lane — Review

Review only the current slice for `agent-experience-narration-mode`.

Current Slice Contract:
Plan file:
- `genesis/plans/015-agent-experience.md`

Child work item: `agent-experience-narration-mode`

Full plan context (read this for domain knowledge, design decisions, and specifications):

# Agent Experience Implementation

**Plan ID:** 015
**Status:** New
**Priority:** MEDIUM — enables autonomous agent operation

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, Fabro agents running Myosu lanes will have a persistent memory system, reflection prompts, a journal, and identity. Agents will not just be functions that parse stdin faster — they will be digital inhabitants with accumulated context.

---

## Progress

- [ ] Implement agent memory system (per-agent persistent memory)
- [ ] Implement reflection prompt (`reflect>`)
- [ ] Implement journal (append-only markdown autobiography)
- [ ] Implement identity persistence
- [ ] Implement rich narration mode (`--narrate`)
- [ ] Verify agent experience across multiple Fabro lane runs

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: Reflection is optional — agents can skip the `reflect>` prompt.
  Rationale: Per `specs/031626-10-agent-experience.md`, forcing reflection reduces quality. It should be available but not mandatory.
  Date/Author: 2026-03-21 / Interim CEO

- Decision: Journal is append-only.
  Rationale: An autobiography is immutable. Agents can read their history but cannot rewrite it. This preserves the integrity of the reflection system.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Implement agent memory system
Per-agent memory stored in `~/.myosu/agents/{agent_id}/memory/`.

Proof: After running two Fabro lanes, `ls ~/.myosu/agents/` shows the agent's memory directory with accumulated context.

### M2: Implement reflection prompt
The `reflect>` prompt is available at the end of each lane run.

Proof: After a Fabro lane completes, the agent is prompted with `reflect>` and can write reflection notes.

### M3: Implement journal
Append-only markdown autobiography at `~/.myosu/agents/{agent_id}/journal.md`.

Proof: `cat ~/.myosu/agents/{agent_id}/journal.md` shows entries for each completed lane, not overwritten between runs.

### M4: Implement narration mode
The `--narrate` flag produces a rich narrative log of agent actions.

Proof: `fabro run --narrate ...` produces a narrative log at `outputs/{lane}/narration.md`.

### M5: Verify agent experience across multiple runs
Run the same agent on three different lanes. Verify memory accumulates correctly and journal is truthful.

Proof: After 3 lane runs, the agent's memory reflects all 3 runs and the journal contains 3 entries in chronological order.

---

## Validation

- `ls ~/.myosu/agents/` shows agent memory directories after first run
- `~/.myosu/agents/{id}/journal.md` is append-only
- `fabro run --narrate ...` produces `narration.md`
- Agent with accumulated memory outperforms agent without on a multi-step task


Workflow archetype: implement

Review profile: ux

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: Rich prose narration engine activated by --narrate flag in pipe mode
- How: Add --narrate flag that renders game state as prose with board texture, session arc, and strategic context
- Required tests: cargo test -p myosu-tui narration::tests
- Verification plan: --narrate produces prose with board texture and session arc; underlying game state identical in both modes; LLM can extract valid action from narrated output
- Rollback condition: --narrate flag missing, narration omits board texture or session context, or game state diverges between modes

Proof commands:
- `cargo test -p myosu-tui narration::tests::narrate_includes_board_texture`
- `cargo test -p myosu-tui narration::tests::narrate_includes_session_context`
- `cargo test -p myosu-tui narration::tests::terse_and_narrate_same_game_state`

Artifacts to write:
- `spec.md`
- `review.md`


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Nemesis-style security review
- Pass 1 — first-principles challenge: question trust boundaries, authority assumptions, and who can trigger the slice's dangerous actions
- Pass 2 — coupled-state review: identify paired state or protocol surfaces and check that every mutation path keeps them consistent or explains the asymmetry
- check secret handling, capability scoping, pairing/idempotence behavior, and privilege escalation paths

Focus on:
- slice scope discipline
- proof-gate coverage for the active slice
- touched-surface containment
- implementation and verification artifact quality
- remaining blockers before the next slice

Deterministic evidence:
- treat `quality.md` as machine-generated truth about placeholder debt, warning debt, manual follow-up, and artifact mismatch risk
- if `quality.md` says `quality_ready: no`, do not bless the slice as merge-ready


Write `promotion.md` in this exact machine-readable form:

merge_ready: yes|no
manual_proof_pending: yes|no
reason: <one sentence>
next_action: <one sentence>

Only set `merge_ready: yes` when:
- `quality.md` says `quality_ready: yes`
- automated proof is sufficient for this slice
- any required manual proof has actually been performed
- no unresolved warnings or stale failures undermine confidence
- the implementation and verification artifacts match the real code.

Review stage ownership:
- you may write or replace `promotion.md` in this stage
- read `quality.md` before deciding `merge_ready`
- when the slice is security-sensitive, perform a Nemesis-style pass: first-principles assumption challenge plus coupled-state consistency review
- include security findings in the review verdict when the slice touches trust boundaries, keys, funds, auth, control-plane behavior, or external process control
- prefer not to modify source code here unless a tiny correction is required to make the review judgment truthful
