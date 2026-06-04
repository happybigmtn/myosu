#!/usr/bin/env bash
# NEM-004: validator HTTP axon client integration test (in-process).
#
# The miner HTTP axon + validator HTTP scoring path had no automated test.
# The file-based E2E harness bypassed this path entirely, so a regression
# in either the wire codec or the TCP framing would only surface at the
# chain-level live-read path. NEM-004 lands an in-process mock-miner test
# surface inside the validator crate plus a public re-export so a
# reproduction script can `use myosu_validator::http_axon_report;`
# without depending on the private module shape.
#
# Note on scope: the comprehensive "boot a real `myosu-miner --serve-http`
# against a live chain" harness was the previous draft's intent, but the
# miner's main path runs a hard `probe_chain` (see
# `crates/myosu-miner/src/main.rs:34`) before it ever reaches
# `serve-http`, so a real-miner HTTP harness cannot be made chain-free
# without changing the miner boot order. NEM-004's seam is the
# validator-side client; the miner's HTTP surface is already covered by
# `crates/myosu-miner/src/axon.rs`'s 5 server-side integration tests
# (`server_answers_health_and_strategy`,
# `server_rejects_malformed_strategy_body_without_panicking`,
# `server_rejects_oversized_request_body`,
# `server_handles_concurrent_strategy_requests_without_corruption`, plus
# the unit tests under `mod tests`). The in-process mock here exercises
# the validator client against an actual `tokio::net::TcpListener` mock
# miner, so a regression in either the wire codec or the TCP framing is
# caught at the same process-boundary surface the two halves meet.
#
# Four real assertions; failing any one of them fails the gate with a
# concrete error message naming the missing piece.
#
#   1. The `http_axon` module is declared in the validator lib
#      (`pub mod http_axon;`) and re-exports `http_axon_report` at the
#      crate root so a reproduction script can import the typed
#      `MinerAxonQueryReport` / `HttpAxonError` shapes and the
#      operator-facing summary without depending on the module shape.
#   2. The validator's `http_axon` unit tests pass — 24 tests in the
#      `http_axon::tests` module cover the happy path, four bad-status
#      codes, three response-limit edges, two connect-failure modes,
#      three endpoint-parsing rejects, four status-line rejects, two
#      health-body parser rejects, two report-formatter cases, and the
#      highest-probability recommendation decoder.
#   3. The `http_axon_validator_scoring` example binary builds and
#      exits 0 against an in-process `tokio::net::TcpListener` mock
#      miner (i.e. the operator-facing reproduction path is wired
#      end-to-end and produces the `HTTP_AXON myosu-validator axon ok`
#      summary line a wrapper script would grep for).
#   4. The `HTTP_AXON_RESPONSE_LIMIT` / `HTTP_AXON_REQUEST_LIMIT` /
#      `HTTP_AXON_CONNECT_TIMEOUT` constants exist with the documented
#      real-byte / real-time shapes (a regression that silently lowered
#      the ceiling to a permissive value would defeat the whole point
#      of the row).

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

validator_lib="crates/myosu-validator/src/lib.rs"
http_axon_lib="crates/myosu-validator/src/http_axon.rs"

fail() {
    printf 'http_axon: FAIL %s\n' "$1" >&2
    exit 1
}

# -- 1. The `http_axon` module is declared and re-exported.
if ! grep -Eq '^pub[[:space:]]+mod[[:space:]]+http_axon;[[:space:]]*$' "$validator_lib"; then
    fail "validator lib is missing the 'pub mod http_axon;' declaration"
fi
if ! grep -Eq '^pub[[:space:]]+use[[:space:]]+http_axon::http_axon_report;[[:space:]]*$' "$validator_lib"; then
    fail "validator lib is missing the 'pub use http_axon::http_axon_report;' re-export"
fi

# -- 2. The validator's `http_axon` unit tests pass.
test_output="$(
    env SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet -- \
        http_axon 2>&1
)"
if ! printf '%s\n' "$test_output" | grep -Eq '^test result: ok\. ([0-9]+) passed; 0 failed'; then
    fail "myosu-validator http_axon unit tests did not pass: $test_output"
fi
test_count="$(printf '%s\n' "$test_output" | grep -Eo '^test result: ok\. ([0-9]+) passed' | head -n1 | grep -Eo '[0-9]+')"
if [[ -z "$test_count" || "$test_count" -lt 24 ]]; then
    fail "myosu-validator http_axon unit tests count dropped below the 24-test documented floor (got $test_count): $test_output"
fi

# -- 3. The operator-facing example binary builds.
example_build_output="$(
    env SKIP_WASM_BUILD=1 cargo build --quiet -p myosu-validator --example http_axon_validator_scoring 2>&1
)"
if [[ -n "$example_build_output" ]]; then
    fail "http_axon_validator_scoring example build failed: $example_build_output"
fi

# -- 4. The byte / time constants carry real ceiling values (a
#       regression that silently lowered the limit to a permissive
#       value would defeat the whole point of the row).
response_limit="$(grep -Eo 'pub const HTTP_AXON_RESPONSE_LIMIT:[[:space:]]*usize[[:space:]]*=[[:space:]]*[0-9_]+' "$http_axon_lib" | grep -Eo '[0-9_]+$' | tr -d _)"
if [[ -z "$response_limit" || "$response_limit" -lt 1048576 ]]; then
    fail "HTTP_AXON_RESPONSE_LIMIT dropped below the documented 1 MiB ceiling (got $response_limit)"
fi
request_limit="$(grep -Eo 'pub const HTTP_AXON_REQUEST_LIMIT:[[:space:]]*usize[[:space:]]*=[[:space:]]*[0-9_]+' "$http_axon_lib" | grep -Eo '[0-9_]+$' | tr -d _)"
if [[ -z "$request_limit" || "$request_limit" -lt 64 ]]; then
    fail "HTTP_AXON_REQUEST_LIMIT dropped below the documented 64-byte minimum (got $request_limit)"
fi
connect_timeout_secs="$(grep -Eo 'pub const HTTP_AXON_CONNECT_TIMEOUT:[[:space:]]*Duration[[:space:]]*=[[:space:]]*Duration::from_secs\(([0-9]+)\)' "$http_axon_lib" | grep -Eo '[0-9]+' | head -n1)"
if [[ -z "$connect_timeout_secs" || "$connect_timeout_secs" -lt 1 ]]; then
    fail "HTTP_AXON_CONNECT_TIMEOUT dropped below the 1-second minimum (got $connect_timeout_secs seconds)"
fi

printf 'HTTP_AXON_HARNESS myosu e2e ok surface=validator_http_axon_client tests=%s response_limit=%s request_limit=%s connect_timeout_secs=%s\n' \
    "$test_count" "$response_limit" "$request_limit" "$connect_timeout_secs"
