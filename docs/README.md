# CleanPing documentation

CleanPing rewrites rough text into clean, clear text with an OpenAI-compatible API, from your terminal.
These pages go deeper than the [README](../README.md), which is the place to start.

| Page | Read it to |
|---|---|
| [Getting started](getting-started.md) | Go from nothing to your first rewrite |
| [Use it your way](use-it-your-way.md) | See which key works where, and set up your own launcher names and keys |
| [Command reference](commands.md) | Look up a command, an option, an exit code or an environment variable |
| [Claude Code and Codex](claude-code-and-codex.md) | Fix your message inside those apps with Ctrl+G |
| [The shell key](shell-key.md) | Rewrite the line you are typing in zsh or bash |
| [Providers and local models](providers-and-local-models.md) | Pick an AI, use several keys, or run a local model with Ollama |
| [System prompts](system-prompts.md) | Change how the AI edits your text |
| [Privacy and safety](privacy-and-safety.md) | Know what is sent, what is stored, and what CleanPing refuses to do |
| [Troubleshooting](troubleshooting.md) | Fix an error message |
| [Architecture](architecture.md) | Find your way around the code |

Contributing? See [CONTRIBUTING.md](../CONTRIBUTING.md). What changed in each version is in the
[CHANGELOG](../CHANGELOG.md). To report a security problem privately, see [SECURITY.md](../SECURITY.md).

## Conventions

- The product is written **CleanPing**; the command, the crates, paths and URLs are lowercase
  (`cleanping`).
- These pages describe the code in this repository. `cleanping --version` tells you which version you
  have, and `cleanping --help` and `cleanping COMMAND --help` always describe what you have installed.
- Examples use a `$` for what you type. Output is shown without it.
- Linux and macOS are supported. Windows is not yet.
