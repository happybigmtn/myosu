# `learning:improvement` Lane Review

## Judgment: **BOOTSTRAP** — lane is defined but not yet operational; first run must establish baseline

This is the bootstrap review. The lane definition (`spec.md`) is sound, but the learning surfaces the lane depends on are unevenly maintained. The first honest recurring run will surface a small number of HIGH-signal items and a larger number of MEDIUM/LOW items. The goal for the first run is not to fix anything — it is to produce a credible `improvements.md` that downstream lanes can trust.

---

## Trustworthy Learning Surfaces That Already Exist

### 1. `plans/*.md` Decision Log sections — TRUSTED (with caveats)

Every live plan has a Decision Log section. Entries include date, author, and rationale. This is the highest-quality retrospective surface in the repository because:

- Decisions are explicitly named and bounded
- Rationale prevents "we forgot why" drift
- Date/author enables recency filtering

**Caveats**: Maintenance is uneven. Some plans (e.g., `031926-iterative-execution-and-raspberry-hardening.md`) have extensive, well-maintained logs. Others have Decision Logs that were populated at creation but not updated as execution proceeded. The `learning:improvement` lane should flag any plan whose Decision Log has no entries in the current calendar month as a stale surface.

**Evidence**: `plans/031926-iterative-execution-and-raspberry-hardening.md` lines 155–204 show 8 well-populated Decision Log entries (dates 2026-03-19, rationales present). `plans/031926-design-myosu-fabro-workflow-library.md` lines 75–109 show 5 Decision Log entries. `plans/031826-bootstrap-fabro-primary-executor-surface.md` Decision Log is present but sparser.

### 2. `plans/*.md` Surprises & Discoveries sections — TRUSTED with HIGH value

These sections contain empirical observations from actual execution, not speculation. They are the lane's best evidence source because they name specific run ids, specific Fabro/Raspberry failures, and specific observations that drove a decision. Example: `plans/031926-iterative-execution-and-raspberry-hardening.md` lines 74–154 document 9 Surprises & Discoveries with concrete evidence (run ids, error messages, path observations).

This surface is trustworthy but has **recency bias**: it is populated heavily during active execution and goes stale during planning-only periods. A healthy recurring learning lane would smooth this by checking whether the most recent Surprises & Discoveries entry is older than 2 weeks and flagging it if no execution has happened in that period.

### 3. `outputs/*/review.md` — TRUSTED for lanes that exist

The reviewed artifact corpus under `outputs/` is small but growing. `outputs/agent/experience/review.md` and `outputs/games/traits/review.md` are well-structured with explicit trust judgments, proof expectations, and remaining blockers. These are the cleanest retrospective surfaces because they are produced by Fabro runs, not by ad hoc planning.

**Gap**: Most `outputs/` directories still only have `.gitkeep` files (e.g., `outputs/strategy/planning/`, `outputs/security/audit/`, `outputs/operations/scorecard/`). The `learning:improvement` lane will have limited downstream visibility until more lanes complete bootstrap.

### 4. `outputs/games/traits/implementation.md` + `verification.md` — TRUSTED

These are the only fully-complete implementation+verification artifacts in the outputs corpus. They demonstrate a closed loop: spec → implementation → verification. The learning lane can use this as a reference for what "healthy lane completion" looks like when evaluating other lanes.

### 5. Git log — USABLE with filtering

Recent commits (`9e88de1` through `a75eabe`) have informative messages that encode decisions (e.g., `feat(fabro): migrate myosu onto fabro lanes`, `chore: track ops/evidence/`). The learning lane can use git log to detect when a decision was committed without a corresponding Decision Log entry. This is a MEDIUM-signal input — git messages are not self-documenting enough to serve as primary evidence.

---

## Stale or Missing Retrospective Surfaces

### 1. `ops/evidence/` — MISSING (only `.gitkeep`)

`ops/evidence/` is listed in `specs/031626-00-master-index.md` as a place for "incident notes, run artifacts, debug traces." It currently contains only a `.gitkeep`. This surface is meant to hold raw execution evidence that plans and outputs can cite, but it has never been populated.

**Impact**: The `learning:improvement` lane has no raw ops evidence to ingest. It must rely entirely on the processed surfaces in `plans/` and `outputs/`. This is acceptable for bootstrap but means the lane is operating with incomplete signal.

**Next step**: The lane should open an improvement entry: "ops/evidence/ has never been populated; establish convention for checking in run artifacts."

### 2. Decision Logs are NOT systematically updated after execution

The Decision Log in `plans/031926-iterative-execution-and-raspberry-hardening.md` is well-maintained because that plan was actively executed. But plans that were created earlier (e.g., the bootstrap plan) have Decision Logs that were not updated as execution proceeded — decisions were made and documented in Surprises & Discoveries but never migrated to the Decision Log.

This is a systematic gap, not a one-off. The improvement entry is: "Decision Log entries should be created at decision time, not retroactively. Surprises & Discoveries is not a substitute."

### 3. No recurring-oversight pattern yet exists

All four recurring lanes in `myosu-recurring.yaml` (strategy, security, operations, learning) are currently defined but not yet producing reviewed artifacts. `outputs/strategy/planning/`, `outputs/security/audit/`, `outputs/operations/scorecard/`, and `outputs/learning/improvement/` all have only `.gitkeep` files.

This means the `learning:improvement` lane is bootstrapping into a regime where its downstream consumers (strategy:planning, in particular) are not yet producing the artifacts that would make the improvement loop meaningful.

**This is honest**: the lane should not pretend the feedback loop is operational when it is not. The first run should surface this as an OPEN improvement with MEDIUM signal.

### 4. `plans/*.md` Outcomes & Retrospective sections — INCONSISTENTLY maintained

Some plans have well-populated Outcomes & Retrospective sections (e.g., `plans/031926-iterative-execution-and-raspberry-hardening.md` lines 206–227). Others have them as stubs ("This plan is in progress"). The retrospective section is the natural input for the learning lane's improvement ranking, but it is not reliably written.

### 5. No pattern analysis across lanes

Currently, each plan's Surprises & Discoveries and Decision Log are maintained in isolation. There is no cross-plan synthesis: e.g., "The `games:traits` lane found X about robopoker dependency; has `agent:experience` noted this dependency risk?" This is a natural improvement opportunity for a later slice.

---

## Next Honest Recurring Implementation Slice

The next slice is **Slice 1: Passive retrospective read** from `spec.md`.

**Concrete first-run target**:

1. Ingest all `plans/*.md` Decision Log and Surprises & Discoveries sections
2. Identify the 3 highest-signal improvement candidates with:
   - Signal source (which plan, which entry)
   - Evidence strength (HIGH/MEDIUM/LOW)
   - Owner (which lane or operator action)
3. Write those 3 entries to `outputs/learning/improvement/improvements.md`
4. Write a `surface-audit.md` with per-surface trust status
5. Mark the lane as bootstrap-complete in the recurring manifest

**Expected honest output of first run**:

| Signal | Source | Strength | Owner |
|--------|--------|----------|-------|
| ops/evidence/ never populated | specs/031626-00-master-index.md | MEDIUM | operator |
| Decision Log not updated retroactively | plans/031926-iterative-execution... | MEDIUM | operator |
| Recurring lanes have no reviewed artifacts | myosu-recurring.yaml | MEDIUM | learning:improvement lane |

**What the first run should NOT do**:
- Do not open more than 3 improvement entries (that indicates the ingestion is not focused)
- Do not claim HIGH evidence strength for any item (no decision has been traced to a concrete outcome yet in this lane)
- Do not mark any improvement RESOLVED (this is the first run; nothing has been addressed yet)
- Do not touch any code or plan file (the lane reads and writes artifacts only)

---

## Lane Readiness Assessment

| Dimension | Status | Notes |
|-----------|--------|-------|
| Lane definition (`spec.md`) | **READY** | Sound; defines purpose, surfaces, outputs, healthy behavior |
| Input surfaces (plans/, outputs/, ops/) | **PARTIAL** | plans/ are unevenly maintained; ops/evidence/ empty; outputs/ sparse |
| Downstream consumers | **MISSING** | strategy:planning and other recurring lanes not yet producing |
| Recurring trigger | **NOT YET WIRED** | Lane runs on-demand only; cron/interval scheduling TBD in Slice 5 |
| Improvement close loop | **NOT YET DEMONSTRATED** | No prior run to show RESOLVED → Decision Log linkage |
| `improvements.md` | **DOES NOT EXIST** | First run will create it |
| `surface-audit.md` | **DOES NOT EXIST** | First run will create it |

---

## Recommendation

**Proceed with Slice 1.** The lane definition is solid. The surfaces it depends on are uneven but usable — the learning lane's first job is to name that unevenness in `surface-audit.md` and surface the top 3 improvement signals in `improvements.md`.

The improvement loop will become meaningful once:
1. The strategy:planning lane starts consuming `improvements.md`
2. At least one improvement can be traced through Decision Log → resolution
3. The recurring trigger is wired so the lane runs on a schedule rather than on-demand

All of those are future slices. Slice 1 is the honest starting point.
