use std::path::PathBuf;

use anyhow::{Error, Ok, Result};
use git2::{RemoteCallbacks, Repository};

use crate::utils::github::common::{parse_github_url, GitRepoInfo};

pub(crate) fn get_credentials_callback<'a>() -> RemoteCallbacks<'a> {
    let mut callbacks = RemoteCallbacks::new();

    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
    });

    callbacks
}

pub(crate) fn get_repo_info(repo: &Repository) -> Result<GitRepoInfo> {
    let remote = repo.find_remote("origin")?;

    let url = remote
        .url()
        .ok_or_else(|| anyhow::anyhow!("Remote URL not found"))?;

    // Parse the URL to get the owner and repository name
    let git_repo_info = parse_github_url(url)?;

    Ok(git_repo_info)
}

fn get_worktree_root_path(repo: &Repository) -> Result<PathBuf> {
    repo.path()
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .ok_or_else(|| Error::msg("Failed to get worktree root path"))
}

pub(crate) fn get_root_repo_path(repo: &Repository) -> Result<PathBuf> {
    if repo.is_worktree() {
        return get_worktree_root_path(repo);
    }

    Ok(repo.path().to_path_buf())
}
