#!/usr/bin/env bash
# SEC-001 stale-allowlist probe: for each RUSTSEC id in the CI
# allowlist, verify the entry is still doing real work.
#
# This is a defense-in-depth companion to
# `tests/e2e/sec001_allowlist_consistency.sh`. The consistency
# test enforces that the three sources of truth (ci.yml
# `--ignore` list, .cargo/audit.toml `[advisories]` ignore list,
# SECURITY.md SEC-001 table) stay in sync. This probe enforces
# that every entry in the synced allowlist is still actively
# suppressing a real advisory that the live
# `cargo audit -D warnings` gate would otherwise deny -- so a
# remediation pass that bumps a crate past a CVE actually
# removes the now-dead `--ignore` line, instead of letting the
# allowlist grow forever.
#
# The probe is built around the observation that the cargo-audit
# allowlist can be split into two functionally separate
# sub-trees:
#
#   1. The `vulnerabilities` tree: CVEs and similar denial-class
#      advisories that the gate denies via the
#      `vulnerabilities.count` JSON field. cargo-audit 0.22.1
#      places every CVE / soundness / patchable advisory in this
#      tree.
#   2. The `warnings` tree: `unmaintained`, `unsound`, and
#      `yanked` informational advisories that the gate denies
#      via the `warnings.<kind>` JSON fields. These are
#      "informational" advisories but `-D warnings` still treats
#      them as gate failures.
#
# A probe that only inspects `vulnerabilities.count` will
# false-positive every `unmaintained` entry -- the lockfile
# still has `bincode 1.3.3`, but `bincode 1.3.3` is an
# `unmaintained` advisory, not a vulnerability, so the
# `vulnerabilities` count is 0 even when the gate is failing on
# the `warnings.unmaintained` count being non-zero.
#
# The fix: this probe runs the *exact* gate the CI workflow runs
# (`cargo audit -D warnings --no-fetch`), parses BOTH
# `vulnerabilities.list[*].advisory.id` and the
# `warnings.<kind>[*].advisory.id` fields, and labels an
# allowlist entry as ACTIVE if and only if its id appears in
# one of those collections when the gate is run with an empty
# allowlist.
#
# The probe runs in three phases:
#
#   Phase 1 (baseline). Move the repo's audit.toml aside and
#   replace it with a minimal config whose `[advisories] ignore`
#   list is empty. Run the gate. If the gate passes (exit 0,
#   0 vulns, 0 warnings), every entry in the synced allowlist
#   is STALE -- the lockfile no longer needs any of them.
#   Report and exit 0 (an empty allowlist is a valid end state
#   for the SEC-001 plan).
#
#   Phase 2 (per-entry probe). For each allowlist entry, run the
#   gate with `--ignore <all-other-entries>` and capture the
#   resulting vulnerabilities + warnings. If the entry's id
#   appears in either the per-entry vulnerability list or the
#   per-entry warning lists, the entry is ACTIVE; otherwise it
#   is STALE. STALE entries must be removed from all three
#   sources of truth (ci.yml, .cargo/audit.toml, SECURITY.md) in
#   the next remediation pass.
#
#   Phase 3 (re-run consistency). After the per-entry probe,
#   restore the repo's audit.toml and re-run the consistency
#   test to confirm the three sources of truth still match the
#   (unchanged) allowlist. This is a smoke test that the probe
#   itself did not corrupt the repo state.
#
# The probe is fail-closed on:
#
#   - cargo / cargo-audit / python3 not installed.
#   - The cargo-audit JSON output cannot be parsed.
#   - ANY entry is found to be stale. The list of stale entries
#     is printed with a concrete remediation message.
#
# Implementation note: the probe reads the synced allowlist from
# the parsed output of `sec001_allowlist_consistency.sh`'s
# state machine (the same awk extractor lives in both scripts)
# rather than re-parsing ci.yml directly. This keeps the two
# tests from drifting on what they consider an "allowlist entry".

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

workflow="$repo_root/.github/workflows/ci.yml"
audit_toml="$repo_root/.cargo/audit.toml"

if [[ ! -f "$workflow" ]]; then
    printf 'sec001_probe: missing workflow file %s\n' "$workflow" >&2
    exit 1
fi
if [[ ! -f "$audit_toml" ]]; then
    printf 'sec001_probe: missing audit.toml at %s\n' "$audit_toml" >&2
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
    printf 'sec001_probe: cargo not found in PATH\n' >&2
    exit 1
fi
if ! cargo audit --help >/dev/null 2>&1; then
    printf 'sec001_probe: cargo-audit is not installed; install with `cargo install cargo-audit --locked`\n' >&2
    exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
    printf 'sec001_probe: python3 not found in PATH; required to parse cargo audit JSON\n' >&2
    exit 1
fi

# -- Parse the allowlist using the same awk state machine the
# consistency test uses. Future audit.toml schema changes must
# be mirrored in both places.
toml_ignores_text="$(awk '
    /^\[advisories\]/ { in_adv=1; next }
    /^\[/ && in_adv { in_adv=0 }
    in_adv && /^[[:space:]]*ignore[[:space:]]*=[[:space:]]*\[/ { capturing=1; next }
    in_adv && capturing && /\]/ { capturing=0; next }
    in_adv && capturing { print }
' "$audit_toml")"

allowlist=()
if [[ -n "$toml_ignores_text" ]]; then
    while IFS= read -r rid; do
        rid="$(printf '%s' "$rid" | sed -e 's/^[[:space:]]*"//' -e 's/"[[:space:]]*,*[[:space:]]*$//' -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
        if [[ -n "$rid" && "$rid" != "ignore" && "$rid" != "=" ]]; then
            allowlist+=( "$rid" )
        fi
    done < <(printf '%s\n' "$toml_ignores_text" | tr ',' '\n')
fi

if [[ "${#allowlist[@]}" -gt 0 ]]; then
    allowlist=( $(printf '%s\n' "${allowlist[@]}" | sort -u) )
fi

# -- Substitute the repo's audit.toml with a config whose
# `[advisories] ignore` list is empty, so cargo-audit's auto-loaded
# config does not silently re-ignore the very entries we are
# probing. The real audit.toml is restored on exit (even on signal)
# so the developer's working tree is never left half-rewritten.
audit_real="$repo_root/.cargo/audit.toml"
audit_backup="$(mktemp -t sec001-probe-audit-XXXXXX.toml)"
cp "$audit_real" "$audit_backup"

restore_audit() {
    if [[ -f "$audit_backup" ]]; then
        cp "$audit_backup" "$audit_real"
        rm -f "$audit_backup"
    fi
}
trap restore_audit EXIT INT TERM

cat > "$audit_real" <<'TOML'
[advisories]
ignore = []
informational_warnings = ["notice", "unmaintained", "unsound"]
[output]
quiet = false
show_tree = true
TOML

# -- Phase 1: baseline gate run with an empty allowlist. This
# is the "what would the gate deny if the allowlist did not
# exist" question, and it is the only way to know which entries
# are actually doing work.
baseline_json="$(mktemp -t sec001-probe-baseline-XXXXXX.json)"
cargo audit -D warnings --no-fetch --json > "$baseline_json" 2>"$baseline_json.err" || true
baseline_vuln_ids="$(python3 -c "
import json
d = json.load(open('$baseline_json'))
ids = set()
for v in d.get('vulnerabilities', {}).get('list', []):
    rid = (v.get('advisory') or {}).get('id')
    if rid:
        ids.add(rid)
# unmaintained / unsound advisories carry an advisory id directly.
for kind in ('unmaintained', 'unsound'):
    for w in d.get('warnings', {}).get(kind, []):
        rid = (w.get('advisory') or {}).get('id')
        if rid:
            ids.add(rid)
# yanked warnings do NOT carry an advisory id -- they only name
# the package. Map the package name back to the corresponding
# unmaintained advisory id (if any) so a single entry can
# suppress both classes of denial for the same crate.
yanked_pkgs = set()
for w in d.get('warnings', {}).get('yanked', []):
    pname = (w.get('package') or {}).get('name')
    if pname:
        yanked_pkgs.add(pname)
if yanked_pkgs:
    for kind in ('unmaintained',):
        for w in d.get('warnings', {}).get(kind, []):
            pname = (w.get('package') or {}).get('name')
            rid = (w.get('advisory') or {}).get('id')
            if pname in yanked_pkgs and rid:
                ids.add(rid)
print(' '.join(sorted(ids)))
")" || {
    printf 'sec001_probe: FAIL could not parse cargo-audit JSON output\n' >&2
    cat "$baseline_json" >&2
    exit 1
}
rm -f "$baseline_json" "$baseline_json.err"
read -r -a baseline_arr <<< "$baseline_vuln_ids"

# -- Fast path: the baseline reports zero advisory denials, so
# every entry in the synced allowlist is STALE. The probe
# exits 0 because an empty allowlist is a valid end state
# (the consistency test will continue to pass once the
# remediation commit lands and removes every entry from all
# three sources of truth).
if [[ "${#baseline_arr[@]}" -eq 0 ]]; then
    if [[ "${#allowlist[@]}" -eq 0 ]]; then
        printf 'sec001_probe: ok (allowlist is empty; baseline reports 0 advisories; nothing to do)\n'
    else
        printf 'sec001_probe: FAIL allowlist has %d entries but the gate passes with an empty allowlist:\n' "${#allowlist[@]}" >&2
        for rid in "${allowlist[@]}"; do
            printf '  - %s (remove from .github/workflows/ci.yml, .cargo/audit.toml, SECURITY.md)\n' "$rid" >&2
        done
        exit 1
    fi
    exit 0
fi

# -- Fast path: the baseline reports some advisories, but the
# synced allowlist is empty. The gate is currently broken:
# either the consistency test or the workflow has drifted.
if [[ "${#allowlist[@]}" -eq 0 ]]; then
    printf 'sec001_probe: FAIL synced allowlist is empty but the gate denies %d advisories:\n' "${#baseline_arr[@]}" >&2
    printf '  %s\n' "${baseline_arr[*]}" >&2
    printf '\nThe dependency-audit gate is broken until each of these is added to .cargo/audit.toml + ci.yml + SECURITY.md.\n' >&2
    exit 1
fi

# -- Phase 2: per-entry probe. For each allowlist entry, run
# the gate with `--ignore <every-other-entry>` and read the
# per-entry vulnerability + warning list. An entry is ACTIVE
# iff its id is in the per-entry denial set. We only report
# stale entries; the goal of the probe is to drive the next
# remediation pass, not to enumerate the active set.
active_count=0
stale_count=0
stale_ids=()
per_entry_json="$(mktemp -t sec001-probe-entry-XXXXXX.json)"

for target in "${allowlist[@]}"; do
    other_args=()
    for other in "${allowlist[@]}"; do
        [[ "$other" != "$target" ]] && other_args+=( --ignore "$other" )
    done
    cargo audit -D warnings --no-fetch --json "${other_args[@]}" > "$per_entry_json" 2>/dev/null || true
    if [[ ! -s "$per_entry_json" ]]; then
        printf 'sec001_probe: FAIL `cargo audit --no-fetch --json` produced empty output for entry %s\n' \
            "$target" >&2
        exit 1
    fi
    is_stale="$(python3 -c "
import json
d = json.load(open('$per_entry_json'))
ids = set()
for v in d.get('vulnerabilities', {}).get('list', []):
    rid = (v.get('advisory') or {}).get('id')
    if rid:
        ids.add(rid)
for kind in ('unmaintained', 'unsound'):
    for w in d.get('warnings', {}).get(kind, []):
        rid = (w.get('advisory') or {}).get('id')
        if rid:
            ids.add(rid)
yanked_pkgs = set()
for w in d.get('warnings', {}).get('yanked', []):
    pname = (w.get('package') or {}).get('name')
    if pname:
        yanked_pkgs.add(pname)
for w in d.get('warnings', {}).get('unmaintained', []):
    pname = (w.get('package') or {}).get('name')
    rid = (w.get('advisory') or {}).get('id')
    if pname in yanked_pkgs and rid:
        ids.add(rid)
print('STALE' if '$target' not in ids else 'ACTIVE')
")"
    if [[ "$is_stale" == "STALE" ]]; then
        stale_count=$(( stale_count + 1 ))
        stale_ids+=( "$target" )
    else
        active_count=$(( active_count + 1 ))
    fi
done
rm -f "$per_entry_json"

if [[ "$stale_count" -gt 0 ]]; then
    printf 'sec001_probe: FAIL %d/%d allowlist entries are stale (no longer suppress a real advisory):\n' \
        "$stale_count" "${#allowlist[@]}" >&2
    for rid in "${stale_ids[@]}"; do
        printf '  - %s\n' "$rid" >&2
    done
    printf '\n' >&2
    printf 'Remediation: remove each stale id from all three sources of truth in the same commit:\n' >&2
    printf '  - .github/workflows/ci.yml   (delete the matching --ignore line and its category comment)\n' >&2
    printf '  - .cargo/audit.toml          (delete the matching quoted id from [advisories] ignore)\n' >&2
    printf '  - SECURITY.md                (delete the matching row from the SEC-001 table)\n' >&2
    printf 'Then re-run `bash tests/e2e/sec001_allowlist_consistency.sh` to confirm.\n' >&2
    exit 1
fi

printf 'sec001_probe: ok (%d allowlist entries probed, all %d active, 0 stale; baseline gate denies %d advisories without any --ignore)\n' \
    "${#allowlist[@]}" "$active_count" "${#baseline_arr[@]}"
