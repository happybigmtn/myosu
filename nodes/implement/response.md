Also a stale background build — exit 0 but predates the test fixes. The foreground build and test run we did after resuming both confirmed the final state:

```
cargo build -p myosu-play   → Finished in 0.51s ✅
cargo test -p myosu-games-poker → 22 passed ✅
```

No action needed on any of these late-arriving notifications.