# Myosu No-Ship Ledger

Last updated: 2026-03-30

No active no-ship condition is open for continued stage-0 development or for a
final stage-0 completion claim on the current proof posture.

Current stage:
- stage-0 core implementation stack is complete
- productization (`009`) is complete
- CI hardening (`010`) is complete on hosted GitHub Actions evidence
- release governance (`011`) is complete, including the current invariant and
  completion-contract sync
- canonical spec freshness (`002`) is complete locally
- no promoted follow-on lane is active beyond the `001` control-plane view

## Policy

"No-ship" means:
- do not trust a completion claim
- do not widen scope
- do not advertise a capability as ready

Conditions that open a no-ship entry:
- named proof is not trustworthy (INV-002)
- validator determinism violated (INV-003)
- solver/gameplay separation breached (INV-004)
- plan/land/runtime truth divergence (INV-005)

If any proof named in `ops/release-gate-stage0.md` stops passing, reopen a
no-ship entry immediately and treat the completion claim as untrusted until the
surfaces are back in sync.
