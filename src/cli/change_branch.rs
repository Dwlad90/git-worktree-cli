use std::ffi::OsString;

use git2::Repository;
use skim::{
    prelude::{SkimItemReader, SkimOptionsBuilder},
    FuzzyAlgorithm,
};

use crate::utils::cli::{change_branch_of_bare_or_worktree_repo, change_branch_of_regular_repo};

pub(crate) fn change_branch_sub_command(
    repo: Repository,
    branch: Option<OsString>,
    worktree: Option<OsString>,
    query: Option<OsString>,
) {
    let query_string = query.map(|os_str| os_str.to_string_lossy().to_string());

    let options = SkimOptionsBuilder::default()
        .query(query_string.as_deref())
        .multi(false)
        .algorithm(FuzzyAlgorithm::SkimV2)
        .build()
        .unwrap();

    let item_reader = SkimItemReader::default();

    let branch = branch.map(|os_str| os_str.to_string_lossy().into_owned());
    let worktree = worktree.map(|os_str| os_str.to_string_lossy().into_owned());

    let command = if repo.is_bare() || repo.is_worktree() {
        change_branch_of_bare_or_worktree_repo(&repo, &options, &item_reader, &branch, &worktree)
    } else {
        change_branch_of_regular_repo(&repo, &options, &item_reader, &branch)
    };

    println!("{}", command.expect("Failed to change branch"));
}
