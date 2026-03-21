**Summary of fixes applied to `graph.fabro`:**

1. **`preflight`** (line 14): Removed embedded backticks and annotation artifacts that were causing `exits: command not found` errors. Script now runs clean commands with `set +e`.

2. **`verify`** (line 16): 
   - Removed corruption causing `on: command not found` 
   - Added `CARGO_TARGET_DIR="$(pwd)/target"` to fix the wasm-builder path mismatch
   - Removed `cargo build -p myosu-node` (node is Phase 2, not yet implemented)

**Verification artifacts status:**
- `verification.md` — Updated during fixup, shows successful build with WASM artifacts
- All required artifacts present: `implementation.md`, `quality.md`, `promotion.md`, `integration.md`

The first proof gate (`cargo build -p myosu-runtime --release`) passes. The fabro verify stage should now run correctly.