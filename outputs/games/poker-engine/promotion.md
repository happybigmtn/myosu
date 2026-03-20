merge_ready: no
manual_proof_pending: no
reason: The slice is not merge-ready because core poker-engine proof still depends on 11 ignored tests, `remote_poker_exploitability` contains placeholder behavior, and the required implementation and verification artifacts are missing from `outputs/games/poker-engine`.
next_action: Implement the real remote exploitability path, restore non-ignored contract proof for the required poker-engine tests, materialize the curated implementation and verification artifacts in `outputs/games/poker-engine`, and rerun the lane proof.
