#!/bin/sh
# install-cleanping-mac.sh - install CleanPing on a Mac.
# Author: Keynodex | Date: 2026-10-01 (first version 2026-09-30)
# Purpose: download the CleanPing build for this Mac from its GitHub release, check its SHA-256
#          checksum, install it as ~/.local/bin/cleanping and make sure new Terminal windows can
#          find it (one PATH line in ~/.zshrc, added once). Changes nothing else.
# Usage:   download this file from a release page, then run:  sh install-cleanping-mac.sh
#          Download it first (and read it if you like); do not pipe it from curl into sh.
# Needs:   curl, tar and shasum (all ship with macOS); sha256sum works too.
#
# The copy attached to each release has its version filled in by the release workflow
# (.github/scripts/installer-for-release.sh). This copy in the repository does not: run it with
# CLEANPING_VERSION=vX.Y.Z to choose a release.
#
# Settings (environment variables, all optional):
#   CLEANPING_VERSION        the release to install, such as v0.5.0; wins over the filled-in version
#   CLEANPING_TARGET         aarch64-apple-darwin or x86_64-apple-darwin (default: this Mac's)
#   CLEANPING_PROFILE        the file that gets the PATH line (default: ~/.zshrc)
#   CLEANPING_DOWNLOAD_BASE  FOR TESTS ONLY: download from here instead of the GitHub releases. Must
#                            be https://, or http:// on 127.0.0.1 or localhost (a local test server).
set -eu

RELEASE_VERSION='@VERSION@'
RELEASES='https://github.com/Keynodex/cleanping/releases'
# shellcheck disable=SC2016 # the line is written to the profile as it is, not expanded here
LINE='export PATH="$HOME/.local/bin:$PATH"'

stop() { # stop CODE LINE...: print the lines and exit
  code=$1
  shift
  printf '%s\n' "$@" >&2
  exit "$code"
}

valid_version() { # vX.Y.Z, digits only
  case $1 in v[0-9]*.[0-9]*.[0-9]*) ;; *) return 1 ;; esac
  case ${1#v} in *[!0-9.]* | *..* | *. | *.*.*.*) return 1 ;; esac
}

[ "$#" -eq 0 ] || stop 2 "This installer takes no arguments. To choose a version, run:" \
  "  CLEANPING_VERSION=vX.Y.Z sh install-cleanping-mac.sh"
[ -n "${HOME:-}" ] || stop 1 "HOME is not set, so there is nowhere to install. Nothing was changed."

if [ -n "${CLEANPING_VERSION:-}" ]; then
  VERSION=$CLEANPING_VERSION
  case $VERSION in v*) ;; *) VERSION="v$VERSION" ;; esac
  valid_version "$VERSION" ||
    stop 2 "CLEANPING_VERSION must look like v0.5.0. Nothing was changed."
elif valid_version "$RELEASE_VERSION"; then
  VERSION=$RELEASE_VERSION
else
  stop 2 "This copy of the installer does not say which CleanPing version to install." \
    "Download install-cleanping-mac.sh from the release page and run that copy:" \
    "  $RELEASES/latest" \
    "Nothing was changed."
fi

BASE=${CLEANPING_DOWNLOAD_BASE:-$RELEASES/download}
case $BASE in
  *[!A-Za-z0-9._~:/-]*) PROTO='' ;; # no user@host, query or escapes
  https://*) PROTO='=https' ;;
  http://127.0.0.1:* | http://127.0.0.1/* | http://localhost:* | http://localhost/*) PROTO='=http' ;;
  *) PROTO='' ;;
esac
[ -n "$PROTO" ] || stop 2 "CLEANPING_DOWNLOAD_BASE is for tests only and must start with https://" \
  "(or http://127.0.0.1 or http://localhost). Nothing was changed."

if [ -n "${CLEANPING_TARGET:-}" ]; then
  TARGET=$CLEANPING_TARGET
else
  case "$(uname -s)-$(uname -m)" in
    Darwin-arm64) TARGET=aarch64-apple-darwin ;;
    Darwin-x86_64) TARGET=x86_64-apple-darwin ;;
    *) stop 1 "This installer is for a Mac. On Linux, follow the Install steps in the README." \
      "Nothing was changed." ;;
  esac
fi
case $TARGET in
  aarch64-apple-darwin | x86_64-apple-darwin) ;;
  *) stop 1 "There is no CleanPing build for that system: CLEANPING_TARGET must be" \
    "aarch64-apple-darwin or x86_64-apple-darwin. Nothing was changed." ;;
esac

if command -v shasum >/dev/null 2>&1; then
  sum256() { shasum -a 256 "$1"; }
elif command -v sha256sum >/dev/null 2>&1; then
  sum256() { sha256sum "$1"; }
else
  stop 1 "Neither shasum nor sha256sum was found, so the download cannot be checked." \
    "Nothing was changed."
fi
command -v curl >/dev/null 2>&1 || stop 1 "curl was not found. Nothing was changed."

NAME="cleanping-$VERSION-$TARGET"
DEST="$HOME/.local/bin"
PART=''
WORK=$(mktemp -d)
cleanup() {
  rm -rf "$WORK"
  if [ -n "$PART" ]; then rm -f "$PART"; fi
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM

fetch() { # fetch FILE: download one release file into WORK
  curl -fsSL --proto "$PROTO" --proto-redir "$PROTO" -o "$WORK/$1" "$BASE/$VERSION/$1" ||
    stop 1 "Could not download $1 for CleanPing $VERSION." \
      "Check the internet connection and that the release exists: $RELEASES" \
      "Nothing was changed."
}

echo "Downloading CleanPing $VERSION ($TARGET)..."
fetch "$NAME.tar.gz"
fetch "$NAME.tar.gz.sha256"

echo "Checking that the download is complete and unchanged..."
expected='' listed='' rest=''
read -r expected listed rest < "$WORK/$NAME.tar.gz.sha256" || :
actual=$(sum256 "$WORK/$NAME.tar.gz")
actual=${actual%% *}
case $listed in "$NAME.tar.gz" | "*$NAME.tar.gz") ;; *) expected='' ;; esac
if [ -n "$rest" ] || [ "${#expected}" -ne 64 ] || [ "$actual" != "$expected" ]; then
  stop 1 "The download does not match its checksum, so it was not installed." \
    "Nothing was changed."
fi

(cd "$WORK" && tar xzf "$NAME.tar.gz" "$NAME/cleanping") ||
  stop 1 "The download could not be unpacked. Nothing was changed."
if [ ! -f "$WORK/$NAME/cleanping" ] || [ -L "$WORK/$NAME/cleanping" ]; then
  stop 1 "The download holds no cleanping program. Nothing was changed."
fi

# Copy under a temporary name in the same folder, check it starts, then rename it into place in one
# step: a failure on the way leaves any existing cleanping as it was.
mkdir -p "$DEST"
PART="$DEST/.cleanping-install.$$"
cp "$WORK/$NAME/cleanping" "$PART"
chmod 755 "$PART"
INSTALLED=$("$PART" --version 2>/dev/null) ||
  stop 1 "The downloaded CleanPing does not start on this Mac, so it was not installed." \
    "Any CleanPing you had before was not changed."
mv -f "$PART" "$DEST/cleanping"
PART=''

# New Terminal windows look for programs in PATH; make sure that folder is on it.
PROFILE=${CLEANPING_PROFILE:-$HOME/.zshrc}
touch "$PROFILE"
if ! grep -qF "$LINE" "$PROFILE"; then
  printf '\n# Added by the CleanPing installer\n%s\n' "$LINE" >> "$PROFILE"
fi

echo
echo "CleanPing is installed ($INSTALLED)."
echo "Now close this Terminal window, open a new one, and type:  cleanping setup"
