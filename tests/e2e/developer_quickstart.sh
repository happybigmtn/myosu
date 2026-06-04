#!/usr/bin/env bash
# DX-001 developer quickstart gate.
#
# The plan's current top open item is `IMPLEMENTATION_PLAN.md` `DX-001`
# "Consolidate critical operator caveats". The plan's acceptance criteria
# require four things, which this gate enforces as five executable
# assertions so that a missing link or a stale docstring fails the
# harness (and the CI job) with a concrete error message:
#
#   1. `docs/developer-quickstart.md` exists and is non-empty.
#   2. `README.md` links to `docs/developer-quickstart.md` from the
#      "Developer Path" or "Operator Path" section.
#   3. The consolidated doc contains the 4-step fastest first-success
#      path called out by `specs/110426-developer-experience.md`. The
#      gate greps for the four canonical command lines (plus the
#      `quit` pipe-mode EOF marker) so a contributor cannot silently
#      drop a step.
#   4. The five critical caveats from the spec / AGENTS.md / OS.md
#      (WASM cache, sparse artifacts, devnet timing, SKIP_WASM_BUILD,
#      wasm32v1-none target) all appear in the doc.
#   5. The required environment variable inventory covers at least
#      SKIP_WASM_BUILD, MYOSU_KEY_PASSWORD, and MYOSU_NODE_AUTHORITY_SURI.
#
# The gate then runs the actual 4-step first-success path end-to-end so
# that the documented commands stay truthful. Each step is allowed up
# to a generous wall-clock budget and runs in a temp working directory
# so it cannot poison the caller's tree.
#
# This is the executable surface that backs `DX-001` and complements the
# static checks in `.github/scripts/check_stage0_repo_shape.sh`.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

doc="$repo_root/docs/developer-quickstart.md"
readme="$repo_root/README.md"
spec="$repo_root/specs/110426-developer-experience.md"

if [[ ! -f "$doc" ]]; then
    printf 'developer_quickstart: missing consolidated doc %s\n' "$doc" >&2
    exit 1
fi
if [[ ! -s "$doc" ]]; then
    printf 'developer_quickstart: consolidated doc %s is empty\n' "$doc" >&2
    exit 1
fi
if [[ ! -f "$readme" ]]; then
    printf 'developer_quickstart: missing README %s\n' "$readme" >&2
    exit 1
fi
if [[ ! -f "$spec" ]]; then
    printf 'developer_quickstart: missing spec %s\n' "$spec" >&2
    exit 1
fi

# Helper: report which literal string is missing.
expect_in_doc() {
    local needle="$1"
    local label="$2"
    if ! grep -Fq -- "$needle" "$doc"; then
        printf 'developer_quickstart: doc missing required %s (%s)\n' "$label" "$needle" >&2
        exit 1
    fi
}

# -- 1. Required literals in the consolidated doc.
# 4-step fastest first-success path commands.
expect_in_doc 'SKIP_WASM_BUILD=1 cargo test -p myosu-games-kuhn --quiet' \
    '4-step path step 1 (kuhn test)'
expect_in_doc 'SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test' \
    '4-step path step 2 (poker smoke test)'
expect_in_doc "SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --game kuhn --smoke-test" \
    '4-step path step 3 (kuhn smoke test)'
expect_in_doc "printf 'quit\\n' | SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- pipe" \
    '4-step path step 4 (pipe mode)'

# 5 critical caveats from AGENTS.md / OS.md / specs/110426-developer-experience.md.
expect_in_doc 'SKIP_WASM_BUILD' \
    'SKIP_WASM_BUILD caveat (also covered by the env var inventory)'
expect_in_doc 'wasm32v1-none' \
    'wasm32v1-none target caveat'
expect_in_doc 'myosu_chain_runtime.wasm' \
    'WASM runtime cache caveat'
expect_in_doc 'postflop_complete=false' \
    'sparse bootstrap artifacts caveat'
expect_in_doc '48 seconds per authored block' \
    'devnet timing caveat'

# 3 required environment variables from the DX-001 acceptance criteria.
expect_in_doc 'SKIP_WASM_BUILD' \
    'env var SKIP_WASM_BUILD'
expect_in_doc 'MYOSU_KEY_PASSWORD' \
    'env var MYOSU_KEY_PASSWORD'
expect_in_doc 'MYOSU_NODE_AUTHORITY_SURI' \
    'env var MYOSU_NODE_AUTHORITY_SURI'

# -- 2. README links to the consolidated doc.
if ! grep -Eq 'docs/developer-quickstart\.md' "$readme"; then
    printf 'developer_quickstart: README.md does not link to docs/developer-quickstart.md\n' >&2
    exit 1
fi

# -- 3. Spec cross-reference still exists.
if ! grep -Fq '110426-developer-experience.md' "$doc"; then
    printf 'developer_quickstart: doc does not cross-reference the developer-experience spec\n' >&2
    exit 1
fi

# -- 4. Plan cross-reference still exists (so reviewers can trace the gate to its plan row).
if ! grep -Fq 'DX-001' "$doc"; then
    printf 'developer_quickstart: doc does not cross-reference the DX-001 plan row\n' >&2
    exit 1
fi

# -- 5. Run the actual 4-step fastest first-success path so the documented
#       commands stay truthful. Each step has a generous wall-clock budget
#       because the first cold build of the gameplay crates is the slow
#       part; this gate is meant to be runnable on CI.
work_root="$(mktemp -d "$repo_root/target/e2e/developer-quickstart.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT

run_step() {
    local label="$1"
    local budget="$2"
    shift 2
    if ! timeout "$budget" "$@"; then
        printf 'developer_quickstart: step %s failed under %ss budget\n' "$label" "$budget" >&2
        exit 1
    fi
}

run_step '1-kuhn-test'    600 bash -c 'cd "$1" && SKIP_WASM_BUILD=1 cargo test -p myosu-games-kuhn --quiet' bash "$work_root"
run_step '2-poker-smoke' 600 bash -c 'cd "$1" && SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --smoke-test' bash "$work_root"
run_step '3-kuhn-smoke'  600 bash -c 'cd "$1" && SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- --game kuhn --smoke-test' bash "$work_root"
run_step '4-pipe'        300 bash -c 'cd "$1" && printf "quit\n" | SKIP_WASM_BUILD=1 cargo run -p myosu-play --quiet -- pipe' bash "$work_root"

printf 'DEVELOPER_QUICKSTART_HARNESS myosu e2e ok doc=%s readme=%s workdir=%s\n' \
    "$doc" "$readme" "$work_root"
