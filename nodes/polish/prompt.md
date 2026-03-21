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
  - Model: gpt-5.4, 10.4m tokens in / 58.4k out
  - Files: README.md, fabro/README.md, fabro/checks/fabro-local-dispatch.sh, fabro/checks/foundations-verify.sh, fabro/checks/foundations-write-artifact.sh, fabro/programs/README.md, fabro/programs/myosu-foundations.yaml, fabro/programs/myosu.yaml, fabro/run-configs/bootstrap/foundations.toml, fabro/workflows/bootstrap/foundations.fabro, outputs/foundations/foundation-plan.md, outputs/foundations/review.md, plans/031926-iterative-execution-and-raspberry-hardening.md
- **review**: fail

## Context
- failure_class: deterministic
- failure_signature: review|deterministic|api_deterministic|openai|not_found


# Foundations Lane — Polish

Polish the durable artifacts for `foundations` so they are clear, repo-specific, and ready for the supervisory plane.
