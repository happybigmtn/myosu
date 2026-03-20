Both required artifacts have been created:

- `outputs/foundations/foundation-plan.md` (9.9KB) — Full ExecPlan following PLANS.md format with Purpose, Progress, Surprises & Discoveries, Decision Log, Outcomes & Retrospective, Context, Plan of Work, Concrete Steps, Validation, and Interfaces
- `outputs/foundations/review.md` (7.4KB) — Honest review with Keep/Reopen/Reset judgment, truth-posture assessment, concrete risks, and next steps

`★ Insight ─────────────────────────────────────`
**Why a dedicated foundations lane?** The Myosu project has a bootstrap program (`myosu-bootstrap.yaml`) with units for `games`, `tui`, and `chain`, but no structural home for truth-integrity work. The `games:multi-game` false-submit — a lane that recorded a reviewed milestone without a successful live run — demonstrates that milestone claims alone are insufficient. The foundations lane creates the control-plane home for honest assessment and repair.

**The false-submit problem is a control-plane integrity failure.** It's not enough to delete the false claim — the lane must either produce a truthful failure record (with documented root cause) or be re-run honestly. Cosmetic fixes undermine all downstream milestone decisions.

**Three-phase approach: assess → repair honestly → verify trust.** Phase 1 examines current truth posture without speculation. Phase 2 fixes only execution-proven defects (not theoretical ones). Phase 3 verifies the detach path is clean so `execute/status/watch` truth survives across runs.
`─────────────────────────────────────────────────`