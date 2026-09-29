#!/usr/bin/env bash
# Author: Keynodex
# Date: 2026-09-30
# Purpose: Check changelog extraction, compare links, and failures using fixtures.
# Usage: bash .github/scripts/release-notes-test.sh
set -euo pipefail

scripts=$(cd -- "$(dirname -- "$0")" && pwd)
fixtures="$scripts/fixtures/release-notes"
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT

fail() {
  printf 'FAIL: %s\n' "$1" >&2
  exit 1
}

expect_notes() {
  local name=$1 version=$2
  cat > "$scratch/expected"
  bash "$scripts/release-notes.sh" "$version" "$fixtures/releases.md" \
    > "$scratch/actual" 2> "$scratch/error" || fail "$name: extraction must succeed"
  diff -u "$scratch/expected" "$scratch/actual" || fail "$name: notes differ"
  [ ! -s "$scratch/error" ] || fail "$name: unexpected error output"
  printf 'PASS: %s\n' "$name"
}

expect_failure() {
  local name=$1 version=$2 file=$3 message=$4
  if bash "$scripts/release-notes.sh" "$version" "$file" \
    > "$scratch/actual" 2> "$scratch/error"; then
    fail "$name: extraction must fail"
  fi
  [ ! -s "$scratch/actual" ] || fail "$name: failure leaked partial notes"
  grep -Fq "$message" "$scratch/error" || fail "$name: missing clear error"
  # Diagnostics must not echo rejected content or an unvalidated version.
  if grep -Eq '@|Example Person' "$scratch/error"; then
    fail "$name: diagnostic leaked attribution"
  fi
  printf 'PASS: %s\n' "$name"
}

expect_notes 'normal section' 0.3.0 <<'NOTES'
- Middle release.

**Full Changelog**: https://github.com/Keynodex/cleanping/compare/v0.2.0...v0.3.0
NOTES

expect_notes 'newest release finds previous version' 0.4.0 <<'NOTES'
- Newest release.

**Full Changelog**: https://github.com/Keynodex/cleanping/compare/v0.3.0...v0.4.0
NOTES

expect_notes 'sub-headings are kept' 0.2.0 <<'NOTES'
### Added

- Keep this sub-heading.

### Fixed

- Keep this too.

**Full Changelog**: https://github.com/Keynodex/cleanping/compare/v0.1.0...v0.2.0
NOTES

expect_failure 'missing version' 9.9.9 "$fixtures/releases.md" 'section not found'
expect_failure 'empty section' 0.3.0 "$fixtures/empty.md" 'section is empty'
printf '## [0.3.0] - 2026-09-29\n\n \t \n\n## [0.2.0]\n' > "$scratch/whitespace.md"
expect_failure 'whitespace-only section' 0.3.0 "$scratch/whitespace.md" 'section is empty'
expect_failure 'headings alone are empty' 0.3.0 "$fixtures/headings-only.md" 'section is empty'
expect_failure 'handle is rejected' 0.3.0 "$fixtures/handle.md" 'attribution is not allowed'
expect_failure 'author is rejected' 0.3.0 "$fixtures/author.md" 'attribution is not allowed'
expect_failure 'previous version is required' 0.1.0 "$fixtures/releases.md" 'previous version not found'
expect_failure 'invalid version' '@example-user' "$fixtures/releases.md" 'version must be X.Y.Z'
expect_failure 'missing file' 0.3.0 "$scratch/missing.md" 'cannot read changelog'

printf 'All release-notes tests passed.\n'
