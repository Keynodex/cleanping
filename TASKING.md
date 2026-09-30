# Next tasking

Written 2026-09-29. Delete a task when its pull request is merged, and delete this file when nothing is
left. Every step that changes `main`, pushes a tag, or publishes needs the maintainer's explicit go.

**Where things stand.** Every v0.3.0 feature is merged, the version is `0.3.0` on `main`, the tag `v0.3.0`
is pushed and the release workflow built a **draft** release (three archives with checksums and
attestations; the Linux one was checked). It is not public until the maintainer publishes it. Release
notes now come from the changelog section (`.github/scripts/release-notes.sh`), and the pinned GitHub
Actions are `checkout` 7.0.1, `upload-artifact` 7.0.1 and `download-artifact` 8.0.1. Task 1 is done
(Rust 1.98.1 here, 398 tests passing).

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

## 2. Finish the v0.3.0 release

1. The maintainer reads the draft on GitHub (Releases, v0.3.0) and publishes it. Publishing is the
   maintainer's step.
2. After publishing, download one archive on a clean machine and follow the README install steps,
   including `sha256sum -c`. The macOS archives have not been run by anyone yet.
3. Delete this task in a small pull request.

**Done when** the release is published and step 2 works.

## 3. Docs page on keynodex.com

`keynodex.com/docs/cleanping/`, built from `docs/` so the two cannot drift. It lives in the website
repository (`Keynodex/keynodex-site`, Next.js on Vercel), so its `CONVENTIONS.md` gates apply. Do it after
task 2 so the page shows the released version. Separate pull request.

## 4. Later

- A browser extension for web text boxes, as its own release. Every site's editor differs and updates
  break it, so plan testing per site.
- Windows support.
- Grok Build as an editor host for `cleanping edit` (untested), and whether Claude Code or Codex throw away
  your text when their editor exits with an error (untested; a failed edit exits `0` as a precaution).
- Things never checked: the terminal tests on macOS, the real `ollama pull` download, pasting a key from
  the clipboard into the hidden prompt, zsh older than 5.3, and wide (CJK) characters.
- A one-time "your text goes to your provider" notice on first use.
