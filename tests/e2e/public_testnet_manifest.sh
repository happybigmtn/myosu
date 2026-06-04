#!/usr/bin/env bash
# W-01 (public-testnet manifest) contract drift guard.
#
# Implements the W-01 row in IMPLEMENTATION_PLAN.md:
#   "Publish a reference testnet manifest + WS endpoint contract
#    (CEO lens: the testnet is the product; no public endpoint = no
#    adoption)."
#
# This proof fail-closes if any of the four sources of truth that
# together describe the public Myosu testnet drift apart:
#
#   1. `docs/operator-guide/public-testnet.md`  - the human-readable
#      public contract (operator-facing).
#   2. `ops/testnet/manifest.yaml`             - the machine-readable
#      public contract (what an external agent / partner `curl`s).
#   3. `ops/testnet/healthcheck.json`          - the canonical
#      `system_health` response body the public endpoints are
#      guaranteed to return.
#   4. `README.md` "Operator Path"              - the public README
#      link to the new doc.
#
# Five real assertions; failing any one of them fails the gate with a
# concrete error message that names the offending file and the
# requirement it broke. The `public-testnet-manifest` CI job in
# `.github/workflows/ci.yml` runs the harness on every PR and push
# to trunk.
#
#   1. `ops/testnet/manifest.yaml` exists, parses as YAML, and carries
#      every required top-level field (`chain_spec`, `rpc_endpoints`,
#      `bootnodes`, `subnet`, `promotable_games`, `authority_uris`,
#      `healthcheck`, `last_rotated_at`). The `chain_spec` field MUST
#      be the literal `test_finney` (the P0 testnet spec the public
#      endpoints serve; a change to a different spec is a different
#      contract, not a doc edit).
#   2. `ops/testnet/healthcheck.json` exists, parses as JSON, and the
#      `result` object carries the three `system_health` fields the
#      public contract guarantees (`peers`, `isSyncing`,
#      `shouldHavePeers`). A field-name change is a contract change.
#   3. `docs/operator-guide/public-testnet.md` exists, is non-empty,
#      and is labeled with the W-01 row name so a future reader can
#      locate the source row in `IMPLEMENTATION_PLAN.md`. The doc
#      names the CORS-enabled WS contract (`wss://` /
#      `--rpc-cors all` / `--rpc-methods unsafe`), the four
#      `//myosu//testnet//authority-*` URIs (matching the manifest's
#      `authority_uris` list), and embeds the example `system_health`
#      body the `healthcheck.json` file carries.
#   4. The `promotable_games` list in the manifest agrees with the
#      `ops/solver_promotion.yaml` ledger's `tier: promotable_local`
#      rows. The public contract is the operator's promise, and the
#      ledger is the engine's truth; they must match.
#   5. `README.md` "Operator Path" links to the new doc so a new
#      operator finds the public contract on the same page as the
#      rest of the operator path.
#
# A drift in any of the above forces the contract to be updated in
# the same commit, not silently overridden by an editor hand-waving
# that "the doc will catch up next PR".
#
# Required pre-conditions:
#   - python3 (for YAML / JSON parsing; no extra Python deps; the
#     harness uses only the standard library).
#
# Usage:  bash tests/e2e/public_testnet_manifest.sh

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

doc_path="$repo_root/docs/operator-guide/public-testnet.md"
manifest_path="$repo_root/ops/testnet/manifest.yaml"
healthcheck_path="$repo_root/ops/testnet/healthcheck.json"
ledger_path="$repo_root/ops/solver_promotion.yaml"
readme_path="$repo_root/README.md"

if ! command -v python3 >/dev/null 2>&1; then
    printf 'public_testnet_manifest: python3 not found in PATH\n' >&2
    exit 1
fi

# -- 1. The manifest exists, parses as YAML, and carries every
#       required field with the literal `test_finney` chain spec.
if [[ ! -s "$manifest_path" ]]; then
    printf 'public_testnet_manifest: manifest missing or empty: %s\n' \
        "$manifest_path" >&2
    exit 1
fi
manifest_json="$(python3 -c '
import json, sys, yaml
d = yaml.safe_load(open(sys.argv[1]))
json.dump(d, sys.stdout, sort_keys=True)
' "$manifest_path" 2>/dev/null)" || {
    printf 'public_testnet_manifest: manifest failed YAML parse: %s\n' \
        "$manifest_path" >&2
    exit 1
}
for field in chain_spec rpc_endpoints bootnodes subnet promotable_games \
    authority_uris healthcheck last_rotated_at; do
    if ! printf '%s\n' "$manifest_json" | python3 -c '
import json, sys
d = json.loads(sys.stdin.read())
sys.exit(0 if "'"$field"'" in d and d["'"$field"'"] is not None else 1)
'; then
        printf 'public_testnet_manifest: manifest missing required field: %s\n' \
            "$field" >&2
        exit 1
    fi
done
if ! printf '%s\n' "$manifest_json" | python3 -c '
import json, sys
d = json.loads(sys.stdin.read())
sys.exit(0 if d.get("chain_spec") == "test_finney" else 1)
'; then
    printf 'public_testnet_manifest: manifest chain_spec is not test_finney: %s\n' \
        "$(printf '%s' "$manifest_json" | python3 -c 'import json,sys; print(json.loads(sys.stdin.read()).get("chain_spec"))')" >&2
    exit 1
fi
manifest_subnet="$(printf '%s' "$manifest_json" | python3 -c 'import json,sys; print(json.loads(sys.stdin.read()).get("subnet"))')"
if [[ "$manifest_subnet" != "7" ]]; then
    printf 'public_testnet_manifest: manifest subnet is not 7: %s\n' \
        "$manifest_subnet" >&2
    exit 1
fi
manifest_authority_count="$(printf '%s' "$manifest_json" | python3 -c '
import json, sys
d = json.loads(sys.stdin.read())
print(len(d.get("authority_uris", [])))
')"
if (( manifest_authority_count < 4 )); then
    printf 'public_testnet_manifest: manifest authority_uris has %d entries; expected >= 4\n' \
        "$manifest_authority_count" >&2
    exit 1
fi
# Every authority URI must be a testnet URI (the public contract is
# the public testnet, not the devnet or any future mainnet).
if ! printf '%s' "$manifest_json" | python3 -c '
import json, sys
d = json.loads(sys.stdin.read())
uris = d.get("authority_uris", [])
sys.exit(0 if all(u.startswith("//myosu//testnet//") for u in uris) else 1)
'; then
    printf 'public_testnet_manifest: manifest authority_uris contain a non-testnet URI\n' >&2
    exit 1
fi
# Every RPC endpoint must be the wss:// / https:// shape (the public
# contract is the public WSS+HTTP+WS API, not raw TCP / IPC).
if ! printf '%s' "$manifest_json" | python3 -c '
import json, sys
d = json.loads(sys.stdin.read())
endpoints = d.get("rpc_endpoints", [])
ok = all(e.startswith("wss://") or e.startswith("https://") for e in endpoints)
sys.exit(0 if ok else 1)
'; then
    printf 'public_testnet_manifest: manifest rpc_endpoints contain a non-https/wss URI\n' >&2
    exit 1
fi
# last_rotated_at must be a parseable ISO 8601 timestamp.
if ! printf '%s' "$manifest_json" | python3 -c '
import datetime, json, sys
d = json.loads(sys.stdin.read())
ts = d.get("last_rotated_at", "")
# Trailing Z means UTC; tolerate +/-HH:MM offsets too.
ts_norm = ts.replace("Z", "+00:00") if ts.endswith("Z") else ts
datetime.datetime.fromisoformat(ts_norm)
'; then
    printf 'public_testnet_manifest: manifest last_rotated_at is not ISO 8601: %s\n' \
        "$(printf '%s' "$manifest_json" | python3 -c 'import json,sys; print(json.loads(sys.stdin.read()).get("last_rotated_at"))')" >&2
    exit 1
fi

# -- 2. The healthcheck.json example exists, parses as JSON, and the
#       `result` object carries the three system_health fields the
#       public contract guarantees.
if [[ ! -s "$healthcheck_path" ]]; then
    printf 'public_testnet_manifest: healthcheck.json missing or empty: %s\n' \
        "$healthcheck_path" >&2
    exit 1
fi
if ! python3 -c '
import json, sys
d = json.load(open(sys.argv[1]))
result = d.get("result", {})
for f in ("peers", "isSyncing", "shouldHavePeers"):
    if f not in result:
        sys.exit(1)
sys.exit(0)
' "$healthcheck_path"; then
    printf 'public_testnet_manifest: healthcheck.json missing required field in result\n' >&2
    exit 1
fi
# The healthcheck.json example must also have the literal
# `system_health` method label somewhere (either in the request id
# context or as a comment field the public doc can quote). The clean
# shape: the file is the canonical response body, not the request,
# so we look for the three fields and `isSyncing` being a bool.
if ! python3 -c '
import json, sys
d = json.load(open(sys.argv[1]))
result = d.get("result", {})
sys.exit(0 if isinstance(result.get("isSyncing"), bool) else 1)
' "$healthcheck_path"; then
    printf 'public_testnet_manifest: healthcheck.json isSyncing field is not a bool\n' >&2
    exit 1
fi

# -- 3. The operator-facing doc exists, is labeled with W-01, names
#       the CORS+WS contract, the four authority URIs, and embeds
#       the example system_health body.
if [[ ! -s "$doc_path" ]]; then
    printf 'public_testnet_manifest: doc missing or empty: %s\n' "$doc_path" >&2
    exit 1
fi
# The doc's H1 title must include the W-01 row label so a future
# reader can locate the source row in `IMPLEMENTATION_PLAN.md` from
# the doc itself.
if ! head -n 1 "$doc_path" | grep -Fq '(W-01)'; then
    printf 'public_testnet_manifest: doc H1 is missing the W-01 row label: %s\n%s\n' \
        "$(head -n 1 "$doc_path")" "$doc_path" >&2
    exit 1
fi
for needle in 'wss://' '--rpc-cors all' '--rpc-methods unsafe' \
    '`peers`' '`isSyncing`' '`shouldHavePeers`' \
    'ops/testnet/manifest.yaml' 'ops/testnet/healthcheck.json' \
    'tests/e2e/public_testnet_manifest.sh' \
    '//myosu//testnet//authority-1' '//myosu//testnet//authority-2' \
    '//myosu//testnet//authority-3' '//myosu//testnet//subnet-owner'; do
    if ! grep -Fq -- "$needle" "$doc_path"; then
        printf 'public_testnet_manifest: doc missing required marker: %s\n%s\n' \
            "$needle" "$doc_path" >&2
        exit 1
    fi
done
# The doc must also mention `test_finney` (the chain spec the public
# endpoints serve) and the `wss://myosu-testnet.example.com:9944`
# endpoint shape.
for needle in 'test_finney' 'myosu-testnet.example.com' 'W-01'; do
    if ! grep -Fq -- "$needle" "$doc_path"; then
        printf 'public_testnet_manifest: doc missing required marker: %s\n' \
            "$needle" >&2
        exit 1
    fi
done

# -- 4. The promotable_games list in the manifest agrees with the
#       `ops/solver_promotion.yaml` ledger's `tier: promotable_local`
#       rows. The harness reads the live ledger and the live manifest
#       and asserts the sets are equal.
if [[ ! -s "$ledger_path" ]]; then
    printf 'public_testnet_manifest: ledger missing: %s\n' "$ledger_path" >&2
    exit 1
fi
ledger_promotable="$(python3 -c '
import yaml, sys
d = yaml.safe_load(open(sys.argv[1]))
games = d.get("games", []) if isinstance(d, dict) else []
names = []
current = None
for entry in games:
    if not isinstance(entry, dict):
        continue
    name = entry.get("game")
    tier = entry.get("tier")
    if name is None or tier is None:
        continue
    if tier == "promotable_local":
        names.append(name)
print(" ".join(sorted(set(names))))
' "$ledger_path")"
manifest_promotable="$(printf '%s' "$manifest_json" | python3 -c '
import json, sys
d = json.loads(sys.stdin.read())
print(" ".join(sorted(d.get("promotable_games", []))))
')"
if [[ "$ledger_promotable" != "$manifest_promotable" ]]; then
    printf 'public_testnet_manifest: promotable_games drift\n  manifest: %s\n  ledger:   %s\n' \
        "$manifest_promotable" "$ledger_promotable" >&2
    exit 1
fi

# -- 5. README.md "Operator Path" links to the new doc so a new
#       operator finds the public contract on the same page as the
#       rest of the operator path.
if [[ ! -s "$readme_path" ]]; then
    printf 'public_testnet_manifest: README missing: %s\n' "$readme_path" >&2
    exit 1
fi
if ! grep -Fq 'public-testnet.md' "$readme_path"; then
    printf 'public_testnet_manifest: README.md does not link to public-testnet.md\n' >&2
    exit 1
fi
if ! grep -Fqi 'Operator Path' "$readme_path"; then
    printf 'public_testnet_manifest: README.md has no Operator Path section\n' >&2
    exit 1
fi
if ! grep -Fqi 'Public testnet' "$readme_path"; then
    printf 'public_testnet_manifest: README.md has no Public testnet subsection\n' >&2
    exit 1
fi

printf 'PUBLIC_TESTNET_MANIFEST_HARNESS myosu e2e ok surface=public_testnet_contract sub_checks=5 manifest_chain_spec=test_finney manifest_subnet=7 promotable_games=%s authority_uris=%d\n' \
    "$manifest_promotable" "$manifest_authority_count"
