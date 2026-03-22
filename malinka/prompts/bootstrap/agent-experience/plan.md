# Agent Experience Implementation Lane — Plan

Lane: `agent-experience`

Goal:
- Agent Experience Implementation

Bootstrap the first honest reviewed slice for this frontier.

Inputs:
- `README.md`
- `SPEC.md`
- `PLANS.md`
- `AGENTS.md`
- `genesis/plans/001-master-plan.md`
- `genesis/plans/002-fabro-cleanup-completion.md`

Current frontier tasks:
- Implement agent memory system (per-agent persistent memory)
- Implement reflection prompt (`reflect>`)
- Implement journal (append-only markdown autobiography)
- Implement identity persistence
- Implement rich narration mode (`--narrate`)
- Verify agent experience across multiple Fabro lane runs

Required durable artifacts:
- `outputs/agent-experience/spec.md`
- `outputs/agent-experience/review.md`

Context:
- Plan file:
- `genesis/plans/015-agent-experience.md`

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


Active plan:
- `genesis/plans/001-master-plan.md`

Active spec:
- `genesis/SPEC.md`

Open tasks:
- Implement agent memory system (per-agent persistent memory)
- Implement reflection prompt (`reflect>`)
- Implement journal (append-only markdown autobiography)
- Implement identity persistence
- Implement rich narration mode (`--narrate`)
- Verify agent experience across multiple Fabro lane runs

Artifacts to write:
- `spec.md`
- `review.md`
