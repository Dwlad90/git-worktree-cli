use std::path::Path;

use git2::{Repository, StatusOptions};

pub(crate) mod branch;
pub(crate) mod commit;
pub(crate) mod worktree;

fn open_repo<P>(repo_path: &P) -> Repository
where
    P: AsRef<Path>,
{
    Repository::open(repo_path).expect("Failed to open repository")
}

pub(crate) fn is_bare_repo<P>(repo_path: &P) -> bool
where
    P: AsRef<Path>,
{
    let repo = open_repo(repo_path);

    repo.is_bare()
}

pub(crate) fn is_worktree<P>(repo_path: &P) -> bool
where
    P: AsRef<Path>,
{
    let repo = open_repo(repo_path);

    repo.is_worktree()
}

pub(crate) fn is_branch_clear<P>(repo_path: &P) -> bool
where
    P: AsRef<Path>,
{
    let repo = Repository::open(repo_path).expect("Failed to open repository");

    let mut status_options = StatusOptions::new();
    status_options.include_untracked(false);

    let statuses = repo
        .statuses(Some(&mut status_options))
        .expect("Failed to get repository statuses");

    if statuses.is_empty() {
        true
    } else {
        if cfg!(debug_assertions) {
            for entry in statuses.iter() {
                let status = entry.status();
                let path = entry.path().unwrap_or("unknown");
                println!("Path: {}, Status: {:?}", path, status);
            }
        }

        false
    }
}
