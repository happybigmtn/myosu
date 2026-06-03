# Security Policy

Myosu is still a stage-0 project. Security reports are welcome, but the only
supported remediation targets are the `trunk` branch and the most recent git
tag. Older tags, archived plans, and historical bootstrap artifacts may not
receive fixes.

## Reporting a Vulnerability

Please do not open a public GitHub issue, pull request, or discussion for a
suspected vulnerability.

Preferred private channel:

- Use GitHub's private vulnerability reporting flow for
  `happybigmtn/myosu` if it is available in the repository UI.

Fallback private channel:

- Contact repository owner `@happybigmtn` privately on GitHub and request a
  security disclosure thread before sharing details publicly.

Include as much of the following as you can:

- affected component, crate, command, or workflow
- impact and realistic attacker model
- exact commit, tag, or branch tested
- reproduction steps or proof-of-concept
- any suggested mitigation or patch direction

## Response Expectations

- Acknowledgement target: within 72 hours
- Initial triage target: within 7 calendar days
- Status updates: at least weekly while the report is active
- Disclosure target: coordinated disclosure after a fix ships or a mitigation
  is documented

If a report cannot be resolved quickly because it depends on inherited
Substrate, Bittensor, or robopoker fork behavior, maintainers will document the
risk, the upstream dependency, and the temporary operator guidance before
closing the report.

## Scope

In scope:

- active crates and binaries used by the stage-0 local loop
- chain runtime, node, and `pallet-game-solver`
- miner, validator, gameplay, key-management, and operator-bundle flows
- artifact, checkpoint, wire-format, and blueprint loading boundaries
- CI and release-gate automation that can change shipped artifacts or operator
  instructions

Out of scope:

- public internet infrastructure not operated from this repository
- third-party services and GitHub platform bugs
- denial-of-service from unrealistic resource exhaustion without a plausible
  operator impact path
- historical or archived planning documents unless they create a live exploit
  path in current code

## Safe Harbor

Myosu supports good-faith security research conducted to improve the project.
Maintainers will not pursue action for research that:

- avoids privacy violations, data destruction, and service interruption
- uses test accounts, local devnets, or self-controlled environments whenever
  possible
- limits proof-of-concept activity to the minimum needed to demonstrate impact
- gives maintainers a reasonable opportunity to investigate and fix the issue
  before public disclosure

Do not exfiltrate secrets, modify other users' data, or run destructive load or
consensus attacks against systems you do not own or operate.

## Current Security Context

The current stage-0 audit snapshot lives in
[`ops/security-audit-stage0.md`](ops/security-audit-stage0.md). It is a release
gate input, not a substitute for reporting a vulnerability privately.

The upstream dependency review and cve-tracking process lives in
[`ops/cve-tracking-process.md`](ops/cve-tracking-process.md). Keep it aligned
with the current `cargo audit` ignore list and any upstream pin changes.

This repository does not currently run a bug bounty program.
