use std::path::{Path, PathBuf};

use crate::codegen;
use crate::github::GitHubClient;
use crate::lockfile::Lockfile;

pub const BASE_DIR: &str = ".github/ghat";
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

pub fn base_dir() -> &'static Path {
    Path::new(BASE_DIR)
}

/// Evict stale cache entries, save the cache, and regenerate `mappings.js`.
pub fn finalize_codegen(
    lockfile: &Lockfile,
    cache: &mut codegen::ManifestCache,
) -> miette::Result<()> {
    let base = base_dir();
    let cache_path = base.join("actions/cache.json");
    cache.evict_stale(lockfile);
    cache.save(&cache_path)?;
    codegen::write_mappings(base, lockfile, cache)?;
    Ok(())
}
