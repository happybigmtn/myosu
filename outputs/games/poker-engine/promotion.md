merge_ready: no
manual_proof_pending: yes
reason: The lane is not merge-ready because `remote_poker_exploitability` is implemented as a sampling proxy instead of the reviewed `Tree::build` plus `Profile::exploitability()` path, and the verification artifact misstates unhydrated `strategy()` behavior while database-backed proof remains unperformed.
next_action: Implement the reviewed exploitability path, rerun the database-backed proof set, and regenerate the implementation and verification artifacts so they truthfully match the code.
