use std::path::Path;

use git2::{Repository, StatusOptions};

pub(crate) mod branch;
pub(crate) mod commit;
pub(crate) mod common;
pub(crate) mod fetch;
pub(crate) mod worktree;

pub(crate) fn open_repo<P>(repo_path: &P) -> Repository
where
    P: AsRef<Path>,
{
    let Ok(repo) = Repository::open(repo_path) else {
        error!("Failed to open repository at {:?}", repo_path.as_ref());
        std::process::exit(exitcode::SOFTWARE);
    };

    repo
}

pub(crate) fn is_branch_clear(repo: &Repository) -> bool {
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
                debug!("Path: {}, Status: {:?}", path, status);
            }
        }

        false
    }
}
