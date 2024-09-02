use std::{path::PathBuf, sync::Arc};

use anyhow::{bail, Result};
use futures::future::join_all;
use git2::Repository;
use octocrab::{models::pulls::PullRequest, params::State, Page};
use tokio::{spawn, task::JoinHandle};

use crate::{
    utils::{
        cli::{add_branch_to_repo, add_worktree_to_repo},
        git::{
            common::get_repo_info,
            open_repo,
            worktree::{worktree_exists_by_branch_name, AddKind},
        },
        search::common::{get_fuzzy_options, handle_final_key},
    },
    PRKind, PrSelection,
};

use super::common::setup_octocrab;
pub async fn add_workspace_by_pull_requests(
    repo: &Repository,
    pr_state: State,
    pr_kind: PRKind,
    pr_selection: PrSelection,
) -> Result<()> {
    let repo_info = get_repo_info(repo)?;
    let gh = setup_octocrab().await?;
    let prs = gh
        .pulls(repo_info.owner, repo_info.repo)
        .list()
        .state(pr_state)
        .send()
        .await?;

    let filtered_prs = filter_prs(prs, repo, pr_kind);
    let selected_prs = select_prs(filtered_prs, pr_selection);

    let repo_path = Arc::new(repo.path().to_path_buf());

    let tasks: Vec<JoinHandle<Result<()>>> = selected_prs
        .await?
        .into_iter()
        .map(|pr| {
            let repo_path = Arc::clone(&repo_path);
            spawn(async move { create_branch_for_pull_request(repo_path, pr).await })
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

fn matches_pr_kind(pr: &PullRequest, pr_kind: PRKind) -> bool {
    pr_kind == PRKind::All
        || (pr_kind == PRKind::Draft && pr.draft.unwrap_or(false))
        || (pr_kind == PRKind::Open && !pr.draft.unwrap_or(false))
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

fn filter_prs(prs: Page<PullRequest>, repo: &Repository, pr_kind: PRKind) -> Vec<PullRequest> {
    prs.into_iter()
        .filter(|pr| {
            let branch_name = &pr.head.ref_field;
            matches_pr_kind(pr, pr_kind)
                && !worktree_exists_by_branch_name(repo, branch_name).unwrap_or(false)
        })
        .collect()
}

async fn select_prs(prs: Vec<PullRequest>, pr_selection: PrSelection) -> Result<Vec<PullRequest>> {
    match pr_selection {
        PrSelection::All => Ok(prs),
        PrSelection::Multiple | PrSelection::Single => {
            let items = prs
                .iter()
                .map(|pr| pr.head.ref_field.clone())
                .collect::<Vec<String>>()
                .join("\n");

            let out = get_fuzzy_options(
                None,
                pr_selection == PrSelection::Multiple,
                String::from("PR branch"),
                items,
            )
            .await;

            match out {
                Some(out) => {
                    let selected_prs = out
                        .selected_items
                        .iter()
                        .map(|selected_item| {
                            (**selected_item)
                                .as_any()
                                .downcast_ref::<String>()
                                .unwrap()
                                .to_owned()
                        })
                        .collect::<Vec<String>>();

                    handle_final_key(&out, &selected_prs)?;

                    let prs = prs
                        .into_iter()
                        .filter(|pr| {
                            selected_prs
                                .iter()
                                .any(|selected_pr| *selected_pr == pr.head.ref_field)
                        })
                        .collect();

                    Ok(prs)
                }
                None => bail!("No available PRs to select"),
            }
        }
    }
}
