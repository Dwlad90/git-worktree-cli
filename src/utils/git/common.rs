use git2::RemoteCallbacks;



pub(crate)  fn get_credentials_callback<'a>() -> RemoteCallbacks<'a> {
  let mut callbacks = RemoteCallbacks::new();

  callbacks.credentials(|_url, username_from_url, _allowed_types| {
      git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
  });

  callbacks
}