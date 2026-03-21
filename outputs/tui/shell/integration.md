# `tui:shell` Integration — Slice 1

## Integration Status

This slice remains internal to [events.rs](/home/r/.fabro/runs/20260320-01KM755GFB9SS6HFPKZYFWVJDP/worktree/crates/myosu-tui/src/events.rs). It improves proof coverage without changing the external shell contract.

The fixup in this lane is also internal: it normalizes the shell lane's proof
commands to exact cargo invocations/selectors so Fabro verification can execute
the intended `myosu-tui` regression surfaces instead of relying on shorthand.

## Preserved Contracts

- `Shell::run()` still depends on `EventLoop::new(tick_rate)`.
- `EventLoop::update_sender()` still exposes the same cloned `mpsc::UnboundedSender<UpdateEvent>` contract.
- Key and resize events still surface as `Event::Key` and `Event::Resize`.
- Focus, mouse, and paste events are still ignored.
- The lane still exports the same shell, screen, input, renderer, schema, theme,
  and pipe surfaces; no downstream crate or runtime contract changed.

## Downstream Impact

- Downstream shells and game renderers do not need code changes.
- Upstream lane consumers now have CI-safe proof that the event loop behaves in headless execution, which matters for Fabro runs and future pipe/agent automation.
- Fabro-style verification can now target real `myosu-tui` tests with exact
  selectors recorded in [spec.md](/home/r/.fabro/runs/20260320-01KM755GFB9SS6HFPKZYFWVJDP/worktree/outputs/tui/shell/spec.md).

## Follow-On Integration Work

- Slice 2 should prove the shell-side integration from input handling into `ScreenManager`.
- Later slices should add render coverage across non-Game screens and address the remaining schema proof gap called out in [review.md](/home/r/.fabro/runs/20260320-01KM755GFB9SS6HFPKZYFWVJDP/worktree/outputs/tui/shell/review.md).
