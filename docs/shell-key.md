# The shell key (zsh and bash)

Type a rough command line, press one key, and CleanPing rewrites it in place. Press the key again to get
your original back.

## Set it up

Add one line to `~/.zshrc`, or to `~/.bashrc` (bash 4 or newer), then open a new terminal:

```sh
eval "$(cleanping init zsh)"     # or: eval "$(cleanping init bash)"
```

`cleanping setup` prints the right line for your shell. You need a saved key first (see
[Getting started](getting-started.md)).

## Use it

1. Type a rough line at the prompt. Do not press Enter.
2. Press **Ctrl-X Ctrl-P**. zsh shows `cleanping: rewriting...` while it works, then the line is replaced
   by the rewrite.
3. Read it, edit it if you like, and press Enter when you are happy. Press the same keys again, without
   editing, to get your original back. An empty line does nothing.

To use a different key, set `CLEANPING_KEYBIND` before the `eval` line. In zsh use `bindkey` notation, and
in bash `bind` notation:

```sh
CLEANPING_KEYBIND='^[r'      # zsh: Alt-R
CLEANPING_KEYBIND='\er'      # bash: Alt-R
```

## What changed

- **zsh** highlights the words the AI changed, until you edit the line. The default style is `standout`.
  Set `CLEANPING_HIGHLIGHT` to another zsh style (`'underline'`, `'bg=yellow'`), or to an empty value to
  turn it off. Highlighting needs zsh 5.3 or newer and a UTF-8 locale; without them the line is still
  rewritten, just not highlighted.
- **bash** cannot highlight part of the command line, so it prints your original on its own line after
  the rewrite, for you to compare: `cleanping: was: ...` (control characters are shown as `?`).

## What the key sends, and what it refuses

The key runs `cleanping --no-history --keep-shape --refuse-secrets` and gives it your line on stdin.

- **The whole line goes to your AI provider** when you press the key. It goes over stdin, never as a
  command-line argument, so it does not show up in `ps`. Nothing is written to CleanPing's history.
- **A line that looks like it holds a secret is not sent** (`--refuse-secrets`): an API key, token,
  private key or password. You see a message and the line stays as it is. The check is a best guess and
  cannot catch everything, so still avoid the key on lines with secrets.
- **A reply has to look like what it replaces** (`--keep-shape`). It is only put on your command line if
  it has no more lines than your text, is not much longer (about twice as long plus 80 characters), and
  does not pad itself with long runs of spaces or tabs (more than 8 in a row, unless your own text
  already had a longer run). Otherwise the line is left alone and you see a message. This stops a reply from pushing a command out of sight above a harmless-looking
  end. It is still a rewrite by an AI, so read the line before you press Enter.
- **A reply may not change the command itself** (also `--keep-shape`). Your line is a command, so a reply
  is refused when it closes a quote you left open or leaves one open that you closed, drops or adds a
  flag (`-f`, `--force`), or changes a path or URL. A missing quote is often the very bug you are asking
  about, and a model that "fixes" it hides the answer, even when told not to. Your line stays as it is
  and you see `cleanping: The reply changed your command (...); not applied.` This is a safety net, not a
  proof: it reads quotes the way a shell does, but it does not understand every command, so it misses
  some changes (a changed word that is not a flag or path) and refuses a few harmless ones (see
  [Privacy and safety](privacy-and-safety.md#a-reply-that-changes-a-command)). The same applies to
  Ctrl+G in `cleanping writer`, which is this key.
- **Errors never end up on your command line.** They are captured and shown as a message, so only the
  reply itself can be placed in your line.

## Troubleshooting

| You see | What it means |
|---|---|
| `cleanping: the shell key needs bash 4 or newer (this is bash ...)` | Your bash is older than 4. Install a newer bash, or use zsh |
| A message that starts `cleanping:` and the line is unchanged | The rewrite was refused or failed; the message says why. See [Troubleshooting](troubleshooting.md) |
| Nothing happens when you press the keys | Check that the key is bound in this shell: `bindkey \| grep cleanping` (zsh) or `bind -X \| grep -i cleanping` (bash) should print a line. If not, the `eval` line has not run in this shell |
| The rewrite works but nothing is highlighted (zsh) | Highlighting needs zsh 5.3 or newer and a UTF-8 locale, or `CLEANPING_HIGHLIGHT` is empty |
