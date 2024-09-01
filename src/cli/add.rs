use anyhow::Result;
use octocrab::params::State;

use std::ffi::OsString;

use git2::Repository;

use crate::{
    utils::{
        cli::{add_branch_to_repo, add_worktree_to_repo},
        github::pr::add_workspace_by_pull_requests,
    },
    PRStatus,
};

pub(crate) fn add_sub_command(repo: Repository, name: OsString) -> Result<()> {
    let (command, _) = if repo.is_bare() || repo.is_worktree() {
        add_worktree_to_repo(&repo, name)?
    } else {
        add_branch_to_repo(&repo, name)?
    };

    println!("{}", command);

    Ok(())
}

pub(crate) async fn add_from_pr_sub_command(
    repo: Repository,
    pr_state: State,
    pr_status: PRStatus,
) -> Result<()> {
    add_workspace_by_pull_requests(&repo, pr_state, pr_status).await
}
