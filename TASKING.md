# Next tasking

Written 2026-09-29. Delete a task when its pull request is merged, and delete this file when nothing is
left. Every step that changes `main`, pushes a tag, or publishes needs the maintainer's explicit go.

**Where things stand.** Every v0.3.0 feature and its documentation is merged. The release pull request
(`release/v0.3.0`) sets the version to `0.3.0`; until it is merged, the tag is pushed and the draft is
published, v0.3.0 is not released. Task 1 is done (Rust 1.98.1 here, 398 tests passing).

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

## 2. Release v0.3.0

Do this before task 3, so the release workflow that shipped v0.2.0 is used unchanged.

1. Branch `release/v0.3.0`. Set `version = "0.3.0"` in the workspace `Cargo.toml` (line 6), then run
   `cargo build` so both `cleanping-*` entries in `Cargo.lock` follow.
2. `CHANGELOG.md`: put a new empty **Unreleased** section on top, rename the old one `[0.3.0] - DATE`,
   and update the two links at the bottom (`compare/v0.3.0...HEAD` and `releases/tag/v0.3.0`).
3. `README.md` lines 25 to 27 and 30 name `v0.2.0` in the install example: change them to `v0.3.0`.
   Check with `grep -rn '0\.2\.0' README.md docs CONTRIBUTING.md Cargo.toml`.
4. Run the four checks from task 1, open the pull request, and wait for CI. **Merge needs a go on the
   exact head.**
5. Wait for CI on `main` to pass for the merge commit, then push the tag (**needs a go**):
   ```sh
   git tag v0.3.0 MERGE_COMMIT_SHA && git push origin v0.3.0
   ```
   `release.yml` refuses a tag that is not on `main`, that differs from the `Cargo.toml` version, or
   whose commit has no successful CI run. It builds Linux (x86_64) and macOS (Apple silicon and Intel)
   archives with checksums and build-provenance attestations, and creates a **draft** release.
6. Read the draft (three archives, three `.sha256` files, generated notes), then the maintainer publishes.
7. After publishing, download one archive on a clean machine and follow the README install steps,
   including `sha256sum -c`.

**Done when** the release is published and step 7 works.

## 3. Dependabot pull requests #1, #2 and #3

They bump `actions/upload-artifact` 4.6.2 to 7.0.1 (#1), `actions/checkout` 4.4.0 to 7.0.1 (#2) and
`actions/download-artifact` 4.3.0 to 8.0.1 (#3). Workflows pin actions by commit SHA, so for each one:

- Check that the SHA in the pull request is the commit the upstream tag points to (annotated tags need
  one more step to reach the commit):
  ```sh
  gh api repos/actions/checkout/git/ref/tags/v7.0.1
  ```
- Read the upstream release notes for breaking changes (runtime versions, changed inputs).
- `upload-artifact` and `download-artifact` are used together in `release.yml`, and #1 and #3 move to
  different major versions (7 and 8). Confirm they are compatible with each other, or change them in one
  pull request.
- CI must be green. Any change to `release.yml` also needs a dry run: **Actions, Release, Run workflow**
  builds and checks everything and publishes nothing.

**Done when** each pull request is merged or closed with a reason, and a Release dry run has passed.

## 5. Docs page on keynodex.com

`keynodex.com/docs/cleanping/`, built from `docs/` so the two cannot drift. It lives in the website
repository (`Keynodex/keynodex-site`, Next.js on Vercel), so its `CONVENTIONS.md` gates apply. Do it after
task 2 so the page shows the released version. Separate pull request.

## 6. Later

- A browser extension for web text boxes, as its own release. Every site's editor differs and updates
  break it, so plan testing per site.
- Windows support.
- Grok Build as an editor host for `cleanping edit` (untested), and whether Claude Code or Codex throw away
  your text when their editor exits with an error (untested; a failed edit exits `0` as a precaution).
- Things never checked: the terminal tests on macOS, the real `ollama pull` download, pasting a key from
  the clipboard into the hidden prompt, zsh older than 5.3, and wide (CJK) characters.
- A one-time "your text goes to your provider" notice on first use.
