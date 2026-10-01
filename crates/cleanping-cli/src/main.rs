//! `cleanping`: a thin command-line front end over `cleanping-core`.

mod args;
mod clipboard;
mod command_warnings;
mod connection;
mod edit;
mod exit;
mod guard;
mod hints;
mod history;
mod init;
mod input;
mod keys;
mod marks;
mod no_history;
mod output;
mod progress;
mod prompt;
mod rewrite;
mod secret_input;
mod services;
mod setup;
mod update;
mod writer;

use clap::Parser;
use cleanping_core::domain::errors::Result;

use args::{Cli, Command};
use services::Services;

fn execute(cli: &Cli) -> Result<()> {
    match &cli.command {
        // Shell start-up runs this on every new terminal: no database, no files.
        Some(Command::Init { shell }) => output::write(init::script(*shell)),
        Some(Command::Edit(args)) => edit::run(&Services::open()?, args),
        Some(Command::Writer) => writer::run().map(|never| match never {}),
        // Only checks: no database, no files. `--check` is today's only behavior.
        Some(Command::Update(_)) => update::run(),
        Some(Command::Setup) => setup::run(&Services::open()?),
        Some(Command::Keys { action }) => keys::run(&Services::open()?, action),
        Some(Command::Prompt { action }) => prompt::run(&Services::open()?, action),
        Some(Command::History { action }) => history::run(&Services::open()?, action),
        None if cli.marks => marks::run(),
        None => rewrite::run(cli, &Services::open()?),
    }
}

fn main() {
    let cli = Cli::parse();
    let code = match execute(&cli) {
        Ok(()) => exit::OK,
        Err(error) => exit::fail(&error),
    };
    std::process::exit(code);
}
