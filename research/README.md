# Myosu Research

Automated research infrastructure for game-solving algorithm analysis,
incentive mechanism design, and game rule documentation.

## Tools

- **AutoResearchClaw** — 23-stage autonomous research pipeline
- **Agent reports** — parallel research agents for targeted analysis
- **Game rules** — idiomatic rule documents for all 20 target games

## Directory Structure

```
research/
  game-solving/           AutoResearchClaw output (23-stage pipeline)
    stage-01/               TOPIC_INIT — goal.md, hardware_profile.json
    stage-02/               PROBLEM_DECOMPOSE — problem_tree.md
    stage-03/               SEARCH_STRATEGY — search_plan.yaml, sources.json
    stage-04/               LITERATURE_COLLECT — candidates.jsonl, references.bib
    stage-05/               LITERATURE_SCREEN — shortlist.jsonl
    stage-06/               KNOWLEDGE_EXTRACT — cards/
    ...through stage-23...
    deliverables/           Final packaged outputs
    agent-reports/          Parallel agent research (faster, less structured)
  game-rules/             Idiomatic rules for all 20 games
    01-nlhe-hu.md
    02-nlhe-6max.md
    ...through 20-backgammon.md
```

## AutoResearchClaw Setup

### Prerequisites

```bash
# Already installed at /home/r/coding/AutoResearchClaw
cd /home/r/coding/AutoResearchClaw
source .venv/bin/activate

# Verify installation
researchclaw doctor --config config.myosu.yaml
```

### Dependencies

- Python 3.12+ (installed via uv in `.venv/`)
- `acpx` (npm, installed globally) — ACP bridge to Codex CLI
- Codex CLI (`codex`) — the LLM backend (authenticated via `~/.codex/auth.json`)
- No separate API keys needed — Codex handles its own auth

### Configuration

The config lives at `/home/r/coding/AutoResearchClaw/config.myosu.yaml`.

Key settings:

```yaml
llm:
  provider: "acp"           # Uses Agent Client Protocol
  primary_model: "codex"    # Codex CLI as the LLM agent
  acp:
    agent: "codex"                    # Which ACP agent binary
    cwd: "/home/r/coding/myosu"      # Working directory for the agent
    session_name: "researchclaw"      # Persistent session name
    timeout_sec: 900                  # Per-prompt timeout (15 min)
```

To switch to OpenAI/OpenRouter instead of ACP:

```yaml
llm:
  provider: "openrouter"
  api_key_env: "OPENROUTER_API_KEY"
  primary_model: "anthropic/claude-sonnet-4"
```

## Running Research

### New research session

```bash
cd /home/r/coding/AutoResearchClaw
source .venv/bin/activate

researchclaw run \
  --config config.myosu.yaml \
  --output /home/r/coding/myosu/research/<topic-name> \
  --auto-approve \
  --skip-noncritical-stage
```

### Override the topic

```bash
researchclaw run \
  --config config.myosu.yaml \
  --output /home/r/coding/myosu/research/cfr-convergence \
  --topic "Survey of CFR convergence rates across imperfect-information game classes" \
  --auto-approve \
  --skip-noncritical-stage
```

### Resume after failure

```bash
# Resume from last checkpoint
researchclaw run \
  --config config.myosu.yaml \
  --output /home/r/coding/myosu/research/game-solving \
  --resume

# Resume from a specific stage
researchclaw run \
  --config config.myosu.yaml \
  --output /home/r/coding/myosu/research/game-solving \
  --from-stage LITERATURE_SCREEN \
  --auto-approve \
  --skip-noncritical-stage
```

### Generate a report from completed run

```bash
researchclaw report --output /home/r/coding/myosu/research/game-solving
```

## Pipeline Stages

```
PHASE A: Research Scoping
  1. TOPIC_INIT           Define SMART goal, novel angle, constraints
  2. PROBLEM_DECOMPOSE    Break topic into sub-problems

PHASE B: Literature Discovery
  3. SEARCH_STRATEGY      Design search queries for Semantic Scholar
  4. LITERATURE_COLLECT   Fetch candidate papers (candidates.jsonl)
  5. LITERATURE_SCREEN    Filter to shortlist (GATE — human approval if not --auto-approve)
  6. KNOWLEDGE_EXTRACT    Extract key findings into knowledge cards

PHASE C: Knowledge Synthesis
  7. SYNTHESIS            Merge knowledge cards into coherent narrative
  8. HYPOTHESIS_GEN       Generate testable hypotheses

PHASE D: Experiment Design
  9. EXPERIMENT_DESIGN    Design experiments to test hypotheses (GATE)
  10. CODE_GENERATION     Generate experiment code
  11. RESOURCE_PLANNING   Estimate compute/time requirements

PHASE E: Experiment Execution
  12. EXPERIMENT_RUN      Execute experiments in sandbox
  13. ITERATIVE_REFINE    Iterate on failed/weak experiments

PHASE F: Analysis & Decision
  14. RESULT_ANALYSIS     Analyze experiment results
  15. RESEARCH_DECISION   Decide what to include in paper

PHASE G: Paper Writing
  16. PAPER_OUTLINE       Structure the paper
  17. PAPER_DRAFT         Write full draft
  18. PEER_REVIEW         Self-review for quality
  19. PAPER_REVISION      Address review feedback

PHASE H: Finalization
  20. QUALITY_GATE        Final quality check (GATE)
  21. KNOWLEDGE_ARCHIVE   Archive findings to knowledge base
  22. EXPORT_PUBLISH      Package deliverables
  23. CITATION_VERIFY     Verify all citations are valid
```

### Stage names for --from-stage

```
TOPIC_INIT, PROBLEM_DECOMPOSE, SEARCH_STRATEGY,
LITERATURE_COLLECT, LITERATURE_SCREEN, KNOWLEDGE_EXTRACT,
SYNTHESIS, HYPOTHESIS_GEN,
EXPERIMENT_DESIGN, CODE_GENERATION, RESOURCE_PLANNING,
EXPERIMENT_RUN, ITERATIVE_REFINE,
RESULT_ANALYSIS, RESEARCH_DECISION,
PAPER_OUTLINE, PAPER_DRAFT, PEER_REVIEW, PAPER_REVISION,
QUALITY_GATE, KNOWLEDGE_ARCHIVE, EXPORT_PUBLISH, CITATION_VERIFY
```

## Monitoring a Running Pipeline

### Check heartbeat

```bash
cat /home/r/coding/myosu/research/game-solving/heartbeat.json
```

Shows current stage, PID, and timestamp.

### Check pipeline summary

```bash
cat /home/r/coding/myosu/research/game-solving/pipeline_summary.json
```

Shows stages executed, done, failed counts.

### Check stage outputs

```bash
ls /home/r/coding/myosu/research/game-solving/stage-*/
```

Each stage directory contains its outputs + `decision.json` + `stage_health.json`.

## Patches Applied

The ACP client in AutoResearchClaw required patches for our environment:

1. **File-based prompts** — Large prompts (e.g., 379 paper candidates) exceed
   shell argument limits. Patched `_send_prompt` to write prompts to temp files
   and pass via `acpx -f <file>`.

2. **Session TTL** — Sessions expire after 300s idle. Patched to use `--ttl 0`
   (keep alive forever) for both session creation and prompt sending.

3. **Session reconnect** — Patched `_ensure_session` to detect `[closed]`
   sessions and recreate them, instead of assuming any named session is valid.

4. **CWD handling** — `acpx` session commands don't accept `--cwd` flag.
   Patched to pass cwd as the subprocess working directory.

5. **Doctor ACP mode** — Doctor crashed on ACP mode (no base_url for HTTP
   checks). Patched `health.py` to skip HTTP checks when provider is "acp".

All patches are in `/home/r/coding/AutoResearchClaw/researchclaw/llm/acp_client.py`
and `/home/r/coding/AutoResearchClaw/researchclaw/health.py`.

## Research Topics Queue

Completed or in-progress research:

| Topic | Status | Output |
|-------|--------|--------|
| 20-game algorithm survey + incentive design | In progress (ARC) | `research/game-solving/` |
| Poker game algorithms (5 games) | Complete (agent) | `research/game-solving/agent-reports/01-poker-games.md` |
| Strategy game algorithms (9 games) | Complete (agent) | `research/game-solving/agent-reports/03-strategy-games.md` |
| Game rules (20 games) | In progress (agents) | `research/game-rules/` |

Future research topics to queue:

- CFR convergence rates and abstraction quality tradeoffs
- Multiplayer scoring metrics (alpha-rank vs CCE distance vs round-robin)
- GPU-accelerated CFR training feasibility
- Abstraction pipeline design (EMD clustering vs k-means vs learned)
- Token economics modeling (emission curves, miner incentives, stake dynamics)
