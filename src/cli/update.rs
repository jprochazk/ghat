use crate::codegen;
use crate::github::{
    GitHubApi, ResolvedAction, parse_version_req, resolve_compatible, resolve_latest,
};
use crate::lockfile::{LockedAction, Lockfile};

#[derive(Debug)]
pub enum UpdateResult {
    Updated {
        name: String,
        old_version: String,
        new: ResolvedAction,
    },
    Unchanged {
        name: String,
        version: String,
    },
}

/// Derive a major-version constraint from a full version string.
///
/// `v2.7.8` → `v2` (parsed as `>=2.0.0, <3.0.0`).
/// Returns `None` if the version doesn't look like semver.
fn major_constraint(version: &str) -> Option<String> {
    let stripped = version.strip_prefix('v').unwrap_or(version);
    let major = stripped.split('.').next()?;
    if major.parse::<u64>().is_ok() {
        Some(format!("v{major}"))
    } else {
        None
    }
}

/// Resolve the latest compatible version for an action already in the lockfile.
///
/// With `breaking = false`: stays within the same major version.
/// With `breaking = true`: resolves to absolute latest release.
fn resolve_update(
    api: &dyn GitHubApi,
    name: &str,
    current: &LockedAction,
    breaking: bool,
) -> miette::Result<ResolvedAction> {
    let (owner, repo) = name
        .split_once('/')
        .ok_or_else(|| miette::miette!("invalid action name in lockfile: {name}"))?;

    if breaking {
        return resolve_latest(api, owner, repo);
    }

    // Try semver-compatible update within the same major version
    if let Some(constraint) = major_constraint(&current.version) {
        let req = parse_version_req(&constraint)?;
        return resolve_compatible(api, owner, repo, &req);
    }

    // Non-semver version — fall back to latest
    resolve_latest(api, owner, repo)
}

/// Core logic: update actions in lockfile.
pub fn update_actions(
    api: &dyn GitHubApi,
    lockfile: &mut Lockfile,
    names: &[String],
    breaking: bool,
) -> miette::Result<Vec<UpdateResult>> {
    // Validate all names exist
    let not_found: Vec<&str> = names
        .iter()
        .filter(|n| !lockfile.actions.contains_key(n.as_str()))
        .map(String::as_str)
        .collect();
    if !not_found.is_empty() {
        return Err(miette::miette!(
            "actions not found in lockfile: {}",
            not_found.join(", ")
        ));
    }

    // Determine which actions to update
    let targets: Vec<String> = if names.is_empty() {
        lockfile.actions.keys().cloned().collect()
    } else {
        names.to_vec()
    };

    let mut results = Vec::new();
    for name in &targets {
        let current = &lockfile.actions[name];
        let resolved = resolve_update(api, name, current, breaking)?;

        if resolved.sha == current.sha {
            results.push(UpdateResult::Unchanged {
                name: name.clone(),
                version: current.version.clone(),
            });
        } else {
            let old_version = current.version.clone();
            lockfile.actions.insert(
                name.clone(),
                LockedAction {
                    version: resolved.version.clone(),
                    sha: resolved.sha.clone(),
                },
            );
            results.push(UpdateResult::Updated {
                name: name.clone(),
                old_version,
                new: resolved,
            });
        }
    }

    Ok(results)
}

/// CLI entry point for `ghat update`.
pub fn run(
    actions: Vec<String>,
    breaking: bool,
    github_token: Option<String>,
) -> miette::Result<()> {
    let (path, mut lockfile) = super::common::load_lockfile()?;

    if lockfile.actions.is_empty() {
        return Err(miette::miette!("lockfile is empty, nothing to update"));
    }

    let client = super::common::github_client(github_token);
    let results = update_actions(&client, &mut lockfile, &actions, breaking)?;
    lockfile.save(&path)?;

    let base = super::common::base_dir();
    let mut cache = codegen::ManifestCache::load(&base.join("actions/cache.json"))?;

    for r in &results {
        match r {
            UpdateResult::Updated {
                name,
                old_version,
                new,
            } => {
                let manifest = codegen::get_or_fetch_manifest(
                    &mut cache,
                    &client,
                    name,
                    &new.sha,
                    &new.version,
                )?;
                codegen::write_action_types(base, name, &manifest)?;
                eprintln!(
                    "updated {name} {old_version} -> {} ({})",
                    new.version,
                    &new.sha[..12]
                );
            }
            UpdateResult::Unchanged { name, version } => {
                eprintln!("{name} {version} is already up to date");
            }
        }
    }

    super::common::finalize_codegen(&lockfile, &mut cache)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::testing::*;

    fn lockfile_with_checkout() -> Lockfile {
        let mut lockfile = Lockfile::new();
        lockfile.actions.insert(
            "actions/checkout".into(),
            LockedAction {
                version: "v4.1.0".into(),
                sha: "8ade135a41bc03ea155e62e844d188df1ea18608".into(),
            },
        );
        lockfile
    }

    fn lockfile_with_rust_cache() -> Lockfile {
        let mut lockfile = Lockfile::new();
        lockfile.actions.insert(
            "Swatinem/rust-cache".into(),
            LockedAction {
                version: "v2.7.0".into(),
                sha: "bd47c6ad4b02e050fd481d890b2ea34778fd09d6".into(),
            },
        );
        lockfile
    }

    #[test]
    fn update_within_major() {
        let mock = mock_checkout();
        let mut lockfile = lockfile_with_checkout();

        let results =
            update_actions(&mock, &mut lockfile, &["actions/checkout".into()], false).unwrap();

        assert_eq!(results.len(), 1);
        assert!(
            matches!(&results[0], UpdateResult::Updated { old_version, new, .. }
                if old_version == "v4.1.0" && new.version == "v4.2.2"
            )
        );
        assert_eq!(lockfile.actions["actions/checkout"].version, "v4.2.2");
    }

    #[test]
    fn update_already_latest() {
        let mock = mock_checkout();
        let mut lockfile = Lockfile::new();
        lockfile.actions.insert(
            "actions/checkout".into(),
            LockedAction {
                version: "v4.2.2".into(),
                sha: "11bd71901bbe5b1630ceea73d27597364c9af683".into(),
            },
        );

        let results =
            update_actions(&mock, &mut lockfile, &["actions/checkout".into()], false).unwrap();

        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], UpdateResult::Unchanged { .. }));
    }

    #[test]
    fn update_all() {
        let mock = mock_multi();
        let mut lockfile = lockfile_with_checkout();
        lockfile.actions.insert(
            "Swatinem/rust-cache".into(),
            LockedAction {
                version: "v2.7.0".into(),
                sha: "bd47c6ad4b02e050fd481d890b2ea34778fd09d6".into(),
            },
        );

        let results = update_actions(&mock, &mut lockfile, &[], false).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(lockfile.actions["Swatinem/rust-cache"].version, "v2.7.8");
        assert_eq!(lockfile.actions["actions/checkout"].version, "v4.2.2");
    }

    #[test]
    fn update_respects_major_boundary() {
        let mock = mock_rust_cache();
        let mut lockfile = Lockfile::new();
        lockfile.actions.insert(
            "Swatinem/rust-cache".into(),
            LockedAction {
                version: "v1.4.0".into(),
                sha: "cd47c6ad4b02e050fd481d890b2ea34778fd09d6".into(),
            },
        );

        // Without --breaking, stays within v1.x
        let results =
            update_actions(&mock, &mut lockfile, &["Swatinem/rust-cache".into()], false).unwrap();

        assert!(matches!(&results[0], UpdateResult::Unchanged { .. }));
        assert_eq!(lockfile.actions["Swatinem/rust-cache"].version, "v1.4.0");
    }

    #[test]
    fn update_breaking_crosses_major() {
        let mock = mock_rust_cache();
        let mut lockfile = Lockfile::new();
        lockfile.actions.insert(
            "Swatinem/rust-cache".into(),
            LockedAction {
                version: "v1.4.0".into(),
                sha: "cd47c6ad4b02e050fd481d890b2ea34778fd09d6".into(),
            },
        );

        // With --breaking, jumps to latest (v2.7.8)
        let results =
            update_actions(&mock, &mut lockfile, &["Swatinem/rust-cache".into()], true).unwrap();

        assert!(
            matches!(&results[0], UpdateResult::Updated { new, .. } if new.version == "v2.7.8")
        );
        assert_eq!(lockfile.actions["Swatinem/rust-cache"].version, "v2.7.8");
    }

    #[test]
    fn update_not_found() {
        let mock = mock_checkout();
        let mut lockfile = lockfile_with_checkout();
        let result = update_actions(&mock, &mut lockfile, &["nonexistent/action".into()], false);
        assert!(result.is_err());
    }
}
