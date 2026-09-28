//! `cleanping init <shell>`: print the shell integration. Touches no files and no database.

use crate::args::Shell;

const ZSH: &str = include_str!("../shell/cleanping.zsh");
const BASH: &str = include_str!("../shell/cleanping.bash");

pub fn script(shell: Shell) -> &'static str {
    match shell {
        Shell::Zsh => ZSH,
        Shell::Bash => BASH,
    }
}
