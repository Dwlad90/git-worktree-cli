use std::env::{self, VarError};

use anyhow::{bail, Result};
use log::info;
use octocrab::Octocrab;
use regex::Regex;

pub(crate) fn retrieve_github_access_token() -> Result<String, VarError> {
    env::var("WORKTREE_CLI_GITHUB_TOKEN")
}

pub async fn setup_octocrab() -> Result<Octocrab, anyhow::Error> {
    let mut builder = octocrab::OctocrabBuilder::new();

    if let Ok(access_token) = retrieve_github_access_token() {
        builder = builder.personal_token(access_token);
        let octocrab = builder.build()?;
        match octocrab.ratelimit().get().await {
            Ok(rate) => info!(
                "GitHub API rate limit: {}/{}.",
                rate.resources.core.used, rate.resources.core.limit
            ),
            Err(e) => {
                bail!(
                "Failed to get rate limit info: {}. GitHub Personal Access Token might be invalid.",
                e
            );
            }
        }
        Ok(octocrab)
    } else {
        builder.build().map_err(anyhow::Error::from)
    }
}

#[derive(Debug)]
pub(crate) struct GitRepoInfo {
    pub owner: String,
    pub repo: String,
}

// Helper function to parse GitHub URL
pub(crate) fn parse_github_url(url: &str) -> Result<GitRepoInfo> {
    let re = Regex::new(r"github\.com[:/](?P<owner>[^/]+)/(?P<repo>[^/.]+)(?:\.git)?$")?;

    let caps = re
        .captures(url)
        .ok_or_else(|| anyhow::anyhow!("Invalid GitHub URL"))?;
    let owner = caps
        .name("owner")
        .ok_or_else(|| anyhow::anyhow!("Owner not found"))?
        .as_str();
    let repo = caps
        .name("repo")
        .ok_or_else(|| anyhow::anyhow!("Repository name not found"))?
        .as_str();

    Ok(GitRepoInfo {
        owner: owner.to_string(),
        repo: repo.to_string(),
    })
}
