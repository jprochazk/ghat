use std::collections::BTreeMap;
use std::path::Path;

use miette::{Context, IntoDiagnostic};

use crate::github::{ActionManifest, GitHubApi};
use crate::lockfile::Lockfile;

/// Cache of fetched action.yml manifests, keyed by `"owner/repo@sha"`.
pub struct ManifestCache {
    entries: BTreeMap<String, ActionManifest>,
}

impl ManifestCache {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    fn cache_key(name: &str, sha: &str) -> String {
        format!("{name}@{sha}")
    }

    pub fn get(&self, name: &str, sha: &str) -> Option<&ActionManifest> {
        self.entries.get(&Self::cache_key(name, sha))
    }

    pub fn insert(&mut self, name: &str, sha: &str, manifest: ActionManifest) {
        self.entries.insert(Self::cache_key(name, sha), manifest);
    }

    pub fn load(path: &Path) -> miette::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let entries: BTreeMap<String, ActionManifest> = serde_json::from_str(&content)
                    .into_diagnostic()
                    .wrap_err("failed to parse manifest cache")?;
                Ok(Self { entries })
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::new()),
            Err(e) => Err(e)
                .into_diagnostic()
                .wrap_err("failed to read manifest cache"),
        }
    }

    pub fn save(&self, path: &Path) -> miette::Result<()> {
        let content = serde_json::to_string_pretty(&self.entries)
            .into_diagnostic()
            .wrap_err("failed to serialize manifest cache")?;
        std::fs::write(path, content)
            .into_diagnostic()
            .wrap_err("failed to write manifest cache")?;
        Ok(())
    }

    /// Remove cache entries whose key doesn't correspond to a current lockfile entry.
    pub fn evict_stale(&mut self, lockfile: &Lockfile) {
        let valid_keys: std::collections::BTreeSet<String> = lockfile
            .actions
            .iter()
            .map(|(name, locked)| Self::cache_key(name, &locked.sha))
            .collect();
        self.entries.retain(|key, _| valid_keys.contains(key));
    }
}

/// Fetch a manifest, storing the result in the cache. Returns a reference to the cached entry.
pub fn get_or_fetch_manifest<'c>(
    cache: &'c mut ManifestCache,
    api: &dyn GitHubApi,
    name: &str,
    sha: &str,
    version: &str,
) -> miette::Result<&'c ActionManifest> {
    if cache.get(name, sha).is_some() {
        return Ok(cache.get(name, sha).unwrap());
    }

    let (owner, repo) = name
        .split_once('/')
        .ok_or_else(|| miette::miette!("invalid action name: {name}"))?;

    let manifest = api.get_action_manifest(owner, repo, version)?;
    cache.insert(name, sha, manifest);
    Ok(cache.get(name, sha).unwrap())
}

/// Convert a kebab-case name to snake_case (replace `-` with `_`).
pub fn to_snake_case(name: &str) -> String {
    name.replace('-', "_")
}

/// Generate the `.d.ts` filename for an action: `"owner__repo.d.ts"`.
pub fn action_dts_filename(action_name: &str) -> String {
    format!("{}.d.ts", action_name.replace('/', "__"))
}

/// Generate the `.d.ts` content for a single action.
pub fn generate_action_dts(action_name: &str, manifest: &ActionManifest) -> String {
    let mut out = String::new();

    // Build input type
    let mut action_has_required_inputs = false;
    let mut input_fields = Vec::new();
    for (name, input) in &manifest.inputs {
        let snake = to_snake_case(name);
        let is_required = input.required == Some(true) && input.default.is_none();
        if is_required {
            action_has_required_inputs = true;
        }

        let mut doc_lines = Vec::new();
        if let Some(desc) = trim_description(input.description.as_deref()) {
            doc_lines.push(escape_jsdoc(desc));
        }
        if let Some(msg) = &input.deprecation_message {
            let msg = msg.trim();
            if msg.is_empty() {
                doc_lines.push("@deprecated".into());
            } else {
                doc_lines.push(format!("@deprecated {}", escape_jsdoc(msg)));
            }
        }

        let mut field = String::new();
        if let Some(comment) = jsdoc("    ", &doc_lines) {
            field.push_str(&comment);
            field.push('\n');
        }
        field.push_str(&format!(
            "    {}{}: string;",
            snake,
            if is_required { "" } else { "?" }
        ));
        input_fields.push(field);
    }

    // Build output type
    let mut output_fields = Vec::new();
    for (name, output) in &manifest.outputs {
        let snake = to_snake_case(name);
        let mut doc_lines = Vec::new();
        if let Some(desc) = trim_description(output.description.as_deref()) {
            doc_lines.push(escape_jsdoc(desc));
        }

        let mut field = String::new();
        if let Some(comment) = jsdoc("    ", &doc_lines) {
            field.push_str(&comment);
            field.push('\n');
        }
        field.push_str(&format!("    {snake}: string;"));
        output_fields.push(field);
    }

    // JSDoc comment
    let mut doc_lines = Vec::new();
    if let Some(desc) = trim_description(manifest.description.as_deref()) {
        doc_lines.push(escape_jsdoc(desc));
    }
    doc_lines.push(format!("@see https://github.com/{action_name}"));
    if let Some(comment) = jsdoc("", &doc_lines) {
        out.push_str(&comment);
        out.push('\n');
    }

    // Options type name
    let options_type = if action_has_required_inputs {
        "UsesOptionsRequired"
    } else {
        "UsesOptions"
    };

    // Function overload
    out.push_str("declare global {\n");
    out.push_str(&format!(
        "  function uses(\n    action: \"{action_name}\",\n"
    ));

    // Options parameter
    if input_fields.is_empty() {
        out.push_str(&format!("    options?: UsesOptions<{{}}>,\n"));
    } else {
        let required_marker = if action_has_required_inputs { "" } else { "?" };
        out.push_str(&format!(
            "    options{required_marker}: {options_type}<{{\n"
        ));
        for field in &input_fields {
            out.push_str(field);
            out.push('\n');
        }
        out.push_str("    }>,\n");
    }

    // Return type
    if output_fields.is_empty() {
        out.push_str("  ): StepRef;\n");
    } else {
        out.push_str("  ): StepRef<{\n");
        for field in &output_fields {
            out.push_str(field);
            out.push('\n');
        }
        out.push_str("  }>;\n");
    }

    out.push_str("}\n");
    out.push_str("export {};\n");

    out
}

/// Generate the `mappings.js` content from the lockfile and cache.
pub fn generate_mappings_js(lockfile: &Lockfile, cache: &ManifestCache) -> String {
    let mut out = String::from("globalThis.__GHAT_ACTION_MAPPINGS = {\n");

    for (name, locked) in &lockfile.actions {
        let Some(manifest) = cache.get(name, &locked.sha) else {
            continue;
        };

        let has_input_renames = manifest.inputs.keys().any(|k| to_snake_case(k) != *k);
        let has_output_renames = manifest.outputs.keys().any(|k| to_snake_case(k) != *k);
        if !has_input_renames && !has_output_renames {
            continue;
        }

        out.push_str(&format!("  \"{name}\": {{\n"));

        if has_input_renames {
            out.push_str("    inputs: {\n");
            write_rename_entries(&mut out, manifest.inputs.keys());
            out.push_str("    },\n");
        }
        if has_output_renames {
            out.push_str("    outputs: {\n");
            write_rename_entries(&mut out, manifest.outputs.keys());
            out.push_str("    },\n");
        }

        out.push_str("  },\n");
    }

    out.push_str("};\n");
    out
}

/// Write `"snake_name": "original-name"` entries for keys that differ when snake_cased.
fn write_rename_entries<'a>(out: &mut String, keys: impl Iterator<Item = &'a String>) {
    let mut first = true;
    for name in keys {
        let snake = to_snake_case(name);
        if snake != *name {
            if !first {
                out.push_str(",\n");
            }
            out.push_str(&format!("      \"{snake}\": \"{name}\""));
            first = false;
        }
    }
    out.push('\n');
}

/// Write the `.d.ts` file for a single action.
pub fn write_action_types(
    base: &Path,
    action_name: &str,
    manifest: &ActionManifest,
) -> miette::Result<()> {
    let dir = base.join("actions");
    std::fs::create_dir_all(&dir)
        .into_diagnostic()
        .wrap_err("failed to create actions directory")?;

    let content = generate_action_dts(action_name, manifest);
    let path = dir.join(action_dts_filename(action_name));
    std::fs::write(&path, content)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

/// Delete the `.d.ts` file for a single action.
pub fn remove_action_types(base: &Path, action_name: &str) -> miette::Result<()> {
    let path = base.join("actions").join(action_dts_filename(action_name));
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e)
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to remove {}", path.display())),
    }
}

/// Write the `mappings.js` file.
pub fn write_mappings(
    base: &Path,
    lockfile: &Lockfile,
    cache: &ManifestCache,
) -> miette::Result<()> {
    let dir = base.join("actions");
    std::fs::create_dir_all(&dir)
        .into_diagnostic()
        .wrap_err("failed to create actions directory")?;

    let content = generate_mappings_js(lockfile, cache);
    let path = dir.join("mappings.js");
    std::fs::write(&path, content)
        .into_diagnostic()
        .wrap_err("failed to write mappings.js")?;
    Ok(())
}

/// Extract a non-empty trimmed description, or `None`.
fn trim_description(s: Option<&str>) -> Option<&str> {
    s.map(str::trim).filter(|d| !d.is_empty())
}

/// Escape text for use inside a JSDoc comment (prevent `*/` from closing it).
fn escape_jsdoc(s: &str) -> String {
    s.replace("*/", "*\\/")
}

/// Format a JSDoc comment at the given indentation level.
///
/// - One line  → `/** line */`
/// - Multiple  → `/**\n * line1\n * line2\n */`
/// - Empty     → returns `None`
fn jsdoc(indent: &str, lines: &[String]) -> Option<String> {
    match lines.len() {
        0 => None,
        1 => Some(format!("{indent}/** {} */", lines[0])),
        _ => {
            let mut out = format!("{indent}/**\n");
            for line in lines {
                out.push_str(&format!("{indent} * {line}\n"));
            }
            out.push_str(&format!("{indent} */"));
            Some(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::{ActionInput, ActionOutput};
    use indexmap::IndexMap;

    fn sample_manifest() -> ActionManifest {
        let mut inputs = IndexMap::new();
        inputs.insert(
            "fetch-depth".into(),
            ActionInput {
                description: Some("Number of commits to fetch.".into()),
                required: Some(false),
                default: Some("1".into()),
                deprecation_message: None,
            },
        );
        inputs.insert(
            "repo-token".into(),
            ActionInput {
                description: Some("The token to use.".into()),
                required: Some(true),
                default: None,
                deprecation_message: None,
            },
        );
        inputs.insert(
            "old-input".into(),
            ActionInput {
                description: Some("Deprecated.".into()),
                required: None,
                default: None,
                deprecation_message: Some("Use something else.".into()),
            },
        );

        let mut outputs = IndexMap::new();
        outputs.insert(
            "my-value".into(),
            ActionOutput {
                description: Some("The output value.".into()),
            },
        );

        ActionManifest {
            name: "Test Action".into(),
            description: Some("A test action for unit tests.".into()),
            inputs,
            outputs,
        }
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("fetch-depth"), "fetch_depth");
        assert_eq!(to_snake_case("simple"), "simple");
        assert_eq!(to_snake_case("a-b-c"), "a_b_c");
    }

    #[test]
    fn test_action_dts_filename() {
        assert_eq!(
            action_dts_filename("actions/checkout"),
            "actions__checkout.d.ts"
        );
        assert_eq!(
            action_dts_filename("Swatinem/rust-cache"),
            "Swatinem__rust-cache.d.ts"
        );
    }

    #[test]
    fn test_generate_action_dts() {
        let manifest = sample_manifest();
        let dts = generate_action_dts("test/action", &manifest);
        insta::assert_snapshot!(dts);
    }

    #[test]
    fn test_generate_action_dts_no_required() {
        let mut inputs = IndexMap::new();
        inputs.insert(
            "my-input".into(),
            ActionInput {
                description: Some("An optional input.".into()),
                required: Some(false),
                default: None,
                deprecation_message: None,
            },
        );

        let manifest = ActionManifest {
            name: "Simple".into(),
            description: Some("A simple action.".into()),
            inputs,
            outputs: IndexMap::new(),
        };
        let dts = generate_action_dts("org/simple", &manifest);
        insta::assert_snapshot!(dts);
    }

    #[test]
    fn test_generate_mappings_js() {
        let manifest = sample_manifest();
        let mut cache = ManifestCache::new();
        cache.insert("test/action", "abc123", manifest);

        let mut lockfile = Lockfile::new();
        lockfile.actions.insert(
            "test/action".into(),
            crate::lockfile::LockedAction {
                version: "v1.0.0".into(),
                sha: "abc123".into(),
            },
        );

        let js = generate_mappings_js(&lockfile, &cache);
        insta::assert_snapshot!(js);
    }

    #[test]
    fn test_generate_mappings_js_no_renames() {
        // Action with no kebab-case names should produce empty mappings
        let mut inputs = IndexMap::new();
        inputs.insert(
            "simple".into(),
            ActionInput {
                description: None,
                required: None,
                default: None,
                deprecation_message: None,
            },
        );

        let manifest = ActionManifest {
            name: "Simple".into(),
            description: None,
            inputs,
            outputs: IndexMap::new(),
        };

        let mut cache = ManifestCache::new();
        cache.insert("org/simple", "def456", manifest);

        let mut lockfile = Lockfile::new();
        lockfile.actions.insert(
            "org/simple".into(),
            crate::lockfile::LockedAction {
                version: "v1.0.0".into(),
                sha: "def456".into(),
            },
        );

        let js = generate_mappings_js(&lockfile, &cache);
        insta::assert_snapshot!(js);
    }

    #[test]
    fn test_cache_evict_stale() {
        let mut cache = ManifestCache::new();
        cache.insert(
            "test/action",
            "sha1",
            ActionManifest {
                name: "Test".into(),
                description: None,
                inputs: IndexMap::new(),
                outputs: IndexMap::new(),
            },
        );
        cache.insert(
            "test/action",
            "sha2",
            ActionManifest {
                name: "Test".into(),
                description: None,
                inputs: IndexMap::new(),
                outputs: IndexMap::new(),
            },
        );

        let mut lockfile = Lockfile::new();
        lockfile.actions.insert(
            "test/action".into(),
            crate::lockfile::LockedAction {
                version: "v2.0.0".into(),
                sha: "sha2".into(),
            },
        );

        cache.evict_stale(&lockfile);
        assert!(cache.get("test/action", "sha1").is_none());
        assert!(cache.get("test/action", "sha2").is_some());
    }
}
