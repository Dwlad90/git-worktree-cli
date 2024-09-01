use std::{ffi::OsString, fs};

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use cli::{
    add::{add_from_pr_sub_command, add_sub_command},
    change_branch::change_branch_sub_command,
    completions::completions_sub_command,
};
use octocrab::params::State;
use utils::git::open_repo;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod cli;
mod utils;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum RepoSource {
    // Source from a git repository
    Git,
    // Source from a Github repository
    PR,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum PRStatus {
    // Source from a git repository
    Draft,
    // Source from a Github repository
    Open,
    All,
}

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

    #[command(about = "Add a new worktree/branch to a git repository from a PR")]
    AddByPR {
        #[clap(
            short = 'p',
            long,
            help = "Path to the git repository",
            default_value = ".",
            value_hint = clap::ValueHint::DirPath
        )]
        repo_path: OsString,
        #[clap(
            short = 's',
            long,
            value_enum,
            help = "Status of PR to add",
            value_name = "PR_STATUS",
            default_value = "open"
        )]
        pr_status: PRStatus,
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

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let opt = CLI::parse();

    match opt.subcommands {
        SubCommands::Add { name, repo_path } => {
            let repo_path = fs::canonicalize(repo_path).expect("Failed to get worktree path");
            let repo = open_repo(&repo_path);

            match add_sub_command(repo, name) {
                Ok(_) => {
                    info!("Worktree/branch was added successfully");
                }
                Err(e) => {
                    error!("Failed to add worktree/branch: {:?}", e);
                }
            }
        }
        SubCommands::AddByPR {
            repo_path,
            pr_status,
        } => {
            let repo_path = fs::canonicalize(repo_path).expect("Failed to get worktree path");
            let repo = open_repo(&repo_path);

            match add_from_pr_sub_command(repo, State::Open, pr_status).await {
                Ok(_) => {
                    info!("All PRs were added successfully");
                }
                Err(e) => {
                    error!("Failed to get PR: {:?}", e);
                }
            }
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
