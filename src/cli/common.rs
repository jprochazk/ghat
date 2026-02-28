use std::path::PathBuf;

use crate::github::GitHubClient;
use crate::lockfile::Lockfile;

const LOCKFILE_PATH: &str = ".github/ghat/ghat.lock";

pub fn lockfile_path() -> miette::Result<PathBuf> {
    let path = PathBuf::from(LOCKFILE_PATH);
    if !path.exists() {
        return Err(miette::miette!(
            "lockfile not found: {LOCKFILE_PATH}\n\nRun `ghat init` first to initialize the project."
        ));
    }
    Ok(path)
}

pub fn load_lockfile() -> miette::Result<(PathBuf, Lockfile)> {
    let path = lockfile_path()?;
    let lockfile = Lockfile::load(&path)?;
    Ok((path, lockfile))
}

pub fn github_client(token: Option<String>) -> GitHubClient {
    GitHubClient::new(token)
}
