# Next tasking

Written 2026-09-29. Delete a task when its pull request is merged, and delete this file when nothing is
left. Every step that changes `main`, pushes a tag, or publishes needs the maintainer's explicit go.

**Where things stand.** v0.3.0 is published (2026-09-30). v0.4.0 adds `cleanping writer`; its release
pull request is open until it is merged, tagged and published. The Linux v0.3.0 archive was downloaded
without signing in, and its checksum, attestation and `--version` were checked; no macOS archive has been
run by anyone. Release notes come from the
changelog section (`.github/scripts/release-notes.sh`), and the pinned GitHub Actions are `checkout` 7.0.1,
`upload-artifact` 7.0.1 and `download-artifact` 8.0.1. The docs have a page for which key works where
([docs/use-it-your-way.md](docs/use-it-your-way.md)). Task 1 is done (Rust 1.98.1 here, 398 tests passing).

## 1. Set up a new Linux dev machine

The packaged Rust on Ubuntu 24.04 is 1.75, older than the 1.89 this project needs, so use rustup.

1. Clone, and set the author identity for this repository right away (a global git identity may be a
   personal address):
   ```sh
   git clone https://github.com/Keynodex/cleanping.git ~/keynodex/active/cleanping-public
   ```
   ```sh
   cd ~/keynodex/active/cleanping-public && git config user.name Keynodex && git config user.email noreply@keynodex.com
   ```
2. Install Rust. Download the official installer to a file and read it before running it:
   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh
   ```
   ```sh
   sh /tmp/rustup-init.sh --profile minimal -y && . ~/.cargo/env && rustup component add clippy rustfmt
   ```
   Then `rustc --version` should show 1.89 or newer.
3. Run the checks CI runs. The terminal tests need `zsh` and `script`:
   ```sh
   cargo fmt --all -- --check && cargo clippy --workspace --all-targets --locked -- -D warnings && cargo test --workspace --locked && RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
   ```

**Done when** all four pass and the test total is **398 passed, 0 failed**, the same as on the first
machine (396 before the query-string check was added). If the total differs, find out why before doing
anything else. Ollama is not needed for the
tests; it is only for trying a real local model by hand.

## 2. Prove the install on a clean machine (v0.4.0, includes `cleanping writer`)

Download one archive from the published release on a machine that has never had CleanPing, and follow the
README install steps, including `sha256sum -c` (`shasum -a 256 -c` on macOS). Do Linux and both macOS
archives if a Mac is available: nobody has run the macOS builds yet. Then delete this task in a small
pull request.

**Done when** each archive you tried installs, passes its checksum and prints `cleanping 0.4.0`, and `cleanping writer` opens, fixes with Ctrl+G and copies with Enter.

## 3. Docs page on keynodex.com

`keynodex.com/docs/cleanping/`, built from `docs/` so the two cannot drift. It lives in the website
repository (`Keynodex/keynodex-site`, Next.js on Vercel), so its `CONVENTIONS.md` gates apply. Do it after
task 2 so the page shows the released version, and include `docs/use-it-your-way.md`. Separate pull request.

## 4. Later

- A browser extension for web text boxes, as its own release. Every site's editor differs and updates
  break it, so plan testing per site.
- Windows support.
- Grok Build as an editor host for `cleanping edit` (untested), and whether Claude Code or Codex throw away
  your text when their editor exits with an error (untested; a failed edit exits `0` as a precaution).
- Things never checked: the terminal tests on macOS, the real `ollama pull` download, pasting a key from
  the clipboard into the hidden prompt, zsh older than 5.3, and wide (CJK) characters.
- A one-time "your text goes to your provider" notice on first use.
- **Prompt quality.** Score the ready-made prompts, and a draft "llm-ready" prompt that lays a rough draft
  out as a clear prompt for an AI (task first, then context, constraints, steps, output). A first test on
  five rough prompts with one model, with three drafts, showed that worked examples in the prompt help most,
  that it sometimes files a constraint under the wrong heading, and that it once added a line the author
  never wrote. Build about 20 rough prompts with the expected result for each, score every version, and
  try one more capable model as well. Ship the preset only with the score in the docs. It suits
  `cleanping edit` and the plain command, not the shell key, which refuses replies that add lines.
- **Ctrl+G as the default shell key.** It already works with `CLEANPING_KEYBIND` (see
  [docs/use-it-your-way.md](docs/use-it-your-way.md)). Decide after some use whether it should be the
  default; that changes both init scripts, their tests, the docs and the changelog. Ctrl+G normally
  cancels at a prompt, so the trade-off is stated in the docs.
- **Copy over SSH.** `--copy` needs a clipboard tool on the machine where CleanPing runs, so it cannot
  reach your own clipboard from a remote shell. A terminal escape sequence for copying is an idea, and
  untested.
- The last screen of `cleanping setup` suggests putting `export VISUAL=...` in your profile. It could point
  to the "Use it your way" page and the one-session launcher instead. That is a code change with a test.
