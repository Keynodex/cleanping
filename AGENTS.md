# cleanping — agent notes

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
  (shell scripts in `shell/`); all stdout goes through `output` (a closed pipe is not a panic)

Dependencies point inward: infrastructure -> application -> domain. Keep files under ~150 lines.

## Rules that matter here

- Tests first; ported behavior is specified by the Python tests in `legacy/python/tests`.
- Never print, log or pass API keys on a command line. `CredentialInput`'s `Debug` redacts the key.
- Never follow HTTP redirects, never echo provider response bodies in errors.
- Provider replies are untrusted: `clean_reply` strips control and invisible characters, and the
  shell key passes `--keep-shape` (`domain/shape.rs`) so a reply cannot add lines, pad itself or
  grow much longer than the text it replaces on a command line.
- A proxy variable that is set but unusable is an error (`infrastructure/proxy_env.rs`), never a
  silent direct connection. Deleted history is overwritten and the database rebuilt.
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
