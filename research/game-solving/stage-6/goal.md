# Stage 6: Architecture Candidates

## Objective

Define the neural network architecture search space for imperfect-information
game solvers. Specify the building blocks, connectivity patterns, and
hyperparameter ranges that the architecture search (stages 7-10) will explore.
The search space must be expressive enough to contain high-performing
architectures while constrained enough to be searchable within the compute
budget.

## Research Questions

### Primary

1. What neural network building blocks are appropriate for imperfect-
   information game solving -- convolutional, recurrent, attention-based,
   graph-based, or hybrid?

2. How should the search space encode the information set structure -- as a
   flat feature vector, a sequence of observations, or a structured graph?

3. What is the right granularity for the search space -- cell-level search
   (NAS-style), block-level search, or full-architecture search?

### Secondary

4. Should the search space include the abstraction layer (how game state is
   encoded) or treat it as fixed from stage 4?

5. What architectural constraints should be enforced to guarantee valid
   architectures (e.g., output size matching action space, information set
   encoding compatibility)?

6. How large a search space can be effectively explored within 10^4 GPU-hours?

7. Should separate search spaces be defined for different game complexity tiers
   (from stage 2) or one unified space?

## Scope

### Building Block Library

The search space is constructed from composable building blocks:

1. **Encoding Layers**:
   - One-hot card/tile encoding
   - Learned embedding lookup
   - Positional encoding for sequential observations
   - Graph encoding for structured game states

2. **Processing Layers**:
   - Fully connected (MLP) blocks with configurable width/depth
   - 1D convolutional blocks for sequential observation patterns
   - Multi-head self-attention blocks
   - Recurrent blocks (LSTM, GRU) for variable-length histories
   - Graph attention networks for relational game structure

3. **Aggregation Layers**:
   - Global average/max pooling
   - Attention-weighted aggregation
   - Belief state summarization (softmax over possible worlds)

4. **Output Heads**:
   - Action probability head (policy network)
   - Value head (state/info-set value estimation)
   - Advantage head (per-action advantage estimation)
   - Regret head (cumulative regret estimation for CFR)

### Search Space Parameters

| Parameter          | Range           | Type        |
|--------------------|-----------------|-------------|
| Encoding type      | {onehot, embed} | Categorical |
| Embedding dim      | [16, 256]       | Log-uniform |
| Processing blocks  | [1, 8]         | Integer     |
| Block type         | {MLP, Conv1D, Attention, LSTM, GAT} | Categorical |
| Hidden dim         | [32, 512]       | Log-uniform |
| Attention heads    | [1, 8]         | Integer     |
| Dropout rate       | [0.0, 0.3]     | Uniform     |
| Activation         | {ReLU, GELU, SiLU} | Categorical |
| Skip connections   | {none, residual, dense} | Categorical |
| Output heads       | {policy, policy+value, policy+regret} | Categorical |
| Normalization      | {none, layer, batch} | Categorical |

### Search Space Size Estimate

With the ranges above, the discrete search space contains approximately 10^8
configurations. Continuous parameters add infinite variation, but the effective
search space (configurations that differ meaningfully in performance) is
estimated at 10^5 -- 10^6.

## Methodology

### Phase 1: Building Block Selection

Review neural architecture search literature and imperfect-information game
solver architectures to select the building block library:

- Survey NAS search spaces (DARTS, NASNet, ProxylessNAS) for proven building
  blocks
- Survey game solver architectures (DeepStack, Pluribus, ReBeL, Player of
  Games) for domain-specific blocks
- Identify blocks that appear in both literatures (strong prior)
- Add blocks unique to game solving that NAS literature has not explored

### Phase 2: Constraint Definition

Define hard constraints that ensure valid architectures:

- Output dimension must match the game's action space
- Information set encoding must be compatible with the selected processing
  blocks
- Memory budget: total parameters < 10^7 (to keep training tractable)
- Inference latency: single forward pass < 1ms on target hardware

### Phase 3: Search Space Validation

Validate the search space by:

- Sampling 50 random architectures and verifying they instantiate correctly
- Training 10 sampled architectures on a Tier 1 game to verify learning signal
- Confirming that the search space contains known-good architectures (e.g.,
  the DeepStack architecture should be representable)
- Measuring the performance variance across random samples (high variance
  indicates the search space contains both good and bad architectures,
  which is desirable)

### Phase 4: Proxy Task Compatibility

Verify that the search space is compatible with the CIFAR-100 proxy task
(stage 7) -- architectures sampled from this space can process both game
states and image inputs with minimal adaptation (only the encoding/output
layers change).

## Expected Outcomes

1. A formal search space specification with building blocks, ranges, and
   constraints.

2. A search space implementation (Python) that can sample random architectures,
   enumerate neighbors, and validate configurations.

3. Validation evidence that the space is expressive (contains good and bad
   architectures) and tractable (random samples instantiate and train).

4. Compatibility verification with the proxy task framework.

## Success Criteria

- 100% of randomly sampled architectures pass constraint validation.
- Performance variance across 10 random architectures on a Tier 1 game spans
  at least 2x from worst to best (indicating the space is expressive).
- Known-good architectures from published solvers are representable in the
  search space.
- Search space implementation samples an architecture in < 100ms.

## Dependencies

- Stage 2 (Complexity Classification): game action space sizes determine
  output head constraints
- Stage 3 (Algorithm-Game Matching): which algorithms pair with architectures
  (determines output head types: policy, value, regret)
- Stage 4 (Abstraction Taxonomy): abstraction choices determine encoding
  layer requirements

## Outputs

- `search_space_spec.json` -- formal specification with blocks, ranges,
  constraints
- `search_space.py` -- implementation with sample/validate/enumerate methods
- `validation_results.json` -- random sample training curves and variance
  measurements
- `known_architecture_mappings.json` -- how published architectures map into
  the search space

## Notes

The search space is deliberately larger than what any single search method can
explore exhaustively. The architecture search methods in stages 7-10 (Random
Search, DARTS, etc.) each explore a different region. The key design decision
is balancing expressiveness (the space could contain a great architecture) with
searchability (the methods can find it within budget).
