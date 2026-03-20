# Foundations Lane Review

## Keep / Reopen / Reset Judgment

**Judgment: KEEP (lane active, awaiting execution)**

The foundations lane exists to repair the control-plane truth path so that all other lanes can trust what `execute/status/watch` reports. This is a genuine bootstrap defect, not a code quality issue.

## Current Lane State

The `foundations` lane is **not yet defined** in `fabro/programs/myosu.yaml`. The lane is being bootstrapped in this slice. Artifacts produced:

- `outputs/foundations/foundation-plan.md` (this file's companion)
- `outputs/foundations/review.md` (this file)

## Concrete Risks

### Risk 1: False-Submit Corruption Scope Is Unknown

The `games:multi-game` false-submit is the known canary. It is unclear whether other lanes have also recorded false submits through the same supervisory dispatch path. Until the repair run completes, the scope of corruption is bounded only by the number of lanes dispatched through the broken path.

**Mitigation:** Fix the `games:multi-game` false-submit first. Use its repair as a verification case for the truth path. If `execute/status/watch` still does not match real outcomes after the repair, the scope of the defect is wider than just `games:multi-game`.

### Risk 2: `execute/status/watch` Truth Is Unverified Across All Lanes

The control-plane truth path has never been independently verified against actual execution outcomes. The `games:traits` lane produced honest reviewed artifacts, but those artifacts were not produced through a Fabro `execute/status/watch` cycle — they were bootstrapped directly. This means the `execute/status/watch` signal path has not been proven trustworthy.

**Mitigation:** After fixing the `games:multi-game` false-submit, run a verification cycle that compares `execute/status/watch` output against the actual lane artifacts and exit codes.

### Risk 3: The Fabro Detach Path May Have Systemic Issues

If the false-submit was caused by a systemic issue in the Fabro detach path (rather than an isolated dispatch error), fixing only the `games:multi-game` lane will not prevent other lanes from false-submitting in the future.

**Mitigation:** The repair run must include a verification step that confirms the control-plane signal reflects the actual execution state. If this verification fails, the foundations lane scope widens to include a systemic Fabro detach repair.

### Risk 4: No `foundations` Unit in Current Program Manifest

`fabro/programs/myosu.yaml` does not have a `foundations` unit. The lane cannot be executed through the normal Raspberry dispatch path until it is added.

**Mitigation:** This review is the first artifact. Adding the `foundations` unit to `myosu.yaml` and creating `myosu-foundations.yaml` is the first concrete step after this artifact is produced.

## Proof Commands

| Context | Command | Expected |
|---------|---------|----------|
| Foundations lane bootstrap | `test -f outputs/foundations/foundation-plan.md && test -f outputs/foundations/review.md` | Exit 0 |
| After foundations manifest added | `test -f fabro/programs/myosu-foundations.yaml` | Exit 0 |
| After `games:multi-game` repair | `cat outputs/games/multi-game/review.md \| grep -q "false-submit"` must return non-zero (false-submit cleared) | Exit non-zero |
| `execute/status/watch` verification | Compare `raspberry status --manifest fabro/programs/myosu-foundations.yaml --lane games-multi-game-repair` against actual lane artifacts | Signals match |

## File Reference Index

| File | Role |
|------|------|
| `outputs/foundations/foundation-plan.md` | This lane's plan artifact |
| `outputs/foundations/review.md` | This file |
| `fabro/programs/myosu.yaml` | Top-level program manifest; needs `foundations` unit added |
| `fabro/programs/myosu-bootstrap.yaml` | Bootstrap program manifest (dependency baseline) |
| `fabro/programs/myosu-foundations.yaml` | Foundations program manifest (to be created) |
| `outputs/games/multi-game/review.md` | Contains the false-submit that this lane must repair |
| `fabro/programs/myosu-games-traits-implementation.yaml` | Example of a lane-specific implementation manifest (for shape reference) |
