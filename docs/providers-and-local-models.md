# Providers and local models

CleanPing works with any service that speaks the OpenAI chat-completions format. You bring the key; there
is no CleanPing account and no CleanPing server.

## Ready-made providers

| Provider (`--provider`) | Address it fills in | Default model |
|---|---|---|
| `DeepSeek` | `https://api.deepseek.com/v1/chat/completions` | `deepseek-chat` |
| `OpenAI` | `https://api.openai.com/v1/chat/completions` | `gpt-4o-mini` |
| `OpenRouter` | `https://openrouter.ai/api/v1/chat/completions` | `openai/gpt-4o-mini` |
| `Ollama (local)` | `http://127.0.0.1:11434/v1/chat/completions` | `qwen2.5:7b` |

```sh
cleanping keys add --provider OpenAI            # asks for the key with a hidden prompt
cleanping keys add --provider OpenAI --model gpt-4o
```

The defaults are starting points, not recommendations: pick the model that suits your budget and quality
needs with `--model`.

## Any other provider

Give the full chat-completions address and a model name:

```sh
cleanping keys add --name work --url https://llm.example.com/v1/chat/completions --model my-model
```

- The address must be `https://`. Plain `http://` is accepted only for `localhost`, `127.0.0.1` and
  `::1`, so a key and your text never travel unencrypted to another machine.
- `localhost` is trusted to mean this machine, as your system resolves it. Use `127.0.0.1` if you want
  to be sure.
- HTTP redirects are never followed, so a key and your text cannot be forwarded to another host.
- The key is sent as a `Bearer` token in the `Authorization` header.
- **Never put a key or token in the address** (for example `...?key=...`). The address is printed in full by
  `cleanping keys list`, saved in the database as plain text, and typed on a command line, where your
  shell may keep it. Keys belong in the hidden prompt or on stdin, which is the only way CleanPing sends
  one. A provider that wants the key in the address is not supported. `keys add` refuses any address
  with a query string or fragment (`?...` or `#...`), so this cannot happen by accident.

## Several keys

You can save more than one (`--name` tells them apart). `cleanping keys list` shows them, and `*` marks
the selected one.

```sh
cleanping keys use work          # make "work" the default
cleanping -c personal "text"     # use another one for a single run
```

CleanPing never guesses between keys: if several are saved and none is selected, it stops and asks you
to choose, so your text cannot go to a provider you did not pick. An exact name always beats a name that
differs only in capital letters.

## Check that a key works

```sh
cleanping keys test            # the selected key
cleanping keys test work       # a named key
```

This sends one tiny fixed request (never your text) and prints how long the answer took. Nothing is saved
to the history.

## A local model with Ollama

With a local model, nothing leaves your computer: the request goes to your own machine only. Local
providers are always reached directly, never through a proxy.

1. **Install Ollama** yourself, from [ollama.com/download](https://ollama.com/download). CleanPing never
   installs it, and never runs an install script from the internet.
2. **Start it**, for example with `ollama serve`.
3. **Save the provider:** `cleanping keys add --provider "Ollama (local)"`. No real key is needed.
4. **Download the model:** `ollama pull qwen2.5:7b` (or whichever model you chose with `--model`).
5. **Check:** `cleanping keys test`.

Or run `cleanping setup` and choose the local model: it checks whether Ollama is running, and if the
model is missing it asks `[y/N]` before running `ollama pull` for you. The download can be several
gigabytes. If the model name starts with `-`, setup refuses to pass it to `ollama`.

If a check fails, CleanPing tells you the next step:

| It says | What to do |
|---|---|
| Ollama does not seem to be installed | Install it, then `ollama pull MODEL` |
| Ollama is installed but not running | `ollama serve` |
| Ollama is running but does not have the model | `ollama pull MODEL` |

"Not installed" and "not running" are only suggested for the standard address (`127.0.0.1:11434`); on
another local address CleanPing cannot know how the server is started. To find out what is running,
CleanPing asks the local server for its model list (`GET /api/tags`, 3-second limit, loopback addresses
only, no redirects, no proxy) and looks for an `ollama` program on your `PATH`.

The first request after Ollama loads a model into memory can take much longer than the ones after it. A
rewrite gives up after 180 seconds; `keys test` after 60.

## Models that think before they answer

Some models think before they answer, and that thinking counts toward the provider's limit on how long a
reply may be. With a long text, such a model can run out of room before the rewrite is complete. When the
provider says it stopped at that limit, CleanPing refuses the reply instead of using part of a rewrite, and
your text stays as it was ([what to do](troubleshooting.md#the-provider)).

**DeepSeek flash models think by default; CleanPing turns that off.** For a key whose address is
`api.deepseek.com` and whose model name starts with `deepseek-flash`, every request asks the model not to
think (`"thinking": {"type": "disabled"}`). Rewriting does not need it: in a test on 2026-10-01, thinking
took 13 to 44 seconds on a long text and sometimes ran out of room, while the same job without it took
about 5 seconds. Other DeepSeek models (such as `deepseek-chat`), other addresses and other
providers get nothing extra. This is not configurable yet.

## Limits

A reply larger than 1 MB is refused, and input above 200,000 bytes is refused before anything is sent.
More in [Privacy and safety](privacy-and-safety.md).
