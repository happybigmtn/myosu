#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage: ops/release.sh [--dry-run] vX.Y.Z

Prepare a Myosu operator-facing release from the current `trunk` checkout.

The script:
  1. validates the requested release tag
  2. updates the workspace version in the root Cargo.toml when needed
  3. derives release notes from CHANGELOG.md
  4. builds a versioned operator bundle
  5. creates an annotated git tag (unless --dry-run)

Environment:
  MYOSU_RELEASE_ROOT          Output root (default: target/releases)
  MYOSU_RELEASE_PASSWORD_ENV  Password env passed into the operator bundle
                              flow (default: MYOSU_KEY_PASSWORD)
EOF
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "${script_dir}/.." && pwd -P)"
cd "$repo_root"

dry_run=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      dry_run=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --*)
      echo "unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
    *)
      break
      ;;
  esac
done

if [[ $# -ne 1 ]]; then
  usage >&2
  exit 1
fi

release_tag="$1"
if [[ ! "$release_tag" =~ ^v([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
  echo "release tag must match vX.Y.Z" >&2
  exit 1
fi

release_version="${release_tag#v}"
release_root="${MYOSU_RELEASE_ROOT:-${repo_root}/target/releases}"
release_dir="${release_root}/${release_tag}"
bundle_dir="${release_dir}/operator-bundle"
config_dir="${release_dir}/config"
notes_file="${release_dir}/release-notes.md"
changelog_path="${repo_root}/CHANGELOG.md"
cargo_toml_path="${repo_root}/Cargo.toml"
password_env_name="${MYOSU_RELEASE_PASSWORD_ENV:-MYOSU_KEY_PASSWORD}"

workspace_version() {
  python - "$cargo_toml_path" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
section = None
for line in path.read_text(encoding="utf-8").splitlines():
    stripped = line.strip()
    if stripped.startswith("[") and stripped.endswith("]"):
        section = stripped[1:-1]
        continue
    if section == "workspace.package":
        match = re.match(r'version\s*=\s*"([^"]+)"', stripped)
        if match:
            print(match.group(1))
            raise SystemExit(0)

raise SystemExit("root Cargo.toml is missing [workspace.package].version")
PY
}

set_workspace_version() {
  local new_version="$1"
  python - "$cargo_toml_path" "$new_version" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
new_version = sys.argv[2]
lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
section = None
updated = False

for index, line in enumerate(lines):
    stripped = line.strip()
    if stripped.startswith("[") and stripped.endswith("]"):
        section = stripped[1:-1]
        continue
    if section == "workspace.package" and re.match(r'version\s*=\s*"[^"]+"', stripped):
        lines[index] = re.sub(
            r'(")[^"]+(")',
            rf'\g<1>{new_version}\g<2>',
            line,
            count=1,
        )
        updated = True
        break

if not updated:
    raise SystemExit("root Cargo.toml is missing [workspace.package].version")

path.write_text("".join(lines), encoding="utf-8")
PY
}

extract_changelog_section() {
  local target_version="$1"
  local allow_unreleased="$2"
  python - "$changelog_path" "$target_version" "$allow_unreleased" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
target_version = sys.argv[2]
allow_unreleased = sys.argv[3] == "1"
lines = path.read_text(encoding="utf-8").splitlines()


def extract(name: str):
    header = re.compile(rf"^## \[{re.escape(name)}\](?:\s+-\s+.*)?$")
    start = None
    for idx, line in enumerate(lines):
        if header.match(line):
            start = idx
            break
    if start is None:
        return None
    end = len(lines)
    for idx in range(start + 1, len(lines)):
        if lines[idx].startswith("## "):
            end = idx
            break
    body = "\n".join(lines[start:end]).strip()
    return body


target = extract(target_version)
if target is not None:
    print(target)
    raise SystemExit(0)

if allow_unreleased:
    unreleased = extract("Unreleased")
    if unreleased is not None:
        print(unreleased)
        raise SystemExit(0)

raise SystemExit(
    f"CHANGELOG.md does not contain a [{target_version}] section"
    + (" or [Unreleased] fallback" if allow_unreleased else "")
)
PY
}

write_release_notes() {
  local source_section="$1"
  local release_commit="$2"
  mkdir -p "$release_dir"
  cat >"$notes_file" <<EOF
# Release ${release_tag}

- Release tag: \`${release_tag}\`
- Workspace version: \`${release_version}\`
- Commit: \`${release_commit}\`
- Bundle: \`${bundle_dir}\`

${source_section}
EOF
}

annotate_bundle_manifest() {
  local release_commit="$1"
  python - "$bundle_dir/bundle-manifest.toml" "$release_tag" "$release_version" "$release_commit" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
release_tag = sys.argv[2]
release_version = sys.argv[3]
release_commit = sys.argv[4]
lines = path.read_text(encoding="utf-8").splitlines(keepends=True)

filtered = [
    line
    for line in lines
    if not line.startswith("release_tag = ")
    and not line.startswith("workspace_version = ")
    and not line.startswith("release_commit = ")
]

insertion = [
    f'release_tag = "{release_tag}"\n',
    f'workspace_version = "{release_version}"\n',
    f'release_commit = "{release_commit}"\n',
]

for index, line in enumerate(filtered):
    if line.startswith("[scripts]"):
        filtered[index:index] = insertion
        break
else:
    filtered.extend(insertion)

path.write_text("".join(filtered), encoding="utf-8")
PY
}

ensure_clean_worktree() {
  if [[ -n "$(git status --porcelain)" ]]; then
    echo "release requires a clean git worktree" >&2
    exit 1
  fi
}

current_workspace_version="$(workspace_version)"
current_branch="$(git branch --show-current)"
release_commit="$(git rev-parse HEAD)"

if [[ "$dry_run" -eq 0 ]]; then
  ensure_clean_worktree
  if [[ "$current_branch" != "trunk" ]]; then
    echo "release must run from branch trunk; current branch is $current_branch" >&2
    exit 1
  fi
  if git rev-parse -q --verify "refs/tags/${release_tag}" >/dev/null; then
    echo "tag already exists: ${release_tag}" >&2
    exit 1
  fi
else
  if [[ -n "$(git status --porcelain)" ]]; then
    echo "dry-run: continuing with a dirty worktree; no git state will change" >&2
  fi
  if git rev-parse -q --verify "refs/tags/${release_tag}" >/dev/null; then
    echo "dry-run: tag ${release_tag} already exists; continuing without creating it" >&2
  fi
fi

allow_unreleased_fallback=0
if [[ "$dry_run" -eq 1 ]]; then
  allow_unreleased_fallback=1
fi
release_notes_section="$(extract_changelog_section "$release_version" "$allow_unreleased_fallback")"

if [[ "$dry_run" -eq 0 && "$current_workspace_version" != "$release_version" ]]; then
  set_workspace_version "$release_version"
  git add "$cargo_toml_path"
  git commit -m "release: ${release_tag}" >/dev/null
  release_commit="$(git rev-parse HEAD)"
elif [[ "$dry_run" -eq 1 && "$current_workspace_version" != "$release_version" ]]; then
  echo "dry-run: would update workspace version ${current_workspace_version} -> ${release_version}" >&2
fi

if [[ "$dry_run" -eq 1 && -z "${!password_env_name:-}" ]]; then
  export "${password_env_name}=release-dry-run"
fi

if [[ "$dry_run" -eq 0 && -z "${!password_env_name:-}" ]]; then
  echo "export ${password_env_name} before running a real release" >&2
  exit 1
fi

mkdir -p "$release_dir"
export MYOSU_OPERATOR_PASSWORD_ENV="$password_env_name"
bash .github/scripts/prepare_operator_network_bundle.sh "$bundle_dir" "$config_dir" >/dev/null
annotate_bundle_manifest "$release_commit"
write_release_notes "$release_notes_section" "$release_commit"

if [[ "$dry_run" -eq 0 ]]; then
  git tag -a "$release_tag" -m "Release ${release_tag}" "$release_commit"
fi

mode_label=""
if [[ "$dry_run" -eq 1 ]]; then
  mode_label="(dry-run) "
fi

cat <<EOF
Release ${release_tag} ${mode_label}prepared:
  workspace version: ${release_version}
  operator bundle: ${bundle_dir}
  release notes: ${notes_file}
  commit: ${release_commit}
EOF
