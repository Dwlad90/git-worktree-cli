use git2::{FetchOptions, Repository};

use super::common::get_credentials_callback;

fn get_fetch_options<'a>() -> FetchOptions<'a> {
    let callbacks = get_credentials_callback();

    let mut fetch_options = FetchOptions::new();

    fetch_options.remote_callbacks(callbacks);

    fetch_options
}

pub(crate) fn fetch_all(repo: &Repository) {
    let mut fetch_options = get_fetch_options();

    repo.find_remote("origin")
        .expect("Failed to find remote")
        .fetch(
            &["+refs/heads/*:refs/remotes/origin/*"],
            Some(&mut fetch_options),
            None,
        )
        .expect("Failed to fetch");
}
