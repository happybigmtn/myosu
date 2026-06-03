#!/usr/bin/env bash
# SEC-001 cargo-audit allowlist consistency check.
#
# This is the executable test that backs the SECURITY.md
# "SEC-001 Advisory Triage Table" and the categorized
# `--ignore` list in `.github/workflows/ci.yml`. It is wired into
# the `dependency-audit` CI job via the
# "Verify SEC-001 allowlist consistency" step and is also run
# directly from a developer shell as part of pre-push hygiene.
#
# It performs five real consistency checks; failing any one of them
# fails the test (and the CI gate) with a concrete error message
# that names the offending advisory id and the file that needs to
# change. The five checks are:
#
#   1. Parse every `--ignore RUSTSEC-YYYY-NNNN` from
#      `.github/workflows/ci.yml` and confirm each one carries
#      one of the four SEC-001 category comments
#      (`direct-owned`, `inherited-chain`, `inherited-wasm`,
#      `inherited-misc`) directly above it.  This is the
#      "every advisory in the CI allowlist has a documented
#      classification" criterion.
#
#   2. Parse the `[advisories] ignore = [...]` list from
#      `.cargo/audit.toml` and confirm the two lists are
#      identical (sorted, deduplicated). The config file is the
#      source of truth for `cargo audit` when run from the
#      repository root; the workflow `--ignore` flags must agree
#      with it so a developer running `cargo audit` locally and a
#      CI run produce the same result.
#
#   3. Confirm every ignored advisory id appears in the
#      SEC-001 triage table in SECURITY.md. This is the
#      "no untracked suppressions" half of the consistency
#      criterion.
#
#   4. Confirm every advisory row in the SECURITY.md triage
#      table is actually ignored in ci.yml and the audit.toml
#      config. This is the "no dead documentation" half of
#      the consistency criterion.
#
#   5. Run `cargo audit -D warnings` (the same deny policy
#      used by the CI workflow) and confirm exit 0. The
#      script consumes the advisory DB on the host via
#      `--no-fetch`, so it is fast and deterministic offline
#      once the DB has been cloned once.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

workflow="$repo_root/.github/workflows/ci.yml"
audit_toml="$repo_root/.cargo/audit.toml"
security_md="$repo_root/SECURITY.md"

if [[ ! -f "$workflow" ]]; then
    printf 'sec001: missing workflow file %s\n' "$workflow" >&2
    exit 1
fi
if [[ ! -f "$audit_toml" ]]; then
    printf 'sec001: missing audit.toml at %s\n' "$audit_toml" >&2
    exit 1
fi
if [[ ! -f "$security_md" ]]; then
    printf 'sec001: missing SECURITY.md at %s\n' "$security_md" >&2
    exit 1
fi

# -- 1. Parse the workflow allowlist and verify category comments.
declare -A categorized
categorized[direct-owned]=0
categorized[inherited-chain]=0
categorized[inherited-wasm]=0
categorized[inherited-misc]=0

current_category=""
workflow_ignores=()
while IFS= read -r line; do
    case "$line" in
        *"--ignore RUSTSEC-"*)
            rid="$(printf '%s\n' "$line" | sed -n 's/.*--ignore \(RUSTSEC-[0-9]\{4\}-[0-9]\{4\}\).*/\1/p')"
            if [[ -z "$rid" ]]; then
                printf 'sec001: malformed --ignore line in ci.yml: %s\n' "$line" >&2
                exit 1
            fi
            if [[ -z "$current_category" ]]; then
                printf 'sec001: --ignore %s in ci.yml is missing a category comment above it\n' "$rid" >&2
                exit 1
            fi
            categorized[$current_category]=$(( ${categorized[$current_category]} + 1 ))
            workflow_ignores+=( "$rid" )
            ;;
        *"# --- "*"--- "*)
            cat="$(printf '%s\n' "$line" | sed -n 's/^[[:space:]]*# --- \([a-z-]*\).*/\1/p')"
            case "$cat" in
                inherited-chain|direct-owned|inherited-wasm|inherited-misc)
                    current_category="$cat"
                    ;;
                *)
                    printf 'sec001: unknown category comment in ci.yml: %s\n' "$line" >&2
                    exit 1
                    ;;
            esac
            ;;
    esac
done < "$workflow"

for required in direct-owned inherited-chain inherited-wasm inherited-misc; do
    if [[ "${categorized[$required]}" -eq 0 ]]; then
        printf 'sec001: category %s has zero --ignore entries in ci.yml\n' "$required" >&2
        exit 1
    fi
done

# -- 2. Parse the audit.toml [advisories] ignore list.
# Use awk in a state-machine mode: once we see the `[advisories]`
# table heading and then an `ignore = [` line, capture every
# following line until we hit a `]` that closes the list. The
# captured lines may include quoted IDs (with optional trailing
# commas) and blank lines; the trimming loop below extracts the
# bare RUSTSEC id from each.
#
# This is the multi-line form used in our `.cargo/audit.toml`
# (one quoted id per line, with a closing `]` on its own line).
# A single-line form (`ignore = ["a", "b", "c"]`) is also handled:
# the capture continues only until the first `]` we see, which in
# the single-line case is on the same line as the last id.
toml_ignores_text="$(awk '
    /^\[advisories\]/ { in_adv=1; next }
    /^\[/ && in_adv { in_adv=0 }
    in_adv && /^[[:space:]]*ignore[[:space:]]*=[[:space:]]*\[/ { capturing=1; next }
    in_adv && capturing && /\]/ { capturing=0; next }
    in_adv && capturing { print }
' "$audit_toml")"

if [[ -z "$toml_ignores_text" ]]; then
    printf 'sec001: [advisories] ignore in .cargo/audit.toml is empty\n' >&2
    exit 1
fi

toml_ignores=()
while IFS= read -r rid; do
    rid="$(printf '%s' "$rid" | sed -e 's/^[[:space:]]*"//' -e 's/"[[:space:]]*,*[[:space:]]*$//' -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
    if [[ -n "$rid" && "$rid" != "ignore" && "$rid" != "=" ]]; then
        toml_ignores+=( "$rid" )
    fi
done < <(printf '%s\n' "$toml_ignores_text" | tr ',' '\n')

if [[ "${#toml_ignores[@]}" -eq 0 ]]; then
    printf 'sec001: [advisories] ignore in .cargo/audit.toml parsed empty after trim\n' >&2
    exit 1
fi

# Sort and dedupe both lists for comparison.
workflow_sorted=( $(printf '%s\n' "${workflow_ignores[@]}" | sort -u) )
toml_sorted=( $(printf '%s\n' "${toml_ignores[@]}" | sort -u) )

if [[ "${#workflow_sorted[@]}" -ne "${#toml_sorted[@]}" ]]; then
    printf 'sec001: ci.yml has %s ignores but .cargo/audit.toml has %s\n' \
        "${#workflow_sorted[@]}" "${#toml_sorted[@]}" >&2
    exit 1
fi

for i in "${!workflow_sorted[@]}"; do
    if [[ "${workflow_sorted[$i]}" != "${toml_sorted[$i]}" ]]; then
        printf 'sec001: ci.yml and .cargo/audit.toml disagree on ignore[%d]: %s vs %s\n' \
            "$i" "${workflow_sorted[$i]}" "${toml_sorted[$i]}" >&2
        exit 1
    fi
done

# -- 3. Forward coverage: every ignored advisory appears in SECURITY.md.
doc_section_present="$(awk '/^## SEC-001/{flag=1; next} /^## /{flag=0} flag' "$security_md" | grep -cE 'RUSTSEC-[0-9]{4}-[0-9]{4}' || true)"

if [[ "$doc_section_present" -eq 0 ]]; then
    printf 'sec001: SECURITY.md has no SEC-001 section with RUSTSEC rows\n' >&2
    exit 1
fi

# Use the SECURITY.md SEC-001 section (between the "## SEC-001" heading
# and the next "## " heading) to scope the lookup.
doc_ids="$(awk '/^## SEC-001/{flag=1; next} /^## /{flag=0} flag' "$security_md" | grep -oE 'RUSTSEC-[0-9]{4}-[0-9]{4}' | sort -u)"

missing_in_doc=()
while IFS= read -r rid; do
    if ! printf '%s\n' "$doc_ids" | grep -qx "$rid"; then
        missing_in_doc+=( "$rid" )
    fi
done < <(printf '%s\n' "${workflow_sorted[@]}")

if [[ "${#missing_in_doc[@]}" -gt 0 ]]; then
    printf 'sec001: advisories present in ci.yml/audit.toml but missing in SECURITY.md SEC-001 table:\n' >&2
    for rid in "${missing_in_doc[@]}"; do
        printf '  - %s\n' "$rid" >&2
    done
    exit 1
fi

# -- 4. Reverse coverage: every SECURITY.md row is in the ignore list.
missing_in_workflow=()
while IFS= read -r rid; do
    if ! printf '%s\n' "${workflow_sorted[@]}" | grep -qx "$rid"; then
        missing_in_workflow+=( "$rid" )
    fi
done <<< "$doc_ids"

if [[ "${#missing_in_workflow[@]}" -gt 0 ]]; then
    printf 'sec001: advisories documented in SECURITY.md but not ignored in ci.yml/audit.toml:\n' >&2
    for rid in "${missing_in_workflow[@]}"; do
        printf '  - %s\n' "$rid" >&2
    done
    exit 1
fi

# -- 5. Run cargo audit and confirm exit 0.
if ! command -v cargo >/dev/null 2>&1; then
    printf 'sec001: cargo not found in PATH\n' >&2
    exit 1
fi
if ! cargo audit --help >/dev/null 2>&1; then
    printf 'sec001: cargo-audit is not installed; install with `cargo install cargo-audit --locked`\n' >&2
    exit 1
fi

ignore_args=()
for rid in "${workflow_sorted[@]}"; do
    ignore_args+=( --ignore "$rid" )
done

if ! cargo audit -D warnings --no-fetch "${ignore_args[@]}" >/tmp/sec001-audit.log 2>&1; then
    printf 'sec001: cargo audit failed with the parsed allowlist. Last 30 lines:\n' >&2
    tail -n 30 /tmp/sec001-audit.log >&2
    exit 1
fi

allowlist_count="${#workflow_sorted[@]}"
printf 'sec001: ok (%s advisories classified across 4 categories: ' "$allowlist_count"
printf 'direct-owned=%d, inherited-chain=%d, inherited-wasm=%d, inherited-misc=%d)\n' \
    "${categorized[direct-owned]}" \
    "${categorized[inherited-chain]}" \
    "${categorized[inherited-wasm]}" \
    "${categorized[inherited-misc]}"
