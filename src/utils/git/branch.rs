use std::ffi::OsStr;

use git2::{BranchType, Error, Oid, Repository};
use indexmap::IndexMap;

use super::{commit::get_commit_time, worktree::get_worktree_names};

pub(crate) fn get_branches(repo: &Repository, branch_type: BranchType) -> Vec<BranchInfo> {
    let branches = repo
        .branches(Some(branch_type))
        .expect("Failed to get local branches");

    let mut local_branches: Vec<(BranchInfo, i64)> = branches
        .filter_map(|branch| {
            branch.ok().and_then(|(branch, _)| {
                branch.name().ok().flatten().map(|name| {
                    (
                        BranchInfo {
                            name: name.to_string(),
                            head: branch.get().target().unwrap().to_string(),
                        },
                        get_commit_time(repo, &branch),
                    )
                })
            })
        })
        .collect();

    local_branches.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by commit time in descending order

    local_branches.into_iter().map(|(name, _)| name).collect()
}
pub(crate) fn get_worktree_branches(repo: &Repository) -> Result<IndexMap<String, String>, Error> {
    let worktree_names = get_worktree_names(repo);
    let mut worktree_branches = IndexMap::new();

    for worktree_name in worktree_names.iter() {
        if let Ok(worktree_branch) = get_worktree_branch(repo, worktree_name) {
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

pub(crate) fn branch_exists_by_name<S>(
    repo: &Repository,
    branch_name: &S,
    branch_type: BranchType,
) -> Result<bool, Error>
where
    S: AsRef<OsStr>,
{
    let branches = repo.branches(Some(branch_type))?;

    for branch in branches {
        let (branch, _) = branch?;
        if let Some(name) = branch.name()? {
            if name == branch_name.as_ref() {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

pub(crate) fn _check_remote_branch_exist(repo: &Repository, branch_name: &String) -> bool {
    let remote_branches = repo.branches(Some(BranchType::Remote)).unwrap();

    for remote_branch in remote_branches {
        let (branch, _) = remote_branch.unwrap();
        if let Some(name) = branch.name().unwrap() {
            if name == branch_name {
                return true;
            }
        }
    }

    false
}

#[derive(Debug)]
pub(crate) struct BranchInfo {
    pub name: String,
    pub head: String,
}

pub(crate) fn get_branch<S>(
    repo: &Repository,
    branch_name: &S,
    branch_type: BranchType,
) -> Option<BranchInfo>
where
    S: AsRef<OsStr>,
{
    let remote_branches = repo.branches(Some(branch_type)).unwrap();

    for remote_branch in remote_branches {
        let (branch, _) = remote_branch.unwrap();
        if let Some(name) = branch.name().unwrap() {
            let comparison_branch_name = if branch_type == BranchType::Remote {
                format!("origin/{}", branch_name.as_ref().to_str().unwrap())
            } else {
                branch_name.as_ref().to_str().unwrap().to_string()
            };

            if name == comparison_branch_name {
                let head = branch.get().target().unwrap().to_string();

                return Some(BranchInfo {
                    name: branch_name.as_ref().to_str().unwrap().to_string(),
                    head,
                });
            }
        }
    }

    None
}

pub(crate) fn _get_remote_branch_reference<'a>(
    repo: &'a Repository,
    branch: &'a BranchInfo,
) -> Result<git2::Reference<'a>, git2::Error> {
    let remote_branch_ref = format!("refs/remotes/origin/{}", branch.name);
    repo.find_reference(&remote_branch_ref)
}

pub(crate) fn get_local_branch_reference<'a>(
    repo: &'a Repository,
    branch: &'a BranchInfo,
) -> Result<git2::Reference<'a>, git2::Error> {
    let local_branch_ref = format!("refs/heads/{}", branch.name);
    let reference = repo.find_reference(&local_branch_ref);

    match reference {
        Ok(reference) => Ok(reference),
        Err(_) => {
            let oid = Oid::from_str(&branch.head)?;
            repo.reference(&local_branch_ref, oid, false, "")
        }
    }
}

pub(crate) fn add_branch<S>(repo: &Repository, branch_name: &S) -> Result<BranchInfo, Error>
where
    S: AsRef<OsStr>,
{
    if branch_exists_by_name(repo, branch_name, BranchType::Local).unwrap_or(false) {
        let branch = get_branch(repo, &branch_name, BranchType::Local).unwrap();

        return Ok(branch);
    }

    let remote_branch = get_branch(repo, &branch_name, BranchType::Remote);

    if let Some(remote_branch) = remote_branch {
        return Ok(remote_branch);
    }

    let branch_name = if let Some(branch) = branch_name.as_ref().to_str() {
        branch.to_string()
    } else {
        branch_name.as_ref().to_string_lossy().to_string()
    };

    let head = repo
        .head()
        .unwrap()
        .peel_to_commit()
        .expect("Failed to get head commit");

    let branch = repo
        .branch(&branch_name, &head, false)
        .expect("Failed to create branch");

    Ok(BranchInfo {
        name: branch.name().unwrap().unwrap().to_string(),
        head: branch.get().target().unwrap().to_string(),
    })
}
