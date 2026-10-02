# CleanPing writer: the startup file of the zsh that `cleanping writer` starts.
# It is written to a private folder on every run, so edits made here do not last.
#
# The idea: a place to type text, where nothing typed is ever run. Ctrl+G fixes the line in
# place (the same key as `cleanping init zsh`, on Ctrl+G), Enter copies it, Ctrl+D leaves.
# This is a guard against accidents for someone new to terminals, not a security sandbox.

bindkey -e

# The shell key, loaded exactly as a ~/.zshrc line would load it, on Ctrl+G. The changed-words
# highlight is off here: after Enter the lit words stay on the lines that scrolled away.
CLEANPING_KEYBIND='^G'
CLEANPING_HIGHLIGHT=''
eval "$(command cleanping init zsh)"

# Ask the terminal itself to copy $1 (the OSC 52 escape sequence). It reaches your own clipboard
# over SSH or on a server with no clipboard tool. The terminal never answers, so the caller can
# only say the request was sent. Fails when `base64` or `tr` is missing, or the text is too long
# for a request (many terminals refuse more than about 75 kB).
_cleanping_writer_ask_terminal() {
  emulate -L zsh
  local encoded
  whence -p -- base64 >/dev/null 2>&1 && whence -p -- tr >/dev/null 2>&1 || return 1
  encoded=$(printf '%s' "$1" | command base64 | command tr -d '\n') || return 1
  [[ -n $encoded ]] && (( ${#encoded} <= 74000 )) || return 1
  print -rn -- $'\e]52;c;'"$encoded"$'\a'
}

# Copy $1 with the first clipboard tool found. Returns 0 when a tool copied it. When there is
# no tool, or it fails (an `xclip` with no display over SSH), it asks the terminal to copy
# instead and returns 2: that request cannot be confirmed. Returns 1 when nothing could be done.
_cleanping_writer_copy() {
  emulate -L zsh
  local candidate
  local -a tool
  for candidate in 'pbcopy' 'wl-copy' 'xclip -selection clipboard' 'xsel --clipboard --input'; do
    tool=( ${=candidate} )
    whence -p -- ${tool[1]} >/dev/null 2>&1 || continue
    if printf '%s' "$1" | command ${tool[@]} >/dev/null 2>&1; then
      return 0
    fi
    break
  done
  _cleanping_writer_ask_terminal "$1" && return 2
  return 1
}

# Enter: copy what was typed, say so below it, and start a fresh line. Never runs anything.
_cleanping_writer_enter() {
  emulate -L zsh
  [[ -z ${BUFFER//[[:space:]]/} ]] && return 0
  local message
  zle -I
  _cleanping_writer_copy "$BUFFER"
  case $? in
    0) message='Copied. Paste it anywhere (Cmd+V on a Mac, Ctrl+V elsewhere).' ;;
    2) message="Sent to your terminal's clipboard. If pasting does not work, select the text above and copy it yourself." ;;
    *) message='Could not copy (no clipboard tool found). Select the text above and copy it yourself.' ;;
  esac
  # The blank line matters: the prompt redraw moves up one line and clears below, so a message on
  # the line just above the prompt would be wiped at once and nobody would see it.
  print -r -- $message
  print
  BUFFER=''
  zle reset-prompt
}

# History search would accept a line on Enter, and there is no history to search: it is off.
# Every way of accepting a line copies instead of running it. A widget is replaced by name, so
# every key bound to an accept-* widget, in every keymap, follows.
zle -N _cleanping_writer_enter
zle -N accept-line _cleanping_writer_enter
for _cleanping_widget in ${(k)widgets}; do
  [[ $_cleanping_widget == accept-* ]] && zle -N $_cleanping_widget _cleanping_writer_enter
done
unset _cleanping_widget
for _cleanping_map in emacs viins vicmd; do
  bindkey -M $_cleanping_map '^M' _cleanping_writer_enter
  bindkey -M $_cleanping_map '^J' _cleanping_writer_enter
  bindkey -M $_cleanping_map '^O' _cleanping_writer_enter
  bindkey -M $_cleanping_map '^Z' undefined-key
  bindkey -M $_cleanping_map -r '^X^E' '^R' '^S'
done
unset _cleanping_map

# Text stays text: no history expansion (`!!`), no saved history, no spelling "corrections".
# No flow control either: Ctrl+S would freeze the screen until Ctrl+Q.
unsetopt BANG_HIST CORRECT CORRECT_ALL FLOW_CONTROL
setopt NO_BEEP
HISTFILE=/dev/null
SAVEHIST=0

PROMPT='> '
RPROMPT=''

print -r -- 'CleanPing writer'
print -r -- 'Type your text. Press Ctrl+G to fix it. Press Enter to copy it. Press Ctrl+D to leave.'
print -r -- 'Nothing you type here is ever run.'
