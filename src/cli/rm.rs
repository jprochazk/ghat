use crate::codegen;
use crate::lockfile::Lockfile;

#[derive(Debug)]
pub enum RmResult {
    Removed { name: String },
}

/// Single-row Levenshtein distance, reusing a caller-provided buffer.
fn edit_distance(a: &str, b: &str, buf: &mut Vec<usize>) -> usize {
    let b_len = b.len();
    buf.clear();
    buf.extend(0..=b_len);

    for (i, ca) in a.bytes().enumerate() {
        let mut prev = i;
        buf[0] = i + 1;
        for (j, cb) in b.bytes().enumerate() {
            let cost = if ca == cb { prev } else { prev + 1 };
            prev = buf[j + 1];
            buf[j + 1] = cost.min(buf[j] + 1).min(prev + 1);
        }
    }
    buf[b_len]
}

/// Find the closest match in the lockfile for a given name.
fn suggest(name: &str, candidates: &[&str]) -> Option<String> {
    let threshold = (name.len() / 3).max(2);
    let mut buf = Vec::new();
    candidates
        .iter()
        .map(|c| (c, edit_distance(name, c, &mut buf)))
        .filter(|(_, d)| *d <= threshold)
        .min_by_key(|(_, d)| *d)
        .map(|(c, _)| c.to_string())
}

/// Core logic: remove actions from lockfile.
///
/// Returns an error if any action is not found in the lockfile.
pub fn rm_actions(lockfile: &mut Lockfile, names: &[String]) -> miette::Result<Vec<RmResult>> {
    // Check all names exist before removing any
    let mut errors = Vec::new();
    let candidates: Vec<&str> = lockfile.actions.keys().map(String::as_str).collect();
    for name in names {
        if !lockfile.actions.contains_key(name) {
            let mut msg = format!("{name} not found in lockfile");
            if let Some(suggestion) = suggest(name, &candidates) {
                msg.push_str(&format!(" (did you mean `{suggestion}`?)"));
            }
            errors.push(msg);
        }
    }
    if !errors.is_empty() {
        return Err(miette::miette!("{}", errors.join("\n")));
    }

    let mut results = Vec::new();
    for name in names {
        lockfile.actions.remove(name);
        results.push(RmResult::Removed { name: name.clone() });
    }
    Ok(results)
}

/// CLI entry point for `ghat rm`.
pub fn run(actions: Vec<String>) -> miette::Result<()> {
    if actions.is_empty() {
        return Err(miette::miette!("no actions specified"));
    }

    let (path, mut lockfile) = super::common::load_lockfile()?;
    let results = rm_actions(&mut lockfile, &actions)?;
    lockfile.save(&path)?;

    let base = super::common::base_dir();
    let mut cache = codegen::ManifestCache::load(&base.join("actions/cache.json"))?;

    for r in &results {
        match r {
            RmResult::Removed { name } => {
                codegen::remove_action_types(base, name)?;
                eprintln!("removed {name}");
            }
        }
    }

    super::common::finalize_codegen(&lockfile, &mut cache)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lockfile::LockedAction;

    fn test_lockfile() -> Lockfile {
        let mut lockfile = Lockfile::new();
        lockfile.actions.insert(
            "actions/checkout".into(),
            LockedAction {
                version: "v4.2.2".into(),
                sha: "11bd71901bbe5b1630ceea73d27597364c9af683".into(),
            },
        );
        lockfile.actions.insert(
            "Swatinem/rust-cache".into(),
            LockedAction {
                version: "v2.7.8".into(),
                sha: "9d47c6ad4b02e050fd481d890b2ea34778fd09d6".into(),
            },
        );
        lockfile
    }

    #[test]
    fn rm_single() {
        let mut lockfile = test_lockfile();
        let results = rm_actions(&mut lockfile, &["actions/checkout".into()]).unwrap();

        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], RmResult::Removed { name } if name == "actions/checkout"));
        assert!(!lockfile.actions.contains_key("actions/checkout"));
        assert!(lockfile.actions.contains_key("Swatinem/rust-cache"));
    }

    #[test]
    fn rm_multiple() {
        let mut lockfile = test_lockfile();
        let results = rm_actions(
            &mut lockfile,
            &["actions/checkout".into(), "Swatinem/rust-cache".into()],
        )
        .unwrap();

        assert_eq!(results.len(), 2);
        assert!(lockfile.actions.is_empty());
    }

    #[test]
    fn rm_not_found() {
        let mut lockfile = test_lockfile();
        let err = rm_actions(&mut lockfile, &["nonexistent/action".into()]).unwrap_err();
        insta::assert_snapshot!(err);
        assert_eq!(lockfile.actions.len(), 2);
    }

    #[test]
    fn rm_not_found_with_suggestion() {
        let mut lockfile = test_lockfile();
        let err = rm_actions(&mut lockfile, &["actions/checkotu".into()]).unwrap_err();
        insta::assert_snapshot!(err);
        assert_eq!(lockfile.actions.len(), 2);
    }

    #[test]
    fn rm_partial_not_found() {
        let mut lockfile = test_lockfile();
        let err = rm_actions(
            &mut lockfile,
            &["actions/checkout".into(), "nonexistent/action".into()],
        )
        .unwrap_err();
        insta::assert_snapshot!(err);
        assert_eq!(lockfile.actions.len(), 2);
    }
}
