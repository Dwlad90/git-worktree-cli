use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use futures::future::join_all;
use git2::Repository;
use octocrab::{models::pulls::PullRequest, params::State};
use tokio::{spawn, task::JoinHandle};

use crate::{
    utils::{
        cli::{add_branch_to_repo, add_worktree_to_repo},
        git::{common::get_repo_info, open_repo, worktree::AddKind},
    },
    PRStatus,
};

use super::common::setup_octocrab;
pub async fn add_workspace_by_pull_requests(
    repo: &Repository,
    pr_state: State,
    pr_status: PRStatus,
) -> Result<()> {
    let repo_info = get_repo_info(repo)?;
    let gh = setup_octocrab().await?;
    let prs = gh
        .pulls(repo_info.owner, repo_info.repo)
        .list()
        .state(pr_state)
        .send()
        .await?;

    let repo_path = Arc::new(repo.path().to_path_buf());

    let tasks: Vec<JoinHandle<Result<()>>> = prs
        .into_iter()
        .filter_map(|pr| {
            if matches_pr_status(&pr, pr_status) {
                let repo_path = Arc::clone(&repo_path);
                Some(spawn(async move {
                    create_branch_for_pull_request(repo_path, pr).await
                }))
            } else {
                None
            }
        })
        .collect();

    let results = join_all(tasks).await;

    // Collect errors for more informative output
    let errors: Vec<_> = results
        .into_iter()
        .filter_map(|result| result.err())
        .collect();

    if !errors.is_empty() {
        error!("Tasks failed: {:?}", errors);
    }

    Ok(())
}

fn matches_pr_status(pr: &PullRequest, pr_status: PRStatus) -> bool {
    pr_status == PRStatus::All
        || (pr_status == PRStatus::Draft && pr.draft.unwrap_or(false))
        || (pr_status == PRStatus::Open && !pr.draft.unwrap_or(false))
}

async fn create_branch_for_pull_request(repo_path: Arc<PathBuf>, pr: PullRequest) -> Result<()> {
    let branch_name = &pr.head.ref_field;
    let repo = open_repo(&repo_path.as_path());

    let (command, add_kind) = if repo.is_bare() || repo.is_worktree() {
        add_worktree_to_repo(&repo, branch_name)?
    } else {
        add_branch_to_repo(&repo, branch_name)?
    };

    let pr_url = if let Some(pr_url) = pr.html_url {
        pr_url.to_string()
    } else {
        "URL not available".to_string()
    };

    if add_kind == AddKind::Added {
        let log = format!(
            "Added workspace for PR #{pr}:
            branch: {branch}
            url: {url}
            goto: {goto}\n
            ",
            pr = pr.number,
            branch = branch_name,
            url = pr_url,
            goto = command
        );

        info!("{}", log);
        println!("{}", log);
    }

    Ok(())
}
