# CleanPing shell integration for zsh.
#
#   Add to ~/.zshrc:   eval "$(cleanping init zsh)"
#
# Press the key (default Ctrl-X Ctrl-P) to rewrite what you have typed on the command line.
# Press it again, without editing, to get your original text back.
# Change the key by setting CLEANPING_KEYBIND before the eval line, e.g. CLEANPING_KEYBIND='^[r'.
#
# The words the AI changed are highlighted until you edit the line (needs zsh 5.3 or newer).
# Set CLEANPING_HIGHLIGHT to another zsh style, e.g. 'underline' or 'bg=yellow', or to an empty
# value to turn the highlight off.
#
# Privacy: the whole command line is sent to your configured AI provider when you press the key,
# unless it looks like it holds a key, token or password: then nothing is sent. Nothing is saved
# to CleanPing's history from here (--no-history).

typeset -g _CLEANPING_ORIGINAL="" _CLEANPING_RESULT=""
typeset -ga _CLEANPING_MARKS=()

# Remove exactly the highlights we added; other plugins' entries stay.
_cleanping_clear_marks() {
  emulate -L zsh
  (( ${#_CLEANPING_MARKS} )) || return 0
  region_highlight=( ${region_highlight:|_CLEANPING_MARKS} )
  _CLEANPING_MARKS=()
}

# Runs before every redraw: highlights are positions in the reply, so they must go as soon as the
# line is anything else.
_cleanping_unmark() {
  emulate -L zsh
  [[ $BUFFER == "$_CLEANPING_RESULT" ]] || _cleanping_clear_marks
}

# $1 = the original line, $2 = the reply. Highlights the parts of the reply that changed.
_cleanping_mark() {
  emulate -L zsh
  local style=${CLEANPING_HIGHLIGHT-standout} output first last
  local -a lines
  [[ -n $style && -n $_CLEANPING_CAN_MARK ]] || return 0
  output=$(printf '%s\0%s' "$1" "$2" | command cleanping --marks 2>/dev/null) || return 0
  lines=("${(@f)output}")
  # The first line is the reply's length in characters. A locale that is not UTF-8 makes zsh
  # count bytes instead, and then the positions would light the wrong text: show no highlight.
  [[ ${lines[1]} == ${#2} ]] || return 0
  for first last in ${=lines[2,-1]}; do
    _CLEANPING_MARKS+=("$first $last $style")
  done
  region_highlight+=("${_CLEANPING_MARKS[@]}")
}

_cleanping_polish() {
  emulate -L zsh
  local original=$BUFFER out msg rc errfile

  # Second press on an unchanged result: put the original back.
  if [[ -n $_CLEANPING_RESULT && $BUFFER == "$_CLEANPING_RESULT" ]]; then
    BUFFER=$_CLEANPING_ORIGINAL
    CURSOR=${#BUFFER}
    _CLEANPING_RESULT=""
    _cleanping_clear_marks
    return 0
  fi

  [[ -z ${BUFFER//[[:space:]]/} ]] && return 0

  zle -M "cleanping: rewriting..." && zle -R
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
    BUFFER=$out
    CURSOR=${#BUFFER}
    _cleanping_clear_marks
    _cleanping_mark "$original" "$out"
    zle -M ""
  else
    zle -M "${msg:-cleanping failed (exit $rc)}"
  fi
}

zle -N _cleanping_polish
bindkey "${CLEANPING_KEYBIND:-^X^P}" _cleanping_polish

# Highlighting is only switched on when the redraw hook that removes it is available.
if autoload -Uz add-zle-hook-widget 2>/dev/null && add-zle-hook-widget line-pre-redraw _cleanping_unmark 2>/dev/null; then
  typeset -g _CLEANPING_CAN_MARK=1
fi
