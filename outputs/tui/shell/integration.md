# tui:shell — Slice 1 Integration: Event Loop Headless Test

## Upstream Integration

### Shell → EventLoop

```
shell.rs:run()
    │
    ├── EventLoop::new(tick_rate)        // Creates event loop with tick timing
    │       └── Spawns OS thread for EventStream (TTY-dependent)
    │
    └── while self.running {
            event_loop.next().await       // Bridge: sync mpsc → async Option<Event>
                  │
                  ├── Event::Tick        → UI refresh tick (no-op, app calls draw)
                  ├── Event::Key(key)    → shell.handle_key(key, renderer)
                  ├── Event::Resize(w,h) → shell.terminal_size = (w, h)
                  ├── Event::Update(u)  → shell.handle_update(u, renderer)
                  └── Event::Quit       → running = false
          }
```

### EventLoop → Shell Contract

| Event | Shell Action |
|-------|--------------|
| `Tick` | No-op (application layer calls `draw` independently) |
| `Key` | Routes to input handler, screen-specific handlers |
| `Resize` | Updates terminal size for layout calculation |
| `Update` | Forwards to `renderer.handle_update()` |
| `Quit` | Sets `running = false`, exits loop |

### Update Injection Path

```
External task (solver, network)
    │
    ├── EventLoop::update_sender() → mpsc::Sender<UpdateEvent>
    │       (cloned and passed to async tasks)
    │
    └── update_tx.send(UpdateEvent::Message(...))
              │
              ├── OS thread receives via update_rx.try_recv()
              └── event_tx.send(Event::Update(...))
                        │
                        └── event_loop.next().await → Event::Update
                                  → shell.handle_update()
```

---

## Compatibility with Existing Tests

### Shell Tests

`shell.rs` currently has no tests that instantiate `EventLoop`. The shell tests mock `GameRenderer` but do not exercise the event loop path.

**Impact**: This slice does not break any existing shell tests.

### Events Tests

All 5 tests use `EventLoop::with_mock()` — no TTY required.

**Impact**: Tests run in headless CI without `#[ignore]`.

---

## Mock vs Production Behavior

| Path | Mock (`with_mock`) | Production (`new`) |
|------|-------------------|-------------------|
| Tick timing | Accurate to `tick_rate` | Accurate to `tick_rate` |
| Key events | From synthetic `Vec<Event>` | From `EventStream` (TTY) |
| Resize events | From synthetic `Vec<Event>` | From `EventStream` (TTY) |
| Update injection | Via `update_sender()` | Via `update_sender()` |

**Key invariant**: Both constructors produce identical `Event` stream structure. The shell's `match event_loop.next().await` handles all variants identically regardless of source.

---

## Limitations

### EventLoop::new() Still TTY-Dependent

`EventLoop::new()` spawns an OS thread that creates `EventStream::new()`. This still panics in headless environments. The `with_mock()` constructor exists specifically to enable headless testing of the shell's event handling logic (handled in Slice 2).

### EventStream Not Actually Polled

In `EventLoop::new()`, the `EventStream` is created (`_reader`) but not polled. The thread only sends tick events. Real terminal event reading would require polling `EventStream::next()` in a loop, which is complex due to the `!Send` nature. This is documented as future work.
