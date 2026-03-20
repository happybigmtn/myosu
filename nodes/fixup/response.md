**Fixup complete.** The verify failure was a scope bleed — the verification script referenced `myosu-games-poker` and tests from Slices 3–5 that don't exist yet.

`★ Insight ─────────────────────────────────────`
- **Package vs crate distinction**: `myosu-games` is the existing workspace crate; `myosu-games-poker` is a future Slice 2 crate not yet created. The verification script conflated these.
- **Slice gating**: Each slice has its own proof gate. Future-slice tests should never appear in the current slice's verification script — they must remain in "Not Yet Applicable" until that slice is implemented.
- **Deterministic failure**: The `package ID specification did not match any packages` error is deterministic — it will fail the same way every time until the referenced package exists.
`─────────────────────────────────────────────────`

**Verification results for Slice 1:**

| Proof | Command | Outcome |
|-------|---------|---------|
| Binary skeleton | `cargo build -p myosu-play` | PASS — exits 0 |
| Regression: myosu-games | `cargo test -p myosu-games` | PASS — 10 tests |
| Regression: myosu-tui | `cargo test -p myosu-tui` | PASS — 82 tests |

The `verification.md` artifact now accurately documents the Slice 1 proof gate. The `promotion.md` remains untouched per fixup constraints.