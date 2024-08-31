use std::ffi::OsStr;

use colored::Colorize;
use git2::{Error, Repository, Worktree, WorktreeAddOptions};

use super::{
    branch::{get_local_branch_reference, get_worktree_branch, BranchInfo},
    commit::get_worktree_commit_time,
};

pub(crate) fn get_worktree_names(repo: &Repository) -> Vec<String> {
    let worktree_names = repo.worktrees().unwrap();

    let mut worktree_paths = worktree_names
        .iter()
        .filter_map(|worktree_name| {
            if let Some(worktree_name) = worktree_name {
                let worktree = repo.find_worktree(worktree_name).ok()?;
                let commit_time = get_worktree_commit_time(repo, worktree_name)?;

                Some((worktree.name().unwrap().to_string(), commit_time))
            } else {
                None
            }
        })
        .collect::<Vec<(String, i64)>>();

    worktree_paths.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by commit time in descending order

    worktree_paths.into_iter().map(|(name, _)| name).collect()
}

pub(crate) fn get_worktree_path_by_name(
    repo: &Repository,
    target_worktree_name: &str,
) -> Option<String> {
    let worktree_names = repo.worktrees().unwrap();

    for worktree_name in worktree_names.iter().flatten() {
        let worktree = repo.find_worktree(worktree_name).unwrap();
        let worktree_branch = worktree.name().unwrap_or("");

        if worktree_branch == target_worktree_name {
            return Some(worktree.path().to_string_lossy().to_string());
        }
    }

    None
}

pub(crate) fn get_worktree_by_branch_name(
    repo: &Repository,
    branch_name: &str,
) -> Result<String, Error> {
    let worktree_names = repo.worktrees()?;

    for worktree_name in worktree_names.iter().flatten() {
        if let Ok(worktree_branch) = get_worktree_branch(repo, worktree_name) {
            if worktree_branch == branch_name {
                return Ok(worktree_name.to_string());
            }
        }
    }

    Err(Error::from_str(
        "No worktree found for the given branch name",
    ))
}

pub(crate) fn worktree_exists_by_name<S>(repo: &Repository, worktree_name: S) -> Result<bool, Error>
where
    S: AsRef<OsStr>,
{
    let worktree_names = repo.worktrees()?;

    Ok(worktree_names
        .iter()
        .flatten()
        .any(|name| name == worktree_name.as_ref()))
}

pub(crate) fn get_worktree_by_name<S>(
    repo: &Repository,
    worktree_name: &S,
) -> Result<Worktree, Error>
where
    S: AsRef<OsStr>,
{
    let worktree_names = repo.worktrees()?;

    for name in worktree_names.iter().flatten() {
        if name == worktree_name.as_ref() {
            return repo.find_worktree(name);
        }
    }

    Err(Error::from_str("Worktree not found"))
}

pub(crate) fn add_worktree<S>(
    repo: &Repository,
    worktree_name: &S,
    remote_branch: &Option<BranchInfo>,
) -> Result<Worktree, Error>
where
    S: AsRef<OsStr>,
{
    // Check if the worktree already exists
    if worktree_exists_by_name(repo, worktree_name)? {
        eprintln!(
            "{}",
            "WARNING: Worktree with the given name already exists".yellow()
        );

        return get_worktree_by_name(repo, worktree_name);
    }

    let worktree_path = repo
        .path()
        .parent()
        .expect("Failed to get repo root path")
        .join(worktree_name.as_ref());

    let mut add_options = WorktreeAddOptions::new();

    let head = match remote_branch {
        Some(remote_branch) => {
            let reference = get_local_branch_reference(repo, remote_branch)?;

            Some(reference)
        }
        None => None,
    };

    add_options.reference(head.as_ref());

    repo.worktree(
        worktree_name
            .as_ref()
            .to_string_lossy()
            .to_string()
            .as_str(),
        worktree_path.as_ref(),
        Some(&add_options),
    )
}
