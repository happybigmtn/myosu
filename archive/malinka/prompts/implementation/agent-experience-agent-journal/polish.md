# Implement journal Lane — Fixup

Fix only the current slice for `agent-experience-agent-journal`.

Current Slice Contract:
Plan file:
- `genesis/plans/015-agent-experience.md`

Child work item: `agent-experience-agent-journal`

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

Review profile: foundation

Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

AC contract:
- Where: Append-only markdown autobiography at ~/.myosu/agents/{agent_id}/journal.md
- How: Add append-only journal that records hand entries and session summaries as markdown prose
- Required tests: cargo test -p myosu-tui journal::tests
- Verification plan: Journal grows across 3 sessions without truncation; entries include board, held cards, result, and optional reflection
- Rollback condition: Journal truncated or overwritten between sessions, or entries missing required fields

Proof commands:
- `cargo test -p myosu-tui journal::tests::append_hand_entry`
- `cargo test -p myosu-tui journal::tests::append_session_summary`
- `cargo test -p myosu-tui journal::tests::never_truncates`

Artifacts to write:
- `spec.md`
- `review.md`


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Priorities:
- unblock the active slice's first proof gate
- stay within the named slice and touched surfaces
- preserve setup constraints before expanding implementation scope
- keep implementation and verification artifacts durable and specific
- do not create or rewrite `promotion.md` during Fixup; that file is owned by the Review stage
- do not hand-author `quality.md`; the Quality Gate rewrites it after verification
