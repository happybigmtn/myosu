# Myosu Design System

Genesis document. Defines the visual language for all user-facing surfaces.

## Design Philosophy

Myosu's interface communicates through structure and restraint, not decoration. The TUI is a thinking tool — it should feel like a well-tuned instrument, not a game. Visual hierarchy comes from typography weight and spatial rhythm, not color. Color is reserved for emotional moments: convergence (win), divergence (loss), instability (warning).

Guiding principles:
1. **Structure carries meaning.** The five-panel layout (header, transcript, state, declaration, input) is fixed. Games only fill the state panel.
2. **Silence is default.** Most of the screen is monochrome gray. Accents appear sparingly.
3. **Readability without color.** The interface must be fully usable in monochrome terminals. Color enhances but never encodes.
4. **Density over chrome.** No decorative borders, shadows, or rounded corners. Every pixel conveys information.

## Color Palette

8-token semantic palette. Defined in `crates/myosu-tui/src/theme.rs`.

```
TOKEN         RGB             ROLE
─────────────────────────────────────────────────────────
fg            192, 192, 192   Primary text. Most content.
fg_bright     255, 255, 255   Emphasis. Active elements.
fg_dim         96,  96,  96   Receding. Folds, shadows.
converge        0, 204, 102   Positive. Wins, convergence.
diverge       204,  51,  51   Negative. Losses, divergence.
unstable      204, 170,   0   Warning. Instability, caution.
focus          68, 136, 204   Selection. Active cursor, focus.
protocol      136,  68, 204   System. Chain events, protocol.
```

Background: terminal default (assumed dark). No background color tokens — the terminal theme controls this.

### Usage Rules

- **During normal gameplay**: only `fg`, `fg_bright`, and `fg_dim` are visible.
- **Accent colors** appear for emotional moments only: win/loss results, convergence metrics, protocol events.
- **Never combine** two accent colors in the same line.
- **All 8 tokens must be visually distinct** from each other (enforced by test `readable_without_color` in theme.rs).

## Typography

Terminal-native. No font control — the user's terminal font applies everywhere.

```
ELEMENT          STYLE                  EXAMPLE
─────────────────────────────────────────────────────────
Header           fg_bright, BOLD        NLHE-HU  HAND 47
Declaration      fg_bright, centered    THE SYSTEM AWAITS YOUR DECISION
Transcript       fg, normal             Hero raises to 6bb
Fold lines       fg_dim, DIM            folds (recedes visually)
Hand shadow      fg_dim, DIM            Previous hands dim out
Assistant text   fg, ITALIC             Signals "not fact" — coach advice
Input prompt     fg_bright              > _
Game state       fg, normal             Board, pot, positions
```

### Modifiers

- **BOLD**: header and declaration only.
- **DIM**: receding content (old hands, folds, inactive elements).
- **ITALIC**: assistant/coach text — visual signal that this is opinion, not fact.
- **UNDERLINE**: not used. Reserved for future hyperlink support.

## Layout

Five-panel vertical stack. Defined in `crates/myosu-tui/src/shell.rs`.

```
┌─────────────────────────────────────┐
│ HEADER                              │  1 line, fixed
│ game_label  context_label           │
├─────────────────────────────────────┤
│                                     │
│ TRANSCRIPT                          │  flex, fills available space
│ (scrollable game history)           │
│                                     │
├─────────────────────────────────────┤
│ STATE PANEL                         │  variable height (0 when inactive)
│ (game-specific: board, pot, etc.)   │  delegated to GameRenderer
├─────────────────────────────────────┤
│ DECLARATION                         │  1 line, centered, ALLCAPS
├─────────────────────────────────────┤
│ > input_                            │  1 line, fixed
└─────────────────────────────────────┘
```

### Panel Sizing

| Panel       | Height     | Constraint         |
|-------------|------------|--------------------|
| Header      | 1 line     | Fixed              |
| Transcript  | remaining  | Flex (fills space) |
| State       | 0-8 lines  | GameRenderer decides; 0 when no active hand |
| Declaration | 1 line     | Fixed              |
| Input       | 1 line     | Fixed              |

### Minimum Dimensions

- **Width**: 40 columns (below this, render "terminal too small" message)
- **Height**: 12 rows (below this, render "terminal too small" message)

### Responsive Tiers

```
TIER          WIDTH        BEHAVIOR
─────────────────────────────────────────────────────────
Narrow        40-59 cols   Compact card display, abbreviated labels
Compact       60-79 cols   Standard layout, full labels
Desktop       80+ cols     Full layout with padding
```

The layout is always a single vertical stack — no side-by-side panels. Responsive adaptation is within panels (card density, label length), not layout structure.

## Screen Architecture

8 screens managed by `ScreenManager`. Defined in `crates/myosu-tui/src/screens.rs`.

```
ONBOARDING ──> LOBBY ──> GAME ──> STATS
                 │         │
                 │         ├──> COACHING (overlay)
                 │         ├──> HISTORY  (overlay)
                 │         └──> SPECTATE
                 │
                 └──> WALLET
```

- **Overlays** (Coaching, History) return to Game on any key.
- **Wallet** supports `/back` navigation to Lobby.
- **Stats** is a terminal screen (end of session).

### Screen Declarations

Each screen has a default declaration (ALLCAPS centered text):

| Screen      | Declaration                          |
|-------------|--------------------------------------|
| Onboarding  | WELCOME TO MYOSU                     |
| Lobby       | SELECT A GAME                        |
| Game        | THE SYSTEM AWAITS YOUR DECISION      |
| Stats       | SESSION SUMMARY                      |
| Coaching    | ANALYSIS                             |
| History     | HAND HISTORY                         |
| Wallet      | ACCOUNT                              |
| Spectate    | SPECTATOR MODE                       |

## Interaction Model

Command-line input with game-specific parsing. No mouse. No menus.

```
INPUT FLOW
─────────────────────────────────────────────────────────
User types  ──>  GameRenderer.parse_input()
                    │
                    ├── Valid action ──> execute
                    ├── Ambiguous   ──> GameRenderer.clarify() prompt
                    └── Invalid     ──> "unknown command" in transcript
```

### Input Conventions

- **Game actions**: bare words (`fold`, `call`, `raise 6`, `check`)
- **System commands**: slash-prefixed (`/analyze`, `/history`, `/quit`, `/back`)
- **Tab completion**: available via `GameRenderer.completions()`
- **History**: up/down arrows navigate input history

### Keyboard Shortcuts

| Key       | Action                |
|-----------|-----------------------|
| Enter     | Submit input          |
| Tab       | Cycle completions     |
| Up/Down   | Input history         |
| Ctrl-C    | Quit                  |
| Ctrl-L    | Clear transcript      |
| ?         | Show help overlay     |

## Pipe Mode

When `--pipe` is passed, the TUI is disabled. Output is plain text to stdout, input is plain text from stdin. This enables programmatic interaction (miners, bots, testing).

Pipe mode uses `GameRenderer.pipe_output()` for state representation — same data, no styling.

## Game Extensibility

Adding a new game requires implementing `GameRenderer` (defined in `crates/myosu-tui/src/renderer.rs`). The shell handles all cross-cutting concerns. A game implementation provides:

1. `render_state()` — draw the state panel
2. `desired_height()` — how tall the state panel should be
3. `declaration()` — current declaration text
4. `completions()` — tab completion candidates
5. `parse_input()` — interpret user commands
6. `clarify()` — disambiguate ambiguous input
7. `pipe_output()` — plain text state for pipe mode
8. `game_label()` — header identifier (e.g., "NLHE-HU")
9. `context_label()` — header context (e.g., "HAND 47")

No shell, event loop, or layout code needs to change.

## Aesthetic Direction

**Terminal brutalism.** The interface trusts the terminal's native rendering. No box-drawing characters for decoration. No progress bars. No animations. Information appears immediately and completely.

The emotional arc comes from the game, not the interface. When a player wins a big pot, `converge` green appears in the transcript. When they lose, `diverge` red. The rest of the time, the screen is calm gray text — a deliberate absence of stimulation that keeps focus on the decision.

This is not minimalism as aesthetic choice. It is minimalism as functional requirement: the TUI must work over SSH, in tmux, on 256-color terminals, and in CI pipe mode.
