# CleanPing shell integration for bash 4+.
#
#   Add to ~/.bashrc:   eval "$(cleanping init bash)"
#
# Press the key (default Ctrl-X Ctrl-P) to rewrite what you have typed on the command line.
# Press it again, without editing, to get your original text back.
# Change the key by setting CLEANPING_KEYBIND before the eval line, e.g. CLEANPING_KEYBIND='\er'.
#
# Bash cannot highlight part of the command line, so after a rewrite the original is printed on
# its own line ("cleanping: was: ...") for you to compare.
#
# Privacy: the whole command line is sent to your configured AI provider when you press the key,
# unless it looks like it holds a key, token or password: then nothing is sent. Nothing is saved
# to CleanPing's history from here (--no-history).

if (( BASH_VERSINFO[0] < 4 )); then
  echo "cleanping: the shell key needs bash 4 or newer (this is bash ${BASH_VERSION})." >&2
else
  _CLEANPING_ORIGINAL=""
  _CLEANPING_RESULT=""

  _cleanping_polish() {
    local original=$READLINE_LINE out msg rc errfile

    # Second press on an unchanged result: put the original back.
    if [[ -n $_CLEANPING_RESULT && $READLINE_LINE == "$_CLEANPING_RESULT" ]]; then
      READLINE_LINE=$_CLEANPING_ORIGINAL
      READLINE_POINT=${#READLINE_LINE}
      _CLEANPING_RESULT=""
      return 0
    fi

    [[ -z ${READLINE_LINE//[[:space:]]/} ]] && return 0

    # --keep-shape: the reply may not have more lines than your text, be much longer, or be padded
    # with blanks. Otherwise the part you can see could hide what runs when you press Enter.
    # --refuse-secrets: a line that looks like it holds a key, token or password is not sent.
    # The line goes over stdin, not the argument list, so it never shows up in `ps`. Errors go to
    # a private temp file, so only the reply itself can ever end up on your command line.
    errfile=$(mktemp 2>/dev/null) || errfile=/dev/null
    out=$(printf '%s' "$original" | command cleanping --no-history --keep-shape --refuse-secrets 2>"$errfile")
    rc=$?
    msg=$(<"$errfile"); [[ $errfile == /dev/null ]] || rm -f "$errfile"

    if (( rc == 0 )) && [[ -n $out ]]; then
      _CLEANPING_ORIGINAL=$original
      _CLEANPING_RESULT=$out
      READLINE_LINE=$out
      READLINE_POINT=${#READLINE_LINE}
      # Control characters are shown as ? so an odd line cannot drive the terminal.
      printf '\ncleanping: was: %s\n' \
        "$(printf '%s' "$original" | LC_ALL=C tr '\000-\010\013-\037\177' '?')" >&2
    else
      # Bash has no message area; print the reason on its own line and keep the text.
      # cleanping's own messages already start with "cleanping: ".
      printf '\n%s\n' "${msg:-cleanping: failed (exit $rc)}" >&2
    fi
  }

  bind -x "\"${CLEANPING_KEYBIND:-\\C-x\\C-p}\": _cleanping_polish"
fi
