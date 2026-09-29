# CleanPing shell integration for zsh.
#
#   Add to ~/.zshrc:   eval "$(cleanping init zsh)"
#
# Press the key (default Ctrl-X Ctrl-P) to rewrite what you have typed on the command line.
# Press it again, without editing, to get your original text back.
# Change the key by setting CLEANPING_KEYBIND before the eval line, e.g. CLEANPING_KEYBIND='^[r'.
#
# Privacy: the whole command line is sent to your configured AI provider when you press the key.
# Nothing is saved to CleanPing's history from here (--no-history).

typeset -g _CLEANPING_ORIGINAL="" _CLEANPING_RESULT=""

_cleanping_polish() {
  emulate -L zsh
  local original=$BUFFER out msg rc errfile

  # Second press on an unchanged result: put the original back.
  if [[ -n $_CLEANPING_RESULT && $BUFFER == "$_CLEANPING_RESULT" ]]; then
    BUFFER=$_CLEANPING_ORIGINAL
    CURSOR=${#BUFFER}
    _CLEANPING_RESULT=""
    return 0
  fi

  [[ -z ${BUFFER//[[:space:]]/} ]] && return 0

  zle -M "cleanping: rewriting..." && zle -R
  # --keep-shape: the reply may not have more lines than your text, be much longer, or be padded
  # with blanks. Otherwise the part you can see could hide what runs when you press Enter.
  # The line goes over stdin, not the argument list, so it never shows up in `ps`. Errors go to
  # a private temp file, so only the reply itself can ever end up on your command line.
  errfile=$(mktemp 2>/dev/null) || errfile=/dev/null
  out=$(printf '%s' "$original" | command cleanping --no-history --keep-shape 2>"$errfile")
  rc=$?
  msg=$(<"$errfile"); [[ $errfile == /dev/null ]] || rm -f "$errfile"

  if (( rc == 0 )) && [[ -n $out ]]; then
    _CLEANPING_ORIGINAL=$original
    _CLEANPING_RESULT=$out
    BUFFER=$out
    CURSOR=${#BUFFER}
    zle -M ""
  else
    zle -M "${msg:-cleanping failed (exit $rc)}"
  fi
}

zle -N _cleanping_polish
bindkey "${CLEANPING_KEYBIND:-^X^P}" _cleanping_polish
