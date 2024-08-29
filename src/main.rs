use std::{ffi::OsString, io, path::Path};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use skim::prelude::*;
use utils::{
    cli::{change_branch_of_bare_or_worktree_repo, change_branch_of_regular_repo},
    git::{is_bare_repo, is_worktree},
};

mod utils;

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
enum SubCommands {
    #[command(about = "Change branch or worktree of a git repository")]
    ChangeBranch {
        #[clap(short='p', long, help = "Path to the git repository", default_value = ".")]
        repo_path: OsString,
        #[clap(short, long, help = "Branch name to change to")]
        branch: Option<OsString>,
        #[clap(short, long, help = "Worktree name to change to")]
        worktree: Option<OsString>,
        #[clap(short, long, help = "Query string to filter results")]
        query: Option<OsString>,
    },
    #[command(arg_required_else_help = true, about = "Generate shell completions")]
    Completions {
        #[arg(value_enum, help="Shell to generate completions for")]
        shell: Shell,
    },
}

#[derive(Debug, Parser)]
#[command(name = "git-worktree-cli", version, about, author)]
#[command(about = "CLI for working with git worktree", long_about = None)]
#[command(propagate_version = true)]
pub struct CLI {
    #[clap(subcommand)]
    subcommands: SubCommands,
}

fn main() {
    let opt = CLI::parse();

    match opt.subcommands {
        SubCommands::Completions { shell } => {
            generate(
                shell,
                &mut CLI::command(),
                "git-worktree-cli",
                &mut io::stdout(),
            );
        }
        SubCommands::ChangeBranch {
            repo_path,
            branch,
            worktree,
            query,
        } => {
            let query_string = query.map(|os_str| os_str.to_string_lossy().to_string());

            let options = SkimOptionsBuilder::default()
                .query(query_string.as_deref())
                .multi(false)
                .algorithm(FuzzyAlgorithm::SkimV2)
                .build()
                .unwrap();

            let item_reader = SkimItemReader::default();

            let repo_path = Path::new(&repo_path);

            let branch = branch.map(|os_str| os_str.to_string_lossy().into_owned());
            let worktree = worktree.map(|os_str| os_str.to_string_lossy().into_owned());

            let command = if is_bare_repo(&repo_path) || is_worktree(&repo_path) {
                change_branch_of_bare_or_worktree_repo(
                    &repo_path,
                    &options,
                    &item_reader,
                    &branch,
                    &worktree,
                )
            } else {
                change_branch_of_regular_repo(&repo_path, &options, &item_reader, &branch)
            };

            println!("{}", command.expect("Failed to change branch"));
        }
    }
}
