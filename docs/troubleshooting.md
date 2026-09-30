# Troubleshooting

CleanPing's messages start with `cleanping:` and say what to do next. This page explains the common ones.
The number in brackets is the [exit code](commands.md#exit-codes).

## Keys

| Message | What it means and what to do |
|---|---|
| `No API key saved yet. Add one: cleanping keys add --provider OpenAI ...` [3] | Nothing is saved. Run `cleanping setup`, or `cleanping keys add --provider NAME` |
| `Several keys are saved and none is selected. Choose one: cleanping keys use NAME ...` [3] | CleanPing will not guess. Run `cleanping keys use NAME`, or pass `-c NAME` for one run. `cleanping keys list` shows the names |
| `No saved key named "NAME".` [3] | The name after `-c` or `keys use` does not exist. Check `cleanping keys list` |
| `Several keys match "NAME"; use the exact name.` [2] | Two saved names differ only by capital letters, and yours matches neither exactly. Type one exactly as `cleanping keys list` shows it |
| `No API key saved for "NAME".` [3] | The key list has an entry but its key is missing (for example `secrets.json` was deleted). Add it again with the command in the message |
| `A key for "NAME" is left over from an earlier setup; enter the API key again.` / `The API address changed; enter the API key again ...` [2] | Changing the address of a saved key, or reusing a key left over from an earlier setup, needs you to type the key again, so a saved key is never sent to a new host without you. Run `cleanping keys add ...` and enter the key |
| `Unknown provider. Use DeepSeek, OpenAI, OpenRouter or "Ollama (local)", or give --url.` [2] | `--provider` must be one of those names (in quotes when it has spaces), or give `--url` for another provider |
| `API URL must use HTTPS (HTTP is allowed only for localhost).` [2] | Use an `https://` address. `http://` works only for `localhost`, `127.0.0.1` and `::1` |
| `API URL must not have a query string or fragment (? or #). Never put a key in the address.` [2] | Remove everything from `?` or `#` onward in `--url`. Give the key through the hidden prompt or `--key-stdin` |
| `API key must be plain visible characters (no spaces, control characters or smart quotes).` [2] | The pasted key has something extra in it, often a trailing space or a curly quote. Copy it again |
| `Cancelled: no key was entered.` [2] | You pressed Ctrl-C or Esc at the key prompt. Nothing was saved |

## Rewriting

| Message | What it means and what to do |
|---|---|
| `Nothing to rewrite. Pass text, or pipe it in: ...` [2] | No text was given and nothing was piped in |
| `Input is too long (limit 200000 bytes).` [2] | Rewrite a smaller piece. The limit is checked before anything is sent |
| `Input is not valid UTF-8 text.` [2] | The input is not text (for example a binary file) |
| `The text looks like it contains a private key / an API key or token / a password or secret, so it was not sent.` [2] | The [secret check](privacy-and-safety.md#text-that-looks-like-a-secret) stopped it. Remove the secret, or, for plain `cleanping "text"`, leave out `--refuse-secrets` |
| `The reply is longer or has more lines than your text; not applied.` | The shell key refused a reply that did not [keep the shape](shell-key.md#what-the-key-sends-and-what-it-refuses) of your line. Your line is unchanged; try again or edit it by hand |

## The provider

| Message | What it means and what to do |
|---|---|
| `Could not reach the API: ...` [1] | No connection: network down, wrong address, a proxy in the way, or a request that took longer than 180 seconds. For a local model, is it running? |
| `API returned HTTP 401.` (or 403) [1] | The provider did not accept the key. Check that the key is right and still active |
| `API returned HTTP 404.` [1] | Usually a wrong address or a model name the provider does not have. With Ollama, run `ollama pull MODEL` |
| `API returned HTTP 429.` [1] | The provider is limiting you (rate limit or quota). Wait, or check your plan |
| `API redirected the request (HTTP N); refusing to follow it.` [1] | CleanPing never follows redirects, so a key cannot be forwarded to another host. Save the final address with `--url` |
| `API response did not contain edited text.` / `API response could not be read.` [1] | The address is not an OpenAI-compatible chat-completions endpoint |
| `API response was too large (limit 1000000 bytes).` [1] | The provider sent more than 1 MB. CleanPing refuses it |
| `API returned an empty edit; nothing was copied.` [1] | The model answered with nothing. Try again, or a different model or [system prompt](system-prompts.md) |

`cleanping keys test` checks a key without using your text and tells you how long the provider took.
For a local Ollama it also says whether to install it, start it, or download the model
([details](providers-and-local-models.md#a-local-model-with-ollama)).

## Setup, `edit` and `writer`

| Message | What it means and what to do |
|---|---|
| `cleanping setup asks questions, so it needs a terminal. ...` [2] | Run it in a real terminal. In a script, use `cleanping keys add --provider NAME --key-stdin` |
| `There is no terminal to show the edit on. Run it from a terminal, or add --yes ...` [1] | `cleanping edit` needs a terminal for its screen. Add `--yes` to edit the file without one |
| `cleanping writer needs a terminal for its input and its output. ...` [2] | `cleanping writer` is for a terminal window. It cannot run in a script or with its input or output redirected |
| `cleanping writer needs zsh, and zsh was not found. ...` [1] | Writing mode runs in zsh. Install it with your system's package manager (macOS and most Linux systems have it) |
| `Could not copy (no clipboard tool found). ...` (shown inside the writer) | No `pbcopy`, `wl-copy`, `xclip` or `xsel` was found, or it failed. Select the text on screen and copy it yourself |
| `Could not read the file to edit.` [2] | The file path is wrong or unreadable |
| `Could not write the edited text back to the file.` | The file is not writable. Check its permissions |
| The edit screen says **Could not edit** | The reason is shown on the screen and your text is unchanged. Any key returns you to the app |

## Still stuck?

`cleanping COMMAND --help` describes every option. If you think you have found a bug, open an issue at
[github.com/Keynodex/cleanping/issues](https://github.com/Keynodex/cleanping/issues) with the command you
ran and the message you saw. **Never paste an API key**, and do not include text you want to keep
private. To report a security problem, use the private route in [SECURITY.md](../SECURITY.md).
