Both required durable artifacts are now in place:

**`outputs/foundations/foundation-plan.md`** (292 lines) — A PLANS.md-style ExecPlan that:
- Establishes the execution truth problem (detach path returns run ID before worker starts)
- Documents the `games:multi-game` false-submit case (`01KM2BS4ASVRXVT2ND1GVVMKJ0`) as a concrete instance
- Splits work into Track A (detach path repair in `/home/r/coding/fabro`) and Track B (foreground Fabro fallback for `games:multi-game`)
- Includes a submit-path trust inventory table across all known lanes
- Defines explicit acceptance criteria for when detach can be marked trustworthy again

**`outputs/foundations/review.md`** (159 lines) — A lane review that:
- Marks `raspberry execute --detach` and `raspberry status/watch` (for detach) as **UNVERIFIED**
- Marks direct foreground Fabro as **TRUSTED** with concrete run ID evidence
- Explicitly requires the `games:multi-game` false-submit to be converted to an honest result
- Provides a submit-path trust inventory across all lanes (8 lanes assessed: 5 trusted via foreground, 1 broken via detach, 2 verified via foreground)
- Includes contributor guidance: use foreground Fabro as the honest fallback until detach is repaired

`★ Insight ─────────────────────────────────────`
This lane is unusual — it reviews the **control plane itself**, not product code. The key insight is that the false-submit problem is a **class defect** (the detach mechanism) not an instance defect (one bad run). The review treats the foreground path as the gold standard because it has consistently produced real manifests and stage labels, while the detach path has produced run IDs without workers. The honest inventory table gives contributors a clear signal about which submit paths to trust today.
`─────────────────────────────────────────────────`