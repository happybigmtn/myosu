# ARCHIVED

## 2026-04-02 review pass

- `ADR-001` commit `28801090f35a3ac056202acf7264c6f28efd5aa2`; validated with `test -d docs/adr`; `test -f docs/adr/000-template.md`; `test -f docs/adr/README.md`
- `ADR-002` commit `6594b059a7c9a5675b5f8fbfdcce72f15d05ed2b`; validated with `test $(ls docs/adr/0*.md | wc -l) -ge 8`; `grep -l 'Status:' docs/adr/001-*.md docs/adr/007-*.md`
- `ADR-003` commit `ff88d493bc22c34b89c00a0fb4a5f59e3378bc7a`; validated with `test -f docs/adr/stage-2-roadmap.md`; `grep -q 'reversib' docs/adr/stage-2-roadmap.md`
- `OBS-001` commit `e1de8b5142f541d9a13d638fc51377d36c190f78`; review note: smoke proof was hardened in this pass so poker smoke mode no longer depends on ambient codexpoker state. Validated with `SKIP_WASM_BUILD=1 cargo test -p myosu-play --quiet smoke_demo_renderer_uses_builtin_poker_surface`; `SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test`; `RUST_LOG=myosu_play=debug SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test 2>&1 | grep -q myosu_play`
- `SEC-002` commit `da1c4ec2d2b96d2b203e0f598550870ca5250786`; validated with `test -f SECURITY.md`; `grep -q SECURITY.md README.md`
- `SEC-003` commit `80fe6a41acc8ba584ef824e525424ae5bece34a7`; validated with `test -f ops/cve-tracking-process.md`; `grep -q cve-tracking SECURITY.md`
- `SEC-004` commit `07158dd798ecef6797f5d77163abaae9760b5f12`; validated with `grep -c 'SAFETY' crates/myosu-games-poker/src/codexpoker.rs | grep -q '[2-9]'`; `cargo test -p myosu-games-poker --quiet`
- `IT-004` commit `9e890e7205e575d3fc226faeb4ad4fe628e40224`; validated with `/tmp/actionlint-bin/actionlint .github/workflows/ci.yml`
- `PY-001` commit `533ab22df004950fc5813a5b6305afa0ba0c4ab3`; validated with `python -c "import methods; print('PY_IMPORT_OK')"`; `grep -c '__import__' methods.py | grep -q '^0$'`
- `PY-002` commit `41bc48c3d34165cfd7f57517a03a4eff0f429f58`; validated with `python -c "from metrics import paired_sign_flip_test; import numpy as np; print(paired_sign_flip_test(np.array([0.1, -0.2, 0.3])))"`; `python -c "from metrics import paired_sign_flip_test; import numpy as np; paired_sign_flip_test(np.random.randn(50)); print('OK')"`
- `PY-003` commit `70309c09a259d7d1cbf283bd42ba19ebacddbb8b`; validated with `ruff check main.py methods.py runner.py metrics.py data.py`
- `PY-004` commit `68b20b71e96dcc69962709504f5e515360bb708b`; validated with `python -m pytest tests/test_metrics.py tests/test_data.py -q`
- `PY-005` commit `8b80c847775509779d503fbf5e604c181b3f2a31`; validated with `ruff check main.py methods.py runner.py metrics.py data.py`; `python -m pytest tests/test_metrics.py tests/test_data.py -q`; `/tmp/actionlint-bin/actionlint .github/workflows/ci.yml`
- `DN-002` commit `0b36470be4cdcbb5675364185b6a2fc8212d57f1`; validated with `shellcheck ops/deploy-bootnode.sh`; `bash ops/deploy-bootnode.sh --dry-run --node-bin target/debug/myosu-chain`
- `DN-004` commit `77973f980ee3122c10bb9b908ab8057ffa8fa230`; validated with `bash .github/scripts/check_operator_network_bootstrap.sh`
- `OP-001` commit `dd04d561beed05b31f6032313d2f33e347e48cdd`; validated with `test -f docs/operator-guide/quickstart.md`; `grep -q 'myosu-keys' docs/operator-guide/quickstart.md`; `grep -q 'docs/operator-guide/quickstart.md' README.md`; `bash .github/scripts/check_operator_network_bootstrap.sh`
- `OP-002` commit `889871ee693c9f97cdff309e5f625b896d5dc57e`; validated with `test -f docs/operator-guide/architecture.md`; `grep -q 'architecture.md' docs/operator-guide/quickstart.md`; `grep -q 'docs/operator-guide/architecture.md' README.md`
- `OP-003` commit `4662046d3ab30c9d2e59c882de8d532aa7ba3d4e`; validated with `test -f docs/operator-guide/troubleshooting.md`; `test "$(grep -c '^## [0-9]' docs/operator-guide/troubleshooting.md)" -ge 10`; `grep -q 'troubleshooting.md' docs/operator-guide/quickstart.md`; `grep -q 'troubleshooting.md' docs/execution-playbooks/operator-network.md`
- `RG-001` commit `2768b772fda35f11ff7066c2dadc8424423bd651`; validated with `test -f CHANGELOG.md`; `grep -q '0.1.0' CHANGELOG.md`; `grep -q 'CHANGELOG.md' README.md`
- `RG-002` commit `3fef79fcbb6c928487e7b6c72567444d37d8d876`; validated with `shellcheck ops/release.sh`; `bash ops/release.sh --dry-run v0.1.0`
- `RG-003` commit `85c0c4645e9c65e94627be8a87518ea7184095bd`; validated with `test -f docs/operator-guide/upgrading.md`; `grep -q 'upgrading.md' docs/operator-guide/quickstart.md`; `grep -q 'docs/operator-guide/upgrading.md' README.md`
- `G3-001` commit `608f1786d9133cbf731e52eed3a069f343c65663`; validated with `cargo test -p myosu-games-kuhn --quiet`; `cargo test -p myosu-games --quiet`
- `G3-002` commit `1ffa3299e719faba5d4c24bcaf72e8624200e6a6`; validated with `cargo test -p myosu-games-kuhn --quiet`
- `G3-003` commit `9aef01245ad7617e6bbf6243db86945dfe88a784`; validated with `SKIP_WASM_BUILD=1 cargo test -p myosu-play --quiet smoke_demo_renderer_uses_builtin_poker_surface`; `cargo test -p myosu-games-kuhn --quiet`; `SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --game kuhn --smoke-test`; `SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test`
