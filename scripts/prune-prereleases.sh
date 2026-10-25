#!/usr/bin/env bash
set -euo pipefail

version_parts() {
  local version=${1#v}
  if [[ "$version" =~ ^([1-9][0-9]{3})\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-beta\.(0|[1-9][0-9]*))?$ ]]; then
    local stable=1 beta=${BASH_REMATCH[5]:-0}
    [[ -z "${BASH_REMATCH[4]}" ]] || stable=0
    printf '%s %s %s %s %s\n' "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}" "${BASH_REMATCH[3]}" "$stable" "$beta"
  else
    return 1
  fi
}

older_than() {
  local -a left right
  local i
  read -r -a left <<< "$1"
  read -r -a right <<< "$2"
  for i in 0 1 2 3 4; do
    # Compare decimal strings without overflowing on a large beta number.
    if (( ${#left[i]} < ${#right[i]} )) ||
       { (( ${#left[i]} == ${#right[i]} )) && [[ "${left[i]}" < "${right[i]}" ]]; }; then
      return 0
    fi
    [[ "${left[i]}" == "${right[i]}" ]] || return 1
  done
  return 1
}

prune_prereleases() {
  local current=$1 tag current_parts candidate_parts
  if ! current_parts=$(version_parts "$current"); then
    echo "unsupported current release tag; skipping prerelease pruning: $current"
    return 0
  fi
  while IFS= read -r tag; do
    if ! candidate_parts=$(version_parts "$tag") || [[ ! "$tag" =~ -beta\.[0-9]+$ ]]; then
      continue
    fi
    if older_than "$candidate_parts" "$current_parts"; then
      echo "pruning older prerelease: $tag"
      gh release delete --yes --cleanup-tag "$tag" || true
    fi
  done < <(gh release list --json tagName,isPrerelease -q '.[] | select(.isPrerelease) | .tagName')
}

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  prune_prereleases "${RELEASE_TAG:?RELEASE_TAG is required}"
fi
