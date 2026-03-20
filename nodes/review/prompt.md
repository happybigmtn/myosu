Goal: Foundations

Bootstrap the first honest reviewed slice for this frontier.

Inputs:
- `README.md`
- `SPEC.md`
- `PLANS.md`
- `AGENTS.md`
- `specs/031626-00-master-index.md`
- `specs/031826-fabro-primary-executor-decision.md`

Current frontier tasks:
- Fix Raspberry/Fabro defects only when they are discovered by real Myosu execution, then rerun the affected frontier until `execute/status/watch` truth is trustworthy again.
- Convert the current Raspberry-dispatched `games:multi-game` false-submit into a truthful failure or successful live run, then rerun the lane with the repaired Fabro detach path.

Required durable artifacts:
- `outputs/foundations/foundation-plan.md`
- `outputs/foundations/review.md`


## Completed stages
- **specify**: success
  - Model: MiniMax-M2.7-highspeed, 747.5k tokens in / 8.7k out
  - Files: outputs/foundations/foundation-plan.md, outputs/foundations/review.md


# Foundations Lane — Review

Review the lane outcome for `foundations`.

Focus on:
- correctness
- milestone fit
- remaining blockers
