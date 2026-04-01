# Myosu Design System (Genesis)

Date: 2026-03-28
Scope: user-facing surfaces in this repo (`myosu-play`, `myosu-tui`, operator text outputs, and future web control surfaces).

## Aesthetic Direction

Myosu should feel like a serious trading desk for strategy quality, not a playful terminal toy.

Design direction:
- Dense but calm information layout.
- High signal-to-noise text hierarchy.
- Monochrome-first baseline with sparse semantic accents.
- Explainability-first surfaces: always show state, legal actions, and advice provenance.

Rationale:
- Users are making decisions under uncertainty; ornamental UI increases cognitive load.
- The product differentiator is trusted decision support, so clarity and provenance outrank visual novelty.

## Typography

### Font Roles
- Display: `Iosevka Aile` (fallback: `JetBrains Mono`, `Menlo`, `monospace`)
- Body/UI: `IBM Plex Sans` (fallback: `Inter`, `system-ui`, `sans-serif`)
- Code/Terminal: `Iosevka` (fallback: `JetBrains Mono`, `monospace`)

### Modular Scale (base 16)
- `xs`: 12
- `sm`: 14
- `md`: 16
- `lg`: 20
- `xl`: 24
- `2xl`: 32

Usage:
- Header game/context label: `lg` bold.
- Declaration line: `md` bold, uppercase.
- Transcript/body rows: `sm` regular.
- Input prompt and state table rows: `sm` medium.

## Color Palette

### Core
- Background: `#0F1217`
- Surface: `#171C24`
- Surface-alt: `#1F2632`
- Text-primary: `#E8ECF2`
- Text-secondary: `#B8C1CF`
- Text-muted: `#7D8898`

### Brand/Semantic
- Primary (focus/info): `#4C8BF5`
- Secondary (protocol): `#7A5AF8`
- Success: `#23A36D`
- Warning: `#D4A72C`
- Error: `#D94F4F`
- Neutral-border: `#2E3746`

### Terminal Token Mapping
- `fg` -> text-secondary
- `fg_bright` -> text-primary
- `fg_dim` -> text-muted
- `converge` -> success
- `diverge` -> error
- `unstable` -> warning
- `focus` -> primary
- `protocol` -> secondary

## Spacing Scale

Base unit: `4px`

Scale:
- `1x`: 4
- `2x`: 8
- `3x`: 12
- `4x`: 16
- `6x`: 24
- `8x`: 32
- `10x`: 40

Terminal mapping guidance:
- Horizontal panel padding: 2 characters (minimum).
- Vertical separation rhythm: 1 line for minor boundaries, 2 lines for phase transitions.
- Keep declaration row isolated from transcript by at least one border/separator.

## Layout by Breakpoint

### Small terminal / mobile narrow (`<80 cols` or `<640px`)
- Collapse non-essential panels.
- Priority order:
  1. Declaration
  2. State snapshot
  3. Legal actions
  4. Input
  5. Transcript (truncated tail)
- Avoid side-by-side columns; use stacked sections.

### Medium (`80-119 cols` or `640-1023px`)
- Current five-panel vertical stack remains default.
- State section includes compact key-value rows.
- Transcript keeps last N lines that fit without wrapping key action lines.

### Large (`>=120 cols` or `>=1024px`)
- Preserve five-panel stack but allow denser state grid and richer advisor block.
- Show additional metadata (advice source, checkpoint hash, encoder hash) in header or state footer.

## Interaction State Requirements

All user-facing actions must define:
- Loading: explicit “computing advice…” line, never silent stall.
- Empty: first-run instructions with 1-2 executable commands.
- Error: human-readable error + recovery command.
- Success: confirm with concrete state delta.
- Partial: identify stale vs fresh components (for example, board updated, advisor pending).

## Accessibility Requirements

- Keyboard-only complete operation for every command path.
- No color-only semantics; pair color with text token (`ERROR`, `WARNING`, `OK`).
- Maintain readable contrast target equivalent to WCAG AA.
- Input and actionable rows must be at least one full text row high; no compressed hit targets.
- Pipe/agent mode output must remain plain text and deterministic.

