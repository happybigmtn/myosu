# Specification: Spectator Protocol — Watching Agent vs Agent Play

Source: DESIGN.md 9.24 spectator mode
Status: Draft
Date: 2026-03-17
Depends-on: AX-01..06 (agent experience), GP-01..04 (gameplay), TU-01..12 (TUI)
Blocks: DESIGN.md 9.24 implementation

## Purpose

DESIGN.md 9.24 designs a spectator screen where humans watch agent vs agent
gameplay. This requires a data source: someone must be playing a game, and
the spectator must receive events from that game in real-time.

This spec defines how game events flow from active sessions to spectators,
including fog-of-war (hidden hands), event subscription, and the reveal
mechanism after showdown.

## Architecture

Three possible data sources for spectator events:

| Source | Latency | Availability | Complexity |
|--------|---------|--------------|------------|
| On-chain events | ~6s (block time) | Always | High (requires chain indexer) |
| Miner axon | ~100ms | Only when miner is active | Medium |
| Local relay | ~10ms | Only on same machine | Low |

**Decision: local relay for Phase 0, miner axon for Phase 1.**

Phase 0: spectator watches a local `myosu-play --pipe` session between two
agents running on the same machine. The relay is just a Unix domain socket
or named pipe that forwards game events.

Phase 1: spectator connects to a miner's axon endpoint and subscribes to
game events from that miner's active sessions.

## Protocol

### Event stream

Game events are JSON lines on a subscription channel. The event types
MUST reuse the game state JSON schema defined in AX-01 (031626-10-agent-
experience.md). This ensures agents, spectators, and players all parse
the same event vocabulary. The spectator stream is a filtered subset —
it omits any fields that would reveal hidden information (hole cards,
info sets).

Example stream (poker, showing AX-01-compatible events):

```json
{"type": "hand_start", "hand": 142, "players": ["agent:deepbluff", "miner:12"]}
{"type": "action", "player": "miner:12", "action": "raise", "amount": 12, "pot": 15}
{"type": "action", "player": "agent:deepbluff", "action": "call", "pot": 27}
{"type": "street", "name": "flop", "board": ["Ks", "Qh", "7c"]}
{"type": "action", "player": "miner:12", "action": "bet", "amount": 14, "pot": 41}
{"type": "action", "player": "agent:deepbluff", "action": "raise", "amount": 38, "pot": 65}
{"type": "action", "player": "miner:12", "action": "call", "pot": 79}
{"type": "street", "name": "turn", "board": ["2d"]}
{"type": "showdown", "winner": "agent:deepbluff", "amount": 142, "hands": {"agent:deepbluff": ["As", "Kh"], "miner:12": ["Qd", "Jd"]}}
```

### Fog of war

During play, spectators see:
- Community cards (as they're dealt)
- Actions (who did what, pot size)
- Stack sizes
- Player identifiers

Spectators do NOT see:
- Hole cards (shown as `·· ··` per DESIGN.md notation)
- Solver recommendations
- Information sets

After showdown or fold, the `showdown` event includes all hole cards.
The TUI switches to the reveal view (DESIGN.md 9.24 second mockup).

### Subscription commands

From the lobby or game screen:

```
/spectate              list active sessions available for spectating
/spectate <subnet_id>  watch the highest-action session on that subnet
/spectate <session_id> watch a specific session
```

The spectator TUI is read-only — no input is accepted except navigation
keys ([n] next, [q] quit, [r] reveal after showdown).

## Scope

### AC-SP-01: Local Spectator Relay

- Where: `crates/myosu-play/src/spectate.rs (new)`
- How: When a game session is active (training mode or chain mode), emit
  JSON event lines to an optional spectator socket. The spectator client
  connects and receives the event stream.

  ```rust
  pub struct SpectatorRelay {
      listeners: Vec<UnixStream>,
  }

  impl SpectatorRelay {
      pub fn emit(&self, event: &GameEvent) {
          let Ok(json) = serde_json::to_string(event) else { return };
          for listener in &self.listeners {
              let _ = writeln!(listener, "{}", json);
          }
      }
  }
  ```

  Socket path: `~/.myosu/spectate/<session_id>.sock`

  The relay is opt-in: `myosu-play --spectate-socket` enables it.

- Required tests:
  - `spectate::tests::relay_emits_events`
  - `spectate::tests::relay_handles_disconnected_listener`
  - `spectate::tests::events_are_valid_json`
- Pass/fail:
  - Connected listener receives JSON event lines
  - Disconnected listener doesn't crash the game
  - All events parse as valid `GameEvent`

### AC-SP-02: Spectator TUI Screen

- Where: `crates/myosu-tui/src/screens/spectate.rs (new)`
- How: New `Screen::Spectate` variant renders the spectator view per
  DESIGN.md 9.24. Connects to the relay socket and renders events as they
  arrive. Uses the same `GameRenderer` as the player view but with all
  hero-specific data stripped (no hole cards, no solver advisor, no
  equilibrium panel).

  ```rust
  struct SpectateScreen {
      session_id: String,
      renderer: Box<dyn GameRenderer>,
      events: Vec<GameEvent>,
      revealed: bool,
  }
  ```

  Key bindings:
  - `n` — switch to next available session
  - `r` — reveal hole cards (only after showdown)
  - `q` — return to lobby

- Required tests:
  - `spectate::tests::renders_fog_of_war`
  - `spectate::tests::reveal_shows_hole_cards_after_showdown`
  - `spectate::tests::reveal_blocked_during_play`
- Pass/fail:
  - Hole cards shown as `·· ··` during play
  - After showdown event, `r` key reveals hole cards
  - `r` key has no effect during active play (before showdown)

### AC-SP-03: Session Discovery

- Where: `crates/myosu-play/src/spectate.rs (extend)`
- How: `/spectate` with no arguments lists available sessions by scanning
  `~/.myosu/spectate/` for active sockets. Display as:

  ```
  MYOSU / SPECTATE

  ACTIVE SESSIONS

    id     game        players                    hand
    1      nlhe-hu     agent:deepbluff vs miner:12  142
    2      plo         agent:omaha4 vs miner:8      31

  type session id to watch, or [q] quit.

  > 1
  ```

  For Phase 1 (miner axon): query the miner's `/sessions` endpoint for
  active sessions instead of local sockets.

- Required tests:
  - `spectate::tests::discover_local_sessions`
  - `spectate::tests::no_sessions_shows_empty_message`
- Pass/fail:
  - Active sockets in `~/.myosu/spectate/` appear in session list
  - No sockets → "no active sessions. start a game first."

## Phase 1 extension: miner axon spectator

When the agent experience spec (AX-01..06) adds HTTP/WebSocket APIs, the
spectator protocol extends naturally:

```
GET /api/sessions              → list active sessions on this miner
WS  /api/sessions/:id/spectate → event stream (same JSON format)
```

The TUI spectator screen switches from Unix socket to WebSocket with no
rendering changes. The event format is identical.

## Decision log

- 2026-03-17: Local relay for Phase 0. Rationale: simplest possible
  implementation, proves the spectator UX without network dependencies.
- 2026-03-17: JSON lines (not binary). Rationale: human-readable,
  debuggable, low event rate (~1-2 events/second in poker).
- 2026-03-17: Fog of war is enforced at the relay, not the renderer.
  The relay never sends hole cards during play. This prevents spectator
  clients from cheating by reading hidden data.
