# Fix your message inside Claude Code and Codex

You type a rough message in Claude Code or Codex, press **Ctrl+G**, and CleanPing fixes it before you
send it. You see what changed and decide.

## Set it up

Both apps open the program named in `$VISUAL` when you press Ctrl+G ("edit in editor"). Point it at
CleanPing, for example in `~/.zshrc` or `~/.bashrc`, and open a new terminal:

```sh
export VISUAL="cleanping edit"
```

`cleanping setup` prints this line for you at the end. You need a saved key first (see
[Getting started](getting-started.md)). To use a specific key, name it before the file:
`export VISUAL="cleanping edit -c work"`. The apps split the value on spaces and add the file last, so
options belong inside the value.

## Use it

1. Type your message in the app.
2. Press **Ctrl+G**. CleanPing opens on your terminal and lights up your text while the AI works
   (**Esc** cancels and keeps your text).
3. When it is done you see the edited text with the words the AI changed highlighted.

| Key | What it does |
|---|---|
| **Enter** | Accept: your message in the app becomes the edited text |
| **N** or **Esc** | Keep your original text |
| **O** | Switch between the edited text and your original |
| **↑ ↓ Page Up Page Down** | Scroll a long message |

## When something goes wrong

- **The edit failed** (no key saved, no network, the provider said no): the screen says **Could not edit**
  and why, then shows your text unchanged. Press any key and you are back in the app with your text
  exactly as it was. The command exits `0` on purpose, as a precaution, so the app has no reason to treat
  the editor as failed (see [What has been tested](#what-has-been-tested)).
- **The text looks like it holds a secret** (an API key, token, private key or password): the screen
  says **Not sent**, names the *kind* of secret (never showing it), and waits. **S** sends it anyway if
  you choose to; any other key keeps your text. The check is a best guess, so it does not replace your
  own care about what you paste.

Nothing you edit here is saved to the history.

## Why it draws on the terminal directly

Claude Code and Codex start the editor with your terminal as its input, but not as its output. A screen
that drew on standard output would not reach you. CleanPing opens the terminal itself (`/dev/tty`), so the
screen appears anyway.

## Editing a file without a screen

For a script, or to check what an edit would do:

```sh
cleanping edit --yes notes.txt
```

writes the edited text into `notes.txt` with no screen and no question. Text that looks like it holds a
secret is still never sent. Without `--yes` and without a terminal, `edit` exits `1` and says so.

## What has been tested

- Claude Code and Codex on Linux: Ctrl+G opens CleanPing, and after **Enter** the app's next editor is
  handed the edited text; after **N** it is handed your original.
- Other programs that run `$VISUAL FILE` should behave the same way, but have not been tested.
  **Grok Build** is one of them and is next on the [roadmap](../README.md#roadmap).
- Whether an app throws your text away when its editor exits with an error is not verified, which is why a
  failed edit exits `0`.
- Windows is not supported yet.
