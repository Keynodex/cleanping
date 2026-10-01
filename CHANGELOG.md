# Changelog

All notable changes to CleanPing are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/) while it is below 1.0: a minor version (0.x) may change
behavior, and the notes below say when it does.

## [Unreleased]

### Added

- `cleanping update` tells you whether a newer CleanPing release exists and how to install it: on a Mac,
  the two installer commands from the latest release, then the manual archive steps as the alternative;
  elsewhere, the release page and the README install steps. It only checks and prints: nothing is downloaded, installed or changed. Installing the update for
  you is planned. It contacts GitHub only when you run it, never by itself, and sends nothing but a
  `User-Agent: cleanping/VERSION` header; see
  [Privacy and safety](docs/privacy-and-safety.md#checking-for-updates). `cleanping update --check` does
  the same and will keep doing so when installing is added.

### Changed

- **`update` is now a command word**, like `writer` and `edit`. Text that starts with it must be given
  after `--` or piped in: `cleanping -- update the docs`.

- **A reply that changes a command is caught.** Models asked to fix spelling often "fix" a command too,
  most often by closing a quote you left open, which hides the very bug you were asking about. CleanPing
  now compares the commands in your text with the reply: a quote closed or opened, a flag dropped or
  added, a path or URL changed. The shell key, Ctrl+G in `cleanping writer`, and anything else using
  `--keep-shape` refuse such a reply (your line stays, exit `1`); `cleanping edit` shows
  `Check the command: ...` on its screen (and `--yes` warns on stderr); the plain command prints
  `cleanping: warning: ...` on stderr and still prints the reply with the same exit code. It is a safety
  net with false negatives and a few false positives, not a proof; see
  [Privacy and safety](docs/privacy-and-safety.md#a-reply-that-changes-a-command).
- **You can see a slow rewrite working.** A long text can take the AI a while, and nothing showed. Now,
  after 1.5 seconds, a line on stderr shows `Fixing your text… ▰▰▰▰▱▱▱▱ about 40% 12s`, and
  `cleanping edit` shows the same bar under its title. The provider answers all at once, so the percent
  is an estimate from the length of your text; it stops at 95% until the reply arrives, and the seconds
  are real. The line is erased before the result or an error, never goes to stdout, and is not shown in
  pipes, scripts or the shell key. `CLEANPING_PROGRESS=off` hides it, `NO_COLOR` drops the green, and a
  terminal that is not UTF-8 gets ASCII. See [Command reference](docs/commands.md#rewrite-text).
- **The progress bar's timing guess now matches a fast provider**: about 1 second plus 0.9 seconds per
  kilobyte of text, from timings measured with DeepSeek flash (thinking off). Before, it guessed a slow
  provider, so a quick rewrite finished while the bar showed under 20%. It is still an estimate: with a
  slower provider (a thinking model, a local model) the bar waits longer at 95%.
- **The `default` system prompt now fixes garbled and badly misspelled words** that it used to leave
  alone, such as "sdf is now evn the way to do I dwnt se the profes bar at allll". It asks the AI to work
  out each intended word from the sentence around it, and carries one worked example. If you already have
  the old default saved, run `cleanping prompt use default` to switch (no `--yes` needed);
  `cleanping prompt presets` reminds you ([the default prompt](docs/system-prompts.md#the-default-prompt)).

- A Mac installer on every release. Download `install-cleanping-mac.sh` from the latest release and run it
  with `sh install-cleanping-mac.sh`. It picks the build for your Mac and checks its SHA-256 checksum,
  installing nothing if the checksum does not match. It installs `cleanping` into `~/.local/bin` and adds
  one `PATH` line to `~/.zshrc` if that line is not already there. It comes with its own `.sha256` file and
  a build-provenance attestation. See [Install](README.md#install).

### Changed

- **Your text now goes to the AI inside `<draft>` tags, with a fixed sentence after your system prompt**
  saying to edit only that text and never answer it, carry it out or translate it. In a blind test this
  stopped models from answering a text instead of editing it, for every provider tried. Tags the AI copies
  into its reply are removed. Your saved prompt and the history are unchanged, and a text that itself
  contains `<draft>` or `</draft>` is sent as before
  ([what CleanPing adds](docs/system-prompts.md#what-cleanping-adds-to-every-request)).
- **DeepSeek flash models no longer think before they rewrite.** For `deepseek-flash` models on
  `api.deepseek.com`, CleanPing turns thinking off: the same rewrite went from 13 to 44 seconds to about 5,
  without running out of room. Other models and providers are unchanged, and this is not configurable yet
  ([providers](docs/providers-and-local-models.md#models-that-think-before-they-answer)).

### Fixed

- **A reply the provider cut off at its length limit is refused**, instead of being used as if it were the
  whole rewrite. This happened most with models that think before they answer, on long texts. The command
  exits `1` with nothing on stdout, the shell key and `edit` leave your text as it was, and the message
  says what to try ([troubleshooting](docs/troubleshooting.md#the-provider)).

### Added

- A `structure` system prompt: after `cleanping prompt use structure`, rewrites ask the AI to fix a draft,
  lay it out as a clear prompt for an AI and clean junk out of pasted terminal text. Best with
  `cleanping edit` (Ctrl+G); see
  [System prompts](docs/system-prompts.md#structure-lay-a-draft-out-as-a-prompt-for-an-ai).

### Changed

- The DeepSeek preset now uses the model `deepseek-flash`, the name DeepSeek lists as current, instead
  of `deepseek-chat`. Keys saved earlier keep their model. To switch one, run
  `cleanping keys add --provider DeepSeek --name DeepSeek --model deepseek-flash` and press Enter at the
  key prompt to keep the saved key. Choosing DeepSeek again in `cleanping setup` also switches it.

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
