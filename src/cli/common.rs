use std::path::{Path, PathBuf};

use miette::{Context, IntoDiagnostic};

use crate::codegen;
use crate::github::GitHubClient;
use crate::lockfile::Lockfile;
use crate::runtime::Runtime;
use crate::workflow::Workflow;

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

/// Evaluate all workflow definitions and return the generated workflows.
///
/// This is the shared core of `ghat generate` and `ghat check`.
pub fn eval_workflow_definitions() -> miette::Result<Vec<(String, Workflow)>> {
    let workflows_dir = PathBuf::from(".github/ghat/workflows");
    miette::ensure!(
        std::fs::exists(&workflows_dir).is_ok_and(|success| success),
        "workflows directory not found: {}",
        workflows_dir.display()
    );

    let mut builder = Runtime::builder();

    let mappings_path = base_dir().join("actions/mappings.js");
    match std::fs::read_to_string(&mappings_path) {
        Ok(s) => builder = builder.mappings(&s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            return Err(e)
                .into_diagnostic()
                .wrap_err("failed to read mappings.js")
        }
    };

    let rt = builder.build()?;

    let entries = std::fs::read_dir(&workflows_dir)
        .into_diagnostic()
        .wrap_err("failed to load directory")?;
    for entry in entries {
        let entry = entry
            .into_diagnostic()
            .wrap_err("failed to load workflow file")?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        if file_name.starts_with('_') {
            continue;
        }

        if file_name.ends_with(".ts") || file_name.ends_with(".js") {
            log::info!("evaluating workflow: {file_name}");
            rt.eval_workflow_definition(&entry.path())?;
        }
    }

    Ok(rt.finish())
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
