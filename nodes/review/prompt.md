Goal: Implement the next approved `play:tui` slice.

Inputs:
- `spec.md`
- `review.md`

Scope:
- work only inside the smallest next approved implementation slice
- treat the reviewed lane artifacts as the source of truth
- keep changes aligned with the owned surfaces for `play:tui`

Required curated artifacts:
- `implementation.md`
- `verification.md`
- `promotion.md`


## Completed stages
- **preflight**: success
  - Script: `set +e
cargo build -p myosu-play
cargo test -p myosu-games-poker
cargo test -p myosu-play training::tests::hand_completes_fold
cargo test -p myosu-play training::tests::hand_completes_showdown
cargo test -p myosu-play blueprint::tests::load_valid_artifact
cargo test -p myosu-play advisor::tests::format_distribution_text
true`
  - Stdout: (empty)
  - Stderr:
    ```
    Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-play` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-play` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-play` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-play` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-play` did not match any packages
    ```
- **implement**: success
  - Model: MiniMax-M2.7-highspeed, 34.0k tokens in / 262 out
  - Files: Cargo.lock, Cargo.toml, crates/myosu-games-poker/Cargo.toml, crates/myosu-games-poker/src/lib.rs, crates/myosu-games-poker/src/renderer.rs, crates/myosu-games-poker/src/truth_stream.rs, crates/myosu-play/Cargo.toml, crates/myosu-play/src/main.rs, outputs/play/tui/implementation.md, outputs/play/tui/verification.md
- **verify**: success
  - Script: `set -e
cargo build -p myosu-play
cargo test -p myosu-games-poker
cargo test -p myosu-play training::tests::hand_completes_fold
cargo test -p myosu-play training::tests::hand_completes_showdown
cargo test -p myosu-play blueprint::tests::load_valid_artifact
cargo test -p myosu-play advisor::tests::format_distribution_text`
  - Stdout:
    ```
    (25 lines omitted)
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    ```
  - Stderr:
    ```
    (335 lines omitted)
       |
    16 | const SUIT_CLUBS: &str = "♣";
       |       ^^^^^^^^^^
    
    warning: function `render_card` is never used
      --> crates/myosu-games-poker/src/renderer.rs:19:4
       |
    19 | fn render_card(area: &Rect, buf: &mut Buffer, x: u16, y: u16, rank: &str, suit: &str, style: Style) {
       |    ^^^^^^^^^^^
    
    warning: function `render_hidden` is never used
      --> crates/myosu-games-poker/src/renderer.rs:31:4
       |
    31 | fn render_hidden(area: &Rect, buf: &mut Buffer, x: u16, y: u16, style: Style) {
       |    ^^^^^^^^^^^^^
    
    warning: function `render_slot` is never used
      --> crates/myosu-games-poker/src/renderer.rs:43:4
       |
    43 | fn render_slot(area: &Rect, buf: &mut Buffer, x: u16, y: u16, style: Style) {
       |    ^^^^^^^^^^^
    
    warning: `myosu-games-poker` (lib) generated 8 warnings (run `cargo fix --lib -p myosu-games-poker` to apply 1 suggestion)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.47s
         Running unittests src/main.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_play-1bb351a867b90541)
    ```
- **quality**: success
  - Script: `set -e
QUALITY_PATH='outputs/play/tui/quality.md'
IMPLEMENTATION_PATH='outputs/play/tui/implementation.md'
VERIFICATION_PATH='outputs/play/tui/verification.md'
placeholder_hits=""
scan_placeholder() {
  surface="$1"
  if [ ! -e "$surface" ]; then
    return 0
  fi
  hits="$(rg -n -i -g '*.rs' -g 'Cargo.toml' -g '*.toml' 'TODO|stub|placeholder|future slice|not yet implemented|compile-only|for now|will implement' "$surface" || true)"
  if [ -n "$hits" ]; then
    if [ -n "$placeholder_hits" ]; then
      placeholder_hits="$(printf '%s\n%s' "$placeholder_hits" "$hits")"
    else
      placeholder_hits="$hits"
    fi
  fi
}
true
artifact_hits="$(rg -n -i 'manual proof still required|future slice|compile-only|placeholder|stub implementation|not yet fully implemented' "$IMPLEMENTATION_PATH" "$VERIFICATION_PATH" 2>/dev/null || true)"
warning_hits="$(rg -n 'warning:' "$IMPLEMENTATION_PATH" "$VERIFICATION_PATH" 2>/dev/null || true)"
manual_hits="$(rg -n -i 'manual proof still required|manual;' "$VERIFICATION_PATH" 2>/dev/null || true)"
placeholder_debt=no
warning_debt=no
artifact_mismatch_risk=no
manual_followup_required=no
[ -n "$placeholder_hits" ] && placeholder_debt=yes
[ -n "$warning_hits" ] && warning_debt=yes
[ -n "$artifact_hits" ] && artifact_mismatch_risk=yes
[ -n "$manual_hits" ] && manual_followup_required=yes
quality_ready=yes
if [ "$placeholder_debt" = yes ] || [ "$warning_debt" = yes ] || [ "$artifact_mismatch_risk" = yes ] || [ "$manual_followup_required" = yes ]; then
  quality_ready=no
fi
mkdir -p "$(dirname "$QUALITY_PATH")"
cat > "$QUALITY_PATH" <<EOF
quality_ready: $quality_ready
placeholder_debt: $placeholder_debt
warning_debt: $warning_debt
artifact_mismatch_risk: $artifact_mismatch_risk
manual_followup_required: $manual_followup_required

## Touched Surfaces
- (none declared)

## Placeholder Hits
$placeholder_hits

## Artifact Consistency Hits
$artifact_hits

## Warning Hits
$warning_hits

## Manual Followup Hits
$manual_hits
EOF
test "$quality_ready" = yes`
  - Stdout: (empty)
  - Stderr: (empty)
- **fixup**: success
  - Model: MiniMax-M2.7-highspeed, 248.6k tokens in / 3.8k out
  - Files: outputs/play/tui/implementation.md, outputs/play/tui/quality.md, outputs/play/tui/verification.md
- **verify**: success
  - Script: `set -e
cargo build -p myosu-play
cargo test -p myosu-games-poker
cargo test -p myosu-play training::tests::hand_completes_fold
cargo test -p myosu-play training::tests::hand_completes_showdown
cargo test -p myosu-play blueprint::tests::load_valid_artifact
cargo test -p myosu-play advisor::tests::format_distribution_text`
  - Stdout:
    ```
    (25 lines omitted)
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    ```
  - Stderr:
    ```
    (335 lines omitted)
       |
    16 | const SUIT_CLUBS: &str = "♣";
       |       ^^^^^^^^^^
    
    warning: function `render_card` is never used
      --> crates/myosu-games-poker/src/renderer.rs:19:4
       |
    19 | fn render_card(area: &Rect, buf: &mut Buffer, x: u16, y: u16, rank: &str, suit: &str, style: Style) {
       |    ^^^^^^^^^^^
    
    warning: function `render_hidden` is never used
      --> crates/myosu-games-poker/src/renderer.rs:31:4
       |
    31 | fn render_hidden(area: &Rect, buf: &mut Buffer, x: u16, y: u16, style: Style) {
       |    ^^^^^^^^^^^^^
    
    warning: function `render_slot` is never used
      --> crates/myosu-games-poker/src/renderer.rs:43:4
       |
    43 | fn render_slot(area: &Rect, buf: &mut Buffer, x: u16, y: u16, style: Style) {
       |    ^^^^^^^^^^^
    
    warning: `myosu-games-poker` (lib) generated 8 warnings (run `cargo fix --lib -p myosu-games-poker` to apply 1 suggestion)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.47s
         Running unittests src/main.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_play-1bb351a867b90541)
    ```
- **quality**: success
  - Script: `set -e
QUALITY_PATH='outputs/play/tui/quality.md'
IMPLEMENTATION_PATH='outputs/play/tui/implementation.md'
VERIFICATION_PATH='outputs/play/tui/verification.md'
placeholder_hits=""
scan_placeholder() {
  surface="$1"
  if [ ! -e "$surface" ]; then
    return 0
  fi
  hits="$(rg -n -i -g '*.rs' -g 'Cargo.toml' -g '*.toml' 'TODO|stub|placeholder|future slice|not yet implemented|compile-only|for now|will implement' "$surface" || true)"
  if [ -n "$hits" ]; then
    if [ -n "$placeholder_hits" ]; then
      placeholder_hits="$(printf '%s\n%s' "$placeholder_hits" "$hits")"
    else
      placeholder_hits="$hits"
    fi
  fi
}
true
artifact_hits="$(rg -n -i 'manual proof still required|future slice|compile-only|placeholder|stub implementation|not yet fully implemented' "$IMPLEMENTATION_PATH" "$VERIFICATION_PATH" 2>/dev/null || true)"
warning_hits="$(rg -n 'warning:' "$IMPLEMENTATION_PATH" "$VERIFICATION_PATH" 2>/dev/null || true)"
manual_hits="$(rg -n -i 'manual proof still required|manual;' "$VERIFICATION_PATH" 2>/dev/null || true)"
placeholder_debt=no
warning_debt=no
artifact_mismatch_risk=no
manual_followup_required=no
[ -n "$placeholder_hits" ] && placeholder_debt=yes
[ -n "$warning_hits" ] && warning_debt=yes
[ -n "$artifact_hits" ] && artifact_mismatch_risk=yes
[ -n "$manual_hits" ] && manual_followup_required=yes
quality_ready=yes
if [ "$placeholder_debt" = yes ] || [ "$warning_debt" = yes ] || [ "$artifact_mismatch_risk" = yes ] || [ "$manual_followup_required" = yes ]; then
  quality_ready=no
fi
mkdir -p "$(dirname "$QUALITY_PATH")"
cat > "$QUALITY_PATH" <<EOF
quality_ready: $quality_ready
placeholder_debt: $placeholder_debt
warning_debt: $warning_debt
artifact_mismatch_risk: $artifact_mismatch_risk
manual_followup_required: $manual_followup_required

## Touched Surfaces
- (none declared)

## Placeholder Hits
$placeholder_hits

## Artifact Consistency Hits
$artifact_hits

## Warning Hits
$warning_hits

## Manual Followup Hits
$manual_hits
EOF
test "$quality_ready" = yes`
  - Stdout: (empty)
  - Stderr: (empty)
- **clear_promotion**: success
  - Script: `rm -f outputs/play/tui/promotion.md`
  - Stdout: (empty)
  - Stderr: (empty)


# Gameplay TUI Implementation Lane — Review

Review only the current slice for `tui-implement`.

Current Slice Contract:
Inspect the relevant repo surfaces, preserve existing doctrine, and produce the lane artifacts honestly.


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Focus on:
- slice scope discipline
- proof-gate coverage for the active slice
- touched-surface containment
- implementation and verification artifact quality
- remaining blockers before the next slice

Deterministic evidence:
- treat `quality.md` as machine-generated truth about placeholder debt, warning debt, manual follow-up, and artifact mismatch risk
- if `quality.md` says `quality_ready: no`, do not bless the slice as merge-ready
