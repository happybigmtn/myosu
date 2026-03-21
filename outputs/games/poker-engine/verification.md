# `games:poker-engine` Verification — Slice 1

## Proof Commands That Passed

| Command | Exit Code | Result |
|---------|-----------|--------|
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo build -p myosu-games-poker --offline` | 0 | Built the new crate and resolved `rbp-nlhe`, `rbp-cards`, and `rbp-gameplay` from the pinned robopoker git source |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-games-poker --offline` | 0 | 2 unit tests passed; 0 failed |
| `cargo tree -p myosu-games-poker -e features --offline` | 0 | Confirmed `rbp-nlhe feature "serde"` and `rbp-mccfr feature "serde"` are active in the resolved feature graph |

## Test Results

```text
running 2 tests
test tests::game_type_reexport_includes_nlhe_heads_up ... ok
test tests::robopoker_nlhe_serde_surface_is_enabled ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Environment Notes

- A plain `cargo build -p myosu-games-poker` was attempted first and failed in this sandbox because Cargo tried to refresh the crates.io index and the environment has no DNS/network access.
- An offline build without overriding `CARGO_TARGET_DIR` also failed because the workspace default target path resolves to a read-only location outside the writable sandbox.
- The slice was therefore verified with `--offline` plus `CARGO_TARGET_DIR=/tmp/myosu-cargo-target`, which stays inside writable local storage and exercises the new package successfully.

## Risks Reduced

- **Workspace package missing:** Reduced. `myosu-games-poker` now exists and is registered in the workspace, so the package is discoverable by Cargo.
- **Serde feature uncertainty on robopoker NLHE types:** Reduced. `NlheInfo` and `NlheEdge` now have both compile-time and runtime smoke-check coverage through the crate root and test suite.
- **Wire-slice ambiguity:** Reduced. `cargo tree -e features` confirms the exact serde feature path that Slice 3 depends on.

## Risks That Remain

- **Shared robopoker rev is still duplicated:** Reduced but not eliminated. This slice pins the new crate to the same reviewed rev, but `crates/myosu-games/Cargo.toml` still carries its own identical literal pin because that crate is outside the poker-engine owned surface.
- **Solver/query/exploit/training surfaces remain unimplemented:** Unchanged. This slice intentionally stops at the crate skeleton.
- **Default workspace build path is not sandbox-friendly:** Unchanged. Proof in this environment still needs a writable `CARGO_TARGET_DIR`.
