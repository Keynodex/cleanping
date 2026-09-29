# CleanPing — agent notes

Prompt polisher: rewrites rough text with an OpenAI-compatible API. Rust workspace (CLI + shell key)
with Clean Architecture layout; the older tkinter desktop window still lives in `legacy/python/`
until it is ported. See `README.md` for users.

## Build, check, test (Rust)

```sh
cargo build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Shell-key tests need `zsh` and `script` (Linux). Locally they skip when a shell is missing; with
`CI` set they fail instead, so they cannot pass without running. They wait for the shell's prompt
(`tests/support/pty.rs`) and then a short settle; do not use sleeps as the way to synchronize.
Minimum Rust is 1.89 (`rust-version`, checked in CI).

## Layout (Rust)

- `crates/cleanping-core/src/domain` — entities and invariants, no I/O (`VersionStack`, URL rules, presets)
- `crates/cleanping-core/src/application` — use cases and ports (traits); fakes live in `test_support.rs`
- `crates/cleanping-core/src/infrastructure` — SQLite, secrets file, HTTP (ureq), paths, clock
- `crates/cleanping-cli` — the `cleanping` binary: `args`, `rewrite`, `keys`, `prompt`, `history`, `init`
  (shell scripts in `shell/`), `guard` (the refuse-if-it-looks-like-a-secret check), `marks` (hidden
  `--marks`, see below), `connection` (`keys test`); all stdout goes through `output` (a closed pipe
  is not a panic)
- Local models: `domain/local_server.rs` decides what to suggest (install, start, pull the model; install
  and start advice only for the standard Ollama address, port 11434); `infrastructure/ollama.rs` asks
  `GET /api/tags` on a loopback address only (no redirects, no proxy, 3 s, 1 MB). `keys test` sends fixed
  words only (`ConnectionCheck`), never the user's text, and never saves to the history.
- `crates/cleanping-cli/src/edit` — `cleanping edit FILE` (the `$VISUAL` screen): `view` + `layout` + `wrap`
  are pure (state and keys in, styled lines out), `terminal` is crossterm on `/dev/tty` only (never
  stdin/stdout: a host app pipes them), `session` is the key/reply loop, `job` the background request
  (never saved to history), `file` reads and writes the host's temp file
- `crates/cleanping-cli/src/setup` — `cleanping setup`: every question goes through the `Console` trait
  and every outside contact (ask a local server, `ollama pull`, the connection test) through `Tools`, so
  the steps (`provider`, `system_prompt`, `check`, `flow`, `usage`) are tested with scripted answers;
  `tests/setup.rs` also drives the real binary in a terminal. Rules: keys are read with the hidden prompt
  and never printed; nothing is downloaded or run without a yes; a model name starting with `-` is never
  passed to `ollama`; setup prints the profile lines and never edits the user's files; tests must never
  reach a real provider (answer "n" to the connection test unless a fake server is used).

Dependencies point inward: infrastructure -> application -> domain. Keep files under ~150 lines.

## Rules that matter here

- Naming: the product is **CleanPing** in prose (docs, help text, release notes). The command, crate
  names, file paths, URLs and the `cleanping:` message prefix stay lowercase.

- Tests first; ported behavior is specified by the Python tests in `legacy/python/tests`.
- Never print, log or pass API keys on a command line. `CredentialInput`'s `Debug` redacts the key.
- Never follow HTTP redirects, never echo provider response bodies in errors.
- Provider replies are untrusted: `clean_reply` strips control and invisible characters, and the
  shell key passes `--keep-shape` (`domain/shape.rs`) so a reply cannot add lines, pad itself or
  grow much longer than the text it replaces on a command line.
- The shell key also passes `--refuse-secrets` (`guard.rs`): a command line that looks like it holds a
  secret is never sent. Plain `cleanping TEXT` only checks when that flag is given.
- Hidden `--marks` is how zsh highlights changes: stdin is `original NUL reply`; stdout is the reply's
  length in characters, then a `start end` line (characters) per changed range, or nothing if the texts
  are too large to compare. It must stay pure (no database, files or network; a test checks). The zsh
  script highlights only when `${#reply}` equals that length, so a non-UTF-8 locale (zsh counts bytes)
  gets no highlight instead of the wrong one.
- A proxy variable that is set but unusable is an error (`infrastructure/proxy_env.rs`), never a
  silent direct connection. Deleted history is overwritten and the database rebuilt.
- `edit` sends nothing that `find_secret` flags unless the user presses S on the screen, and never with
  `--yes`; the screen names the kind of secret, never the secret. A failed edit on the screen exits 0
  with the file untouched, so the host app keeps the user's text.
- Never guess which key to use (`CredentialService::resolve`); an exact name beats a case variant.
- `.env*` and secret files are never read by tools or committed; the test fixture DB holds fake data only.

## Python desktop window (legacy, until ported)

```sh
cd legacy/python
python3 -m pytest        # 95 tests; the UI smoke test needs a display
bin/cleanping            # start the window
```

- `legacy/python/src/cleanping/{domain,application,infrastructure,ui}` — same layering; `ui` is tkinter views only
- CI does not run these tests; the window is being ported to Rust.

## Desktop window shortcuts (legacy)

Enter rewrite (replaces the text; earlier versions are kept) · Shift+Enter newline · Alt+O original⇄latest · Alt+Z back · Alt+Shift+Z forward · Alt+C clear (undoable) · Alt+K keys · Alt+P system prompt · Alt+T light/dark · Ctrl+Shift+C copy · Ctrl+S save in dialogs · Esc close dialog

## Runtime data (never in git)

- DB: `~/.local/share/cleanping/cleanping.db`
- Secrets: `~/.config/cleanping/secrets.json` (mode 0600; API keys never displayed or logged)
