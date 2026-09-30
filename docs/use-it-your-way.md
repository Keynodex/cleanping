# Use CleanPing your way

CleanPing works in three places, and each has its own key. This page shows how to set up each one, and how
to make it fit the way you work.

| Where you are | What you do | Key |
|---|---|---|
| A plain terminal prompt | Type a rough line. Do not press Enter. | **Ctrl-X**, then **Ctrl-P** |
| Inside Claude Code or Codex | Type your message | **Ctrl+G** |
| Anywhere | `cleanping "rough text"` | none |

Ctrl-X Ctrl-P is one shortcut pressed in two steps. It does nothing inside Claude Code or Codex, and Ctrl+G
does nothing at a plain prompt. If a key seems dead, you are probably in the other place.

## 1. Save a key

```sh
cleanping setup
```

It asks which AI to use, then for your key (shown as stars). See [Getting started](getting-started.md).

## 2. At a plain terminal prompt

1. Add this line to `~/.zshrc`, or to `~/.bashrc` (bash 4 or newer):

   ```sh
   eval "$(cleanping init zsh)"     # or: eval "$(cleanping init bash)"
   ```

2. Open a new terminal.
3. Type a rough line, then press **Ctrl-X**, then **Ctrl-P**. The line is replaced by the fixed version.
   Press the same keys again, without editing, to get your original back.

To undo: delete that line. More in [The shell key](shell-key.md).

### Use a different key

Ctrl-X Ctrl-P is the default. To use another key, set `CLEANPING_KEYBIND` on the line **above** the `eval`
line. To use **Ctrl+G**, the same key as in Claude Code and Codex:

```sh
CLEANPING_KEYBIND='^G'           # zsh
CLEANPING_KEYBIND='\C-g'         # bash
```

At a prompt, Ctrl+G normally means "cancel" (for example out of a search or a menu). With this setting it
fixes your line instead, so you lose that cancel key. Ctrl-C still abandons the line.

## 3. Inside Claude Code or Codex

Both apps open the program named in `$VISUAL` when you press Ctrl+G. There are two ways to point it at
CleanPing.

**Option A: one session at a time (recommended).** Add these lines to `~/.zshrc` or `~/.bashrc`.
`fixclaude` and `fixcodex` are only examples: call them whatever you like.

```sh
fixclaude() { VISUAL="cleanping edit" claude "$@"; }
fixcodex()  { VISUAL="cleanping edit" codex "$@"; }
```

Pick a name that is not already a command. Check with `type NAME`: if it says "not found", the name is
free. Open a new terminal, run `fixclaude` instead of `claude`, type your message and press Ctrl+G. Extra
options still work (`fixclaude --resume`). To use a specific saved key, write `cleanping edit -c work`
inside the value.

**Option B: every session.** Add `export VISUAL="cleanping edit"` to the same file. Other programs read
`$VISUAL` too. For example, git uses it when no other editor is set, so it would open CleanPing when you
write a commit message. If you do not want that, use option A.

More in [Claude Code and Codex](claude-code-and-codex.md).

## 4. Copy the result

`cleanping --copy "rough text"` also copies the result to the clipboard. It needs `wl-copy`, `xclip`, `xsel`
or `pbcopy` on the machine where CleanPing runs. Without one it still prints the result and warns that it
could not copy. Over SSH that machine is the remote one, so select the text with your terminal instead.

## When it does not work

| What you see | Why, and what to do |
|---|---|
| The command is red, or "command not found" | Your terminal has not loaded your profile. Open a new terminal, or run `source ~/.zshrc` (`source ~/.bashrc` in bash). |
| Nothing happens at the prompt | Are you inside Claude Code or Codex? Use Ctrl+G there. Otherwise check that the key is bound: `bindkey \| grep cleanping` (zsh) or `bind -X \| grep -i clean` (bash). |
| Ctrl+G opens something else in Claude Code or Codex | The app was started without CleanPing as its editor. Quit it and start it with your `fixclaude` or `fixcodex`. |

Other problems: [Troubleshooting](troubleshooting.md).
