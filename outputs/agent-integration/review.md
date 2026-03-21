# `agent-integration` Review

## Judgment: KEEP — product should advance to implementation-family work

The honest next move for `myosu-product` is an implementation family, not
another upstream unblock.

Both product lanes already have reviewed artifacts:

- `outputs/play/tui/spec.md` + `outputs/play/tui/review.md`
- `outputs/agent/experience/spec.md` + `outputs/agent/experience/review.md`

The remaining gaps are not missing doctrine and not missing reviewed upstream
artifacts. They are missing product-owned code and missing product-owned
implementation control-plane surfaces.

## Why This Is the Right Judgment

1. `play:tui` is explicitly ready for implementation-family work now. Its
   review says the lane is unblocked for an implementation-family workflow and
   defines a concrete first slice: create `myosu-play`.

2. `agent:experience` also says to proceed to implementation-family work, but
   its own review makes clear that most of the lane extends `myosu-play`.
   Slices 3 through 9 depend on `play:tui` Slice 1.

3. The major upstream blocker cited in the product reviews has already been
   reduced. `outputs/games/traits/verification.md` shows that the absolute-path
   robopoker dependency in `myosu-games` was replaced with pinned git
   dependencies and verified with `cargo fetch`.

4. The workflow library already classifies interface bringup like `play:tui`
   into the `implement/` family when the work is pure code/UI bringup. That is
   the current product situation.

## Integration Decision

Product needs implementation-family execution next.

The first lane should be `play:tui`, because it creates the executable binary
surface that `agent:experience` extends. After `play:tui` Slice 1 lands,
product should immediately promote `agent:experience` into its own
implementation-family loop. If early overlap is useful, only the independent
`myosu-tui` additions from `agent:experience` should start before the binary
exists.

## Remaining Real Risks

- No product implementation-family surfaces exist yet under
  `fabro/workflows/implement/`, `fabro/run-configs/implement/`, or
  `fabro/programs/`. This is the next control-plane gap to fix.
- `crates/myosu-play/` and `crates/myosu-games-poker/` are still absent, so
  product has no executable surface yet.
- The product reviews still mention the old robopoker portability blocker. That
  language is now outdated and should be read through the newer
  `games:traits` verification artifact.

## Concrete Recommendation

Seed a lane-scoped `play:tui` implementation program next, following the same
artifact contract already used by `myosu-games-traits-implementation.yaml`:

- consume `outputs/play/tui/spec.md` and `outputs/play/tui/review.md`
- produce `outputs/play/tui/implementation.md` and
  `outputs/play/tui/verification.md`

Once `play:tui` Slice 1 is verified, seed the matching implementation-family
surface for `agent:experience`.
