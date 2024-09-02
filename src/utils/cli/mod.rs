use std::ffi::OsStr;

use anyhow::Result;
use git2::{BranchType, Error, Repository};

use crate::utils::git::{
    branch::{branch_exists_by_name, get_branch, get_worktree_branches},
    worktree::{get_worktree_by_branch_name, get_worktree_path_by_name, worktree_exists_by_name},
};

use super::{
    git::{
        branch::{add_branch, get_branches, BranchInfo},
        fetch::fetch_all,
        is_branch_clear,
        worktree::{add_worktree, AddKind},
    },
    search::common::{get_fuzzy_options, handle_final_key},
};

pub(crate) async fn change_branch_of_bare_or_worktree_repo(
    repo: &Repository,
    branch_name_arg: &Option<String>,
    worktree_name_arg: &Option<String>,
    query: Option<String>,
) -> Result<String> {
    let worktree_name_from_args = if let Some(worktree_name) = worktree_name_arg {
        if worktree_exists_by_name(repo, &worktree_name).unwrap_or(false) {
            Some(worktree_name.to_string())
        } else {
            None
        }
    } else if let Some(branch_name) = branch_name_arg {
        if branch_exists_by_name(repo, &branch_name, BranchType::Local).unwrap_or(false) {
            Some(get_worktree_by_branch_name(repo, branch_name).unwrap())
        } else {
            None
        }
    } else {
        None
    };

    let worktree_name = if let Some(worktree_name_from_args) = worktree_name_from_args {
        worktree_name_from_args
    } else {
        let branch_icon = "";

        let worktree_branch_map =
            get_worktree_branches(repo).expect("Failed to get worktree branches");

        let items = worktree_branch_map
            .into_iter()
            .map(|(workspace, branch)| {
                if workspace == branch {
                    workspace
                } else {
                    format!("{} -> {}{}", workspace, branch_icon, branch)
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        let out = get_fuzzy_options(query, false, String::from("Worktree branch"), items).await;

        if let Some(out) = out {
            let selected_items = out
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

            handle_final_key(&out, &selected_items)?;

            let selected_item = selected_items.first().expect("Workspace not selected");

            let selected_branch_name = selected_item
                .split(branch_icon)
                .last()
                .expect("Failed to get selected worktree name");

            selected_branch_name.to_string()
        } else {
            panic!("Workspace not selected")
        }
    };

    let worktree_path = get_worktree_path_by_name(repo, &worktree_name)?;

    Ok(format!("cd {}", worktree_path))
}

pub(crate) async fn change_branch_of_regular_repo(
    repo: &Repository,
    branch_name_arg: &Option<String>,
    query: Option<String>,
) -> Result<String> {
    if !is_branch_clear(repo) {
        warn!("Branch has uncommitted changes");
        std::process::exit(exitcode::SOFTWARE);
    }
    let branch_name_arg = if let Some(selected_branch_name) = branch_name_arg {
        if branch_exists_by_name(repo, &selected_branch_name, BranchType::Local).unwrap_or(false) {
            Some(selected_branch_name.to_string())
        } else {
            None
        }
    } else {
        None
    };

    let branch_name = if let Some(branch_name) = branch_name_arg {
        branch_name
    } else {
        let local_branches: Vec<BranchInfo> = get_branches(repo, BranchType::Local);

        let items = local_branches
            .iter()
            .map(|branch| branch.name.clone())
            .collect::<Vec<String>>()
            .join("\n");

        let out = get_fuzzy_options(query, false, String::from("Branch"), items).await;

        if let Some(out) = out {
            let selected_items = out
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

            handle_final_key(&out, &selected_items)?;

            let selected_branch = selected_items.first().expect("Branch not selected");

            selected_branch.to_string()
        } else {
            panic!("Branch not selected");
        }
    };

    Ok(format!("git checkout {}", branch_name))
}

pub(crate) fn add_worktree_to_repo<S>(
    repo: &Repository,
    worktree_name: S,
) -> Result<(String, AddKind)>
where
    S: AsRef<OsStr>,
{
    fetch_all(repo);

    let remote_branch = get_branch(repo, &worktree_name, BranchType::Remote);

    let (worktree, add_kind) = add_worktree(repo, &worktree_name, &remote_branch)?;

    let worktree_path = worktree.path().to_string_lossy().to_string();

    Ok((format!("cd {}", worktree_path), add_kind))
}

pub(crate) fn add_branch_to_repo<S>(
    repo: &Repository,
    branch_name: S,
) -> Result<(String, AddKind), Error>
where
    S: AsRef<OsStr>,
{
    fetch_all(repo);

    let (branch, add_kind) = add_branch(repo, &branch_name)?;

    Ok((format!("git checkout {}", branch.name), add_kind))
}
