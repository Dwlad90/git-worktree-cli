use anyhow::Result;
use git2::Repository;
use octocrab::params::State;

use crate::{
    utils::{
        cli::{add_branch_to_repo, add_worktree_to_repo},
        git::{common::get_repo_info, worktree::AddKind},
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

    for pr in prs.clone() {
        if pr_status == PRStatus::All
            || pr_status == PRStatus::Draft && pr.draft.unwrap_or(false)
            || pr_status == PRStatus::Open && !pr.draft.unwrap_or(false)
        {
            let branch_name = &pr.head.ref_field;

            let (command, add_kind) = if repo.is_bare() || repo.is_worktree() {
                add_worktree_to_repo(repo, branch_name)?
            } else {
                add_branch_to_repo(repo, branch_name)?
            };

            if add_kind == AddKind::Added {
                let log = format!(
                    r#"Added workspace for PR #{pr}:
                    branch: {branch}
                    url: {url}
                    goto: {goto}\n
                    "#,
                    pr = pr.number,
                    branch = branch_name,
                    url = pr.url,
                    goto = command
                );

                info!("{}", log);
                println!("{}", log);
            }
        }
    }

    Ok(())
}
