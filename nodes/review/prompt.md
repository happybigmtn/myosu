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
- **specify**: fail

## Context
- failure_class: transient_infra
- failure_signature: specify|transient_infra|handler error: cli command exited with code <n>: ponses_websocket: failed to connect to websocket: http error: <n> internal server error,url: wss://api.openai.com/v1/responses <n>-<n>-21t02:<n>:<n>.407380z error codex_api::endpoint::respons


# Agent Integration Lane — Review

Review the lane outcome for `agent-integration`.

Focus on:
- correctness
- milestone fit
- remaining blockers
