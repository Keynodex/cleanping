# CleanPing writer: the startup file of the zsh that `cleanping writer` starts.
# It is written to a private folder on every run, so edits made here do not last.

unsetopt BANG_HIST CORRECT CORRECT_ALL
setopt NO_BEEP
HISTFILE=/dev/null
SAVEHIST=0

PROMPT='> '
RPROMPT=''

print -r -- 'CleanPing writer'
print -r -- 'Type your text. Press Ctrl+G to fix it. Press Enter to copy it. Press Ctrl+D to leave.'
print -r -- 'Nothing you type here is ever run.'
