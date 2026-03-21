Created the durable integration artifacts at [agent-adapter.md](/home/r/.fabro/runs/20260320-01KM761TJDPXHRVPA7KXWHJARF/worktree/outputs/agent-integration/agent-adapter.md) and [review.md](/home/r/.fabro/runs/20260320-01KM761TJDPXHRVPA7KXWHJARF/worktree/outputs/agent-integration/review.md).

The call is now explicit: product should move into an implementation-family next, not wait for another upstream unblock. The first honest slice is `play:tui` implementation, because `agent:experience` still depends on a missing `myosu-play` binary and concrete poker renderer. I also normalized one important drift in older reviews: the `robopoker` absolute-path blocker is no longer live in the current tree.

Verification I ran:
- `CARGO_TARGET_DIR=/tmp/myosu-agent-integration-target cargo test -p myosu-games` -> 10 unit tests and 4 doctests passed
- `CARGO_TARGET_DIR=/tmp/myosu-agent-integration-target cargo test -p myosu-tui schema::tests` -> 12 tests passed
- `CARGO_TARGET_DIR=/tmp/myosu-agent-integration-target cargo test -p myosu-tui pipe::tests` -> 5 tests passed

The artifacts also note that product still lacks any checked-in implementation-family manifest, so the next control-plane move is to seed and run `play:tui` implementation assets, then follow immediately with `agent:experience` implementation.