# Changelog

All notable changes to CleanPing are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/) while it is below 1.0: a minor version (0.x) may change
behavior, and the notes below say when it does.

## [Unreleased]

### Added

- **A reply that changes a command is caught.** Models asked to fix spelling often "fix" a command too,
  most often by closing a quote you left open, which hides the very bug you were asking about. CleanPing
  now compares the commands in your text with the reply: a quote closed or opened, a flag dropped or
  added, a path or URL changed. The shell key, Ctrl+G in `cleanping writer`, and anything else using
  `--keep-shape` refuse such a reply (your line stays, exit `1`); `cleanping edit` shows
  `Check the command: ...` on its screen (and `--yes` warns on stderr); the plain command prints
  `cleanping: warning: ...` on stderr and still prints the reply with the same exit code. It is a safety
  net with false negatives and a few false positives, not a proof; see
  [Privacy and safety](docs/privacy-and-safety.md#a-reply-that-changes-a-command).

## [0.4.0] - 2026-09-30

### Added

- `cleanping writer`: a text-only window for people who have never used a shell. Nothing you type there is
  ever run. Press Ctrl+G to fix your text in place, Enter to copy it to the clipboard, Ctrl+D to leave. It
  needs zsh and a terminal, reads none of your own zsh startup files, and never edits your files. It
  guards against accidents and is not a security sandbox. To open it in every new zsh window, add
  `[[ -o interactive ]] && command -v cleanping >/dev/null && exec cleanping writer` to the end of
  `~/.zshrc` yourself; see
  [Use it your way](docs/use-it-your-way.md#5-writing-mode-for-text-only).
- Documentation: [Use it your way](docs/use-it-your-way.md) shows which key works where (the shell prompt,
  Claude Code and Codex, the plain command), how to name your own launcher commands, how to use a different
  shell key such as Ctrl+G, and what to do when a key does nothing.

### Changed

- **`writer` is now a command word**, like `edit` and `setup`. Text that starts with it must be given after
  `--` or piped in: `cleanping -- writer notes`.

## [0.3.0] - 2026-09-29

### Added

- `cleanping edit FILE`: fix a message inside Claude Code and Codex. Set `VISUAL="cleanping edit"` and
  press their editor key (Ctrl+G). A screen shows the result with the changed words highlighted; Enter
  accepts it, N or Esc keeps your text, O switches between the two. See
  [docs/claude-code-and-codex.md](docs/claude-code-and-codex.md).
- The zsh shell key highlights the words the AI changed. Bash prints your original on its own line
  (`cleanping: was: ...`) so you can compare.
- `--refuse-secrets`: text that looks like it holds an API key, token, private key or password is not
  sent (exit code `2`). The shell key uses it, and `cleanping edit` applies the same check.
- `cleanping setup`: a guided first run (choose the AI, save its key, pick a system prompt, test the
  connection, see how to use it) and, once a key is saved, a small menu to change any of those.
- `cleanping keys test`: sends one tiny fixed request to check that a saved key works. It never sends
  your text and saves nothing to the history.
- `cleanping prompt presets` and `cleanping prompt use NAME`: ready-made system prompts (`default`,
  `typos`, `concise`, `friendly`).
- Local-model help for Ollama: a failed test says whether to install Ollama, start it, or download the
  model, and `cleanping setup` offers to run `ollama pull` after asking. CleanPing never installs
  Ollama or runs anything without a yes.
- Documentation: the `docs/` folder, `CONTRIBUTING.md`, and doc comments on every public item of
  `cleanping-core` (the build now warns when one is missing).

### Changed

- **`edit` and `setup` are now command words**, like `keys` and `history`. Text that starts with either
  word must be given after `--` or piped in: `cleanping -- setup my laptop`.
- The hidden key prompt shows one star per character. Backspace and Ctrl-U edit; Ctrl-C or Esc cancel
  and save nothing, and the terminal is always restored (the previous prompt could leave typing hidden
  after Ctrl-C in some shells).
- The "no key saved" message now also points to `cleanping setup`.
- **`cleanping keys add` refuses an address with a query string or fragment** (`?...` or `#...`), so a
  key cannot end up in the address, where `keys list` would print it and the database would keep it as
  plain text. It exits with code `2`. Addresses saved earlier keep working; remove and add them again
  without the query string to clear it.
- The product is written **CleanPing** in prose; the command, crate names, paths and URLs stay lowercase.

## [0.2.0] - 2026-09-29

First public release.

- Rewrite text from arguments or a pipe; only the result goes to stdout; exit codes `0` to `3`.
- Shell key for zsh and bash: rewrite the line you are typing, press again to restore your original.
- Keys saved in a file only you can read, never taken from a command-line argument.
- Provider replies are treated as untrusted: control and invisible characters are removed, and the shell
  key refuses a reply that could hide part of itself (`--keep-shape`).
- A local history you can list, clear and purge; deleted text is overwritten.
- Linux (x86_64) and macOS (Apple silicon and Intel) builds with checksums and build-provenance
  attestations.

[Unreleased]: https://github.com/Keynodex/cleanping/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/Keynodex/cleanping/releases/tag/v0.4.0
[0.3.0]: https://github.com/Keynodex/cleanping/releases/tag/v0.3.0
[0.2.0]: https://github.com/Keynodex/cleanping/releases/tag/v0.2.0
