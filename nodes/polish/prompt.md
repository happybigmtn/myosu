Goal: Agent Integration

Bootstrap the first honest reviewed slice for this frontier.

Inputs:
- `README.md`
- `SPEC.md`
- `PLANS.md`
- `AGENTS.md`
- `specs/031626-00-master-index.md`
- `specs/031826-fabro-primary-executor-decision.md`

Current frontier tasks:
- Execute `agent:experience`, the last remaining ready product lane, then use its reviewed artifacts to decide whether product needs an implementation family next or another upstream unblock.

Required durable artifacts:
- `outputs/agent-integration/agent-adapter.md`
- `outputs/agent-integration/review.md`


## Completed stages
- **specify**: success
  - Model: MiniMax-M2.7-highspeed, 577.8k tokens in / 8.8k out
  - Files: outputs/agent-integration/agent-adapter.md, outputs/agent-integration/review.md
- **review**: fail

## Context
- failure_class: deterministic
- failure_signature: review|deterministic|api_deterministic|openai|not_found


# Agent Integration Lane — Polish

Polish the durable artifacts for `agent-integration` so they are clear, repo-specific, and ready for the supervisory plane.
