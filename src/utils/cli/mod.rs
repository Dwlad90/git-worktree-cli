use std::{error::Error, io::Cursor, path::Path};

use colored::*;

use skim::{prelude::SkimItemReader, Skim, SkimOptions};

use crate::utils::git::{
    branch::{branch_exists_by_name, get_local_branches, get_worktree_branches},
    worktree::{get_worktree_by_branch_name, get_worktree_path_by_name, worktree_exists_by_name},
};

use super::git::is_branch_clear;

pub(crate) fn change_branch_of_bare_or_worktree_repo<P>(
    repo_path: &P,
    options: &SkimOptions,
    item_reader: &SkimItemReader,
    branch_name_arg: &Option<String>,
    worktree_name_arg: &Option<String>,
) -> Result<String, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let worktree_name = worktree_name_arg
        .clone()
        .and_then(|worktree_name: String| {
            if worktree_exists_by_name(repo_path, &worktree_name).unwrap_or(false) {
                Some(worktree_name)
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            let selected_branch_name = branch_name_arg
                .clone()
                .and_then(|branch_name: String| {
                    if branch_exists_by_name(repo_path, &branch_name).unwrap_or(false) {
                        Some(branch_name)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| {
                    let branch_icon = "";

                    let worktree_branch_map =
                        get_worktree_branches(repo_path).expect("Failed to get worktree branches");

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

            get_worktree_by_branch_name(repo_path, &selected_branch_name).unwrap()
        });

    let worktree_path =
        get_worktree_path_by_name(repo_path, &worktree_name).expect("Failed to get worktree path");

    Ok(format!("cd {}", worktree_path))
}

pub(crate) fn change_branch_of_regular_repo<P>(
    repo_path: &P,
    options: &SkimOptions,
    item_reader: &SkimItemReader,
    branch_name_arg: &Option<String>,
) -> Result<String, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    if !is_branch_clear(repo_path) {
        eprintln!("{}", "WARNING: Branch has uncommitted changes".yellow());
        std::process::exit(exitcode::SOFTWARE);
    }

    let selected_branch_name = branch_name_arg
        .clone()
        .and_then(|branch_name: String| {
            if branch_exists_by_name(repo_path, &branch_name).unwrap_or(false) {
                Some(branch_name)
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            let local_branches: Vec<String> = get_local_branches(repo_path);

            let items = item_reader.of_bufread(Cursor::new(local_branches.join("\n").into_bytes()));

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
