use std::path::Path;

use git2::{BranchType, Error, Repository};
use indexmap::IndexMap;

use super::{commit::get_commit_time, open_repo, worktree::get_worktree_names};

pub(crate) fn get_local_branches<P>(repo_path: &P) -> Vec<String>
where
    P: AsRef<Path>,
{
    let repo = open_repo(repo_path);

    let branches = repo
        .branches(Some(BranchType::Local))
        .expect("Failed to get local branches");

    let mut local_branches: Vec<(String, i64)> = branches
        .filter_map(|branch| {
            branch.ok().and_then(|(branch, _)| {
                branch
                    .name()
                    .ok()
                    .flatten()
                    .map(|name| (String::from(name), get_commit_time(&repo, &branch)))
            })
        })
        .collect();

    local_branches.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by commit time in descending order

    local_branches.into_iter().map(|(name, _)| name).collect()
}
pub(crate) fn get_worktree_branches<P>(repo_path: &P) -> Result<IndexMap<String, String>, Error>
where
    P: AsRef<Path>,
{
    let repo = open_repo(repo_path);
    let worktree_names = get_worktree_names(repo_path);
    let mut worktree_branches = IndexMap::new();

    for worktree_name in worktree_names.iter() {
        if let Ok(worktree_branch) = get_worktree_branch(&repo, worktree_name) {
            worktree_branches.insert(worktree_name.to_string(), worktree_branch);
        }
    }

    Ok(worktree_branches)
}

pub(crate) fn get_worktree_branch(repo: &Repository, worktree_name: &str) -> Result<String, Error> {
    let worktree_path = repo.path().parent().unwrap().join(worktree_name);
    let worktree_repo = Repository::open(worktree_path)?;
    let head = worktree_repo.head()?;
    let branch_name = head
        .shorthand()
        .ok_or_else(|| Error::from_str("Failed to get branch name"))?;
    Ok(branch_name.to_string())
}

pub(crate) fn get_worktree_branch_name(
    repo: &Repository,
    worktree_name: &str,
) -> Result<String, Error> {
    let worktree_path = repo.path().parent().unwrap().join(worktree_name);
    let worktree_repo = Repository::open(worktree_path)?;
    let head = worktree_repo.head()?;
    let branch_name = head
        .shorthand()
        .ok_or_else(|| Error::from_str("Failed to get branch name"))?;
    Ok(branch_name.to_string())
}

pub(crate) fn branch_exists_by_name<P>(repo_path: &P, branch_name: &String) -> Result<bool, Error>
where
    P: AsRef<Path>,
{
    let repo = open_repo(repo_path);

    let branches = repo.branches(None)?;

    for branch in branches {
        let (branch, _) = branch?;
        if let Some(name) = branch.name()? {
            if name == branch_name {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
