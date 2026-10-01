#!/usr/bin/env bash
# Author: Keynodex
# Date: 2026-10-01
# Purpose: Check the Mac installer against a fake release served on 127.0.0.1 only: install, the PATH
#          line, checksum and download failures, version, target and address checks, the release copy,
#          and missing checksum tools. Works on Linux too (CLEANPING_TARGET picks a Mac build).
# Usage: bash .github/scripts/install-mac-test.sh
set -euo pipefail

scripts=$(cd -- "$(dirname -- "$0")" && pwd)
root=$(cd -- "$scripts/../.." && pwd)
installer="$root/scripts/install-cleanping-mac.sh"
for_release="$scripts/installer-for-release.sh"
scratch=$(mktemp -d)
server_pid=''

cleanup() {
  if [ -n "$server_pid" ]; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT

fail() {
  printf 'FAIL: %s\n' "$1" >&2
  if [ -s "$scratch/out" ]; then
    printf -- '--- installer output ---\n' >&2
    cat "$scratch/out" >&2
  fi
  exit 1
}
pass() { printf 'PASS: %s\n' "$1"; }

[ -f "$installer" ] || fail 'scripts/install-cleanping-mac.sh not found'
[ -f "$for_release" ] || fail '.github/scripts/installer-for-release.sh not found'

sh_bin=$(command -v sh)
target=aarch64-apple-darwin
line='export PATH="$HOME/.local/bin:$PATH"'

hash_file() {
  if command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1"; else sha256sum "$1"; fi
}

# A fake release: an archive holding a `cleanping` stub, and its .sha256. "bad" has a wrong
# checksum; "broken" has a right one but a program that does not start.
release() {
  local version=$1 kind=$2
  local dir="$scratch/www/$version" name="cleanping-$version-$target"
  mkdir -p "$dir/$name"
  printf '#!/bin/sh\necho "cleanping %s"\n' "${version#v}" > "$dir/$name/cleanping"
  [ "$kind" != broken ] || printf '#!/bin/sh\nexit 1\n' > "$dir/$name/cleanping"
  chmod 755 "$dir/$name/cleanping"
  (cd "$dir" && tar czf "$name.tar.gz" "$name" && rm -r "$name")
  if [ "$kind" != bad ]; then
    (cd "$dir" && hash_file "$name.tar.gz" > "$name.tar.gz.sha256")
  else
    printf '%064d  %s\n' 0 "$name.tar.gz" > "$dir/$name.tar.gz.sha256"
  fi
}
release v9.9.9 good
release v9.9.7 good
release v9.9.8 bad
release v9.9.5 broken

# Pick a free port here instead of reading it from the server's log (whose wording and buffering
# vary), start the server on it, and wait until it answers a real request.
server_fail() {
  printf -- '--- server log (%s, %s) ---\n' "$(command -v python3)" "$(python3 --version 2>&1)" >&2
  cat "$scratch/server.log" >&2 || true
  fail "$1"
}
port=$(python3 -c 'import socket
s = socket.socket()
s.bind(("127.0.0.1", 0))
print(s.getsockname()[1])
s.close()') || fail 'could not pick a free port'
case $port in '' | *[!0-9]*) fail "could not pick a free port: '$port'" ;; esac
base="http://127.0.0.1:$port"
python3 -u -m http.server "$port" --bind 127.0.0.1 --directory "$scratch/www" \
  > "$scratch/server.log" 2>&1 &
server_pid=$!
ready=''
for _ in $(seq 1 300); do
  kill -0 "$server_pid" 2>/dev/null || server_fail 'the local test server exited'
  if curl -fsS --noproxy '*' -o /dev/null --max-time 2 "$base/v9.9.9/cleanping-v9.9.9-$target.tar.gz.sha256" \
    2>/dev/null; then
    ready=1
    break
  fi
  sleep 0.1
done
[ -n "$ready" ] || server_fail 'the local test server did not answer within 30 seconds'

# run SCRIPT HOME [VAR=value...]: a clean environment, so nothing from the caller leaks in.
run() {
  local script=$1 home=$2
  shift 2
  mkdir -p "$home"
  set +e
  env -i HOME="$home" PATH="${tool_path:-$PATH}" CLEANPING_DOWNLOAD_BASE="$base" \
    CLEANPING_TARGET="$target" "$@" "$sh_bin" "$script" > "$scratch/out" 2>&1
  status=$?
  set -e
}

expect_status() {
  [ "$status" -eq "$1" ] || fail "$2: exit status $status, expected $1"
}
expect_failed() {
  [ "$status" -ne 0 ] || fail "$1: must fail"
}
expect_output() {
  grep -Fq -- "$1" "$scratch/out" || fail "$2: output lacks '$1'"
}
expect_untouched_home() {
  [ -z "$(ls -A "$1")" ] || fail "$2: the home folder was changed"
}
expect_runs() {
  local got
  got=$("$1/.local/bin/cleanping" --version) || fail "$3: installed binary does not run"
  [ "$got" = "$2" ] || fail "$3: installed '$got', expected '$2'"
}
path_lines() {
  grep -cxF "$line" "$1/.zshrc" || true
}
only_cleanping_in_bin() {
  [ "$(ls -A "$1/.local/bin")" = cleanping ] || fail "$2: stray files in ~/.local/bin"
}

home="$scratch/home-install"
run "$installer" "$home" CLEANPING_VERSION=v9.9.9
expect_status 0 'installs and runs'
expect_runs "$home" 'cleanping 9.9.9' 'installs and runs'
expect_output 'cleanping setup' 'installs and runs'
only_cleanping_in_bin "$home" 'installs and runs'
[ "$(path_lines "$home")" = 1 ] || fail 'installs and runs: PATH line missing'
pass 'installs and runs'

run "$installer" "$home" CLEANPING_VERSION=v9.9.9
expect_status 0 'second run'
expect_runs "$home" 'cleanping 9.9.9' 'second run'
[ "$(path_lines "$home")" = 1 ] || fail 'second run: PATH line duplicated'
only_cleanping_in_bin "$home" 'second run'
pass 'second run keeps one PATH line and a working binary'

home="$scratch/home-profile"
run "$installer" "$home" CLEANPING_VERSION=v9.9.9 CLEANPING_PROFILE="$home/.bash_profile"
expect_status 0 'CLEANPING_PROFILE'
grep -qxF "$line" "$home/.bash_profile" || fail 'CLEANPING_PROFILE: line not added'
[ ! -e "$home/.zshrc" ] || fail 'CLEANPING_PROFILE: ~/.zshrc was touched'
pass 'CLEANPING_PROFILE chooses the file'

home="$scratch/home-badsum"
run "$installer" "$home" CLEANPING_VERSION=v9.9.8
expect_failed 'wrong checksum, fresh'
expect_output 'checksum' 'wrong checksum, fresh'
expect_untouched_home "$home" 'wrong checksum, fresh'
pass 'wrong checksum installs nothing'

home="$scratch/home-existing"
mkdir -p "$home/.local/bin"
printf '#!/bin/sh\necho "cleanping 0.0.1"\n' > "$home/.local/bin/cleanping"
chmod 755 "$home/.local/bin/cleanping"
cp "$home/.local/bin/cleanping" "$scratch/old-cleanping"
run "$installer" "$home" CLEANPING_VERSION=v9.9.8
expect_failed 'wrong checksum, existing'
cmp -s "$home/.local/bin/cleanping" "$scratch/old-cleanping" ||
  fail 'wrong checksum, existing: the existing binary changed'
only_cleanping_in_bin "$home" 'wrong checksum, existing'
[ ! -e "$home/.zshrc" ] || fail 'wrong checksum, existing: profile was touched'
pass 'wrong checksum leaves the existing binary untouched'

run "$installer" "$home" CLEANPING_VERSION=v9.9.5
expect_failed 'program does not start'
expect_output 'does not start' 'program does not start'
cmp -s "$home/.local/bin/cleanping" "$scratch/old-cleanping" ||
  fail 'program does not start: the existing binary changed'
only_cleanping_in_bin "$home" 'program does not start'
[ ! -e "$home/.zshrc" ] || fail 'program does not start: profile was touched'
pass 'a program that does not start is not installed'

home="$scratch/home-missing"
run "$installer" "$home" CLEANPING_VERSION=v9.9.6
expect_failed 'missing release'
expect_output 'Could not download' 'missing release'
expect_untouched_home "$home" 'missing release'
pass 'a failed download installs nothing'

home="$scratch/home-target"
run "$installer" "$home" CLEANPING_VERSION=v9.9.9 CLEANPING_TARGET=x86_64-unknown-linux-gnu
expect_failed 'unknown target'
expect_output 'no CleanPing build' 'unknown target'
expect_untouched_home "$home" 'unknown target'
pass 'unknown target gets a plain message'

if [ "$(uname -s)" != Darwin ]; then
  home="$scratch/home-notmac"
  run "$installer" "$home" CLEANPING_VERSION=v9.9.9 CLEANPING_TARGET=
  expect_failed 'not a Mac'
  expect_output 'is for a Mac' 'not a Mac'
  expect_untouched_home "$home" 'not a Mac'
  pass 'not a Mac gets a plain message'
fi

home="$scratch/home-noversion"
run "$installer" "$home"
expect_status 2 'no version'
expect_output 'release page' 'no version'
expect_untouched_home "$home" 'no version'
pass 'placeholder and no CLEANPING_VERSION exits 2'

for bad in v1.2 v1.2.3.4 'v1.2.3/../x' 'v1..2' latest; do
  home="$scratch/home-badversion"
  rm -rf "$home"
  run "$installer" "$home" CLEANPING_VERSION="$bad"
  expect_status 2 "bad version $bad"
  expect_untouched_home "$home" "bad version $bad"
done
pass 'a malformed CLEANPING_VERSION exits 2'

home="$scratch/home-noprefix"
run "$installer" "$home" CLEANPING_VERSION=9.9.9
expect_status 0 'version without v'
expect_runs "$home" 'cleanping 9.9.9' 'version without v'
pass 'CLEANPING_VERSION works without the leading v'

# .invalid never resolves, so a broken check cannot reach a real host from this test.
for bad in 'http://example.invalid' 'http://localhost.example.invalid' \
  "http://127.0.0.1:$port@example.invalid" 'ftp://127.0.0.1' 'file:///etc'; do
  home="$scratch/home-base"
  rm -rf "$home"
  run "$installer" "$home" CLEANPING_VERSION=v9.9.9 CLEANPING_DOWNLOAD_BASE="$bad"
  expect_status 2 "base $bad"
  expect_output 'CLEANPING_DOWNLOAD_BASE' "base $bad"
  expect_untouched_home "$home" "base $bad"
done
pass 'a non-https remote download base is refused'

home="$scratch/home-args"
mkdir -p "$home"
status=0
env -i HOME="$home" PATH="$PATH" CLEANPING_DOWNLOAD_BASE="$base" CLEANPING_TARGET="$target" \
  "$sh_bin" "$installer" v9.9.9 > "$scratch/out" 2>&1 || status=$?
expect_status 2 'arguments'
expect_output 'CLEANPING_VERSION' 'arguments'
expect_untouched_home "$home" 'arguments'
pass 'arguments are refused with a pointer to CLEANPING_VERSION'

# Release copy: the workflow fills in the tag with installer-for-release.sh.
copy="$scratch/release/install-cleanping-mac.sh"
mkdir -p "$scratch/release"
bash "$for_release" v9.9.9 "$installer" "$copy" || fail 'release copy: must be written'
grep -Fq '@VERSION@' "$copy" && fail 'release copy: placeholder left in'
home="$scratch/home-release"
run "$copy" "$home"
expect_status 0 'release copy'
expect_runs "$home" 'cleanping 9.9.9' 'release copy'
pass 'release copy installs its own version'

home="$scratch/home-release-override"
run "$copy" "$home" CLEANPING_VERSION=v9.9.7
expect_status 0 'release copy with CLEANPING_VERSION'
expect_runs "$home" 'cleanping 9.9.7' 'release copy with CLEANPING_VERSION'
pass 'CLEANPING_VERSION overrides the filled-in version'

for bad in 'v1.2.3/x' 'v1.2.3&' '1.2.3' 'v1.2' 'v1.2.3-rc.1' "v1.2.3
x"; do
  out="$scratch/release/bad.sh"
  if bash "$for_release" "$bad" "$installer" "$out" 2> "$scratch/out"; then
    fail "release copy: tag '$bad' must be refused"
  fi
  [ ! -e "$out" ] && [ ! -e "$out.part" ] || fail "release copy: '$bad' left a file"
  expect_output 'tag must be vX.Y.Z' "release copy tag '$bad'"
done
pass 'release copy refuses a tag that is not vX.Y.Z'

# Missing tools: a PATH holding only what the installer needs, minus the checksum tools.
toolbox() {
  local box=$1 skip=$2 tool
  mkdir -p "$box"
  for tool in uname mktemp rm curl tar gzip mkdir cp mv chmod grep touch shasum sha256sum; do
    case " $skip " in *" $tool "*) continue ;; esac
    if command -v "$tool" >/dev/null 2>&1; then
      ln -s "$(command -v "$tool")" "$box/$tool"
    fi
  done
}
toolbox "$scratch/box-none" 'shasum sha256sum'
home="$scratch/home-notools"
tool_path="$scratch/box-none" run "$installer" "$home" CLEANPING_VERSION=v9.9.9
expect_failed 'no checksum tool'
expect_output 'shasum' 'no checksum tool'
expect_untouched_home "$home" 'no checksum tool'
pass 'missing shasum and sha256sum installs nothing'

if command -v sha256sum >/dev/null 2>&1; then
  toolbox "$scratch/box-sha256sum" 'shasum'
  home="$scratch/home-sha256sum"
  tool_path="$scratch/box-sha256sum" run "$installer" "$home" CLEANPING_VERSION=v9.9.9
  expect_status 0 'sha256sum only'
  expect_runs "$home" 'cleanping 9.9.9' 'sha256sum only'
  pass 'sha256sum works when shasum is missing'
fi

if command -v shasum >/dev/null 2>&1; then
  toolbox "$scratch/box-shasum" 'sha256sum'
  home="$scratch/home-shasum"
  tool_path="$scratch/box-shasum" run "$installer" "$home" CLEANPING_VERSION=v9.9.9
  expect_status 0 'shasum only'
  expect_runs "$home" 'cleanping 9.9.9' 'shasum only'
  pass 'shasum alone is enough'
fi

printf 'All Mac installer tests passed.\n'
