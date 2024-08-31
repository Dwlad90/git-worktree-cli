use std::{ffi::OsString, fs};

use clap::{Parser, Subcommand};
use clap_complete::Shell;
use cli::{
    add::add_sub_command, change_branch::change_branch_sub_command,
    completions::completions_sub_command,
};
use utils::git::open_repo;

mod cli;
mod utils;

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
enum SubCommands {
    #[command(
        arg_required_else_help = true,
        about = "Add a new worktree/branch to a git repository"
    )]
    Add {
        #[arg(
            value_enum,
            help = "Name of the worktree/branch to add",
            required = true,
            value_name = "name"
        )]
        name: OsString,
        #[clap(
            short = 'p',
            long,
            help = "Path to the git repository",
            default_value = ".",
            value_hint = clap::ValueHint::DirPath
        )]
        repo_path: OsString,
    },
    #[command(about = "Change branch or worktree of a git repository")]
    ChangeBranch {
        #[clap(
            short = 'p',
            long,
            help = "Path to the git repository",
            default_value = ".",
            value_hint = clap::ValueHint::DirPath
        )]
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
        #[arg(
            value_enum,
            help = "Shell to generate completions for",
            required = true
        )]
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
        SubCommands::Add { name, repo_path } => {
            let repo_path = fs::canonicalize(repo_path).expect("Failed to get worktree path");
            let repo = open_repo(&repo_path);

            add_sub_command(repo, name);
        }
        SubCommands::Completions { shell } => {
            completions_sub_command(shell);
        }
        SubCommands::ChangeBranch {
            repo_path,
            branch,
            worktree,
            query,
        } => {
            let repo_path = fs::canonicalize(repo_path).expect("Failed to get worktree path");

            let repo = open_repo(&repo_path);

            change_branch_sub_command(repo, branch, worktree, query);
        }
    }
}
