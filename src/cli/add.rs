use crate::github::{parse_action_ref, resolve_action, ActionRef, GitHubApi, ResolvedAction};
use crate::lockfile::{LockedAction, Lockfile};

pub enum AddResult {
    Added {
        name: String,
        resolved: ResolvedAction,
    },
    Skipped {
        name: String,
    },
}

/// Core logic: resolve actions and insert into lockfile.
///
/// Actions already present in the lockfile are skipped (returned as `Skipped`).
pub fn add_actions(
    api: &dyn GitHubApi,
    lockfile: &mut Lockfile,
    refs: &[ActionRef],
) -> miette::Result<Vec<AddResult>> {
    let mut results = Vec::new();
    for r in refs {
        let name = r.name();
        if lockfile.actions.contains_key(&name) {
            results.push(AddResult::Skipped { name });
            continue;
        }

        let resolved = resolve_action(api, r)?;
        let name = resolved.name();
        lockfile.actions.insert(
            name.clone(),
            LockedAction {
                version: resolved.version.clone(),
                sha: resolved.sha.clone(),
            },
        );
        results.push(AddResult::Added { name, resolved });
    }

    Ok(results)
}

impl ResolvedAction {
    fn name(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

/// CLI entry point for `ghat add`.
pub fn run(actions: Vec<String>, _auto: bool, github_token: Option<String>) -> miette::Result<()> {
    let (refs, errors): (Vec<_>, Vec<_>) = actions
        .iter()
        .map(|s| parse_action_ref(s))
        .partition(Result::is_ok);
    let refs: Vec<ActionRef> = refs.into_iter().map(Result::unwrap).collect();

    if !errors.is_empty() {
        let msgs: Vec<String> = errors.into_iter().map(|e| e.unwrap_err().to_string()).collect();
        return Err(miette::miette!("{}", msgs.join("\n")));
    }

    if refs.is_empty() {
        return Err(miette::miette!("no actions specified"));
    }

    let (path, mut lockfile) = super::common::load_lockfile()?;
    let client = super::common::github_client(github_token);

    let results = add_actions(&client, &mut lockfile, &refs)?;
    lockfile.save(&path)?;

    for r in &results {
        match r {
            AddResult::Added { name, resolved } => {
                eprintln!("added {} {} ({})", name, resolved.version, &resolved.sha[..12]);
            }
            AddResult::Skipped { name } => {
                eprintln!("skipped {name} (already in lockfile)");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::testing::{mock_checkout, mock_multi, mock_rust_cache};

    #[test]
    fn add_single_action_latest() {
        let mock = mock_rust_cache();
        let mut lockfile = Lockfile::new();
        let refs = vec![parse_action_ref("Swatinem/rust-cache").unwrap()];

        let results = add_actions(&mock, &mut lockfile, &refs).unwrap();

        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], AddResult::Added { name, resolved }
            if name == "Swatinem/rust-cache"
            && resolved.version == "v2.7.8"
            && resolved.sha == "9d47c6ad4b02e050fd481d890b2ea34778fd09d6"
        ));
        assert!(lockfile.actions.contains_key("Swatinem/rust-cache"));
    }

    #[test]
    fn add_single_action_with_tag() {
        let mock = mock_checkout();
        let mut lockfile = Lockfile::new();
        let refs = vec![parse_action_ref("actions/checkout@v4.2.2").unwrap()];

        let results = add_actions(&mock, &mut lockfile, &refs).unwrap();

        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], AddResult::Added { resolved, .. }
            if resolved.version == "v4.2.2"
            && resolved.sha == "11bd71901bbe5b1630ceea73d27597364c9af683"
        ));
    }

    #[test]
    fn add_multiple_actions() {
        let mock = mock_multi();
        let mut lockfile = Lockfile::new();
        let refs = vec![
            parse_action_ref("actions/checkout").unwrap(),
            parse_action_ref("Swatinem/rust-cache").unwrap(),
        ];

        let results = add_actions(&mock, &mut lockfile, &refs).unwrap();

        assert_eq!(results.len(), 2);
        assert!(matches!(&results[0], AddResult::Added { .. }));
        assert!(matches!(&results[1], AddResult::Added { .. }));
        assert_eq!(lockfile.actions.len(), 2);
    }

    #[test]
    fn duplicate_action_is_skipped() {
        let mock = mock_rust_cache();
        let mut lockfile = Lockfile::new();
        lockfile.actions.insert(
            "Swatinem/rust-cache".into(),
            LockedAction {
                version: "v2.7.7".into(),
                sha: "oldsha".into(),
            },
        );

        let refs = vec![parse_action_ref("Swatinem/rust-cache").unwrap()];
        let results = add_actions(&mock, &mut lockfile, &refs).unwrap();

        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], AddResult::Skipped { name } if name == "Swatinem/rust-cache"));
        // Lockfile unchanged
        assert_eq!(lockfile.actions["Swatinem/rust-cache"].version, "v2.7.7");
    }

    #[test]
    fn duplicate_in_batch_skipped_new_ones_added() {
        let mock = mock_multi();
        let mut lockfile = Lockfile::new();
        lockfile.actions.insert(
            "actions/checkout".into(),
            LockedAction {
                version: "v4.2.1".into(),
                sha: "oldsha".into(),
            },
        );

        let refs = vec![
            parse_action_ref("actions/checkout").unwrap(),
            parse_action_ref("Swatinem/rust-cache").unwrap(),
        ];

        let results = add_actions(&mock, &mut lockfile, &refs).unwrap();
        assert_eq!(results.len(), 2);
        assert!(matches!(&results[0], AddResult::Skipped { .. }));
        assert!(matches!(&results[1], AddResult::Added { .. }));
        // checkout unchanged, rust-cache added
        assert_eq!(lockfile.actions["actions/checkout"].version, "v4.2.1");
        assert!(lockfile.actions.contains_key("Swatinem/rust-cache"));
    }
}
