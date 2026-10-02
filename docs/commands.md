# Command reference

Every command, option, exit code and environment variable. `cleanping --help` and
`cleanping COMMAND --help` print the same information for the version you have installed.

- [Rewrite text](#rewrite-text): `cleanping [TEXT]...`
- [`edit`](#edit): fix a file's text on a screen
- [`setup`](#setup): guided setup
- [`writer`](#writer): a text-only window where nothing typed is run
- [`update`](#update): check for a newer version
- [`keys`](#keys): list, add, remove, use and test API keys
- [`prompt`](#prompt): the system prompt and its presets
- [`history`](#history): look at or delete saved rewrites
- [`init`](#init): shell integration
- [Exit codes](#exit-codes), [environment variables](#environment-variables), [files](#files)

## Rewrite text

```
cleanping [OPTIONS] [TEXT]...
```

Rewrites the text and prints only the result to stdout. Give the text as arguments (quote it, so your
shell does not act on special characters) or pipe it in. Errors go to stderr.

```console
$ cleanping "plz fix teh login pgae, buttons r too smal"
Please fix the login page; the buttons are too small.
$ echo "plz fix teh login pgae" | cleanping
```

| Option | What it does |
|---|---|
| `-c`, `--credential NAME` | Use this saved key instead of the selected one |
| `--copy` | Also copy the result to the clipboard (needs `wl-copy`, `xclip`, `xsel` or `pbcopy`) |
| `--no-history` | Do not save this run to the local history |
| `--keep-shape` | Refuse a reply with more lines than your text, much longer than it, or padded with blanks, or one that changes the command in your text (a quote closed or opened, a flag or a path changed); exit `1` (the [shell key](shell-key.md) uses this) |
| `--refuse-secrets` | Send nothing if the text looks like it holds a key, token or password; exit `2` (the shell key uses this) |

**Text or command?** If the first word is `keys`, `prompt`, `history`, `edit`, `setup`, `writer`, `update` or
`init`, it is read as that command. To rewrite text that starts with one of those words, put it after `--`
or pipe it in: `cleanping -- keys are broken`, `cleanping -- update the docs`. Any other text, including `help me fix this`, is rewritten as it is.

With a single saved key, CleanPing just uses it. With several, it uses the selected one. If several are
saved and none is selected (for example after you removed the selected one), it stops with exit code `3`
and asks you to choose (`cleanping keys use NAME` or `-c NAME`) instead of guessing, so your text cannot
go to a provider you did not pick.

Input is limited to 200,000 bytes and is checked before any request is made.

**While it waits.** A long text can take the AI a while. When a rewrite takes more than half a second and
stderr is a terminal, a line on stderr shows how it is going:

```console
Fixing your text… ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱ about 40% 12s
```

The provider answers all at once, so the percent is an **estimate** from the length of your text and the
time waited; it stops at 95% until the reply arrives. The seconds are real. The line is erased before the
result or an error is printed, and nothing of it goes to stdout. It is not shown when stderr is a pipe or
a file (scripts, the [shell key](shell-key.md)) or with `CLEANPING_PROGRESS=off`. The filled blocks are
green unless `NO_COLOR` is set; with a locale that is not UTF-8 or `TERM=dumb` the line is plain ASCII
(`[########............]`).

**A changed command is a warning.** When the reply changes a command in your text (a quote closed or
opened, a flag dropped or added, a path or URL changed), each change is printed on stderr and the reply is
still printed, with the usual exit code:

```console
$ cleanping < question.txt
Why is it waiting?
curl -X POST http://localhost:8080/items -d '{"name": "lamp"}'
cleanping: warning: a quote was closed in the command starting `curl -X POST`.
```

With `--keep-shape` the reply is refused instead (exit `1`). Which lines count as commands is in
[Privacy and safety](privacy-and-safety.md#a-reply-that-changes-a-command).

## `edit`

```
cleanping edit [-c NAME] [--yes] FILE
```

Fixes the text in `FILE` with the AI and lets you review it on a screen. Set `VISUAL="cleanping edit"` so
Claude Code and Codex open it with Ctrl+G. The full guide is
[Claude Code and Codex](claude-code-and-codex.md).

| Option | What it does |
|---|---|
| `-c`, `--credential NAME` | Use this saved key instead of the selected one |
| `--yes` | No screen and no question: write the edited text into the file. Text that looks like it holds a secret is still never sent |

When the edit changes a command in your text, the screen says `Check the command: ...` under the title
(Enter still accepts it), and `--yes` prints the same as a `cleanping: warning:` line on stderr.

While the edit is on its way, after half a second the screen shows the same
[estimate](#rewrite-text) under the title: a green bar and `about 40% 12s`. `--yes` in a terminal shows
the line on stderr and erases it before anything else is printed.

On the screen, a failed edit exits `0` with the file untouched, so the app that opened it keeps your text.
With `--yes` the normal [exit codes](#exit-codes) apply. Options go before the file (that is how the apps
call it: the file is always last). With no terminal and no `--yes` there is nowhere to show the screen, so
`edit` says so and exits `1`.

## `setup`

```
cleanping setup
```

The guided setup: choose the AI, save its key, pick a system prompt, test the connection, and see how to
use CleanPing. It needs a terminal and exits `2` without one. With a key already saved it shows a menu
instead. It prints profile lines and never edits your files, and it never runs `ollama pull` without
asking. See [Getting started](getting-started.md).

## `writer`

```
cleanping writer
```

A text-only mode for people who have never used a shell: an interactive zsh where typed text is never
run. **Ctrl+G** fixes the line in place, **Enter** copies it to the clipboard (with `pbcopy`, `wl-copy`,
`xclip` or `xsel`, the first one found; if none works it asks your terminal to copy, which also works over
SSH), **Ctrl+D** on an empty line leaves. It needs a terminal for both
input and output (exit `2` without one) and zsh (exit `1` without it). It starts zsh with `-d -i` and a
private `.zshrc` in `writer/` in the data folder, rewritten on every start, so none of your own zsh startup
files are read. The exit status is zsh's. It never edits your files. It is a guard against accidents, not a
security sandbox. The guide is [Use it your way](use-it-your-way.md#5-writing-mode-for-text-only).

## `update`

```
cleanping update [--check]
```

Asks GitHub's public API for the newest CleanPing release and compares it with the version you are
running. It prints one of:

- `CleanPing 0.4.0 is the latest version.`
- `CleanPing 0.5.0 is available (you have 0.4.0).`, the release page, the commands to install it on a Mac
  (the [README](../README.md#install) steps for your Mac's archive, with `shasum -a 256 -c`) or, on other
  systems, a pointer to the README install section, and `Nothing was changed: this command only checks
  for a newer version.`
- that your version is newer than the latest release (a development build), with no install steps.

It exits `0` whether or not an update exists, and `1` when GitHub cannot be reached or its reply is not
understood (see [Troubleshooting](troubleshooting.md#checking-for-updates)). It only checks: nothing is
downloaded, installed or changed, and no file or database is opened. Installing the update for you is
planned; `--check` does exactly what `update` does today and stays valid when that arrives. What the check
sends is in [Privacy and safety](privacy-and-safety.md#checking-for-updates).

## `keys`

| Command | What it does |
|---|---|
| `cleanping keys list` | List saved keys, one per line. `*` marks the selected key. The key itself is never shown |
| `cleanping keys add [OPTIONS]` | Save a key |
| `cleanping keys use NAME` | Make a saved key the default |
| `cleanping keys remove NAME` | Delete a saved key (history that used it is kept) |
| `cleanping keys test [NAME]` | Send one tiny fixed request to check that a key works (default: the selected key) |

### `keys add`

| Option | What it does |
|---|---|
| `--provider PRESET` | `DeepSeek`, `OpenAI`, `OpenRouter` or `"Ollama (local)"`: fills in the address and a default model |
| `--name NAME` | A name for this key (defaults to the provider) |
| `--url URL` | The API address, for a provider that has no preset. It must be `https://`, or `http://` for `localhost`, `127.0.0.1` or `::1`, and must not have a query string or fragment (`?...` or `#...`) |
| `--model MODEL` | The model name (defaults to the provider's) |
| `--key-stdin` | Read the key as one line from stdin. On a terminal it is asked for with a hidden prompt |

The key is never taken from an argument, so it does not end up in your shell history. Piping one in:

```sh
printf '%s\n' "$OPENAI_API_KEY" | cleanping keys add --provider OpenAI --key-stdin
```

A local model needs no real key: `cleanping keys add --provider "Ollama (local)"`. See
[Providers and local models](providers-and-local-models.md).

### `keys test`

Prints `OK: "NAME" answered in N ms (model M).` and exits `0`, or says what failed and exits `1`. It exits
`3` when no key is saved. It sends a fixed word, never your text, and saves nothing to the history. For a
local address a failure can also say what to do: download the model, or, on the standard Ollama address
(`127.0.0.1:11434`), install or start Ollama.

## `prompt`

The system prompt is the instruction sent with every rewrite. See [System prompts](system-prompts.md).

| Command | What it does |
|---|---|
| `cleanping prompt show` | Print the current system prompt |
| `cleanping prompt set [TEXT]` | Replace it with `TEXT`, or with stdin when `TEXT` is left out and input is piped |
| `cleanping prompt presets` | List the ready-made prompts (`default`, `typos`, `concise`, `friendly`, `structure`); `*` marks the one in use, and a last line says when yours is an [earlier version](system-prompts.md#the-default-prompt) of it |
| `cleanping prompt use NAME [--yes]` | Switch to a preset. Replacing a prompt you wrote yourself needs `--yes` |

## `history`

Each rewrite is saved locally (your text, the result, the system prompt and the model), except with
`--no-history`, and never from the shell key or `edit`. See [Privacy and safety](privacy-and-safety.md).

| Command | What it does |
|---|---|
| `cleanping history list [--limit N]` | Show the newest rewrites, one line each (default 20). Times are UTC |
| `cleanping history clear --yes` | Delete every saved rewrite |
| `cleanping history purge --older-than DAYS` | Delete rewrites older than `DAYS` days (1 to 36500) |

Deleted text is overwritten and the database file is rebuilt, so it cannot be read back from the file.

## `init`

```
cleanping init zsh|bash
```

Prints the shell integration. Add `eval "$(cleanping init zsh)"` to `~/.zshrc`, or
`eval "$(cleanping init bash)"` to `~/.bashrc`. See [The shell key](shell-key.md).

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Success |
| `1` | The request failed: network, provider (including a reply the provider cut off at its length limit) or storage, or `update` could not check for a newer version |
| `2` | Bad input: empty or too long text, an invalid address or option, text refused by `--refuse-secrets`, or `setup` or `writer` without a terminal |
| `3` | No usable key: none saved, several saved and none selected, or the name you gave does not exist |

One exception: on the `edit` screen a failed edit shows why and exits `0`, so the app that opened the file
keeps your text.

## Environment variables

| Variable | Used for |
|---|---|
| `XDG_CONFIG_HOME`, `XDG_DATA_HOME` | Where keys and history live (see below). Relative paths are ignored |
| `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, `NO_PROXY` | Proxy for remote providers and for `cleanping update`. A proxy that is set but unusable is an error, never a silent direct connection. Local addresses are always reached directly |
| `CLEANPING_KEYBIND` | The shell key. Set it before the `eval` line |
| `CLEANPING_HIGHLIGHT` | zsh only: the style for changed words (default `standout`; empty turns it off) |
| `CLEANPING_PROGRESS` | `off` (or `0`) hides the [progress line](#rewrite-text) shown on stderr while a slow rewrite waits |
| `NO_COLOR` | Any value that is not empty: the progress line is drawn without color |
| `LC_ALL`, `LC_CTYPE`, `LANG`, `TERM` | The first locale variable that is set decides the progress line's characters: `▰▱` for UTF-8, ASCII otherwise and with `TERM=dumb` |
| `VISUAL` | Read by Claude Code and Codex, not by CleanPing: `export VISUAL="cleanping edit"` |
| `SHELL` | `cleanping setup` uses it to show the right profile file |
| `PATH` | Searched for the `ollama` program, to tell "not installed" from "not running" when a local server does not answer |
| `CLEANPING_UPDATE_URL` | **Tests only.** Replaces the address `cleanping update` asks, and is used only when it is `http://` to `localhost`, `127.0.0.1` or `[::1]` (without a user name or password); any other value is ignored, so it can never send the check to another computer |

## Files

| What | Where | Mode |
|---|---|---|
| API keys | `~/.config/cleanping/secrets.json` (and a `secrets.json.lock` beside it) | `0600`; directory `0700` |
| History, saved prompt, state | `~/.local/share/cleanping/cleanping.db` (SQLite) | `0600`; directory `0700` |
| Startup file of `cleanping writer` | `~/.local/share/cleanping/writer/.zshrc`, rewritten on every start | `0600`; directory `0700` |
