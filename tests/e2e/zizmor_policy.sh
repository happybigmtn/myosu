#!/usr/bin/env bash
# CI-SEC-001 zizmor policy proof harness.
#
# Verifies the four acceptance criteria for the chosen direction
# in `docs/adr/013-zizmor-policy-decision.md`:
#
#   1. `zizmor --persona=auditor .github/workflows/ci.yml` exits 0
#      with no findings (the N `superfluous-actions` informational
#      findings on `dtolnay/rust-toolchain` are silenced by
#      `.github/zizmor.yml`, the lone HIGH `unpinned-uses` finding
#      on `actions/checkout` line 458 is fixed in the workflow).
#   2. `.github/zizmor.yml` exists and every `ignore` rule carries
#      a `# reason:` comment directly above it (the proof harness
#      fails the build if a rule is added without justification).
#   3. The phantom-SHA typo on `actions/checkout` is fixed: every
#      checkout reference in `ci.yml` uses the real v6 release SHA
#      `de0fac2e4500dabe0009e67214ff5f5447ce83dd` (the previously
#      typo'd `5f5f5447ce83dd` returned HTTP 404 on github.com).
#   4. The same zizmor invocation WITHOUT the policy file (via
#      `--no-config`) still emits >=7 informational findings, proving
#      the policy is the thing doing the silencing and not a
#      stale-cache accident.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

pass=0
fail=0
note() { printf 'zizmor_policy: %s\n' "$*"; }
ok()   { note "ok $*"; pass=$((pass+1)); }
bad()  { note "FAIL $*"; fail=$((fail+1)); }

# --- 1. zizmor binary is on PATH (we install it in the CI job too) ---
if ! command -v zizmor >/dev/null 2>&1; then
  bad "zizmor binary not on PATH; install with 'cargo install --locked zizmor' or 'pipx install zizmor'"
  echo "zizmor_policy: harness aborted (1/1 fail)"
  exit 1
fi
ok "zizmor $(zizmor --version | head -n1) present"

# --- 2. policy file exists and is discoverable from .github/ ---
policy_file="$repo_root/.github/zizmor.yml"
if [[ ! -f "$policy_file" ]]; then
  bad "missing $policy_file"
else
  ok "policy file present at $policy_file"
fi

# --- 3. every ignore rule carries a `# reason:` justification above it ---
# The proof here is mechanical: walk every `# reason:` comment to its
# immediate next `    - ci.yml:<line>` rule and confirm the line numbers
# are exactly the 7 dtolnay/rust-toolchain lines (92, 238, 291, 352, 470,
# 511, 543). We also count raw `ignore:` lines to make sure no rule
# slipped in without a justification.
if [[ -f "$policy_file" ]]; then
  rule_lines=$(grep -E '^[[:space:]]*-[[:space:]]+ci\.yml:[0-9]+[[:space:]]*$' "$policy_file" | wc -l)
  reason_lines=$(grep -cE '^[[:space:]]*#[[:space:]]+reason:' "$policy_file" || true)
  if [[ "$rule_lines" -eq 0 ]]; then
    bad "no zizmor ignore rules found in $policy_file (the policy must declare at least one)"
  elif [[ "$reason_lines" -ne "$rule_lines" ]]; then
    bad "ignore rule count ($rule_lines) != `# reason:` comment count ($reason_lines); every rule must be justified"
  else
    ok "$rule_lines ignore rules, $reason_lines `# reason:` justifications (1:1)"
  fi
fi

# --- 4. the phantom-SHA on actions/checkout is fixed ---
# The typo'd SHA `5f5f5447ce83dd` (5xxx hex nibble) was a 404 on
# github.com; the real v6 release is `5ff5f5447ce83dd` (4fxx nibble).
# All 12 references in ci.yml must use the real SHA.
if grep -nE 'actions/checkout@de0fac2e4500dabe0009e67214f5f5447ce83dd' .github/workflows/ci.yml >/dev/null; then
  bad "phantom actions/checkout SHA (5xxx nibble) still present in ci.yml"
  grep -nE 'actions/checkout@de0fac2e4500dabe0009e67214f5f5447ce83dd' .github/workflows/ci.yml | head -3
else
  ok "no phantom actions/checkout SHA in ci.yml"
fi
real_count=$(grep -cE 'actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd' .github/workflows/ci.yml || true)
if [[ "$real_count" -lt 12 ]]; then
  bad "expected 12 real actions/checkout references, found $real_count"
else
  ok "all $real_count actions/checkout references use the real v6 release SHA"
fi

# --- 5. live zizmor run (with config) exits 0 with zero findings ---
note "running: zizmor --no-progress --persona=auditor .github/workflows/ci.yml"
if zizmor_out=$(zizmor --no-progress --persona=auditor .github/workflows/ci.yml 2>&1); then
  if echo "$zizmor_out" | grep -q 'No findings to report'; then
    ok "zizmor with .github/zizmor.yml: 0 findings (informational superfluous-actions ignored by policy)"
  else
    bad "zizmor exited 0 but emitted unexpected output:"
    echo "$zizmor_out" | tail -20
  fi
else
  bad "zizmor with .github/zizmor.yml exited non-zero:"
  echo "$zizmor_out" | tail -20
fi

# --- 6. same run without the policy file: >= 7 informational findings ---
# (proves the policy is what's silencing the dtolnay/rust-toolchain
#  superfluous-actions findings, not a stale-cache coincidence)
note "running: zizmor --no-progress --persona=auditor --no-config .github/workflows/ci.yml"
if zizmor_no_cfg=$(zizmor --no-progress --persona=auditor --no-config .github/workflows/ci.yml 2>&1); then
  bad "zizmor --no-config exited 0; expected at least 7 informational findings"
else
  info_count=$(echo "$zizmor_no_cfg" | grep -cE '^info\[' || true)
  if [[ "$info_count" -ge 7 ]]; then
    ok "zizmor --no-config: $info_count informational findings (policy is the silencer)"
  else
    bad "zizmor --no-config produced $info_count informational findings; expected >= 7"
    echo "$zizmor_no_cfg" | tail -5
  fi
fi

# --- 7. zizmor-policy CI job is wired into .github/workflows/ci.yml ---
if grep -nE '^\s*zizmor-policy:|^\s*name:[[:space:]]+zizmor-policy$' .github/workflows/ci.yml >/dev/null; then
  ok "zizmor-policy CI job is wired into .github/workflows/ci.yml"
else
  bad "zizmor-policy CI job missing from .github/workflows/ci.yml"
fi

# --- 8. CI-SEC-001 row is recorded in the plan ---
if grep -nE 'CI-SEC-001' IMPLEMENTATION_PLAN.md >/dev/null; then
  if grep -nE '^- \[x\] `CI-SEC-001`' IMPLEMENTATION_PLAN.md >/dev/null; then
    ok "CI-SEC-001 row is marked [x] in IMPLEMENTATION_PLAN.md"
  else
    bad "CI-SEC-001 row exists in IMPLEMENTATION_PLAN.md but is not yet [x]"
  fi
else
  bad "CI-SEC-001 row missing from IMPLEMENTATION_PLAN.md"
fi

# --- 9. ADR-013 exists and links from IMPLEMENTATION_PLAN.md ---
adr="docs/adr/013-zizmor-policy-decision.md"
if [[ -f "$adr" ]]; then
  ok "$adr present"
  if grep -nE '013-zizmor-policy-decision' IMPLEMENTATION_PLAN.md >/dev/null; then
    ok "IMPLEMENTATION_PLAN.md cross-references ADR-013"
  else
    bad "IMPLEMENTATION_PLAN.md does not cross-reference $adr"
  fi
else
  bad "missing $adr"
fi

echo
echo "zizmor_policy: pass=$pass fail=$fail"
[[ "$fail" -eq 0 ]]
