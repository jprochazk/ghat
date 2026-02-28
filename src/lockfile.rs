use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedAction {
    pub version: String,
    pub sha: String,
}

/// Line-based lockfile: `owner/repo version sha`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lockfile {
    pub actions: BTreeMap<String, LockedAction>,
}

impl Lockfile {
    pub fn new() -> Self {
        Self {
            actions: BTreeMap::new(),
        }
    }

    pub fn load(path: &Path) -> miette::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| miette::miette!("failed to read lockfile {}: {e}", path.display()))?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> miette::Result<Self> {
        let mut actions = BTreeMap::new();

        for (i, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 3 {
                return Err(miette::miette!(
                    "invalid lockfile entry on line {}: expected `owner/repo version sha`, got: {line}",
                    i + 1,
                ));
            }

            actions.insert(
                parts[0].to_string(),
                LockedAction {
                    version: parts[1].to_string(),
                    sha: parts[2].to_string(),
                },
            );
        }

        Ok(Self { actions })
    }

    pub fn save(&self, path: &Path) -> miette::Result<()> {
        let content = self.to_string();
        std::fs::write(path, &content)
            .map_err(|e| miette::miette!("failed to write lockfile {}: {e}", path.display()))?;
        Ok(())
    }
}

impl std::fmt::Display for Lockfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, action) in &self.actions {
            writeln!(f, "{name} {ver} {sha}", ver = action.version, sha = action.sha)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_lockfile() {
        let lockfile = Lockfile::parse("").unwrap();
        assert!(lockfile.actions.is_empty());
    }

    #[test]
    fn comments_and_blank_lines() {
        let content = "# this is a comment\n\n# another comment\n";
        let lockfile = Lockfile::parse(content).unwrap();
        assert!(lockfile.actions.is_empty());
    }

    #[test]
    fn parse_and_serialize() {
        let content = "\
Swatinem/rust-cache v2.7.8 779680da715d629ac1d338a641029a2f4372abb5
taiki-e/install-action v2.44.3 288875dd3d64326724fa6d9593062d9f8ba0b131
";
        let lockfile = Lockfile::parse(content).unwrap();
        insta::assert_debug_snapshot!(lockfile);
    }

    #[test]
    fn roundtrip() {
        let mut lockfile = Lockfile::new();
        lockfile.actions.insert(
            "taiki-e/install-action".into(),
            LockedAction {
                version: "v2.44.3".into(),
                sha: "288875dd3d64326724fa6d9593062d9f8ba0b131".into(),
            },
        );
        lockfile.actions.insert(
            "Swatinem/rust-cache".into(),
            LockedAction {
                version: "v2.7.8".into(),
                sha: "779680da715d629ac1d338a641029a2f4372abb5".into(),
            },
        );

        let serialized = lockfile.to_string();
        let parsed = Lockfile::parse(&serialized).unwrap();
        assert_eq!(lockfile, parsed);

        insta::assert_snapshot!(serialized);
    }

    #[test]
    fn invalid_line() {
        let content = "bad-line\n";
        let result = Lockfile::parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ghat.lock");

        let mut lockfile = Lockfile::new();
        lockfile.actions.insert(
            "actions/checkout".into(),
            LockedAction {
                version: "v4".into(),
                sha: "abc123".into(),
            },
        );

        lockfile.save(&path).unwrap();
        let loaded = Lockfile::load(&path).unwrap();
        assert_eq!(lockfile, loaded);
    }
}
