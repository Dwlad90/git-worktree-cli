use git2::{Branch, BranchType, Repository};

use super::branch::get_worktree_branch_name;

pub(crate) fn get_commit_time(repo: &Repository, branch: &Branch) -> i64 {
    let oid = branch.get().target().expect("Failed to get target OID");
    let commit = repo.find_commit(oid).expect("Failed to find commit");
    commit.time().seconds()
}

pub(crate) fn get_worktree_commit_time(repo: &Repository, worktree_name: &str) -> Option<i64> {
    let branch_name = get_worktree_branch_name(repo, worktree_name).ok()?;

    match repo.find_branch(&branch_name, BranchType::Local) {
        Ok(branch) => {
            let oid = branch.get().target()?;
            let commit = repo.find_commit(oid).ok()?;
            Some(commit.time().seconds())
        }
        Err(e) => {
            eprintln!("Failed to find branch {}: {}", branch_name, e); // Log the error
            None
        }
    }
}
