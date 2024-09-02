use std::ffi::OsString;

use anyhow::Result;
use git2::Repository;

use crate::utils::cli::{change_branch_of_bare_or_worktree_repo, change_branch_of_regular_repo};

pub(crate) async fn change_branch_sub_command(
    repo: Repository,
    branch: Option<OsString>,
    worktree: Option<OsString>,
    query: Option<OsString>,
) -> Result<()> {
    let branch = branch.map(|os_str| os_str.to_string_lossy().into_owned());
    let worktree = worktree.map(|os_str| os_str.to_string_lossy().into_owned());
    let query = query.map(|os_str| os_str.to_string_lossy().into_owned());

    let command = if repo.is_bare() || repo.is_worktree() {
        change_branch_of_bare_or_worktree_repo(&repo, &branch, &worktree, query).await
    } else {
        change_branch_of_regular_repo(&repo, &branch, query).await
    };

    println!("{}", command?);

    Ok(())
}
