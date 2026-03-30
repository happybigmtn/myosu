# Specification: Agent Coordination Mechanism

Status: Superseded as an active implementation target by
`031626-12-nlhe-incentive-mechanism.md` for the current chain economics lane.
This document is retained only to preserve the original topic slot and to keep
the active spec corpus non-duplicative.

Source: `OS.md` agent doctrine, `specs/031626-10-agent-experience.md`,
bootstrap-stage gameplay and orchestration discussion
Date: 2026-03-29
Depends-on: AX-01..06, GP-01..04, MG-01..04

## Supersession Note

The previous contents of `031626-11` were byte-identical to
`031626-12-nlhe-incentive-mechanism.md`. That made the active spec corpus lie
about having two distinct design surfaces when it really had one duplicated
document. The incentive-mechanism material belongs in `031626-12`, so `11` now
records the narrower agent-coordination problem it was named for and defers any
implementation work until the post-bootstrap agent lane is real.

## Purpose

Define the coordination problem for multiple agents inhabiting the same game
and operational surfaces without turning Stage 0 into an orchestration science
project. This is not the emission-mechanics spec. It is the future-facing
design slot for questions like:

- how multiple autonomous players join, identify, and yield control
- how spectator agents observe without mutating game state
- how shared session metadata, journals, and declarations stay attributable
- how coordination works across games without embedding game-specific logic in
  the transport layer

## Current Truth

Today, the repo proves a **single-agent-at-a-time** interaction model well
enough for bootstrap:

- `myosu-play train` proves the human-facing loop
- `myosu-play pipe` proves the agent-facing loop
- the shared schema and pipe output prove state completeness

What it does **not** yet implement is a multi-agent session manager,
turn-handoff protocol, or persisted agent identity/journal surface. Treat those
as future work rather than hidden assumptions inside the incentive lane.

## Scope

In scope:
- preserving the design slot for future multi-agent gameplay/session rules
- naming the open coordination questions honestly
- preventing further collision with the NLHE incentive doc

Out of scope:
- chain emission mechanics
- validator scoring and reward modifiers
- Stage 0 gameplay transport implementation

## Open Questions For The Future

1. How should an agent claim or release a seat in a persistent session?
2. What metadata must be durable across reconnects?
3. How do spectator agents subscribe without gaining mutation rights?
4. How should journals, declarations, or reflection surfaces be exposed without
   breaking the one-interface doctrine?
5. Which parts of coordination belong in gameplay transport versus external
   orchestration tooling?

## Acceptance Condition

This spec slot is considered normalized when:

- it is no longer byte-identical with `031626-12`
- its supersession status is explicit
- downstream readers are redirected to `031626-12` for incentive mechanics and
  to `031626-10` for the live Stage 0 agent contract
