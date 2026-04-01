# Doctrine Cutover and OS Refresh

**Plan ID:** 019
**Status:** New
**Priority:** CRITICAL — live doctrine still points at retired Malinka/autodev surfaces

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, Myosu will have one current control-plane story. The
live doctrine will stop treating Malinka and `raspberry autodev` as active
operator surfaces, `OS.md` will describe the Fabro/Raspberry execution model
that actually exists, and a fresh Genesis planning pass can happen against a
clean control plane instead of mixed historical guidance.

---

## Progress

- [x] (2026-03-28 18:27Z) Verify that `fabro synth genesis` is a clean-room generator that refuses a populated `genesis/` tree
- [x] (2026-03-28 18:27Z) Attempt a safe disposable-worktree Genesis refresh run against the current repo state
- [ ] Inventory every live doctrine reference to Malinka, autodev, and retired control files
- [ ] Cut the live operator entrypoints over to Fabro/Raspberry direct execution only
- [ ] Promote any still-needed content out of historical Malinka/specsarchive surfaces into active doctrine
- [ ] Resolve the blocked fresh synth Genesis pass and capture a reviewed next-steps summary from its output

---

## Surprises & Discoveries

- Discovery: `fabro synth genesis` will not run in-place against a repo with an
  existing populated `genesis/` directory, even with `--plans-only`. The safe
  path is a disposable worktree or copy where `genesis/` is empty, then a
  selective merge-back of the generated corpus.
  Evidence: `fabro synth genesis --target-repo /home/r/coding/myosu --plans-only`
  failed with `genesis directory ... is not empty`.
  Date/Author: 2026-03-28 / Codex

- Discovery: The current disposable-worktree Genesis refresh did not finish
  cleanly. `fabro synth genesis` spawned a Codex planning subprocess and
  remained live for several minutes without writing any `genesis/` outputs, so
  the refresh is currently blocked on Fabro/Codex execution behavior rather
  than repo doctrine alone.
  Evidence: the run against `/tmp/myosu-genesis-refresh.8PCqSc` spawned
  `codex exec --json --yolo -m gpt-5.3-codex` but wrote no files under
  `genesis/` before termination.
  Date/Author: 2026-03-28 / Codex

---

## Decision Log

- Decision: We should not delete historical artifacts blind.
  Rationale: Some archive-era files still contain source material that has not
  yet been promoted into active doctrine. The safe sequence is inventory,
  promote-or-mark-historical, then delete or quarantine.
  Date/Author: 2026-03-28 / Codex

- Decision: A fresh synth planning pass should happen only after doctrine cutover.
  Rationale: Running synthesis against mixed live and retired control surfaces
  would just generate another layer of conflicting plans.
  Date/Author: 2026-03-28 / Codex

- Decision: The Genesis refresh should run in a disposable worktree, not the
  live repo tree.
  Rationale: The fabro command requires an empty `genesis/` output directory and
  is designed to produce a clean-room planning corpus. Running it in-place
  would force destructive repo surgery just to start the analysis.
  Date/Author: 2026-03-28 / Codex

---

## Outcomes & Retrospective

*(To be written at completion of major milestones or the full plan)*

---

## Milestones

### M1: Inventory live doctrine drift

Read the live operator surfaces and classify every Malinka/autodev reference as
one of: delete, promote, mark historical, or retain. Cover at least `OS.md`,
`README.md`, `AGENTS.md`, `genesis/`, `fabro/programs/`, and the highest-signal
spec indexes.

Proof: `rg -n "Malinka|malinka|autodev|raspberry autodev|specsarchive|ralph/IMPLEMENT" OS.md README.md AGENTS.md genesis fabro/programs specs`
returns only intentional historical references or tracked work items after the
inventory pass.

### M2: Cut live entrypoints over to the current operator loop

Update the user-facing entrypoints so the repo stops advertising retired loops.
That means `README.md`, `OS.md`, and any active Genesis summaries should point
at `raspberry plan`, `status`, `execute`, and the checked-in Fabro program
manifests that actually govern current work.

Proof: `rg -n "autodev --manifest|raspberry autodev" README.md OS.md genesis`
returns no live-entrypoint matches.

### M3: Promote doctrine, quarantine history

Move any still-needed guidance out of Malinka-only or archive-only sources into
active doctrine, then mark the remainder as historical or remove it if it no
longer serves a purpose. The goal is that active work no longer depends on
`specsarchive/`, `ralph/IMPLEMENT.md`, or Malinka-specific specs for current
execution decisions.

Proof: `rg -n "source of truth|only source of truth|specsarchive/|ralph/IMPLEMENT" outputs specs README.md genesis`
shows active surfaces using current doctrine, not archive doctrine, for live
execution decisions.

### M4: Stage and complete the post-cutover Genesis refresh

Once doctrine is clean, decide whether to run a fresh synth-assisted Genesis
planning pass. If yes, define the exact inputs, scope, and expected outputs so
the run clarifies next development steps instead of reopening settled control
plane questions. Run it in a disposable worktree, review the generated corpus,
and copy back only the parts we trust.

Proof: a disposable-worktree refresh run completes, produces a new
`genesis/{ASSESSMENT,SPEC,PLANS,plans/}` corpus, and a checked-in review note
summarizes which changes should merge back into the live repo.

---

## Validation

- `rg -n "autodev --manifest|raspberry autodev" README.md OS.md genesis` returns no live-loop matches
- `rg -n "Malinka|malinka" README.md OS.md genesis fabro/programs specs` returns only intentional historical references or the archived Malinka spec itself
- `rg -n "specsarchive/|ralph/IMPLEMENT" README.md OS.md genesis outputs specs` shows only explicitly historical references after promotion/quarantine work
