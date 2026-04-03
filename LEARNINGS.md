# LEARNINGS

## 2026-04-02

- `myosu-play --smoke-test` must bypass `~/.codexpoker` auto-discovery when it is being used as a smoke proof. Ambient local blueprint state can hide real regressions; the truthful default for smoke mode is a built-in demo surface unless the proof explicitly targets blueprint loading.
- `cargo audit` is not a truthful security gate in this repo unless warning-class advisories are handled explicitly. A zero exit status paired with `13 allowed warnings found` still leaves live security debt in the tree.
- Moving owned code from `bincode` 1.x to 2.x is not a fix for `RUSTSEC-2025-0141`; both major lines are currently flagged unmaintained, so the real decision is replacement, isolation, or explicit acceptance.
