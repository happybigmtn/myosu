**Fixup complete.** The verification artifact now reflects the actual working command.

`★ Insight ─────────────────────────────────────`
- **Shared `CARGO_TARGET_DIR` causes wasm-builder path mismatches**: The substrate wasm-builder resolves package paths relative to the build cache location. When multiple worktrees share a `CARGO_TARGET_DIR`, the builder looks for packages at paths from the wrong worktree.
- **Worktree isolation requires `CARGO_TARGET_DIR` per-worktree**: Each worktree must have its own `CARGO_TARGET_DIR` pointing to a local `target/` directory to ensure wasm-builder finds the correct `Cargo.toml` and source paths.
`─────────────────────────────────────────────────`

## Summary

**Root cause**: The verification script ran `cargo build -p myosu-runtime --release` without setting `CARGO_TARGET_DIR`, causing the wasm-builder to look in the wrong build cache.

**Fix**: Updated `verification.md` with:
- Correct `CARGO_TARGET_DIR` pointing to worktree-local target
- Actual build output showing exit 0
- Actual WASM artifact sizes (717KB / 679KB / 152KB)

**Verification**: Build passes with `CARGO_TARGET_DIR=/home/r/.fabro/runs/20260320-01KM6SVGFXHVETGJZ7MN3HTMMM/worktree/target cargo build -p myosu-runtime --release` — WASM artifacts confirmed at expected locations.