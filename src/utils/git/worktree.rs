use std::ffi::OsStr;

use anyhow::{bail, Result};
use git2::{Error, Repository, Worktree, WorktreeAddOptions};

use crate::utils::git::common::get_root_repo_path;

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
) -> Result<String> {
    let worktree_names = repo.worktrees().unwrap();

    for worktree_name in worktree_names.iter().flatten() {
        let worktree = repo.find_worktree(worktree_name).unwrap();
        let worktree_name = worktree.name().expect("Failed to get worktree name");

        if worktree_name == normalize_workspace_name(target_worktree_name) {
            return Ok(worktree.path().to_string_lossy().to_string());
        }
    }

    bail!("Worktree not found");
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

pub(crate) fn worktree_exists_by_branch_name(
    repo: &Repository,
    branch_name: &str,
) -> Result<bool, Error> {
    let worktree_names = repo.worktrees()?;

    Ok(worktree_names.iter().flatten().any(|name| {
        if let Ok(worktree_branch) = get_worktree_branch(repo, name) {
            worktree_branch == branch_name
        } else {
            false
        }
    }))
}

pub(crate) fn worktree_exists_by_name<S>(
    repo: &Repository,
    worktree_name: &S,
) -> Result<bool, Error>
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

#[derive(Debug, PartialEq)]
pub(crate) enum AddKind {
    Existed,
    Added,
}

pub(crate) fn add_worktree<S>(
    repo: &Repository,
    worktree_name: &S,
    remote_branch: &Option<BranchInfo>,
) -> Result<(Worktree, AddKind)>
where
    S: AsRef<OsStr>,
{
    let worktree_name = normalize_workspace_name(&worktree_name.as_ref().to_string_lossy());

    // Check if the worktree already exists
    if worktree_exists_by_name(repo, &worktree_name)? {
        warn!("Worktree with name `{}` already exists", worktree_name);

        return Ok((
            get_worktree_by_name(repo, &worktree_name)?,
            AddKind::Existed,
        ));
    }

    let worktree_path = get_root_repo_path(repo)
        .expect("Failed to get repo root path")
        .join(&worktree_name);

    let mut add_options = WorktreeAddOptions::new();

    let head = match remote_branch {
        Some(remote_branch) => {
            let reference = get_local_branch_reference(repo, remote_branch)?;

            Some(reference)
        }
        None => None,
    };

    add_options.reference(head.as_ref());

    Ok((
        repo.worktree(&worktree_name, worktree_path.as_ref(), Some(&add_options))?,
        AddKind::Added,
    ))
}

fn normalize_workspace_name(workspace_name: &str) -> String {
    workspace_name.replace("/", "_")
}
