# Privacy and safety

The short version: your text goes to the AI provider you chose, and that provider can read it. CleanPing
does not encrypt it end to end, and it has no server of its own in the middle. If the text must never
leave your computer, use a [local model](providers-and-local-models.md#a-local-model-with-ollama).

To report a security problem privately, see [SECURITY.md](../SECURITY.md).

## What leaves your computer

| When | What is sent, and to whom |
|---|---|
| You rewrite text (`cleanping`, `cleanping edit`, the shell key, Ctrl+G in `cleanping writer`) | Your text and the [system prompt](system-prompts.md), plus the `<draft>` tags and the fixed sentence CleanPing adds ([what it adds](system-prompts.md#what-cleanping-adds-to-every-request)), over an encrypted (TLS) connection, to the provider address you saved, together with your key for that provider |
| `cleanping keys test` | One fixed word (`ping`) with a fixed instruction, framed the same way, to that provider. Never your text, history or saved system prompt |
| You use a local model | To an address on your own machine only. Nothing leaves it |
| `cleanping setup`, local model | A request to the local server's model list (`GET /api/tags`) on a loopback address, and, only if you say yes, Ollama's own `ollama pull` |
| `cleanping update` | One request to GitHub's public API asking for the newest release, with only a `User-Agent: cleanping/VERSION` header ([details](#checking-for-updates)) |

CleanPing itself makes no other network connections: it never checks for updates by itself and sends no
usage data. The only programs it starts are your clipboard tool (with `--copy`, and on Enter in `cleanping writer`),
zsh (for `cleanping writer`) and `ollama pull` (with your yes). When no clipboard tool works, Enter in
`cleanping writer` also starts `base64` and `tr` to encode a copy request, which it writes to your own
terminal (over the SSH connection you are already using, if you have one) and nowhere else.

## What is stored on your computer

| What | Where | Mode |
|---|---|---|
| API keys | `~/.config/cleanping/secrets.json` | `0600`, in a `0700` directory |
| History, your saved system prompt, which key is selected | `~/.local/share/cleanping/cleanping.db` (SQLite) | `0600`, in a `0700` directory |

`$XDG_CONFIG_HOME` and `$XDG_DATA_HOME` are respected (relative paths are ignored). Keys are never
printed, logged, or passed on a command line; the hidden prompt shows only stars, and
`cleanping keys list` shows a key's name and address but never the key.

### History

Each plain rewrite is saved: your text, the result, the system prompt and the model, on your machine
only. A rewrite that failed is saved too, with the error message, so your text is in the history even
when the provider did not answer. It has no expiry and is never trimmed automatically. **Nothing is saved
with `--no-history`, and nothing is ever saved from the shell key, from `cleanping edit` or from `cleanping writer`.**

```sh
cleanping history list                       # newest first, one line each, times in UTC
cleanping history purge --older-than 30      # delete rewrites older than 30 days
cleanping history clear --yes                # delete everything
```

Deleted text is overwritten and the database file is rebuilt, so it cannot be read back from the file.
Backups and disk snapshots you made yourself are out of CleanPing's reach. Deleting a key does not delete
the history that used it.

Your system prompts are kept too: each `cleanping prompt set` or `prompt use` adds a new entry and leaves
the earlier ones in the database, and `history clear` does not remove them. Keep anything private out of a
system prompt. A key's address is stored in the database as well, and `cleanping keys list` shows it, which
is why an address must never contain a key (see
[Providers](providers-and-local-models.md#any-other-provider)).

If you delete the database file to start over, saved keys stay in `secrets.json`. CleanPing will not
reuse such a key for a new address without you typing it again; remove `~/.config/cleanping/secrets.json`
as well for a fully clean slate.

## Text that looks like a secret

CleanPing can refuse to send text that looks like it holds a private key, an API key or token, or a
password. It names the *kind* of secret and never shows it.

| Where | Behavior |
|---|---|
| [Shell key](shell-key.md) | Always on (`--refuse-secrets`). The line is not sent; you see a message |
| [`cleanping writer`](use-it-your-way.md#5-writing-mode-for-text-only) | Always on: Ctrl+G is the shell key, so it behaves the same. The line is not sent; you see a message |
| [`cleanping edit`](claude-code-and-codex.md) | Always on. The screen says **Not sent**; **S** sends it anyway if you choose. `--yes` never sends it |
| `cleanping "text"` | Off, because you chose the text yourself. Add `--refuse-secrets` to turn it on (exit code `2`) |

This check is a best guess: it looks for common key formats and for words like `password` or `token`
followed by a value. It cannot catch everything, so it does not replace judgment about what you paste.

## What a reply can and cannot do

The AI's reply is treated as untrusted.

- Terminal control characters, text-direction overrides and zero-width or other invisible characters are
  removed, so a reply cannot move your cursor, rewrite the screen or hide text in itself.
- The shell key puts a reply on your command line only if it [keeps the shape](shell-key.md#what-the-key-sends-and-what-it-refuses)
  of your text, so a long reply cannot push a command out of sight.
- A reply that [changes a command](#a-reply-that-changes-a-command) in your text is refused by the shell
  key and named everywhere else.
- A reply is still an AI's rewrite. Read a rewritten command before you press Enter.

## Checking for updates

`cleanping update` is the only time CleanPing contacts anything other than the provider you chose, and it
happens only when you run that command: never in the background, never from another command, never on
a schedule.

- It sends one `GET` request to `https://api.github.com/repos/Keynodex/cleanping/releases/latest`. The
  only header CleanPing adds is `User-Agent: cleanping/VERSION` (GitHub requires one). No id, key,
  history, setting or text of yours is sent. GitHub, like any web server, sees your IP address.
- It follows no redirects, uses HTTPS only, gives up after 15 seconds, refuses a reply larger than
  256 KB, and uses the same proxy rules as a remote provider (below).
- The reply is untrusted data. The version must be plain `vX.Y.Z` (a pre-release tag counts as not
  understood), the release page is shown only if it is on `https://github.com/Keynodex/cleanping/`
  (otherwise the standard releases page is shown), and no control character from the reply reaches your
  terminal. An error never includes the reply.
- It only prints. It downloads, installs, runs and changes nothing, and opens no file or database.
  Installing the update for you is planned and will ask first.

## A reply that changes a command

Models asked to fix spelling often "fix" a command too, most often by closing a quote you left open.
When the open quote is the problem you are asking about, that hides the answer. CleanPing compares the
commands in your text with the reply and reports a quote closed or opened, a flag (`-x`, `--name`)
dropped or added, and a path or URL changed.

| Where | Behavior |
|---|---|
| [Shell key](shell-key.md) and Ctrl+G in [`cleanping writer`](use-it-your-way.md#5-writing-mode-for-text-only) | The whole line is the command. A changed command is **refused**: the line stays as it is and a message says why (`--keep-shape`) |
| [`cleanping edit`](claude-code-and-codex.md) | The screen shows `Check the command: ...` under the title. Enter still accepts the edit. With `--yes` the same warning goes to stderr |
| `cleanping "text"` | Each change is a `cleanping: warning: ...` line on stderr. The reply is still printed and the exit code does not change |

Outside the shell key, only these lines count as commands: lines inside a fenced block (three
backticks), a line after a `$ ` prompt (and its `> ` continuation lines), a line indented by 4 spaces
or a tab that starts with a plain word followed by a flag or a path, and any other line that starts
with a plain word followed by a flag. Everything else is prose and is never checked. Messages show at
most 40 characters of a command or word, and never a part that looks like a secret.

It is a safety net, not a proof. It misses changes to words that are not quotes, flags or paths, and
commands that do not match the rules above. It also reports some harmless changes: an apostrophe
between two letters (`don't`) is read as an apostrophe, but one at the edge of a word (`the users'
files`) counts as a quote, so adding one can be refused on the shell key; a prose line with a flag in it
(`use -v`) is checked like a command; and a command the reply moves into a sentence in backticks is
reported as no longer in the reply.

## Network rules

- API addresses must be `https://`. `http://` is accepted only for `localhost`, `127.0.0.1` and `::1`.
- HTTP redirects are never followed, so a key and your text cannot be forwarded to another host.
- Error messages never include the server's response body.
- `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` and `NO_PROXY` are honored for remote providers and for
  `cleanping update`. Only `http://`
  and `https://` proxies are supported. If one of these variables is set to anything else (for example a
  `socks5://` address) or to something that does not parse, CleanPing stops and sends nothing rather than
  quietly connecting without the proxy. Local providers are always reached directly, never through a proxy.
- TLS uses a built-in copy of Mozilla's root certificates, not your system's. A company proxy that
  inspects TLS with its own certificate authority will not be trusted.
- `localhost` is trusted to mean this machine, as your system resolves it. Use `127.0.0.1` to be sure.

## Limits

| Limit | Value |
|---|---|
| Text you can rewrite | 200,000 bytes; larger input is refused before any request is made |
| An API key | 4,096 bytes |
| A provider's reply | 1,000,000 bytes; larger replies are refused |
| A rewrite request | Gives up after 180 seconds |
| `cleanping keys test` | Gives up after 60 seconds |
| `cleanping update` | Gives up after 15 seconds; a reply larger than 256 KB is refused |
| Asking a local server what it has | Gives up after 3 seconds |
| Shell key reply | At most as many lines as your text, at most twice its length plus 80 characters, and no run of more than 8 spaces or tabs (unless your text had one) |

## What CleanPing does not do

- It does not make your text private from the provider you chose.
- It does not sign in or sync, and it keeps nothing outside the key file and the database in your own
  directories (listed above).
- It does not run the text you give it or the reply it gets; it only prints or places them.
