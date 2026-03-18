Myosu game-solving autoresearch workspace

Goal:
- Improve `candidate_config.json` to maximize architecture ranking accuracy
  on the CIFAR-100 proxy benchmark across the 20-game survey.
- The config controls solver method selection, hyperparameters, and abstraction
  strategies. The evaluation harness is fixed — you optimize the config only.

The 20-game survey:
- Myosu studies 20 imperfect-information games (NLHE HU through Backgammon).
- Each game exercises a solver architecture (method family + hyperparameters)
  on a CIFAR-100 proxy that measures how well the architecture ranks methods.
- The primary_metric is architecture ranking accuracy: does the config produce
  a method ordering that matches ground truth on the proxy benchmark?

Mutation rules:
- Edit only `candidate_config.json`.
- Make one small, reversible change per iteration.
- Do not edit the evaluation harness, corpus files, or scoring logic.
- Do not change the method names — the harness expects exact name matches.
- Do not remove methods from the list — only toggle `enabled`, change
  hyperparameters, adjust abstraction strategy, or tune scoring weights.

Valid mutation categories:
1. Hyperparameter tuning — adjust steps, candidates, iterations, budget
2. Seed selection — change seed list (keep 5 seeds minimum)
3. Abstraction strategy — change bucket_count, opponent_model, strategy
4. Method toggling — enable/disable specific methods (keep >= 4 enabled)
5. Algorithm family reweighting — change family assignments if misclassified
6. Calibration thresholds — adjust success_threshold, degenerate_tolerance
7. Time budget allocation — shift budget between methods

Scoring:
- primary_metric from results.json: architecture ranking accuracy on CIFAR-100
- Higher is better. The champion holds the current best total_score.
- Smoke test uses 3 representative games (NLHE HU, Liar's Dice, Backgammon).
- Full eval uses all 20 games.

Known learnings:
- Seeds 42, 123, 456, 789, 1024 are the baseline set from the initial results.
- established_method_1 has the highest mean primary_metric (0.3786) in the
  baseline results. proposed_variant shows high variance across seeds.
- Random Search and DARTS are consistently the weakest performers.
- graph_iterations and darts_steps have the most direct impact on ranking
  quality — small changes here propagate through all method evaluations.
- bootstrap_samples at 1000 is sufficient; increasing it adds runtime without
  improving ranking accuracy.

Acceptance bar:
- Candidate must produce a higher total_score than champion on smoke corpus.
- If smoke passes, candidate must also beat champion on full 20-game corpus.
- Config must be valid JSON and parseable by the evaluation harness.
