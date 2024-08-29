use std::{ffi::OsStr, path::Path};

use git2::Error;

use super::{branch::get_worktree_branch, commit::get_worktree_commit_time, open_repo};

pub(crate) fn get_worktree_names<P>(repo_path: &P) -> Vec<String>
where
    P: AsRef<Path>,
{
    let repo = open_repo(repo_path);

    let worktree_names = repo.worktrees().unwrap();

    let mut worktree_paths = worktree_names
        .iter()
        .filter_map(|worktree_name| {
            if let Some(worktree_name) = worktree_name {
                let worktree = repo.find_worktree(worktree_name).ok()?;
                let commit_time = get_worktree_commit_time(&repo, worktree_name)?;

                Some((worktree.name().unwrap().to_string(), commit_time))
            } else {
                None
            }
        })
        .collect::<Vec<(String, i64)>>();

    worktree_paths.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by commit time in descending order

    worktree_paths.into_iter().map(|(name, _)| name).collect()
}

pub(crate) fn get_worktree_path_by_name<P>(
    repo_path: &P,
    target_worktree_name: &str,
) -> Option<String>
where
    P: AsRef<Path>,
{
    let repo = open_repo(repo_path);

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

pub(crate) fn get_worktree_by_branch_name<P>(
    repo_path: &P,
    branch_name: &str,
) -> Result<String, Error>
where
    P: AsRef<Path>,
{
    let repo = open_repo(repo_path);

    let worktree_names = repo.worktrees()?;

    for worktree_name in worktree_names.iter().flatten() {
        if let Ok(worktree_branch) = get_worktree_branch(&repo, worktree_name) {
            if worktree_branch == branch_name {
                return Ok(worktree_name.to_string());
            }
        }
    }

    Err(Error::from_str(
        "No worktree found for the given branch name",
    ))
}

pub(crate) fn worktree_exists_by_name<P, S>(repo_path: &P, worktree_name: S) -> Result<bool, Error>
where
    P: AsRef<Path>,
    S: AsRef<OsStr>,
{
    let repo = open_repo(repo_path);

    let worktree_names = repo.worktrees()?;

    Ok(worktree_names
        .iter()
        .flatten()
        .any(|name| name == worktree_name.as_ref()))
}
