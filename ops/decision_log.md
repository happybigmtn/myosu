# Myosu Decision Log

## 2026-03-16: Fork Bittensor rather than deploy as native subnet

Decision: Hard fork subtensor to create a dedicated game-solving chain.
Alternatives considered:
- Deploy as Bittensor subnet (Option A): ~$1-2M TAO lock, no chain control
- Hybrid with codexpoker L1 (Option C): clean separation but more integration work
Rationale: Need control over chain parameters, game-specific tokenomics, and
subnet lifecycle. Forking subtensor gives us Substrate + proven incentive math
at fraction of the cost of building from scratch.

## 2026-03-16: Robopoker v1.0.0 as core solver engine

Decision: Depend on robopoker via git tag, not vendor/fork.
Rationale: v1.0.0 is the first stable release with production MCCFR, clustering,
and blueprint infrastructure. Maintaining upstream fidelity (INV-006) keeps us
on the improvement track and avoids maintenance burden.

## 2026-03-16: Malinka as autonomous development framework

Decision: Structure the repo for malinka's task-first development loop.
Rationale: The project has clear specs, bounded stages, and proof gates. Malinka's
structured RESULT/BLOCKED closure and plan-based tracking fits the multi-crate
Substrate build.

## 2026-03-16: Project name "myosu" (묘수)

Decision: Korean word meaning "brilliant move" or "masterstroke."
Rationale: Reflects game-solving focus, Korean gaming culture (baduk, StarCraft),
and is clean/memorable as a romanized word.
