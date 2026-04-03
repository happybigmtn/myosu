# Operator Architecture Overview

## Goal

Give operators a plain-language mental model of the current Myosu stage-0
system before they start running commands from the quickstart.

## The Short Version

Myosu is a network for training and judging game-playing strategy.

- the chain is the shared scoreboard and settlement layer
- miners do the solving work
- validators check the quality of that work
- gameplay is where humans or agents consume the result
- keys let an operator prove "this miner" or "this validator" belongs to them

If you only remember one thing, remember this loop:

1. a miner trains a strategy and serves it
2. a validator scores that strategy
3. the chain records scores and distributes emissions
4. gameplay surfaces consume the best available strategy

## The System At A Glance

```text
                      +----------------------+
                      |      myosu-keys      |
                      | operator identity    |
                      | and signing material |
                      +----------+-----------+
                                 |
                                 v
+-------------------+    register / stake / weights    +-------------------+
|    myosu-miner    | -------------------------------> |    myosu-chain    |
| solver + strategy |                                  | shared network    |
| serving endpoint  | <------------------------------- | state + emissions |
+---------+---------+      subnet state / incentives   +---------+---------+
          ^                                                        ^
          |                                                        |
          | strategy query                                         | validator results
          |                                                        |
+---------+---------+                                  +-----------+--------+
|    myosu-play     | <------------------------------- |  myosu-validator   |
| human/agent       |    advice can come from live     | quality scoring +  |
| gameplay surface  |    miner or local artifact       | on-chain reporting |
+-------------------+                                  +--------------------+
```

## What Each Piece Does

### `myosu-chain`

Think of the chain as the neutral control plane.

It keeps track of:

- which subnets exist
- which miner and validator hotkeys are registered
- where a miner says its strategy endpoint lives
- what weights validators submitted
- how emissions should be distributed for an epoch

The chain does not train poker strategy itself. It stores shared truth about
who participated and how they were scored.

### `myosu-miner`

The miner is the solver worker.

It:

- trains or loads a strategy profile
- publishes an axon/HTTP endpoint so others can query that strategy
- answers strategy queries for a specific game state
- writes local artifacts such as checkpoints and encoder data

In operator terms, the miner is the machine that tries to be useful enough to
earn emissions.

### `myosu-validator`

The validator is the quality checker.

It:

- queries a miner's strategy output
- compares that output against the expected response for the same game state
- turns that comparison into a score
- submits weights back to the chain so consensus can reward stronger solvers

In operator terms, validators are referees, not trainers.

### `myosu-play`

`myosu-play` is the consumer-facing game surface.

It is how a human or agent interacts with the trained strategy through the same
text interface. It matters to operators because it is the product-facing proof
that the network's output is actually usable, but it is not required for the
basic miner/validator bring-up path.

### `myosu-keys`

`myosu-keys` is the operator identity helper.

It:

- creates and stores encrypted operator keys
- marks one key as active
- prints the bootstrap commands for miner and validator
- lets the other binaries sign chain actions without pasting a secret into
  each command

This is a keystore helper, not a finished wallet.

## How Data Moves Through The System

### 1. Identity and configuration

An operator creates or imports a key with `myosu-keys`. That key is then used
by the miner and validator when they register on-chain or submit signed
transactions.

### 2. Miner training

The miner trains a bounded strategy profile for a game such as heads-up no-limit
hold'em. The useful local outputs are checkpoint files and encoder artifacts.

### 3. Miner serving

The miner publishes where it can be reached and starts answering strategy
queries. This is what lets validators and gameplay ask, "what action would you
take from this position?"

### 4. Validator scoring

The validator sends the same query shape to a miner, inspects the answer, and
computes a quality score. In stage-0, this is still a bounded bootstrap/scoring
flow rather than a polished forever-running service.

### 5. Chain accounting

The validator submits weights to the chain. The chain aggregates those weights,
runs the incentive logic, and updates on-chain emission state.

### 6. Gameplay consumption

`myosu-play` can use a live miner or local artifacts to show strategy advice to
a human or agent. This is the easiest way to see the system's output as an
actual playable experience instead of just chain state.

## What Lives On-Chain Vs Off-Chain

| Where it lives | Examples | Why operators care |
|---|---|---|
| On chain | subnet ids, registration state, axon endpoints, stake, submitted weights, emission state | this is the shared truth every operator must agree on |
| On miner disk | checkpoints, encoder artifacts, query/response files during proofs | losing these means retraining or rebuilding local serving state |
| On validator disk | local scoring inputs and temporary proof artifacts | useful for local validation but not the network source of truth |
| In `~/.myosu/` | encrypted operator keyfiles and active-account config | losing this means losing the ability to operate the existing identity |

## How To Think About Failures

- If the miner is down, strategy serving stops even if the chain is healthy.
- If the validator is wrong or offline, scoring quality drops and weight
  submission can stall.
- If the chain is unavailable, miners and validators cannot register, stake, or
  submit results even if their local processes are fine.
- If the key material is unavailable, the binaries may still start, but they
  cannot perform signed on-chain actions.

That separation matters because many operator issues are not "the network is
broken"; they are one layer failing while the others are healthy.

## Stage-0 Reality Check

The honest current operator story is:

- the chain, miner, validator, and gameplay surfaces all exist and work for the
  stage-0 proof path
- the validator is still a bounded scoring/bootstrap tool, not a finished
  autonomous service
- `myosu-play` is a useful consumer and smoke surface, not a required part of
  running a miner or validator
- `myosu-keys` is an operator keystore helper, not a full wallet product

## Where To Use This Next

- Read the [quickstart](./quickstart.md) when you want the exact zero-to-running
  commands.
- Read the [operator network playbook](../execution-playbooks/operator-network.md)
  when you need bundle details, bootnode packaging, or extended key operations.
