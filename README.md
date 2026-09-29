# CleanPing

Rewrite rough text into clean, clear text with any OpenAI-compatible API, from your terminal.

```console
$ cleanping "plz fix teh login pgae, buttons r too smal"
Please fix the login page; the buttons are too small.
```

**What makes it different:** you pick the AI, bring your own key, edit with your own system prompt, and can run a local model so nothing leaves your computer.

- **Pipe-friendly.** Only the rewritten text goes to stdout. Errors go to stderr, with clear exit codes.
- **Works with your key.** DeepSeek, OpenAI, OpenRouter, or a local model such as Ollama. No account with us, no server of ours.
- **A key for your shell.** Type a rough command line, press one key, get it rewritten in place. Press again to restore your original.
- **Fix your message inside Claude Code and Codex.** Press their edit key (Ctrl+G), see the changed words highlighted, and press Enter to accept or N to keep yours.
- **Private by default.** Your API key stays in a file only you can read and is never printed or passed on a command line. Text goes only to the provider you configured.

Works on Linux and macOS. Windows is not supported yet.

## Install

**Prebuilt binary** (Linux x86_64, macOS Apple silicon and Intel): download the archive for your system from the [latest release](https://github.com/Keynodex/cleanping/releases/latest), then check it and put it on your `PATH`:

```sh
sha256sum -c cleanping-v0.2.0-x86_64-unknown-linux-gnu.tar.gz.sha256   # macOS: shasum -a 256 -c
tar xzf cleanping-v0.2.0-x86_64-unknown-linux-gnu.tar.gz
install cleanping-v0.2.0-x86_64-unknown-linux-gnu/cleanping ~/.local/bin/
```

(Replace `v0.2.0` with the version you downloaded.) The Linux build needs glibc 2.34 or newer (Ubuntu 22.04+, Debian 12+). The macOS builds are not signed or notarized by Apple, so macOS may ask you to allow them. Each archive has a build-provenance attestation from this repository's release workflow.

**From source** (needs [Rust](https://rustup.rs) 1.89 or newer):

```sh
cargo install --git https://github.com/Keynodex/cleanping cleanping-cli --locked
```

This installs a `cleanping` binary into `~/.cargo/bin`. An `npm` package is planned.

## First run

Save a key. You are asked for it with a hidden prompt (nothing is shown as you paste). It is never taken from an argument, so it does not land in your shell history:

```sh
cleanping keys add --provider OpenAI
```

In a script, pipe the key in on stdin instead, with `--key-stdin`:

```sh
printf '%s\n' "$OPENAI_API_KEY" | cleanping keys add --provider OpenAI --key-stdin
```

For a local model no key is needed:

```sh
cleanping keys add --provider "Ollama (local)"
```

Rewrite something:

```sh
cleanping "so first i want to say well done on that assessment"
echo "plz fix teh login pgae" | cleanping
cleanping --copy "rough text"      # also copies to the clipboard (wl-copy, xclip, xsel or pbcopy)
```

Text that starts with a word that is also a command (`keys`, `prompt`, `history`, `edit`, `init`) can be passed after `--`, or piped in. Any other text, including `help me fix this`, is rewritten as it is.

If you have several keys, pick one with `cleanping keys use NAME` or `-c NAME`. CleanPing never guesses between keys, so your text cannot go to a provider you did not choose. With a single saved key it just uses it.

## The shell key (zsh and bash)

Add one line to `~/.zshrc` (or `~/.bashrc`, which needs bash 4 or newer):

```sh
eval "$(cleanping init zsh)"     # or: eval "$(cleanping init bash)"
```

Type a rough line at the prompt (do not press Enter), then press **Ctrl-X Ctrl-P**. The line is replaced by the rewrite. Press the same keys again, without editing, to get your original back. To use a different key, set `CLEANPING_KEYBIND` before that line (zsh: `'^[r'`, bash: `'\er'`).

The whole command line is sent to your AI provider when you press the key, so do not use it on lines that contain secrets. The line is sent over stdin, never as a command-line argument, and nothing is written to CleanPing's history from the shell key.

**The reply has to look like what it replaces.** A reply is only put on your command line if it has no more lines than your text, is not much longer (about twice as long plus a short sentence), and does not pad itself with long runs of spaces or tabs. Otherwise the line is left alone and you see a message. This stops a reply from pushing a command out of sight above a harmless-looking end. It is still a rewrite by an AI, so read the line before you press Enter.

## Fix your message inside Claude Code and Codex

Claude Code and Codex open the program named in `$VISUAL` when you press **Ctrl+G** ("edit in editor"). Point it at CleanPing, for example in `~/.zshrc` or `~/.bashrc`:

```sh
export VISUAL="cleanping edit"
```

Type your message in the app and press Ctrl+G. CleanPing lights up your text while the AI works, then shows the result with the changed words highlighted:

| Key | What it does |
|---|---|
| **Enter** | Accept: your message in the app becomes the edited text |
| **N** or **Esc** | Keep your original text |
| **O** | Switch between the edited and the original text |
| **↑ ↓ Page Up Page Down** | Scroll a long message |

If the edit fails (no key saved, no network), the screen says why and any key leaves your text exactly as it was. Text that looks like it holds an API key, token or password is not sent: the screen says so, and **S** sends it anyway if you choose to. Nothing from this screen is saved to the history. It draws on your terminal directly, so it works even though the app captures the editor's output.

It works with Claude Code and Codex, as tested. Other apps that run `$VISUAL FILE` should work the same way. To edit a file with no screen at all (for a script), use `cleanping edit --yes notes.txt`.

## Commands

| Command | What it does |
|---|---|
| `cleanping [TEXT]...` | Rewrite the text (or piped input). Options: `-c/--credential NAME`, `--copy`, `--no-history`, `--keep-shape` (refuse a reply with more lines, much longer, or padded) |
| `cleanping edit FILE` | Fix the text in a file and review it on a screen (set it as `VISUAL`). Options: `-c/--credential NAME`, `--yes` (no screen: write the edit into the file) |
| `cleanping keys list` | Show saved keys (never the key itself) |
| `cleanping keys add ...` | Save a key: `--provider`, `--name`, `--url`, `--model`, `--key-stdin` |
| `cleanping keys use NAME` / `remove NAME` | Choose the default key / delete one |
| `cleanping prompt show` / `set [TEXT]` | Read or change the system prompt sent with every rewrite |
| `cleanping history list [--limit N]` | Show your newest rewrites, one line each |
| `cleanping history clear --yes` | Delete every saved rewrite |
| `cleanping history purge --older-than DAYS` | Delete rewrites older than DAYS days |
| `cleanping init zsh\|bash` | Print the shell integration |

Exit codes: `0` ok, `1` the request failed (network, provider, storage), `2` bad input (empty, too long, invalid URL), `3` no usable key. The one exception is the `edit` screen: when an edit fails there, it shows why and exits `0` with the file untouched, so the app that opened it keeps your text. With `--yes` the codes above apply.

## Where things live

| What | Where | Mode |
|---|---|---|
| API keys | `~/.config/cleanping/secrets.json` | `0600` (directory `0700`) |
| History, saved prompt, state | `~/.local/share/cleanping/cleanping.db` (SQLite) | `0600` |

`$XDG_CONFIG_HOME` and `$XDG_DATA_HOME` are respected (relative paths are ignored).

## History

Each rewrite is saved to a local history in the database above: your text, the result, the system prompt and the model, on your machine only. It has no expiry and is never trimmed automatically. Nothing is saved with `--no-history`, and never from the shell key or `cleanping edit`.

Look at it with `cleanping history list`. Delete it with `cleanping history clear --yes`, or trim it with `cleanping history purge --older-than 30`. Deleted text is overwritten and the database file is rebuilt, so it cannot be read back from the file (backups and disk snapshots you made yourself are out of CleanPing's reach). Deleting a key does not delete the history that used it.

If you delete the database file to start over, saved API keys stay in `secrets.json`. CleanPing will not reuse such a key for a new address without you typing it again; remove `~/.config/cleanping/secrets.json` as well for a fully clean slate.

## Safety

- Your text goes over an encrypted connection to the provider you chose, and that provider can read it. CleanPing is not end-to-end encrypted. If the text must never leave your computer, use a local model such as Ollama: then nothing is sent anywhere.
- API URLs must be `https://`; plain `http://` is accepted only for `localhost`, `127.0.0.1` and `::1`.
- HTTP redirects are never followed, so a key and your text cannot be forwarded to another host.
- Error messages never include the server's response body.
- `cleanping edit` will not send text that looks like an API key, token, private key or password unless you press **S** on the screen; with `--yes` it never does. The check is a best guess and cannot catch everything, so it does not replace judgment about what you paste. The screen only ever names the kind of secret, never shows it.
- Replies are cleaned before they are shown: terminal control characters, text-direction overrides and zero-width or other invisible characters are removed, so a reply cannot move your cursor, rewrite the screen or hide text in it.
- Input is capped at 200,000 bytes, and refused before any request is made. A reply larger than 1 MB is refused.
- `HTTP_PROXY`, `HTTPS_PROXY` and `ALL_PROXY` (and `NO_PROXY`) are honored for remote providers. Only `http://` and `https://` proxies are supported: if one of these variables is set to anything else (for example a `socks5://` address) or to something that does not parse, CleanPing stops and sends nothing, rather than quietly connecting without the proxy. Local providers are always reached directly, never through a proxy.
- `localhost` is trusted to mean this machine, as your system resolves it. Use `127.0.0.1` if you want to be sure.
- TLS uses a built-in copy of Mozilla's root certificates, not your system's. A company proxy that inspects TLS with its own certificate authority will not be trusted.

To report a security problem, see [SECURITY.md](SECURITY.md).

## Roadmap

Planned, not built yet, and the order may change:

1. **Highlighted changes in the shell key** (zsh first).
2. **`cleanping setup`**: a first-run guide to pick a provider, enter your key (shown masked), set your system prompt, and optionally install a free local model with Ollama.
3. **Grok Build** as an editor host for `cleanping edit` (not tested yet).
4. **A browser extension** for web text boxes. Every website's editor behaves differently and sites change without notice, so each one needs its own testing and an update can break it.
5. Windows support.

## Documentation

- This README: install, first run, commands, where things are stored, safety.
- `cleanping --help` and `cleanping <command> --help`.
- [SECURITY.md](SECURITY.md): how to report a problem privately.
- [AGENTS.md](AGENTS.md): how the code is organized and the rules contributors follow.

A full documentation site is planned.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The code is a Cargo workspace: `cleanping-core` (domain rules, use cases, adapters) and `cleanping-cli` (the command). Tests run the real binary against a fake local API server, and drive real interactive zsh and bash sessions for the shell key (Linux; needs `zsh` and `script`).

The earlier desktop window (Python, tkinter) is kept in [`legacy/python`](legacy/python) until it is ported to Rust; it reads the same database and key file.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
