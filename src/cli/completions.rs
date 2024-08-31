use clap::CommandFactory;
use clap_complete::{generate, Shell};

use crate::CLI;

pub(crate) fn completions_sub_command(shell: Shell) {
    generate(
        shell,
      &mut CLI::command(),
        "git-worktree-cli",
        &mut std::io::stdout(),
    );
}
