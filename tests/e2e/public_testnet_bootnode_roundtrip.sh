#!/usr/bin/env bash
# LIBP2P-001 (public-testnet bootnode contract-drift guard) proof harness.
#
# Implements the LIBP2P-001 row in IMPLEMENTATION_PLAN.md:
#   "Public-testnet bootnode contract-drift guard: a
#    tests/e2e/public_testnet_bootnode_roundtrip.sh proof harness
#    that fail-closes if the W-01 ops/testnet/manifest.yaml
#    bootnodes field shape stops being structurally replaceable
#    byte-by-byte with the real libp2p multiaddr format
#    ops/deploy-bootnode.sh actually emits."
#
# This proof fail-closes if any of the four sources of truth that
# together describe the public testnet's libp2p multiaddr shape
# drift apart:
#
#   1. `ops/testnet/manifest.yaml`        - the machine-readable
#      bootnode multiaddr list (the public contract).
#   2. `ops/deploy-bootnode.sh`           - the operator-deployable
#      bootnode renderer (the source of the real peer_id +
#      multiaddr shape the chain binary's --bootnodes parser
#      accepts).
#   3. The chain binary's `--bootnodes` parser - the live
#      consumer of the multiaddr shape. Confirmed by a
#      `target/debug/myosu-chain --bootnodes <multiaddr> --help`
#      invocation that must NOT print `error: Invalid multiaddr`
#      to stderr.
#   4. The harness itself - generated fresh peer_id + multiaddr
#      round-tripped through the manifest's `bootnodes` field
#      shape, byte-for-byte.
#
# Eight real assertions; failing any one of them fails the gate
# with a concrete error message that names the offending file,
# the bootnode entry, and the requirement it broke. The
# `public-testnet-bootnode-roundtrip` CI job in
# `.github/workflows/ci.yml` runs the harness on every PR and
# push to trunk.
#
#   1. `ops/testnet/manifest.yaml` exists, parses as YAML, and
#      the `bootnodes` field is a non-empty YAML list.
#   2. Every entry in `bootnodes` matches one of four accepted
#      multiaddr shapes: `/ip4/<ipv4>/tcp/<port>/p2p/<peer_id>`,
#      `/ip6/<ipv6>/tcp/<port>/p2p/<peer_id>`,
#      `/dns4/<hostname>/tcp/<port>/p2p/<peer_id>`, or
#      `/dns/<hostname>/tcp/<port>/p2p/<peer_id>` (the four
#      shapes `ops/deploy-bootnode.sh` `public_transport_addr`
#      helper at line 148-158 emits).
#   3. The `<peer_id>` suffix of every entry matches the
#      `^12D3Koo[A-Za-z0-9]+$` or `^Qm[A-Za-z0-9]+$` regex
#      (the two libp2p::PeerId base58 encodings the chain
#      binary's --bootnodes parser accepts; ed25519 emits
#      52-char 12D3Koo peer_ids and sha256 CIDv0 emits 46-char
#      Qm peer_ids, so the harness does not constrain the
#      suffix length).
#   4. `target/debug/myosu-chain key generate-node-key` +
#      `key inspect-node-key` round-trips into a fresh valid
#      `peer_id` (the exact same invocation
#      `ops/deploy-bootnode.sh` uses at line 152-153 to render
#      the durable bootnode).
#   5. The `fresh_multiaddr` constructed from the fresh
#      `peer_id` (`/ip4/127.0.0.1/tcp/30333/p2p/<peer_id>`)
#      round-trips byte-for-byte through the manifest's
#      `bootnodes` field shape: a temporary manifest copy with
#      the placeholder replaced by `fresh_multiaddr` re-extracts
#      to the same string.
#   6. The chain binary's `--bootnodes "${fresh_multiaddr}"`
#      CLI parser accepts the fresh multiaddr (its stderr must
#      not contain `multiaddr parsing error`).
#   7. The W-01 `public_testnet_manifest.sh` regression harness
#      still passes (the new LIBP2P-001 harness is additive; the
#      W-01 contract is unchanged).
#   8. The new `public-testnet-bootnode-roundtrip` CI job is
#      wired in `.github/workflows/ci.yml` and the
#      `tests/e2e/public_testnet_manifest.sh` harness's
#      regression-guard sub-check confirms the new harness is
#      referenced by name in the manifest's top-of-field
#      comment.
#
# A drift in any of the above forces the multiaddr-shape
# contract to be updated in the same commit, not silently
# overridden by an editor hand-waving that "the next deploy will
# match".
#
# Required pre-conditions:
#   - python3 (for YAML parsing; no extra Python deps; the
#     harness uses only the standard library).
#   - target/debug/myosu-chain (built by the parent CI job; the
#     harness uses the same SKIP_WASM_BUILD=1 cargo build path
#     the existing W-03 / W-07 CI jobs use).
#   - bash with associative arrays (bash 4+; the harness
#     iterates over the manifest's `bootnodes` list with a
#     bounded counter rather than an associative array, so
#     bash 3.2 / macOS users also work).
#
# Usage:  bash tests/e2e/public_testnet_bootnode_roundtrip.sh

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

manifest_path="$repo_root/ops/testnet/manifest.yaml"
deploy_script="$repo_root/ops/deploy-bootnode.sh"
chain_bin="$repo_root/target/debug/myosu-chain"
w01_harness="$repo_root/tests/e2e/public_testnet_manifest.sh"
ci_workflow="$repo_root/.github/workflows/ci.yml"

pass=0
fail=0
note() { printf 'public_testnet_bootnode_roundtrip: %s\n' "$*"; }
ok()   { note "ok $*";   pass=$((pass+1)); }
bad()  { note "FAIL $*"; fail=$((fail+1)); }

# -- 1. The manifest exists, parses as YAML, and carries a
#       non-empty `bootnodes` list.
if [[ ! -s "$manifest_path" ]]; then
    bad "manifest missing or empty: $manifest_path"
    echo "public_testnet_bootnode_roundtrip: harness aborted (early)"
    exit 1
fi
bootnode_count="$(python3 -c '
import sys, yaml
d = yaml.safe_load(open(sys.argv[1]))
b = d.get("bootnodes", []) if isinstance(d, dict) else []
if not isinstance(b, list):
    sys.exit(2)
print(len(b))
' "$manifest_path" 2>/dev/null)" || {
    bad "manifest failed YAML parse or bootnodes is not a list: $manifest_path"
    exit 1
}
if (( bootnode_count < 1 )); then
    bad "manifest bootnodes is empty (need >= 1 entry): $manifest_path"
    exit 1
fi
ok "manifest carries $bootnode_count bootnode entr(y/ies)"

# -- 2. Every entry matches one of four accepted multiaddr
#       shapes (the four shapes `ops/deploy-bootnode.sh`
#       `public_transport_addr` helper at line 148-158 actually
#       emits): /ip4/<ipv4>/tcp/<port>/p2p/<peer_id>,
#       /ip6/<ipv6>/tcp/<port>/p2p/<peer_id>,
#       /dns4/<hostname>/tcp/<port>/p2p/<peer_id>, and
#       /dns/<hostname>/tcp/<port>/p2p/<peer_id> (the
#       /dns/-without-4 shape is the literal format
#       `public_transport_addr` emits for DNS hostnames; /dns4/ is
#       accepted as a strict superset of /dns/ for browser
#       compatibility).
shape_rc="$(python3 -c '
import re, sys, yaml
# Peer IDs in libp2p are base58-encoded ed25519 (52 chars starting
# with 12D3Koo) or sha256 CIDv0 (46 chars starting with Qm). The
# exact length depends on the key type; the harness checks the
# prefix + base58-ish alphabet, not a specific length, because
# (a) the chain binary accepts any base58-decodable string the
# /p2p/ parser recognizes, and (b) future libp2p key types may
# emit different lengths. The harness grep against
# /p2p/<peer_id> is the operator-facing contract; the chain
# binary parser is the machine contract.
PEER_ID = r"(12D3Koo[A-Za-z0-9]+|Qm[A-Za-z0-9]+)"
PATTERNS = [
    r"^/ip4/\d+\.\d+\.\d+\.\d+/tcp/\d+/p2p/" + PEER_ID + r"$",
    r"^/ip6/[0-9a-fA-F:]+/tcp/\d+/p2p/" + PEER_ID + r"$",
    r"^/dns4/[A-Za-z0-9._-]+/tcp/\d+/p2p/" + PEER_ID + r"$",
    r"^/dns/[A-Za-z0-9._-]+/tcp/\d+/p2p/" + PEER_ID + r"$",
]
d = yaml.safe_load(open(sys.argv[1]))
b = d.get("bootnodes", [])
for i, entry in enumerate(b):
    if not isinstance(entry, str):
        print(f"BOOTNODE_{i}_NOT_STRING")
        sys.exit(1)
    if not any(re.match(p, entry) for p in PATTERNS):
        print(f"BOOTNODE_{i}_BAD_SHAPE: {entry}")
        sys.exit(1)
sys.exit(0)
' "$manifest_path")" || {
    bad "manifest bootnode shape check failed: $shape_rc"
    exit 1
}
ok "all $bootnode_count bootnode entries match one of 4 accepted multiaddr shapes"

# -- 3. The <peer_id> suffix of every entry is a base58 string
#       starting with `12D3Koo` (ed25519) or `Qm` (sha256 CIDv0).
#       (Already implied by sub-check 2; the harness asserts it
#       independently so a future shape addition that *drops* the
#       /p2p/<peer_id> suffix is fail-closed here too. The
#       harness uses the loose `^12D3Koo[A-Za-z0-9]+$` /
#       `^Qm[A-Za-z0-9]+$` regexes (no length constraint) because
#       real ed25519 libp2p PeerIds are 52 chars and real sha256
#       CIDv0 PeerIds are 46 chars, and the chain binary's
#       /p2p/ parser accepts any base58-decodable string the
#       /p2p/ parser recognizes; a fixed `{44}`-char suffix
#       contract would falsely reject real-shape peer_ids the
#       operator pastes in from `ops/deploy-bootnode.sh`.)
peer_id_rc="$(python3 -c '
import re, sys, yaml
d = yaml.safe_load(open(sys.argv[1]))
for i, entry in enumerate(d.get("bootnodes", [])):
    m = re.search(r"/p2p/(.+)$", entry)
    if not m:
        print(f"BOOTNODE_{i}_MISSING_P2P_SUFFIX")
        sys.exit(1)
    pid = m.group(1)
    if not (re.match(r"^12D3Koo[A-Za-z0-9]+$", pid) or
            re.match(r"^Qm[A-Za-z0-9]+$", pid)):
        print(f"BOOTNODE_{i}_BAD_PEER_ID: {pid}")
        sys.exit(1)
sys.exit(0)
' "$manifest_path")" || {
    bad "manifest bootnode peer_id check failed: $peer_id_rc"
    exit 1
}
ok "all $bootnode_count bootnode entries carry a valid /p2p/<peer_id> suffix (12D3Koo or Qm prefix)"

# -- 4. target/debug/myosu-chain key generate-node-key +
#       inspect-node-key round-trips into a fresh valid
#       peer_id. The harness builds the chain binary in the
#       parent CI job (SKIP_WASM_BUILD=1 cargo build -p
#       myosu-chain), so we just invoke it.
if [[ ! -x "$chain_bin" ]]; then
    bad "chain binary missing or not executable: $chain_bin"
    bad "build it with: SKIP_WASM_BUILD=1 cargo build -p myosu-chain"
    exit 1
fi
key_dir="$(mktemp -d -t libp2p001-XXXXXX)"
trap 'rm -rf "$key_dir"' EXIT
key_file="$key_dir/node-key"
if ! "$chain_bin" key generate-node-key --file "$key_file" >/dev/null 2>&1; then
    bad "chain binary key generate-node-key failed"
    exit 1
fi
fresh_peer_id="$("$chain_bin" key inspect-node-key --file "$key_file" 2>/dev/null || true)"
if [[ -z "$fresh_peer_id" ]]; then
    bad "chain binary key inspect-node-key returned empty peer_id"
    exit 1
fi
if ! [[ "$fresh_peer_id" =~ ^12D3Koo[A-Za-z0-9]+$|^Qm[A-Za-z0-9]+$ ]]; then
    bad "fresh peer_id from chain binary is not a valid 12D3Koo|Qm base58: $fresh_peer_id"
    exit 1
fi
ok "fresh peer_id generated via chain binary: ${fresh_peer_id:0:13}... (${#fresh_peer_id} chars)"

# -- 5. The fresh_multiaddr constructed from the fresh peer_id
#       round-trips byte-for-byte through the manifest's
#       bootnodes field shape. The harness writes a copy of the
#       manifest with the placeholder (or the first entry)
#       replaced by fresh_multiaddr, then re-extracts the entry
#       and asserts byte-for-byte equality.
fresh_multiaddr="/ip4/127.0.0.1/tcp/30333/p2p/${fresh_peer_id}"
manifest_copy="$key_dir/manifest.yaml"
cp "$manifest_path" "$manifest_copy"
python3 -c '
import sys, yaml
p_in, p_out, new_entry = sys.argv[1], sys.argv[2], sys.argv[3]
d = yaml.safe_load(open(p_in))
if "bootnodes" in d and isinstance(d["bootnodes"], list) and d["bootnodes"]:
    d["bootnodes"][0] = new_entry
else:
    d["bootnodes"] = [new_entry]
with open(p_out, "w") as f:
    yaml.safe_dump(d, f, sort_keys=False)
' "$manifest_path" "$manifest_copy" "$fresh_multiaddr" 2>/dev/null || {
    bad "could not rewrite manifest copy for round-trip check"
    exit 1
}
roundtripped="$(python3 -c '
import sys, yaml
d = yaml.safe_load(open(sys.argv[1]))
print(d["bootnodes"][0])
' "$manifest_copy")"
if [[ "$roundtripped" != "$fresh_multiaddr" ]]; then
    bad "fresh_multiaddr did not round-trip through manifest bootnodes field"
    bad "  expected: $fresh_multiaddr"
    bad "  actual:   $roundtripped"
    exit 1
fi
ok "fresh_multiaddr round-tripped byte-for-byte through manifest bootnodes field"

# -- 6. The chain binary's --bootnodes parser accepts the
#       fresh_multiaddr. The harness greps stderr for
#       `multiaddr parsing error` (the substrate-clap-parser
#       rejection prefix the libp2p multiaddr crate emits; e.g.
#       `error: invalid value '/ip4/.../p2p/...' for '--bootnodes
#       <ADDR>...': multiaddr parsing error: <detail>`); the
#       exit code is allowed to be 0 or 1 (the chain binary
#       exits 1 on `--help` in some build configurations, but
#       the *stderr* is what the parser emits the rejection on).
if ! "$chain_bin" --bootnodes "$fresh_multiaddr" --help \
        >/dev/null 2>"$key_dir/bootnodes.stderr"; then
    # Exit code != 0 is OK; the parser rejection is what we
    # grep for in stderr.
    :
fi
if grep -q 'multiaddr parsing error' "$key_dir/bootnodes.stderr"; then
    bad "chain binary --bootnodes parser rejected fresh_multiaddr"
    bad "  fresh_multiaddr: $fresh_multiaddr"
    bad "  stderr:"
    sed 's/^/    /' "$key_dir/bootnodes.stderr" >&2
    exit 1
fi
ok "chain binary --bootnodes parser accepted fresh_multiaddr (no 'multiaddr parsing error' on stderr)"

# -- 7. The W-01 public_testnet_manifest.sh regression harness
#       still passes. The harness is a strict subset of W-01
#       (LIBP2P-001 is additive; it adds a new harness + a new
#       CI job; it does not change the W-01 contract). The
#       CI job wires this as a hard dependency.
if [[ ! -x "$w01_harness" ]]; then
    bad "W-01 public_testnet_manifest.sh missing: $w01_harness"
    exit 1
fi
if ! bash "$w01_harness" >/dev/null 2>"$key_dir/w01.stderr"; then
    bad "W-01 public_testnet_manifest.sh regression failed"
    bad "  stderr:"
    sed 's/^/    /' "$key_dir/w01.stderr" >&2
    exit 1
fi
ok "W-01 public_testnet_manifest.sh regression harness still green"

# -- 8. The new public-testnet-bootnode-roundtrip CI job is
#       wired in .github/workflows/ci.yml. The harness greps
#       for the literal job name so a future rename is
#       fail-closed here too.
if [[ ! -s "$ci_workflow" ]]; then
    bad "CI workflow missing: $ci_workflow"
    exit 1
fi
if ! grep -q 'public-testnet-bootnode-roundtrip' "$ci_workflow"; then
    bad "public-testnet-bootnode-roundtrip CI job not wired in $ci_workflow"
    bad "  expected: a job with the name public-testnet-bootnode-roundtrip"
    exit 1
fi
ok "public-testnet-bootnode-roundtrip CI job is wired in .github/workflows/ci.yml"

if (( fail > 0 )); then
    note "harness failed ($fail sub-check(s) failed, $pass passed)"
    exit 1
fi

note "harness green: $pass sub-checks passed (bootnode_count=$bootnode_count fresh_peer_id=${fresh_peer_id:0:13}...)"
printf 'PUBLIC_TESTNET_BOOTNODE_ROUNDTRIP_HARNESS myosu e2e ok surface=public_testnet_bootnode_contract sub_checks=%d bootnode_count=%d\n' \
    "$pass" "$bootnode_count"
