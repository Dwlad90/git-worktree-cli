use std::{ffi::OsStr, io::Cursor};

use colored::*;

use git2::{BranchType, Error, Repository};
use skim::{prelude::SkimItemReader, Skim, SkimOptions};

use crate::utils::git::{
    branch::{branch_exists_by_name, get_branch, get_worktree_branches},
    worktree::{get_worktree_by_branch_name, get_worktree_path_by_name, worktree_exists_by_name},
};

use super::git::{
    branch::{add_branch, get_branches, BranchInfo},
    fetch::fetch_all,
    is_branch_clear,
    worktree::add_worktree,
};

pub(crate) fn change_branch_of_bare_or_worktree_repo(
    repo: &Repository,
    options: &SkimOptions,
    item_reader: &SkimItemReader,
    branch_name_arg: &Option<String>,
    worktree_name_arg: &Option<String>,
) -> Result<String, Error> {
    let worktree_name = worktree_name_arg
        .clone()
        .and_then(|worktree_name: String| {
            if worktree_exists_by_name(repo, &worktree_name).unwrap_or(false) {
                Some(worktree_name)
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            let selected_branch_name = branch_name_arg
                .clone()
                .and_then(|branch_name: String| {
                    if branch_exists_by_name(repo, &branch_name, BranchType::Local).unwrap_or(false)
                    {
                        Some(branch_name)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| {
                    let branch_icon = "";

                    let worktree_branch_map =
                        get_worktree_branches(repo).expect("Failed to get worktree branches");

                    let items = item_reader.of_bufread(Cursor::new(
                        worktree_branch_map
                            .into_iter()
                            .map(|(workspace, branch)| {
                                if workspace == branch {
                                    workspace
                                } else {
                                    format!("{} -> {}{}", workspace, branch_icon, branch)
                                }
                            })
                            .collect::<Vec<String>>()
                            .join("\n")
                            .into_bytes(),
                    ));

                    let selected_items = Skim::run_with(options, Some(items))
                        .map(|out| out.selected_items)
                        .unwrap_or_default();

                    let selected_item = selected_items
                        .first()
                        .expect("Workspace not selected")
                        .output();

                    let selected_branch_name = selected_item
                        .split(branch_icon)
                        .last()
                        .expect("Failed to get selected worktree name");

                    selected_branch_name.to_string()
                });

            get_worktree_by_branch_name(repo, &selected_branch_name).unwrap()
        });

    let worktree_path =
        get_worktree_path_by_name(repo, &worktree_name).expect("Failed to get worktree path");

    Ok(format!("cd {}", worktree_path))
}

pub(crate) fn change_branch_of_regular_repo(
    repo: &Repository,
    options: &SkimOptions,
    item_reader: &SkimItemReader,
    branch_name_arg: &Option<String>,
) -> Result<String, Error> {
    if !is_branch_clear(repo) {
        eprintln!("{}", "WARNING: Branch has uncommitted changes".yellow());
        std::process::exit(exitcode::SOFTWARE);
    }

    let selected_branch_name = branch_name_arg
        .clone()
        .and_then(|branch_name: String| {
            if branch_exists_by_name(repo, &branch_name, BranchType::Local).unwrap_or(false) {
                Some(branch_name)
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            let local_branches: Vec<BranchInfo> = get_branches(repo, BranchType::Local);

            let items = item_reader.of_bufread(Cursor::new(
                local_branches
                    .iter()
                    .map(|branch| branch.name.clone())
                    .collect::<Vec<String>>()
                    .join("\n")
                    .into_bytes(),
            ));

            let selected_items = Skim::run_with(options, Some(items))
                .map(|out| out.selected_items)
                .unwrap_or_default();

            let selected_branch = selected_items
                .first()
                .expect("Branch not selected")
                .output();

            selected_branch.into_owned()
        });

    Ok(format!("git checkout {}", selected_branch_name))
}

pub(crate) fn add_worktree_to_repo<S>(repo: &Repository, worktree_name: S) -> Result<String, Error>
where
    S: AsRef<OsStr>,
{
    fetch_all(repo);

    let remote_branch = get_branch(repo, &worktree_name, BranchType::Remote);

    let worktree = add_worktree(repo, &worktree_name, &remote_branch).unwrap();

    let worktree_path = worktree.path().to_string_lossy().to_string();
    Ok::<String, Error>(format!("cd {}", worktree_path))
}

pub(crate) fn add_branch_to_repo<S>(repo: &Repository, branch_name: S) -> Result<String, Error>
where
    S: AsRef<OsStr>,
{
    fetch_all(repo);

    let branch = add_branch(repo, &branch_name)?;

    Ok::<String, Error>(format!("git checkout {}", branch.name))
}
