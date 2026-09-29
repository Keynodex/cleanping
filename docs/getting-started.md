# Getting started

About five minutes, from nothing to your first rewrite.

## 1. Install

Follow the [Install section of the README](../README.md#install): a prebuilt binary for Linux and macOS,
or `cargo install` from source. Then check that it runs:

```console
$ cleanping --version
```

## 2. Run the guided setup

```console
$ cleanping setup
```

Setup needs a terminal, because it asks questions. It walks through four steps:

1. **Choose the AI.** DeepSeek, OpenAI, OpenRouter, a local model (Ollama), or a custom address. Then
   paste your API key. You see one star per character, never the key; Backspace and Ctrl-U edit, and
   Ctrl-C or Esc cancel and save nothing. A local model needs no key.
2. **Pick a system prompt.** This is the instruction the AI follows for every rewrite. The default fixes
   spelling, grammar and clarity and keeps your meaning. See [System prompts](system-prompts.md).
3. **Test the connection** (optional, and it asks first). One tiny fixed request, never your text.
4. **How to use it.** Setup prints the lines that connect CleanPing to Claude Code, Codex and your
   shell. It prints them and does not edit any of your files, so you stay in control of your profile.

Run `cleanping setup` again later and, once a key is saved, it shows a small menu instead: change the AI
or key, change the system prompt, test the connection, or see the usage lines again.

Prefer to do it by hand, or in a script? Each step is a command:

```sh
cleanping keys add --provider OpenAI      # asks for the key with a hidden prompt
cleanping keys test
cleanping prompt use concise
```

See the [command reference](commands.md).

## 3. Rewrite something

```console
$ cleanping "plz fix teh login pgae, buttons r too smal"
Please fix the login page; the buttons are too small.
```

Only the rewritten text is printed, so it works in pipes:

```sh
echo "plz fix teh login pgae" | cleanping
pbpaste | cleanping | pbcopy              # macOS: fix what is on the clipboard
cleanping --copy "rough text"             # also copies the result to the clipboard
```

## 4. Use it where you write

- **Inside Claude Code or Codex:** `export VISUAL="cleanping edit"`, then press Ctrl+G in the app.
  [Guide](claude-code-and-codex.md).
- **At your shell prompt:** add `eval "$(cleanping init zsh)"` (or `bash`) to your profile, then press
  Ctrl-X Ctrl-P on a line you have typed. [Guide](shell-key.md).

## What to know before you rely on it

- The text you rewrite is sent to the AI provider you chose, and that provider can read it. CleanPing is
  not end-to-end encrypted. If the text must never leave your computer, use a local model. See
  [Privacy and safety](privacy-and-safety.md).
- Something went wrong? See [Troubleshooting](troubleshooting.md).
