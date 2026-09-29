#!/usr/bin/env bash
# Author: Keynodex
# Date: 2026-09-30
# Purpose: Extract reader-facing release notes and a compare link from the changelog.
# Usage: bash .github/scripts/release-notes.sh 0.3.0 CHANGELOG.md
set -euo pipefail

fail() {
  printf 'release-notes: %s\n' "$1" >&2
  exit 1
}

[ "$#" -eq 2 ] || fail 'usage: release-notes.sh VERSION CHANGELOG'
[[ $1 =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail 'version must be X.Y.Z'
[ -f "$2" ] && [ -r "$2" ] || fail 'cannot read changelog'

# Buffer the section: no partial notes may reach stdout if validation fails.
awk -v version="$1" '
  function reject(message) {
    print "release-notes: " message > "/dev/stderr"
    exit 1
  }
  /^## / {
    if (collecting) collecting = 0
    if ($0 ~ /^## \[[0-9]+\.[0-9]+\.[0-9]+\]/) {
      heading = $0
      sub(/^## \[/, "", heading)
      sub(/\].*$/, "", heading)
      if (found && previous == "") previous = heading
      if (!found && heading == version &&
          $0 ~ /\] - [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9][[:space:]]*$/) {
        found = 1
        collecting = 1
      }
    }
    next
  }
  collecting {
    lines[++count] = $0
    if ($0 ~ /[^[:space:]]/ && $0 !~ /^#+[[:space:]]/) has_content = 1
    if ($0 ~ /@/ || tolower($0) ~ /^[#*_[:space:]-]*(authors?|contributors?|new contributors)([[:space:]:]|$)/)
      has_attribution = 1
  }
  END {
    if (!found) reject("section not found")
    if (!has_content) reject("section is empty")
    if (has_attribution) reject("attribution is not allowed")
    if (previous == "") reject("previous version not found")
    first = 1
    last = count
    while (first <= last && lines[first] ~ /^[[:space:]]*$/) first++
    while (last >= first && lines[last] ~ /^[[:space:]]*$/) last--
    for (i = first; i <= last; i++) print lines[i]
    printf "\n**Full Changelog**: https://github.com/Keynodex/cleanping/compare/v%s...v%s\n", previous, version
  }
' < "$2"
