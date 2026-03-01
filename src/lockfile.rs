use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    Tag,
    Branch,
}

impl fmt::Display for RefKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefKind::Tag => write!(f, "tag"),
            RefKind::Branch => write!(f, "branch"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedAction {
    pub ref_kind: RefKind,
    pub version: String,
    pub sha: String,
}

/// Line-based lockfile: `owner/repo tag:version sha` or `owner/repo branch:name sha`
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
                    "invalid lockfile entry on line {}: expected `owner/repo tag:version sha`, got: {line}",
                    i + 1,
                ));
            }

            let (ref_kind, version) = parse_ref_field(parts[1]).ok_or_else(|| {
                miette::miette!(
                    "invalid version on line {}: expected `tag:VERSION` or `branch:NAME`, got: {}",
                    i + 1,
                    parts[1],
                )
            })?;

            actions.insert(
                parts[0].to_string(),
                LockedAction {
                    ref_kind,
                    version: version.to_string(),
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

fn parse_ref_field(field: &str) -> Option<(RefKind, &str)> {
    let (prefix, value) = field.split_once(':')?;
    let kind = match prefix {
        "tag" => RefKind::Tag,
        "branch" => RefKind::Branch,
        _ => return None,
    };
    Some((kind, value))
}

impl fmt::Display for Lockfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, action) in &self.actions {
            writeln!(
                f,
                "{name} {kind}:{ver} {sha}",
                kind = action.ref_kind,
                ver = action.version,
                sha = action.sha
            )?;
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
Swatinem/rust-cache tag:v2.7.8 779680da715d629ac1d338a641029a2f4372abb5
taiki-e/install-action tag:v2.44.3 288875dd3d64326724fa6d9593062d9f8ba0b131
";
        let lockfile = Lockfile::parse(content).unwrap();
        insta::assert_debug_snapshot!(lockfile);
    }

    #[test]
    fn parse_branch_ref() {
        let content = "dtolnay/rust-toolchain branch:stable abc123def456\n";
        let lockfile = Lockfile::parse(content).unwrap();
        let action = &lockfile.actions["dtolnay/rust-toolchain"];
        assert_eq!(action.ref_kind, RefKind::Branch);
        assert_eq!(action.version, "stable");
        assert_eq!(action.sha, "abc123def456");
    }

    #[test]
    fn parse_mixed_refs() {
        let content = "\
actions/checkout tag:v4.2.2 11bd71901bbe5b1630ceea73d27597364c9af683
dtolnay/rust-toolchain branch:stable e97e2d8cc328f1b50210efc529dca0028893a2d9
";
        let lockfile = Lockfile::parse(content).unwrap();
        assert_eq!(lockfile.actions["actions/checkout"].ref_kind, RefKind::Tag);
        assert_eq!(
            lockfile.actions["dtolnay/rust-toolchain"].ref_kind,
            RefKind::Branch
        );
    }

    #[test]
    fn roundtrip() {
        let mut lockfile = Lockfile::new();
        lockfile.actions.insert(
            "taiki-e/install-action".into(),
            LockedAction {
                ref_kind: RefKind::Tag,
                version: "v2.44.3".into(),
                sha: "288875dd3d64326724fa6d9593062d9f8ba0b131".into(),
            },
        );
        lockfile.actions.insert(
            "Swatinem/rust-cache".into(),
            LockedAction {
                ref_kind: RefKind::Tag,
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
    fn invalid_version_no_prefix() {
        let content = "actions/checkout v4.2.2 abc123\n";
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
                ref_kind: RefKind::Tag,
                version: "v4".into(),
                sha: "abc123".into(),
            },
        );

        lockfile.save(&path).unwrap();
        let loaded = Lockfile::load(&path).unwrap();
        assert_eq!(lockfile, loaded);
    }
}
