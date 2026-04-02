# Specification: Validator Process

Source: Reverse-engineered from crates/myosu-validator (main.rs, cli.rs, lib.rs)
Status: Draft
Depends-on: 001-game-trait-framework, 006-chain-rpc-client

## Purpose

The validator process scores miner-produced strategies for exploitability and
submits quality weights to the chain. It is the quality verification component
of the Myosu system: it deterministically evaluates a miner's strategy response
against an expected result, computes a similarity score, and submits that score
as a weight vector that the chain's Yuma Consensus uses to distribute emissions.
Deterministic scoring is a system invariant (INV-003): two validators must
produce identical scores for the same miner.

The primary consumer is a validator operator running the binary against a live
chain node.

## Whole-System Goal

Current state: The validator binary is fully implemented with a sequential
startup pipeline covering key resolution, chain probing, optional registration,
optional subnet parameter configuration, optional staking, deterministic
scoring, and optional weight submission.

This spec adds: Nothing new. This documents the existing behavioral contract.

If all ACs land: An operator can start a validator that registers on-chain,
configures subnet parameters, stakes for a validator permit, scores a miner's
strategy deterministically, and submits quality weights to the chain.

Still not solved here: Miner training, gameplay consumption, emission
distribution mechanics, and cross-validator determinism proof are separate
concerns.

## Scope

In scope:
- Sequential startup pipeline with 12 ordered phases
- Key resolution (shared pattern with miner)
- Chain connectivity probing (shared pattern with miner)
- Optional on-chain registration
- Optional subnet bootstrap: subtoken enablement, tempo override, weights rate
  limit override, commit-reveal toggle
- Optional stake bootstrap with validator permit waiting
- Deterministic strategy scoring with L1 distance and exact match detection
- Optional weight submission (direct or commit-reveal, auto-selected)
- Structured reporting at each phase (10 report formats)
- Game selection (Poker or LiarsDice)

Out of scope:
- Strategy training or MCCFR computation
- Gameplay rendering or user interaction
- Emission calculation or distribution
- Multi-validator coordination or consensus
- Key generation or management

## Current State

The binary exists at crates/myosu-validator with approximately 1,600 lines of
code. It depends on myosu-chain-client for chain interaction, game crates for
strategy decoding, and tracing for structured logging.

The startup pipeline executes sequentially: (1) initialize tracing, (2) parse
CLI, (3) resolve key, (4) probe chain, (5) optionally register, (6) optionally
enable subtoken for the subnet, (7) optionally override subnet tempo via sudo,
(8) optionally override weights set rate limit via sudo, (9) optionally disable
commit-reveal via sudo, (10) optionally bootstrap stake and wait for validator
permit, (11) score miner response deterministically, (12) optionally submit
weights.

Unlike the miner, the validator always exits after completing its configured
phases. It does not run a persistent server.

Scoring loads a wire-encoded miner response and compares it against the expected
output derived from a checkpoint. The validation report includes game type,
action count, exact match flag, L1 distance between probability distributions,
a score between 0.0 and 1.0, and the expected vs. observed recommended actions.

Weight submission supports an optional target hotkey (--weight_hotkey) for
weighting a specific miner, defaulting to the validator's own key. The client
automatically selects direct set_weights or commit-reveal based on the chain's
CommitRevealWeightsEnabled configuration.

Subnet bootstrap parameters (tempo, rate limit, commit-reveal) use sudo
transactions, implying the validator key has sudo authority. Each bootstrap
operation includes idempotency checking (already-set detection) and prints a
structured report.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| CLI parsing | Clap-based with mutual exclusion groups | Reuse | Shared pattern with miner |
| Chain probing | probe_chain with startup_report | Reuse | Shared pattern with miner |
| Registration | ensure_registered via chain client | Reuse | Shared flow with miner |
| Subnet bootstrap | 4 sudo parameter overrides with idempotency | Reuse | Stage-0 configuration |
| Stake bootstrap | ensure_validator_permit_ready | Reuse | Permit acquisition |
| Scoring | Deterministic strategy comparison (L1 distance) | Reuse | INV-003 compliance |
| Weight submission | Direct or commit-reveal auto-selection | Reuse | Chain-aware submission |

## Non-goals

- Continuously polling miners for strategy updates.
- Running as a persistent daemon or server.
- Computing exploitability independently (scoring compares against expected
  output).
- Managing multi-validator consensus or agreement protocols.
- Training strategies or producing checkpoints.

## Behaviors

The validator executes phases sequentially. Each phase is gated by CLI flags:
phases with unset flags are skipped entirely.

Key resolution and chain probing follow the same pattern as the miner. Probe
failure terminates the process.

Registration (when --register is set) submits a burned_register transaction and
waits for UID assignment.

Subnet bootstrap phases (when corresponding flags are set) configure the subnet
for stage-0 operation: enable subtoken staking, set a custom tempo, set a
custom weights rate limit, and optionally disable commit-reveal. Each operation
checks current state first and reports whether the value was already set. These
operations require sudo authority.

Stake bootstrap (when --stake_amount is provided) calls
ensure_validator_permit_ready, which adds stake if the current stake is below
the requested amount, then waits for a validator permit to appear for the
validator's UID on the subnet. The permit report includes requested, final, and
added stake amounts.

Scoring (when --query_file and --response_file are provided) loads the miner's
wire-encoded response, derives the expected response from a checkpoint, and
computes: (a) whether the responses match exactly, (b) the L1 distance between
the probability distributions, (c) a normalized score between 0.0 (worst) and
1.0 (identical), (d) the expected and observed recommended actions.

Weight submission (when --submit_weights is set) resolves the target hotkey
(explicit or self), determines the submission mode (direct or commit-reveal
based on chain state), and submits the weight vector. Timing information
(elapsed milliseconds) is logged.

All failures terminate the process with a descriptive error. There is no retry
logic. The process always exits after completing its pipeline.

## Acceptance Criteria

- The validator starts and probes the chain successfully, printing a startup
  report.
- Registration succeeds and the validator's UID appears on the specified subnet.
- Subnet bootstrap operations are idempotent: re-running with the same
  parameters reports already-set without error.
- Stake bootstrap adds the correct amount and waits for validator permit.
- Scoring produces identical results for identical inputs across multiple runs
  (determinism).
- Scoring correctly computes L1 distance and exact match for known
  query/response pairs.
- Weight submission automatically selects direct or commit-reveal mode.
- Weight submission targets the specified hotkey when --weight_hotkey is
  provided.
- Each phase prints a structured report to stdout.
- The process exits after completing all configured phases.
