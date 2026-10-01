#!/usr/bin/env bash
# Author: Keynodex
# Date: 2026-10-01
# Purpose: Write the release copy of the Mac installer, with the release tag filled in.
# Usage: bash .github/scripts/installer-for-release.sh v0.5.0 scripts/install-cleanping-mac.sh OUT
set -euo pipefail

fail() {
  printf 'installer-for-release: %s\n' "$1" >&2
  exit 1
}

[ "$#" -eq 3 ] || fail 'usage: installer-for-release.sh TAG INSTALLER OUT'
tag=$1 source=$2 out=$3
# Only digits and dots after the v: nothing in the tag can act on sed or the shell script.
[[ $tag =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail 'tag must be vX.Y.Z'
[ -f "$source" ] && [ -r "$source" ] || fail 'cannot read the installer'

placeholder='@VERSION@'
[ "$(grep -cF "$placeholder" "$source")" = 1 ] || fail 'the placeholder must appear exactly once'
grep -qxF "RELEASE_VERSION='$placeholder'" "$source" || fail 'placeholder line not found'

part="$out.part"
trap 'rm -f "$part"' EXIT
sed "s/$placeholder/$tag/" "$source" > "$part"
! grep -qF "$placeholder" "$part" || fail 'placeholder left in'
grep -qxF "RELEASE_VERSION='$tag'" "$part" || fail 'version line not written'
sh -n "$part" || fail 'the release copy does not parse'
chmod 755 "$part"
mv "$part" "$out"
