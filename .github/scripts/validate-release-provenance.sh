#!/usr/bin/env bash
set -euo pipefail

tag="${1:-${GITHUB_REF_NAME:-}}"
if [[ ! "$tag" =~ ^v([0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?)$ ]]; then
  echo "invalid release tag: $tag" >&2
  exit 1
fi

git fetch --no-tags origin main
tag_commit="$(git rev-list -n 1 "$tag")"
git merge-base --is-ancestor "$tag_commit" origin/main || {
  echo "release tag commit is not reachable from origin/main" >&2
  exit 1
}

workspace_version="$(cargo metadata --manifest-path cli/Cargo.toml --no-deps --format-version 1 | jq -r '.packages[0].version')"
if [[ "${BASH_REMATCH[1]}" != "$workspace_version" ]]; then
  echo "tag version ${BASH_REMATCH[1]} does not match workspace version $workspace_version" >&2
  exit 1
fi
