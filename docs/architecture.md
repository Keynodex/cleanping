# Architecture

A map of the code for people who want to change it. The rules contributors follow are in
[CONTRIBUTING.md](../CONTRIBUTING.md) and [AGENTS.md](../AGENTS.md). The API of the library crate is
documented in its doc comments (`cargo doc --workspace --no-deps --open`).

## Two crates, three layers

```
cleanping-cli    the `cleanping` binary: arguments, screens, shell scripts
    │ uses
cleanping-core   the library, in three layers whose dependencies point inward
    infrastructure  ──▶  application  ──▶  domain
    (SQLite, files,       (use cases,        (rules and data,
     HTTP, clock)          ports)             no I/O at all)
```

| Layer | Owns | Must not own |
|---|---|---|
| `domain` | Rules and data: what a key or a run is, address rules, the secret check, the reply-shape rule, word diffs, prompt and provider presets | HTTP, files, SQL, the terminal |
| `application` | Use cases, and the **ports** (traits) they need: `Rewriter`, `SecretStore`, `CredentialRepository`, `RunRepository`, `PromptRepository`, `StateRepository` | Concrete storage or network code |
| `infrastructure` | Adapters that implement the ports: SQLite repositories, the JSON key file, the HTTP rewriter (`ureq`), the Ollama probe, paths, the clock, proxy settings | Product rules |
| `cleanping-cli` | Turning arguments and keystrokes into use-case calls, and results into text on a screen | Business rules, SQL |

Fakes for the ports live in `application/test_support.rs`, so use cases are tested without a database or a
network.

## Where things are

`crates/cleanping-core/src`

| Module | What it holds |
|---|---|
| `domain/models.rs`, `errors.rs`, `validation.rs` | The data types, the error type (its variants decide the exit code), and address and key rules |
| `domain/secret_scan.rs`, `shape.rs`, `sanitize.rs` | The "looks like a secret" check, the "reply must look like the text" rule, and cleaning of replies |
| `domain/command_change.rs`, `command_lines.rs`, `command_pairs.rs`, `command_match.rs`, `shell_words.rs` | The "reply changed a command" check: which lines are commands, which reply line each became, what changed (quotes, flags, paths), and splitting a command into words the way a shell quotes them |
| `domain/diff.rs` | The word diff behind the highlighting (`Diff::changed_ranges`) |
| `domain/progress.rs` | The estimate of how far a rewrite has got, from the text's size and the time waited (`estimate_progress`); its timing guesses are in `progress::timing` |
| `domain/providers.rs`, `prompt_presets.rs`, `local_server.rs` | Provider and prompt presets, and what to suggest for a local model that is not ready |
| `application/polisher.rs` | The rewrite use case (`PolishText`) |
| `application/credentials.rs`, `prompts.rs`, `history.rs`, `app_state.rs`, `connection_check.rs` | The other use cases |
| `infrastructure/http_rewriter.rs`, `ollama.rs`, `proxy_env.rs` | Everything that touches the network |
| `infrastructure/sqlite_db.rs`, `sqlite_repositories*`, `secrets_file.rs`, `private_fs.rs`, `paths.rs` | Everything that touches disk |

`crates/cleanping-cli/src`

| Module | What it holds |
|---|---|
| `args.rs`, `main.rs`, `services.rs`, `exit.rs`, `output.rs` | The command line (clap), the wiring of services, the exit codes, and stdout that treats a closed pipe as normal |
| `rewrite.rs`, `keys.rs`, `prompt.rs`, `history.rs`, `connection.rs`, `init.rs` | One module per command |
| `guard.rs`, `marks.rs`, `hints.rs`, `no_history.rs` | The secret refusal, the hidden `--marks` output, copy-pasteable commands for error messages, and the "never save this" switch |
| `command_warnings.rs` | The `cleanping: warning:` lines when a reply changed a command (the plain command and `edit --yes`) |
| `secret_input.rs` | The hidden key prompt (raw mode on `/dev/tty`, one star per character) |
| `progress/` | The line on stderr while a rewrite waits: `bar` (the text, pure), `look` (whether and how to draw it, from the environment; pure), `ticker` (the thread that draws it and erases it when dropped) |
| `edit/` | `cleanping edit`: `view`, `layout`, `estimate` and `wrap` are pure (state and keys in, styled lines out); `terminal` is the only file that touches the terminal; `session` is the key and reply loop; `job` runs the request in the background; `file` reads and writes the host's temp file |
| `setup/` | `cleanping setup`: every question goes through a `Console` trait and every outside contact through a `Tools` trait, so the steps (`provider`, `system_prompt`, `check`, `flow`, `usage`) are tested with scripted answers |
| `writer/` | `cleanping writer`: `terminal` checks for a terminal, `rc` writes the private startup file, `launch` builds the PATH and replaces the process with zsh; the behavior lives in `shell/writer.zsh` |
| `shell/cleanping.zsh`, `cleanping.bash` | The shell key, printed by `cleanping init` |
| `shell/writer.zsh` | The startup file of the writer's zsh: loads the shell key on Ctrl+G, makes every accept-line key copy instead of run |

## How a rewrite flows

1. `args.rs` parses the command line; `rewrite.rs` reads the text (arguments or stdin, capped at 200,000
   bytes).
2. With `--refuse-secrets`, `guard.rs` asks `domain::secret_scan` first and stops before anything is sent.
3. `CredentialService` picks the key (never guessing between several) and the `PromptService` supplies the
   system prompt.
4. `PolishText` reads the key from the `SecretStore` and calls the `Rewriter` port. The HTTP adapter sends
   the request without following redirects, caps the reply at 1 MB, and cleans the reply (`clean_reply` in
   `domain/sanitize.rs`: control, direction-override and invisible characters are removed).
5. With `--keep-shape`, a reply that does not pass the shape rule, or that changes the command
   (`domain::command_change::command_line_changes`), counts as a failure. Either way the
   attempt is recorded through `RunRepository`, failures included (with the error message), unless
   `--no-history` swaps in a repository that saves nothing (`no_history.rs`).
6. Only the result is written to stdout. Without `--keep-shape`, each change the reply made to a command
   in the text (`command_changes`) is a warning on stderr.

## Design decisions worth knowing

- **The terminal is `/dev/tty`, not stdin or stdout.** Claude Code and Codex give the editor a tty on
  stdin but capture stdout, so the edit screen and the key prompt open `/dev/tty` themselves.
- **The key prompt is our own.** A prompt library that only turns echo off leaves typing hidden after
  Ctrl-C in a plain `sh`. `secret_input.rs` reads in raw mode, treats Ctrl-C as a key, and always restores
  the terminal.
- **Highlighting is computed in Rust, applied in the shell.** The zsh script calls the hidden
  `cleanping --marks`: stdin is `original NUL reply`; stdout is the reply's length in characters, then a
  `start end` line per changed range. It is pure (no database, files or network) and is not a stable
  interface. zsh counts bytes in a non-UTF-8 locale, so the script highlights only when its own length of
  the reply equals the count from `--marks`.
- **Nothing is trusted from the outside.** Replies are cleaned and size-capped, redirects are refused,
  proxy settings that cannot be honored are an error, and error messages never include a response body.
- **Failure on the edit screen exits 0**, leaving the file untouched, so a host app has no reason to
  discard the user's text.

## Tests

- Unit tests sit next to the code in `*_tests.rs` files (pulled in with `#[path]`), so production files
  stay small.
- Integration tests in `crates/cleanping-cli/tests` run the real binary against a fake local API server
  (`tests/support`). Terminal tests (`edit.rs`, `setup.rs`, `hidden_key.rs`, `shell.rs`, `shell_marks.rs`, `writer.rs`, `writer_keys.rs`)
  drive real interactive sessions through a pseudo-terminal with `script`; they are Linux-only. See
  [CONTRIBUTING.md](../CONTRIBUTING.md#tests).

## Legacy

`legacy/python` is the earlier desktop window (Python, tkinter). It reads the same database and key file and
is kept until it is ported to Rust. CI does not run its tests.
