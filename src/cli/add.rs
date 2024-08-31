use std::ffi::OsString;

use git2::Repository;

use crate::utils::cli::{add_branch_to_repo, add_worktree_to_repo};

pub(crate) fn add_sub_command(repo: Repository, name: OsString) {
    let command = if repo.is_bare() || repo.is_worktree() {
        add_worktree_to_repo(&repo, name)
    } else {
        add_branch_to_repo(&repo, name)
    };

    println!("{}", command.expect("Failed to change branch"));
}
