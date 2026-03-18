# Stage 1: Literature Survey

## Objective

Catalog the existing solver approaches, equilibrium concepts, and computational
techniques applicable to 20 imperfect-information games. Establish the
theoretical foundation that all subsequent stages build upon.

## Research Questions

### Primary

1. For each of the 20 games in the myosu survey, what solver families have been
   applied in published work (CFR variants, policy gradient, MCTS with
   information sets, etc.)?

2. What equilibrium concepts are targeted per game -- Nash equilibrium,
   correlated equilibrium, quantal response equilibrium, or team-maxmin?

3. Which games have existing exploitability benchmarks, and what are the
   best-known bounds?

### Secondary

4. What are the publication venues and research groups most active in each
   game's solver literature?

5. Are there games in the 20-game set with no prior computational work,
   requiring novel formulation?

6. What open-source solver implementations exist, and what is their maturity
   level (research prototype vs. production-grade)?

## Scope

### Games Covered

The full 20-game set as defined in the myosu survey. Games span a range from
well-studied domains (Texas Hold'em variants, Kuhn Poker) to less-explored
titles (partnership card games, tile-laying games with hidden information,
asymmetric role games).

### Literature Sources

- Conference proceedings: NeurIPS, ICML, AAAI, IJCAI, AAMAS
- Journals: Artificial Intelligence, Games and Economic Behavior, JAIR
- Preprint servers: arXiv cs.GT, cs.AI, cs.MA
- Theses and technical reports from CMU, U of Alberta, DeepMind, FAIR

### Time Horizon

Papers published from 2000 through the survey cutoff date. Emphasis on work
from 2015 onward given the rapid progress in deep learning approaches to
imperfect-information games.

## Methodology

### Phase 1: Systematic Search

For each game, execute structured queries across Google Scholar, Semantic
Scholar, and DBLP. Record:

- Paper title, authors, year, venue
- Game(s) addressed
- Algorithm family (CFR, Deep CFR, DREAM, NFSP, PPO-based, etc.)
- Equilibrium concept targeted
- State space / info set size reported
- Exploitability or other performance metric reported
- Whether source code is available

### Phase 2: Taxonomy Construction

Organize findings into a game-by-algorithm matrix. For each cell:

- Record whether the combination has been attempted
- Note the best reported result
- Flag gaps where no work exists

### Phase 3: Gap Analysis

Identify which games lack solver results entirely, which have results only from
a single algorithm family, and which have competitive baselines across multiple
approaches.

## Expected Outcomes

1. A structured bibliography of 150-300 papers organized by game and algorithm
   family.

2. A 20-by-N matrix (games x algorithm families) showing coverage, with cells
   containing best-known exploitability or win-rate results.

3. A gap report identifying under-studied games and algorithm-game combinations
   that represent open research opportunities.

4. A shortlist of open-source implementations suitable for baseline
   reproduction in later stages.

## Success Criteria

- Every game in the 20-game set has at least one literature entry or is
  explicitly flagged as having no prior computational work.
- The taxonomy covers at least 5 distinct algorithm families.
- Gap analysis identifies at least 3 under-explored combinations that motivate
  the architecture search direction in stages 6-10.

## Dependencies

- None (this is the first stage).

## Outputs

- `bibliography.json` -- structured citation data
- `coverage_matrix.csv` -- game x algorithm family coverage
- `gap_report.md` -- narrative analysis of literature gaps

## Notes

The literature survey deliberately casts a wide net. Later stages (2, 3) will
narrow focus based on complexity classification and algorithm-game matching.
The survey should capture enough detail to support those downstream decisions
without requiring re-review.
