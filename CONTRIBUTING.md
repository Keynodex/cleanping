# Contributing to CleanPing

Thank you for helping. Small fixes are welcome as a pull request straight away. For anything larger (a new
command, a change in behavior), please open an issue first so we can agree on the shape before you spend
time on it. To report a security problem, do **not** open an issue: see [SECURITY.md](SECURITY.md).

CleanPing handles API keys and sends people's text to an AI provider, so changes are judged on safety
first (see [What must stay true](#what-must-stay-true)), then on correctness, then on structure.

## Set up

You need [Rust](https://rustup.rs) 1.89 or newer (`rust-toolchain.toml` selects the stable toolchain with
`clippy` and `rustfmt`). The terminal and shell-key tests also need `zsh` and `script` (Linux).

```sh
git clone https://github.com/Keynodex/cleanping
cd cleanping
cargo build
```

## Before you open a pull request

These are the checks CI runs. Run them locally:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
```

CI also builds with the oldest supported Rust (1.89, `rust-version` in `Cargo.toml`) and runs the tests on
Ubuntu and macOS. If you use a language feature newer than 1.89, that job fails.

## Tests

Behavior changes come with tests, written first: see the test fail for the reason you expect, make it
pass, then tidy.

- **Unit tests** sit beside the code in `*_tests.rs` files, pulled in with `#[cfg(test)] #[path = "..."]
  mod tests;`. Production files stay small and tests never live in them.
- **Integration tests** in `crates/cleanping-cli/tests` run the real `cleanping` binary against a fake
  local API server (`tests/support`). **Tests must never reach a real provider.**
- **Terminal tests** (`edit.rs`, `setup.rs`, `hidden_key.rs`, `shell.rs`, `shell_marks.rs`, `writer.rs`, `writer_keys.rs`) drive real
  interactive zsh and bash sessions through a pseudo-terminal. They are Linux-only. Locally they skip when
  `zsh` or `script` is missing; when the `CI` environment variable is set they fail instead, so they
  cannot pass without running. They wait for a prompt or for text on the screen: do not use sleeps as the
  way to synchronize.
- **Test data that looks like a secret.** GitHub's push protection rejects strings shaped like real API
  tokens, even in tests. Build such fixtures from two fragments (`concat!` or `format!`), never as one
  literal, and never use a real key. Before pushing, check `git log -p main..HEAD` for anything
  token-shaped.
- A test that asserts something does **not** happen needs a positive control that shows the same setup
  does happen when it should, or it can pass for the wrong reason.

## Code

- **Layers.** `cleanping-core` is `domain` (rules, no I/O) ← `application` (use cases and ports) ←
  `infrastructure` (SQLite, files, HTTP). Dependencies point inward. `cleanping-cli` only turns arguments and
  keystrokes into use-case calls. See [docs/architecture.md](docs/architecture.md).
- **Small files.** Aim for about 150 lines, look hard at 200, and audit at 300. Split by responsibility, not
  by line count.
- **No `unsafe`** (the workspace forbids it), and clippy warnings are errors.
- **Document public items** in `cleanping-core`: what it means, what a caller must know, what it returns
  on error. The build warns on a missing doc comment, and CI treats warnings as errors.
- **Naming in prose.** The product is **CleanPing** in docs, help text and release notes. The command,
  crate names, file paths, URLs and the `cleanping:` message prefix stay lowercase.
- **Keep the docs true.** If you change behavior, update `--help` text, the README or the matching page in
  `docs/`, and add a line under **Unreleased** in [CHANGELOG.md](CHANGELOG.md).

## What must stay true

Reviewers will look for these first. Details are in [AGENTS.md](AGENTS.md).

- An API key is never printed, logged, or put on a command line, and is read only from a hidden prompt or
  stdin. Files holding keys or history are `0600`.
- Text and keys go only to the provider the user configured: HTTPS (plain HTTP only for loopback), no
  redirects followed, and a proxy setting that cannot be honored is an error, not a silent direct
  connection.
- A provider's reply is untrusted. It is cleaned of control and invisible characters, size-limited, and
  for the shell key it must look like the text it replaces.
- Text that looks like a secret is not sent by the shell key or `cleanping edit` without the user's clear
  choice.
- Nothing is downloaded or run without the user saying yes, and no install script from the internet is
  ever run.
- Error messages do not include a provider's response body or the user's text.

## Pull requests

- One logical change per pull request, with small, focused commits.
- Say **what** changed and **why**, **how you tested it**, and **what you did not verify** (for example
  "macOS not tried").
- CI must be green: `rust (ubuntu-latest)`, `rust (macos-latest)` and `msrv`. `main` is protected, so
  changes land by pull request and a maintainer squash-merges them.
- Do not include real API keys, real provider responses, or other people's text in code, tests, issues or
  screenshots.

## Releases (maintainers)

Release notes come from the version’s section of [CHANGELOG.md](CHANGELOG.md), followed by a full
changelog compare link. Write that section for a reader: explain what people can do now, rather than
listing pull requests, and leave out author names and handles. Missing or empty notes fail the release.
The extraction script is tested in CI on Ubuntu and macOS.

GitHub shows the account that opened each pull request on merged commits and pull requests; this project
cannot change that attribution.

1. Set `version` in the workspace `Cargo.toml`, run `cargo build` so `Cargo.lock` follows, and move the
   **Unreleased** notes in [CHANGELOG.md](CHANGELOG.md) under the new version and date.
2. Merge that pull request and wait for CI on `main` to pass.
3. Push a tag `vX.Y.Z` on that commit. `.github/workflows/release.yml` refuses a tag that is not on `main`,
   that does not match the version in `Cargo.toml`, or whose commit has no successful CI run. It then builds
   Linux (x86_64) and macOS (Apple silicon and Intel) archives with SHA-256 checksums and build-provenance
   attestations, and creates a **draft** release.
4. Read the draft, then publish it.

After any change to `.github/workflows/release.yml`, wait for CI to pass, then run **Actions → Release →
Run workflow** on the changed branch. This dry run builds and checks the archives, but does not exercise
the publish job. The script itself is tested in CI; the first real proof of the publish step is the next
release.

## License

CleanPing is licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.
Unless you explicitly state otherwise, any contribution you intentionally submit for inclusion is dual
licensed the same way, without any additional terms or conditions.
