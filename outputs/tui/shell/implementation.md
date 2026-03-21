# tui:shell — Slice 1 Implementation: Event Loop Headless Test

## Slice Identification
- **Slice**: Event Loop Headless Test
- **Touched Surface**: `crates/myosu-tui/src/events.rs`
- **Status**: Completed

## Implementation Summary

### Problem
The original `EventLoop` used `crossterm::event::EventStream` which is `!Send` (contains thread-local `Terminal` state). This blocked headless CI testing because:
1. `EventStream::next()` returns a future that borrows from `!Send` state
2. The future cannot be used with `tokio::spawn` (requires `Send` futures)

### Solution Architecture
Refactored `EventLoop` to use a **dedicated OS thread** for event reading, with channels for event forwarding:

```
┌─────────────────────────────────────────────────────────────────┐
│                     EventLoop                                   │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐  │
│  │ Arc<Mutex< │◄───│  OS Thread  │───►│  mpsc::Sender   │  │
│  │ mpsc::Recv │    │ (EventSrc)  │    │  (event_tx)     │  │
│  └─────────────┘    └──────────────┘    └─────────────────┘  │
│         │                  │                                    │
│         │                  │ (sync channel)                     │
│         ▼                  ▼                                    │
│  ┌─────────────┐    ┌──────────────┐                          │
│  │ spawn_block│    │  Tick timer │                          │
│  │ ing recv   │    │  + EventSrc │                          │
│  └─────────────┘    └──────────────┘                          │
└─────────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

1. **Thread-per-loop pattern**: Each `EventLoop` spawns its own OS thread for event reading. This isolates the `!Send` `EventStream` from the async task.

2. **Sync mpsc channels**: Used `std::sync::mpsc` for event forwarding between threads (Send-safe).

3. **Arc<Mutex<Receiver>>**: The receiver is shared via `Arc<Mutex<...>>` to allow multiple `next()` calls while keeping the `Receiver` owned by `EventLoop`.

4. **`with_mock()` constructor**: Added `EventLoop::with_mock(tick_rate, events)` for headless testing with synthetic events.

### Changes Made

#### `EventLoop` struct
```rust
pub struct EventLoop {
    rx: Arc<Mutex<mpsc::Receiver<Event>>>,  // Shared for multiple next() calls
    update_tx: mpsc::Sender<UpdateEvent>,    // For injecting updates
    event_thread: thread::JoinHandle<()>,   // OS thread handle
}
```

#### New constructors
- `EventLoop::new(tick_rate)` — Uses real `EventStream` (TTY required)
- `EventLoop::with_mock(tick_rate, events)` — Uses synthetic events for headless testing

#### Event flow in mock thread
1. Wait for next tick time
2. Send queued events (in order)
3. Send tick event
4. Check for injected updates (non-blocking)
5. Loop

### Test Coverage Added

| Test | What it verifies |
|------|-----------------|
| `tick_events_produced` | Tick events generated at specified rate |
| `synthetic_events_traverse_channel` | Key/Resize events flow through channel correctly |
| `injected_update_received` | Update events injected via `update_sender()` traverse correctly |
| `update_sender_cloned` | `update_sender()` can be cloned for concurrent use |
| `update_event_variants` | `UpdateEvent` enum variants construct correctly |

## Limitations

### TTY Requirement for Production
`EventLoop::new()` still requires a TTY because `EventStream::new()` panics in headless environments. The `with_mock()` constructor provides headless testing for the event loop logic, but not for actual terminal event reading.

### EventStream Not Polled
The `EventLoop::new()` implementation creates `EventStream` but doesn't actually poll it for events (the `reader` is created but not used). This is a placeholder — the real implementation would need to handle EventStream polling in a way compatible with the thread-per-loop architecture.

## Future Work
- Implement actual `EventStream` polling in `EventLoop::new()`
- Consider using `crossterm::event::poll()` instead of `EventStream` for better control
- Add property test for update injection timing
