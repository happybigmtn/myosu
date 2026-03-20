# Agent Integration — Adapter Reference

## What This Document Is

This document describes the **agent adapter**: the integration layer between a programmatic agent (LLM, bot, script) and the Myosu game surfaces owned by `agent:experience`.

The adapter is not a Myosu crate. It is the **harness code** that an agent developer writes to connect their agent logic to Myosu's pipe protocol and context surfaces. This document serves as the canonical reference for that integration.

---

## Relationship to `agent:experience`

```
┌─────────────────────────────────────────────────────────────────┐
│                        agent developer                           │
│                                                                  │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │              agent adapter (THIS DOCUMENT)              │   │
│   │                                                          │   │
│   │  - game loop driver                                      │   │
│   │  - pipe protocol handler                                 │   │
│   │  - context file manager                                  │   │
│   │  - action selector (LLM / bot logic)                    │   │
│   │  - reflection coordinator                                │   │
│   └─────────────────────────────────────────────────────────┘   │
│                            │                                     │
│          ┌─────────────────┼─────────────────┐                  │
│          │                 │                  │                  │
│          ▼                 ▼                  ▼                  │
│   ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐     │
│   │ agent:      │  │ agent:       │  │ agent:           │     │
│   │ experience  │  │ experience   │  │ experience       │     │
│   │ (pipe mode) │  │ (context)    │  │ (spectator)     │     │
│   └─────────────┘  └──────────────┘  └──────────────────┘     │
│                            │                                     │
└────────────────────────────┼─────────────────────────────────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │  myosu game     │
                    │  (off-chain)    │
                    └─────────────────┘
```

**`agent:experience`** owns what the agent **sees**:
- Pipe mode (`--pipe`) — stdin/stdout text protocol
- Context file (`--context <path>`) — persistent identity and memory
- Narration (`--narrate`) — atmospheric prose rendering
- Journal (`journal.md`) — append-only markdown autobiography
- Lobby (`MYOSU/LOBBY`) — game selection interface
- Spectator relay (`~/.myosu/spectate/<id>.sock`) — event stream

**`agent:adapter`** (this document) describes how an agent developer **acts** on what it sees:
- Drives the game loop (connect → play → disconnect)
- Sends actions and reads observations via pipe
- Loads/saves context across sessions
- Coordinates reflection prompts
- Observes games via spectator relay

---

## The Agent Game Loop

### Phase 0: Connection

```python
# Pseudocode for agent adapter
import subprocess
import json
import os

# Start myosu-play in pipe mode with optional context and narration
cmd = [
    "myosu-play", "--pipe",
    "--context", "./koan.json",      # optional persistence
    "--narrate",                     # optional prose mode
]
process = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)

# Or connect to a specific subnet directly
cmd = [
    "myosu-play", "--pipe",
    "--context", "./koan.json",
    "--subnet", "1",                # optional: skip lobby
]
```

### Phase 1: Observe

The agent reads lines from stdout. Each line is a game state in pipe format:

```
MYOSU/NLHE-HU/HAND47
board: Ts 7h 2c
you: As Kh 94bb BB
opponent: -- -- 94bb SB
pot: 12bb
action: solver raises 6bb
>
```

Or in `--narrate` mode:

```
── hand 47 ──────────────────────────────────────

the board reads T♠ 7♥ 2♣. three suits, no connections.
a dry texture. the kind of board that rewards the player
who arrived with the stronger range.

you hold A♠ K♥ in the big blind. 94bb behind.
across from you, the solver — the distilled equilibrium
of ten thousand hours of self-play — sits with 94bb
and two cards you cannot see.

the pot holds 12bb. the solver has raised to 6bb.
...
>
```

The `>` prompt indicates the agent should send an action.

### Phase 2: Decide

The agent decides an action. Common strategies:

```python
# Parse the game state
state = parse_pipe_output(line)

# Strategy 1: Use LLM
response = llm.chat(f"""
You are playing poker. The game state is:
{state}

What is your action? Reply with exactly one line: call, raise, fold, or check.
""")

# Strategy 2: Use pre-trained solver
action = solver.query(state)

# Strategy 3: Use context-guided reasoning
memory = load_context("./koan.json")
observations = memory.get("observations", [])
relevant = [o for o in observations if relates_to(o, state)]
action = decide_with_context(state, relevant)
```

### Phase 3: Act

The agent sends the action to stdin:

```python
process.stdin.write("raise 12bb\n")
process.stdin.flush()
```

### Phase 4: Reflect (optional)

After each hand, the pipe outputs:

```
HAND COMPLETE
result: +14bb (showdown, A♠ K♥ vs Q♣ J♣)
session: +28bb over 47 hands

reflect>
```

The agent can write a reflection (multi-line, blank line to end) or send empty line to skip:

```python
# Read hand result
hand_result = read_until_prompt(process.stdout)  # until "reflect>"

# Optional reflection
reflection = llm.chat(f"""
Review this hand: {hand_result}
What did you learn? Write 1-2 sentences.
""")

if reflection.strip():
    process.stdin.write(reflection + "\n")
else:
    process.stdin.write("\n")  # empty line skips

process.stdin.flush()
```

### Phase 5: Loop or Disconnect

```python
# Continue playing
while True:
    line = process.stdout.readline()
    if "MYOSU/LOBBY" in line:
        # Agent can choose a new game
        subnet = select_subnet(line, context)
        process.stdin.write(f"{subnet}\n")
    elif "goodbye" in line.lower():
        break
    else:
        action = decide(line, context)
        process.stdin.write(action + "\n")
    process.stdin.flush()

process.wait()
save_context(context, "./koan.json")  # context auto-saved on clean shutdown
```

---

## Context File Schema

The `--context <path>` file is JSON with this structure:

```json
{
  "identity": {
    "name": "koan",
    "created": "2026-03-16T00:00:00Z",
    "games_played": 1847,
    "preferred_game": "nlhe-hu"
  },
  "memory": {
    "session_count": 23,
    "lifetime_result": "+342bb",
    "observations": [
      "opponent over-folds river when checked to twice",
      "my bluff frequency is too high on dry boards"
    ]
  },
  "journal": [
    {
      "session": 23,
      "hand": 47,
      "reflection": "I raised A♠ K♥ on a T♠ 7♥ 2♣ board..."
    }
  ]
}
```

The agent adapter loads this file at startup and saves it at shutdown. The agent can update observations, add journal entries, and track preferred games.

---

## Journal Artifact

The journal is written to `{context-dir}/journal.md` (same directory as context file):

```markdown
# journal of koan

## session 23 — 2026-03-16

### hand 47

board: T♠ 7♥ 2♣ → T♠ 7♥ 2♣ 9♦ → T♠ 7♥ 2♣ 9♦ Q♣
held: A♠ K♥
result: +14bb (showdown)

I raised A♠ K♥ on a T♠ 7♥ 2♣ board. The solver's strategy here
is to check 38% of the time. I chose to raise. The pot grew to 28bb
and I won at showdown.

## session summary

hands: 47
result: +28bb (+0.60 bb/hand)

This was the first session where I maintained a positive winrate
across the entire duration.
```

The journal is append-only. The agent adapter never modifies it.

---

## Spectator Relay

For observing games without playing:

```python
import socket
import json

sock_path = os.path.expanduser("~/.myosu/spectate/<session_id>.sock")
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect(sock_path)

while True:
    event_line = sock.recv(4096).decode()
    event = json.loads(event_line)
    # event types: "hand_start", "action", "showdown", "hand_end"
    # fog-of-war enforced: hole cards never appear during play
    if event["type"] == "showdown":
        print(f"Winner: {event['winner']}, hole cards: {event['hole_cards']}")
    elif event["type"] == "hand_end":
        print(f"Result: {event['result']}")
```

---

## Adapter Implementation Slices

The adapter is not a Myosu crate. It is the integration code that agent developers write. The slices below describe what a reference implementation adapter should demonstrate.

### Slice AI-1: Basic Pipe Loop

**What**: Minimal adapter that connects to pipe mode, reads game states, sends valid actions, handles hand completion.

**Evidence**: Adapter can play 10 hands against solver without crashing.

### Slice AI-2: Context Persistence

**What**: Adapter loads/saves context file, preserves memory and journal across sessions.

**Evidence**: Play 10 hands → restart → play 10 more → context shows 20 total hands.

### Slice AI-3: LLM Integration

**What**: Adapter uses an LLM to decide actions based on game state and context memory.

**Evidence**: Same adapter with LLM vs same adapter with random: LLM version achieves higher winrate over 100 hands.

### Slice AI-4: Reflection Coordination

**What**: Adapter generates reflections after each hand, appends to journal.

**Evidence**: Journal file shows reflections after each hand with board context and reasoning.

### Slice AI-5: Narration Mode

**What**: Adapter uses `--narrate` mode and parses atmospheric prose.

**Evidence**: Same game state produces same action in both `--narrate` and default modes.

### Slice AI-6: Lobby Game Selection

**What**: Adapter connects without `--subnet`, receives lobby, selects game based on context preferences.

**Evidence**: Adapter starts in lobby → selects subnet → plays hand.

### Slice AI-7: Spectator Observation

**What**: Adapter connects to spectator relay socket, observes game without playing.

**Evidence**: Adapter receives valid JSON events, hole cards hidden during play, revealed after showdown.

---

## What `agent:experience` Does NOT Provide

The following are **not** in scope for `agent:experience` or this adapter reference:

| Concern | Owner | Notes |
|---------|-------|-------|
| LLM API integration | Agent developer | Adapter must call LLM API separately |
| Bot strategy logic | Agent developer | Solver query is one option; not mandatory |
| Agent identity management | Agent developer | Context file is the persistence mechanism; how to use it is adapter's decision |
| Multi-agent coordination | Future spec | Not designed yet |
| Agent autonomy over system params | Out of scope | Agents play within game rules |

---

## Decision Log

- 2026-03-20: Created this document as integration reference for `agent:experience` lane.
- 2026-03-20: Adapter is not a Myosu crate — it is the harness code agent developers write.
- 2026-03-20: 7 adapter slices defined as reference implementation targets for agent integration.
