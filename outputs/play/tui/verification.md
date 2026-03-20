# `play:tui` Lane — Slice 1 Verification

## Slice: `tui-implement/slice-1-binary-skeleton`

**Date**: 2026-03-20

---

## Automated Proof Commands

### Proof 1: Binary skeleton builds ✅

```bash
$ cargo build -p myosu-play
   Compiling myosu-play v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.49s
```

**Exit code**: 0
**Result**: PASS

### Proof: Upstream still works (regression check)

```bash
$ cargo test -p myosu-tui
test result: ok. 82 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out

$ cargo test -p myosu-games
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Result**: PASS — no regression in trusted upstream crates

---

## Proof Expectations for Later Slices

These proofs are not yet applicable (Slice 1 scope):

| Proof | Command | Slice |
|-------|---------|-------|
| NLHE renderer | `cargo test -p myosu-games-poker` | Slice 2 |
| Training fold | `cargo test -p myosu-play training::tests::hand_completes_fold` | Slice 3 |
| Training showdown | `cargo test -p myosu-play training::tests::hand_completes_showdown` | Slice 3 |
| Blueprint load | `cargo test -p myosu-play blueprint::tests::load_valid_artifact` | Slice 4 |
| Advisor format | `cargo test -p myosu-play advisor::tests::format_distribution_text` | Slice 5 |

---

## Notes

- **Slice 1 proof gate satisfied**: `cargo build -p myosu-play` exits 0
- **Trusted upstream preserved**: `myosu-tui` (82 tests) and `myosu-games` (10 tests) all pass
- **No breaking changes**: workspace `Cargo.toml` only added a new member
