//! Command-line shape (clap). No behavior here.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "cleanping",
    version,
    about = "Rewrite rough text into clean, clear text with an OpenAI-compatible API.",
    long_about = "Rewrite rough text into clean, clear text with an OpenAI-compatible API.\n\n\
        Give the text as arguments or pipe it in; only the rewritten text goes to stdout.\n\
        Exit codes: 0 ok, 1 failed (network/provider/storage), 2 bad input, 3 no usable key.",
    args_conflicts_with_subcommands = true,
    // "help me fix this" is text to rewrite, not a request for a subcommand's help.
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Text to rewrite. When omitted, the text is read from a pipe.
    pub text: Vec<String>,

    /// Which saved key to use (default: the selected one).
    #[arg(short, long, value_name = "NAME")]
    pub credential: Option<String>,

    /// Also copy the result to the clipboard (needs wl-copy, xclip, xsel or pbcopy).
    #[arg(long)]
    pub copy: bool,

    /// Do not save this run to the local history.
    #[arg(long)]
    pub no_history: bool,

    /// Refuse a reply with more lines than your text, or much longer, or padded with blanks.
    /// The shell key uses this so a reply cannot hide part of itself on your command line.
    #[arg(long)]
    pub keep_shape: bool,

    /// Refuse to send text that looks like it holds a secret (a key, token or password). The
    /// shell key uses this, because command lines often carry them.
    #[arg(long)]
    pub refuse_secrets: bool,

    /// Used by the shell key: read the original and the reply from stdin, separated by a NUL
    /// byte, and print which character ranges of the reply changed. Sends nothing anywhere.
    #[arg(
        long,
        hide = true,
        conflicts_with_all = ["text", "credential", "copy", "no_history", "keep_shape", "refuse_secrets"]
    )]
    pub marks: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage saved API keys.
    Keys {
        #[command(subcommand)]
        action: KeysAction,
    },
    /// Show or change the system prompt sent with every rewrite.
    Prompt {
        #[command(subcommand)]
        action: PromptAction,
    },
    /// Look at, or delete, the rewrites saved on this computer.
    History {
        #[command(subcommand)]
        action: HistoryAction,
    },
    /// Fix the text in FILE with the AI, then review it. Set `VISUAL="cleanping edit"` so the
    /// editor key of Claude Code or Codex (Ctrl+G) opens it.
    Edit(EditArgs),
    /// Guided setup: pick the AI, save its key, choose a system prompt and test the connection.
    /// Run it again later for a small menu.
    Setup,
    /// Print shell integration: add `eval "$(cleanping init zsh)"` to your shell's rc file.
    Init {
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Args)]
pub struct EditArgs {
    /// The file whose text is fixed.
    pub file: PathBuf,
    /// Which saved key to use (default: the selected one).
    #[arg(short, long, value_name = "NAME")]
    pub credential: Option<String>,
    /// Write the edited text into the file without a screen or a question. Text that looks
    /// like it holds a secret is still never sent.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    Zsh,
    Bash,
}

#[derive(Subcommand)]
pub enum KeysAction {
    /// List saved keys (never shows the key itself).
    List,
    /// Save a key. The key is read from stdin or a hidden prompt, never from an argument.
    Add(AddArgs),
    /// Delete a saved key.
    Remove { name: String },
    /// Make a saved key the default.
    Use { name: String },
    /// Send one tiny fixed request to check that a saved key works (default: the selected key).
    Test { name: Option<String> },
}

#[derive(Args)]
pub struct AddArgs {
    /// A preset: DeepSeek, OpenAI, OpenRouter, "Ollama (local)".
    #[arg(long)]
    pub provider: Option<String>,
    /// A name for this key (defaults to the provider).
    #[arg(long)]
    pub name: Option<String>,
    /// API URL (defaults to the provider's).
    #[arg(long)]
    pub url: Option<String>,
    /// Model name (defaults to the provider's).
    #[arg(long)]
    pub model: Option<String>,
    /// Read the API key as one line from stdin (on a terminal it is asked for, hidden).
    #[arg(long)]
    pub key_stdin: bool,
}

#[derive(Subcommand)]
pub enum HistoryAction {
    /// Show the newest rewrites, one line each.
    List {
        /// How many to show.
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Delete every saved rewrite.
    Clear {
        /// Confirm the deletion.
        #[arg(long)]
        yes: bool,
    },
    /// Delete rewrites older than DAYS days.
    Purge {
        #[arg(long, value_name = "DAYS", value_parser = clap::value_parser!(u32).range(1..=36500))]
        older_than: u32,
    },
}

#[derive(Subcommand)]
pub enum PromptAction {
    /// Print the current system prompt.
    Show,
    /// Replace it with TEXT, or with stdin when TEXT is omitted and input is piped.
    Set { text: Option<String> },
    /// List the ready-made system prompts (the one in use is marked with *).
    Presets,
    /// Use a ready-made system prompt; `cleanping prompt presets` lists them.
    Use {
        name: String,
        /// Replace a system prompt you wrote yourself.
        #[arg(long)]
        yes: bool,
    },
}
