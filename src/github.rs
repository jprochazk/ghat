use indexmap::IndexMap;
use serde::Deserialize;

use crate::lockfile::RefKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRef {
    pub owner: String,
    pub repo: String,
    pub tag: Option<String>,
}

/// Parse `owner/repo` or `owner/repo@tag`.
pub fn parse_action_ref(input: &str) -> miette::Result<ActionRef> {
    let (owner_repo, tag) = match input.split_once('@') {
        Some((owner_repo, tag)) => (owner_repo, Some(tag.to_string())),
        None => (input, None),
    };

    let (owner, repo) = owner_repo.split_once('/').ok_or_else(|| {
        miette::miette!("invalid action reference: expected `owner/repo`, got: {input}")
    })?;

    if owner.is_empty() || repo.is_empty() {
        return Err(miette::miette!(
            "invalid action reference: owner and repo must not be empty: {input}"
        ));
    }

    if repo.contains('/') {
        return Err(miette::miette!(
            "invalid action reference: too many slashes: {input}"
        ));
    }

    Ok(ActionRef {
        owner: owner.to_string(),
        repo: repo.to_string(),
        tag,
    })
}

impl ActionRef {
    pub fn name(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAction {
    pub owner: String,
    pub repo: String,
    pub version: String,
    pub sha: String,
    pub ref_kind: RefKind,
}

// API response types

#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    pub tag_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitRef {
    pub object: GitObject,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitObject {
    pub sha: String,
    #[serde(rename = "type")]
    pub object_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitTag {
    pub object: GitObject,
}

// action.yml types

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct ActionManifest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub inputs: IndexMap<String, ActionInput>,
    #[serde(default)]
    pub outputs: IndexMap<String, ActionOutput>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct ActionInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(
        default,
        rename = "deprecationMessage",
        skip_serializing_if = "Option::is_none"
    )]
    pub deprecation_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct ActionOutput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// Trait abstracting the raw GitHub API

pub trait GitHubApi {
    fn get_latest_release(&self, owner: &str, repo: &str) -> miette::Result<Release>;
    fn list_releases(
        &self,
        owner: &str,
        repo: &str,
    ) -> miette::Result<Box<dyn Iterator<Item = miette::Result<Vec<Release>>> + '_>>;
    fn get_git_ref(&self, owner: &str, repo: &str, r#ref: &str) -> miette::Result<GitRef>;
    fn get_git_tag(&self, owner: &str, repo: &str, sha: &str) -> miette::Result<GitTag>;
    fn get_action_manifest(
        &self,
        owner: &str,
        repo: &str,
        version: &str,
    ) -> miette::Result<ActionManifest>;
}

// High-level resolve logic

pub fn resolve_action(
    api: &dyn GitHubApi,
    action_ref: &ActionRef,
) -> miette::Result<ResolvedAction> {
    match &action_ref.tag {
        Some(tag) if is_semver_like(tag) => {
            let req = parse_version_req(tag)?;
            resolve_compatible(api, &action_ref.owner, &action_ref.repo, &req)
        }
        Some(tag) => resolve_tag_or_branch(api, &action_ref.owner, &action_ref.repo, tag),
        None => resolve_latest(api, &action_ref.owner, &action_ref.repo),
    }
}

/// Resolve a ref that could be a tag or a branch. Tries tags first, falls back to branches.
fn resolve_tag_or_branch(
    api: &dyn GitHubApi,
    owner: &str,
    repo: &str,
    name: &str,
) -> miette::Result<ResolvedAction> {
    match resolve_tag(api, owner, repo, name) {
        Ok(resolved) => Ok(resolved),
        Err(_) => resolve_branch(api, owner, repo, name),
    }
}

pub fn resolve_latest(
    api: &dyn GitHubApi,
    owner: &str,
    repo: &str,
) -> miette::Result<ResolvedAction> {
    let release = api.get_latest_release(owner, repo)?;
    let sha = resolve_tag_to_sha(api, owner, repo, &release.tag_name)?;

    Ok(ResolvedAction {
        owner: owner.to_string(),
        repo: repo.to_string(),
        version: release.tag_name,
        sha,
        ref_kind: RefKind::Tag,
    })
}

/// Returns true if `tag` looks like a semver constraint (starts with a digit after stripping `v`).
fn is_semver_like(tag: &str) -> bool {
    let stripped = tag.strip_prefix('v').unwrap_or(tag);
    stripped.starts_with(|c: char| c.is_ascii_digit())
}

/// Parse a version constraint string like `v2`, `v2.7`, or `v2.7.8` into a `VersionReq`.
///
/// - `2` or `v2` → `>=2.0.0, <3.0.0`
/// - `2.7` or `v2.7` → `>=2.7.0, <2.8.0`
/// - `2.7.8` or `v2.7.8` → `=2.7.8`
pub fn parse_version_req(tag: &str) -> miette::Result<semver::VersionReq> {
    use semver::{Comparator, Op};

    let stripped = tag.strip_prefix('v').unwrap_or(tag);
    let parts: Vec<&str> = stripped.split('.').collect();

    let parse = |s: &str| -> miette::Result<u64> {
        s.parse()
            .map_err(|_| miette::miette!("invalid version constraint: {tag}"))
    };

    let comparators = match parts[..] {
        [maj] => {
            let major = parse(maj)?;
            vec![
                Comparator {
                    op: Op::GreaterEq,
                    major,
                    minor: Some(0),
                    patch: Some(0),
                    pre: semver::Prerelease::EMPTY,
                },
                Comparator {
                    op: Op::Less,
                    major: major + 1,
                    minor: Some(0),
                    patch: Some(0),
                    pre: semver::Prerelease::EMPTY,
                },
            ]
        }
        [maj, min] => {
            let major = parse(maj)?;
            let minor = parse(min)?;
            vec![
                Comparator {
                    op: Op::GreaterEq,
                    major,
                    minor: Some(minor),
                    patch: Some(0),
                    pre: semver::Prerelease::EMPTY,
                },
                Comparator {
                    op: Op::Less,
                    major,
                    minor: Some(minor + 1),
                    patch: Some(0),
                    pre: semver::Prerelease::EMPTY,
                },
            ]
        }
        [maj, min, pat] => {
            let major = parse(maj)?;
            let minor = parse(min)?;
            let patch = parse(pat)?;
            vec![Comparator {
                op: Op::Exact,
                major,
                minor: Some(minor),
                patch: Some(patch),
                pre: semver::Prerelease::EMPTY,
            }]
        }
        _ => return Err(miette::miette!("invalid version constraint: {tag}")),
    };

    Ok(semver::VersionReq { comparators })
}

/// Find the newest release matching a semver constraint.
///
/// GitHub returns releases newest-first, so the first match is the best.
pub fn resolve_compatible(
    api: &dyn GitHubApi,
    owner: &str,
    repo: &str,
    req: &semver::VersionReq,
) -> miette::Result<ResolvedAction> {
    let pages = api.list_releases(owner, repo)?;

    for page_result in pages {
        let page = page_result?;
        for release in &page {
            let tag = &release.tag_name;
            let stripped = tag.strip_prefix('v').unwrap_or(tag);
            if let Ok(ver) = semver::Version::parse(stripped) {
                if req.matches(&ver) {
                    let sha = resolve_tag_to_sha(api, owner, repo, tag)?;
                    return Ok(ResolvedAction {
                        owner: owner.to_string(),
                        repo: repo.to_string(),
                        version: tag.clone(),
                        sha,
                        ref_kind: RefKind::Tag,
                    });
                }
            }
        }
    }

    Err(miette::miette!(
        "no release matching {req} found for {owner}/{repo}"
    ))
}

pub fn resolve_tag(
    api: &dyn GitHubApi,
    owner: &str,
    repo: &str,
    tag: &str,
) -> miette::Result<ResolvedAction> {
    let sha = resolve_tag_to_sha(api, owner, repo, tag)?;

    Ok(ResolvedAction {
        owner: owner.to_string(),
        repo: repo.to_string(),
        version: tag.to_string(),
        sha,
        ref_kind: RefKind::Tag,
    })
}

pub fn resolve_branch(
    api: &dyn GitHubApi,
    owner: &str,
    repo: &str,
    branch: &str,
) -> miette::Result<ResolvedAction> {
    let git_ref = api.get_git_ref(owner, repo, &format!("heads/{branch}"))?;

    Ok(ResolvedAction {
        owner: owner.to_string(),
        repo: repo.to_string(),
        version: branch.to_string(),
        sha: git_ref.object.sha,
        ref_kind: RefKind::Branch,
    })
}

fn resolve_tag_to_sha(
    api: &dyn GitHubApi,
    owner: &str,
    repo: &str,
    tag: &str,
) -> miette::Result<String> {
    let git_ref = api.get_git_ref(owner, repo, &format!("tags/{tag}"))?;

    match git_ref.object.object_type.as_str() {
        "commit" => Ok(git_ref.object.sha),
        "tag" => {
            let git_tag = api.get_git_tag(owner, repo, &git_ref.object.sha)?;
            Ok(git_tag.object.sha)
        }
        other => Err(miette::miette!(
            "unexpected git ref type for {owner}/{repo}@{tag}: {other}"
        )),
    }
}

// Real HTTP implementation

pub struct GitHubClient {
    agent: ureq::Agent,
    token: Option<String>,
    api_base: String,
    content_base: String,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Self {
        let api_base = std::env::var("__GHAT_TEST_GITHUB_API_URL")
            .unwrap_or_else(|_| "https://api.github.com".to_string());
        let content_base = std::env::var("__GHAT_TEST_GITHUB_CONTENT_URL")
            .unwrap_or_else(|_| "https://raw.githubusercontent.com".to_string());
        Self {
            agent: ureq::Agent::new_with_defaults(),
            token,
            api_base,
            content_base,
        }
    }

    fn get_raw(&self, url: &str) -> miette::Result<String> {
        let mut req = self.agent.get(url);

        if let Some(token) = &self.token {
            req = req.header("Authorization", &format!("Bearer {token}"));
        }

        let mut response = req
            .header("User-Agent", "ghat")
            .call()
            .map_err(|e| miette::miette!("request failed for {url}: {e}"))?;

        response
            .body_mut()
            .read_to_string()
            .map_err(|e| miette::miette!("failed to read response for {url}: {e}"))
    }

    fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> miette::Result<T> {
        let url = format!("{}{path}", self.api_base);
        let mut req = self.agent.get(&url);

        if let Some(token) = &self.token {
            req = req.header("Authorization", &format!("Bearer {token}"));
        }

        let mut response = req
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "ghat")
            .call()
            .map_err(|e| miette::miette!("GitHub API request failed for {path}: {e}"))?;

        let body: T = response
            .body_mut()
            .read_json()
            .map_err(|e| miette::miette!("failed to parse GitHub API response for {path}: {e}"))?;

        Ok(body)
    }
}

/// Paginated iterator over GitHub API list endpoints.
struct ReleasePaginator<'a> {
    client: &'a GitHubClient,
    next_url: Option<String>,
}

impl Iterator for ReleasePaginator<'_> {
    type Item = miette::Result<Vec<Release>>;

    fn next(&mut self) -> Option<Self::Item> {
        let url = self.next_url.take()?;

        let mut req = self.client.agent.get(&url);
        if let Some(token) = &self.client.token {
            req = req.header("Authorization", &format!("Bearer {token}"));
        }

        let mut response = match req
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "ghat")
            .call()
        {
            Ok(r) => r,
            Err(e) => {
                return Some(Err(miette::miette!(
                    "GitHub API request failed for {url}: {e}"
                )));
            }
        };

        // Parse Link header for next page
        if let Some(link) = response.headers().get("link") {
            if let Ok(link) = link.to_str() {
                self.next_url = parse_link_next(link);
            }
        }

        let releases: Vec<Release> = match response.body_mut().read_json() {
            Ok(r) => r,
            Err(e) => {
                return Some(Err(miette::miette!(
                    "failed to parse GitHub API response for {url}: {e}"
                )));
            }
        };

        if releases.is_empty() {
            return None;
        }

        Some(Ok(releases))
    }
}

/// Parse the `rel="next"` URL from a GitHub `Link` header.
fn parse_link_next(link: &str) -> Option<String> {
    for part in link.split(',') {
        let part = part.trim();
        if part.ends_with("rel=\"next\"") {
            // Extract URL between < and >
            let start = part.find('<')? + 1;
            let end = part.find('>')?;
            return Some(part[start..end].to_string());
        }
    }
    None
}

impl GitHubApi for GitHubClient {
    fn get_latest_release(&self, owner: &str, repo: &str) -> miette::Result<Release> {
        self.get(&format!("/repos/{owner}/{repo}/releases/latest"))
    }

    fn list_releases(
        &self,
        owner: &str,
        repo: &str,
    ) -> miette::Result<Box<dyn Iterator<Item = miette::Result<Vec<Release>>> + '_>> {
        let url = format!(
            "{}/repos/{owner}/{repo}/releases?per_page=100",
            self.api_base
        );
        Ok(Box::new(ReleasePaginator {
            client: self,
            next_url: Some(url),
        }))
    }

    fn get_git_ref(&self, owner: &str, repo: &str, r#ref: &str) -> miette::Result<GitRef> {
        self.get(&format!("/repos/{owner}/{repo}/git/ref/{ref}"))
    }

    fn get_git_tag(&self, owner: &str, repo: &str, sha: &str) -> miette::Result<GitTag> {
        self.get(&format!("/repos/{owner}/{repo}/git/tags/{sha}"))
    }

    fn get_action_manifest(
        &self,
        owner: &str,
        repo: &str,
        version: &str,
    ) -> miette::Result<ActionManifest> {
        let yaml = self.get_raw(&format!(
            "{}/{owner}/{repo}/{version}/action.yml",
            self.content_base
        ))?;
        serde_yaml_ng::from_str(&yaml).map_err(|e| {
            miette::miette!("failed to parse action.yml for {owner}/{repo}@{version}: {e}")
        })
    }
}

#[cfg(test)]
pub mod testing {
    use super::*;
    use std::collections::HashMap;

    pub struct MockGitHubApi {
        /// Latest release per `owner/repo` (for `get_latest_release`).
        pub latest_release: HashMap<String, Release>,
        /// All releases per `owner/repo` (for `list_releases`), newest first.
        pub all_releases: HashMap<String, Vec<Release>>,
        pub refs: HashMap<String, GitRef>,
        pub tags: HashMap<String, GitTag>,
        pub manifests: HashMap<String, ActionManifest>,
    }

    impl MockGitHubApi {
        pub fn new() -> Self {
            Self {
                latest_release: HashMap::new(),
                all_releases: HashMap::new(),
                refs: HashMap::new(),
                tags: HashMap::new(),
                manifests: HashMap::new(),
            }
        }

        /// Merge another mock's data into this one.
        pub fn merge(&mut self, other: MockGitHubApi) {
            self.latest_release.extend(other.latest_release);
            for (key, releases) in other.all_releases {
                self.all_releases.entry(key).or_default().extend(releases);
            }
            self.refs.extend(other.refs);
            self.tags.extend(other.tags);
            self.manifests.extend(other.manifests);
        }
    }

    impl GitHubApi for MockGitHubApi {
        fn get_latest_release(&self, owner: &str, repo: &str) -> miette::Result<Release> {
            let key = format!("{owner}/{repo}");
            self.latest_release
                .get(&key)
                .cloned()
                .ok_or_else(|| miette::miette!("no release for {key}"))
        }

        fn list_releases(
            &self,
            owner: &str,
            repo: &str,
        ) -> miette::Result<Box<dyn Iterator<Item = miette::Result<Vec<Release>>> + '_>> {
            let key = format!("{owner}/{repo}");
            let releases = self.all_releases.get(&key).cloned().unwrap_or_default();
            Ok(Box::new(std::iter::once(Ok(releases))))
        }

        fn get_git_ref(&self, owner: &str, repo: &str, r#ref: &str) -> miette::Result<GitRef> {
            let key = format!("{owner}/{repo}/{ref}");
            self.refs
                .get(&key)
                .cloned()
                .ok_or_else(|| miette::miette!("no ref for {key}"))
        }

        fn get_git_tag(&self, owner: &str, repo: &str, sha: &str) -> miette::Result<GitTag> {
            let key = format!("{owner}/{repo}/{sha}");
            self.tags
                .get(&key)
                .cloned()
                .ok_or_else(|| miette::miette!("no tag for {key}"))
        }

        fn get_action_manifest(
            &self,
            owner: &str,
            repo: &str,
            version: &str,
        ) -> miette::Result<ActionManifest> {
            let key = format!("{owner}/{repo}/{version}");
            self.manifests
                .get(&key)
                .cloned()
                .ok_or_else(|| miette::miette!("no manifest for {key}"))
        }
    }

    pub fn release(tag: &str) -> Release {
        Release {
            tag_name: tag.to_string(),
        }
    }

    pub fn commit_ref(sha: &str) -> GitRef {
        GitRef {
            object: GitObject {
                sha: sha.to_string(),
                object_type: "commit".to_string(),
            },
        }
    }

    pub fn tag_ref(sha: &str) -> GitRef {
        GitRef {
            object: GitObject {
                sha: sha.to_string(),
                object_type: "tag".to_string(),
            },
        }
    }

    pub fn annotated_tag(commit_sha: &str) -> GitTag {
        GitTag {
            object: GitObject {
                sha: commit_sha.to_string(),
                object_type: "commit".to_string(),
            },
        }
    }

    /// actions/checkout - lightweight tag (object type "commit")
    /// Releases: v4.2.2, v4.2.1, v4.1.0, v3.6.0
    pub fn mock_checkout() -> MockGitHubApi {
        let mut mock = MockGitHubApi::new();
        mock.latest_release
            .insert("actions/checkout".into(), release("v4.2.2"));
        mock.all_releases.insert(
            "actions/checkout".into(),
            vec![
                release("v4.2.2"),
                release("v4.2.1"),
                release("v4.1.0"),
                release("v3.6.0"),
            ],
        );
        mock.refs.insert(
            "actions/checkout/tags/v4.2.2".into(),
            commit_ref("11bd71901bbe5b1630ceea73d27597364c9af683"),
        );
        mock.refs.insert(
            "actions/checkout/tags/v4.2.1".into(),
            commit_ref("b4ffde65f46336ab88eb53be808477a3936bae11"),
        );
        mock.refs.insert(
            "actions/checkout/tags/v4.1.0".into(),
            commit_ref("8ade135a41bc03ea155e62e844d188df1ea18608"),
        );
        mock.refs.insert(
            "actions/checkout/tags/v3.6.0".into(),
            commit_ref("f43a0e5ff2bd294095638e18286ca9a3d1956744"),
        );
        mock
    }

    /// Swatinem/rust-cache - annotated tag (object type "tag", needs dereferencing)
    /// Releases: v2.7.8, v2.7.7, v2.7.0, v1.4.0
    pub fn mock_rust_cache() -> MockGitHubApi {
        let mut mock = MockGitHubApi::new();
        mock.latest_release
            .insert("Swatinem/rust-cache".into(), release("v2.7.8"));
        mock.all_releases.insert(
            "Swatinem/rust-cache".into(),
            vec![
                release("v2.7.8"),
                release("v2.7.7"),
                release("v2.7.0"),
                release("v1.4.0"),
            ],
        );
        mock.refs.insert(
            "Swatinem/rust-cache/tags/v2.7.8".into(),
            tag_ref("aa7c1c80a07a27a84c0aa76d0cef0aad3830e330"),
        );
        mock.tags.insert(
            "Swatinem/rust-cache/aa7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
            annotated_tag("9d47c6ad4b02e050fd481d890b2ea34778fd09d6"),
        );
        mock.refs.insert(
            "Swatinem/rust-cache/tags/v2.7.7".into(),
            tag_ref("bb7c1c80a07a27a84c0aa76d0cef0aad3830e330"),
        );
        mock.tags.insert(
            "Swatinem/rust-cache/bb7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
            annotated_tag("ad47c6ad4b02e050fd481d890b2ea34778fd09d6"),
        );
        mock.refs.insert(
            "Swatinem/rust-cache/tags/v2.7.0".into(),
            tag_ref("cc7c1c80a07a27a84c0aa76d0cef0aad3830e330"),
        );
        mock.tags.insert(
            "Swatinem/rust-cache/cc7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
            annotated_tag("bd47c6ad4b02e050fd481d890b2ea34778fd09d6"),
        );
        mock.refs.insert(
            "Swatinem/rust-cache/tags/v1.4.0".into(),
            tag_ref("dd7c1c80a07a27a84c0aa76d0cef0aad3830e330"),
        );
        mock.tags.insert(
            "Swatinem/rust-cache/dd7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
            annotated_tag("cd47c6ad4b02e050fd481d890b2ea34778fd09d6"),
        );
        mock
    }

    /// Combined mock with both checkout and rust-cache fixtures.
    pub fn mock_multi() -> MockGitHubApi {
        let mut mock = mock_checkout();
        mock.merge(mock_rust_cache());
        mock
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testing::*;

    // parse_action_ref tests

    #[test]
    fn parse_owner_repo() {
        let r = parse_action_ref("Swatinem/rust-cache").unwrap();
        insta::assert_debug_snapshot!(r, @r#"
        ActionRef {
            owner: "Swatinem",
            repo: "rust-cache",
            tag: None,
        }
        "#);
    }

    #[test]
    fn parse_owner_repo_tag() {
        let r = parse_action_ref("taiki-e/install-action@v2").unwrap();
        insta::assert_debug_snapshot!(r, @r#"
        ActionRef {
            owner: "taiki-e",
            repo: "install-action",
            tag: Some(
                "v2",
            ),
        }
        "#);
    }

    #[test]
    fn parse_owner_repo_full_tag() {
        let r = parse_action_ref("actions/checkout@v4.1.0").unwrap();
        insta::assert_debug_snapshot!(r, @r#"
        ActionRef {
            owner: "actions",
            repo: "checkout",
            tag: Some(
                "v4.1.0",
            ),
        }
        "#);
    }

    #[test]
    fn parse_invalid_no_slash() {
        assert!(parse_action_ref("invalid").is_err());
    }

    #[test]
    fn parse_invalid_empty_parts() {
        assert!(parse_action_ref("/repo").is_err());
        assert!(parse_action_ref("owner/").is_err());
    }

    #[test]
    fn parse_invalid_too_many_slashes() {
        assert!(parse_action_ref("a/b/c").is_err());
    }

    #[test]
    fn action_ref_name() {
        let r = parse_action_ref("Swatinem/rust-cache@v2").unwrap();
        assert_eq!(r.name(), "Swatinem/rust-cache");
    }

    // resolve tests with lightweight tag (actions/checkout)

    #[test]
    fn resolve_lightweight_tag() {
        let mock = mock_checkout();
        let result = resolve_tag(&mock, "actions", "checkout", "v4.2.2").unwrap();
        insta::assert_debug_snapshot!(result, @r#"
        ResolvedAction {
            owner: "actions",
            repo: "checkout",
            version: "v4.2.2",
            sha: "11bd71901bbe5b1630ceea73d27597364c9af683",
            ref_kind: Tag,
        }
        "#);
    }

    #[test]
    fn resolve_latest_lightweight() {
        let mock = mock_checkout();
        let result = resolve_latest(&mock, "actions", "checkout").unwrap();
        insta::assert_debug_snapshot!(result, @r#"
        ResolvedAction {
            owner: "actions",
            repo: "checkout",
            version: "v4.2.2",
            sha: "11bd71901bbe5b1630ceea73d27597364c9af683",
            ref_kind: Tag,
        }
        "#);
    }

    // resolve tests with annotated tag (Swatinem/rust-cache)

    #[test]
    fn resolve_annotated_tag() {
        let mock = mock_rust_cache();
        let result = resolve_tag(&mock, "Swatinem", "rust-cache", "v2.7.8").unwrap();
        insta::assert_debug_snapshot!(result, @r#"
        ResolvedAction {
            owner: "Swatinem",
            repo: "rust-cache",
            version: "v2.7.8",
            sha: "9d47c6ad4b02e050fd481d890b2ea34778fd09d6",
            ref_kind: Tag,
        }
        "#);
    }

    #[test]
    fn resolve_latest_annotated() {
        let mock = mock_rust_cache();
        let result = resolve_latest(&mock, "Swatinem", "rust-cache").unwrap();
        insta::assert_debug_snapshot!(result, @r#"
        ResolvedAction {
            owner: "Swatinem",
            repo: "rust-cache",
            version: "v2.7.8",
            sha: "9d47c6ad4b02e050fd481d890b2ea34778fd09d6",
            ref_kind: Tag,
        }
        "#);
    }

    // resolve_action dispatch

    #[test]
    fn resolve_action_with_tag() {
        let mock = mock_checkout();
        let action_ref = parse_action_ref("actions/checkout@v4.2.2").unwrap();
        let result = resolve_action(&mock, &action_ref).unwrap();
        assert_eq!(result.version, "v4.2.2");
        assert_eq!(result.sha, "11bd71901bbe5b1630ceea73d27597364c9af683");
    }

    #[test]
    fn resolve_action_without_tag() {
        let mock = mock_rust_cache();
        let action_ref = parse_action_ref("Swatinem/rust-cache").unwrap();
        let result = resolve_action(&mock, &action_ref).unwrap();
        assert_eq!(result.version, "v2.7.8");
        assert_eq!(result.sha, "9d47c6ad4b02e050fd481d890b2ea34778fd09d6");
    }

    // action.yml parsing

    #[test]
    fn parse_manifest_checkout() {
        // Trimmed from real actions/checkout@v4.2.2 action.yml
        let yaml = r#"
name: 'Checkout'
description: 'Checkout a Git repository at a particular version'
inputs:
  repository:
    description: 'Repository name with owner. For example, actions/checkout'
    default: ${{ github.repository }}
  ref:
    description: 'The branch, tag or SHA to checkout.'
  token:
    description: 'Personal access token (PAT) used to fetch the repository.'
    default: ${{ github.token }}
  fetch-depth:
    description: 'Number of commits to fetch. 0 indicates all history for all branches and tags.'
    default: 1
outputs:
  ref:
    description: 'The branch, tag or SHA that was checked out'
  commit:
    description: 'The commit SHA that was checked out'
runs:
  using: node20
  main: dist/index.js
  post: dist/index.js
"#;
        let manifest: ActionManifest = serde_yaml_ng::from_str(yaml).unwrap();
        insta::assert_debug_snapshot!(manifest);
    }

    #[test]
    fn parse_manifest_rust_cache() {
        // Trimmed from real Swatinem/rust-cache@v2.7.8 action.yml
        let yaml = r#"
name: "Rust Cache"
description: "A GitHub Action that implements smart caching for rust/cargo projects."
inputs:
  prefix-key:
    description: "The prefix cache key."
    required: false
    default: "v0-rust"
  shared-key:
    description: "A shared cache key stable over multiple jobs."
    required: false
  cache-on-failure:
    description: "Cache even if the build fails."
    required: false
  save-if:
    description: "Whether the cache should be saved."
    required: false
    default: "true"
outputs:
  cache-hit:
    description: "A boolean value that indicates an exact match was found."
runs:
  using: "node20"
  main: "dist/restore/index.js"
  post: "dist/save/index.js"
"#;
        let manifest: ActionManifest = serde_yaml_ng::from_str(yaml).unwrap();
        insta::assert_debug_snapshot!(manifest);
    }

    #[test]
    fn parse_manifest_no_inputs_no_outputs() {
        let yaml = r#"
name: "Minimal"
description: "A minimal action"
runs:
  using: node20
  main: index.js
"#;
        let manifest: ActionManifest = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(manifest.inputs.is_empty());
        assert!(manifest.outputs.is_empty());
    }

    #[test]
    fn parse_manifest_deprecation_message() {
        let yaml = r#"
name: "Deprecated Input"
inputs:
  old-input:
    description: "Don't use this"
    deprecationMessage: "Use new-input instead"
  new-input:
    description: "Use this"
runs:
  using: node20
  main: index.js
"#;
        let manifest: ActionManifest = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(
            manifest.inputs["old-input"].deprecation_message.as_deref(),
            Some("Use new-input instead")
        );
        assert!(manifest.inputs["new-input"].deprecation_message.is_none());
    }

    // parse_version_req tests

    #[test]
    fn version_req_major_only() {
        let req = parse_version_req("v2").unwrap();
        assert!(req.matches(&semver::Version::new(2, 0, 0)));
        assert!(req.matches(&semver::Version::new(2, 99, 99)));
        assert!(!req.matches(&semver::Version::new(3, 0, 0)));
        assert!(!req.matches(&semver::Version::new(1, 9, 9)));
    }

    #[test]
    fn version_req_major_minor() {
        let req = parse_version_req("v2.7").unwrap();
        assert!(req.matches(&semver::Version::new(2, 7, 0)));
        assert!(req.matches(&semver::Version::new(2, 7, 99)));
        assert!(!req.matches(&semver::Version::new(2, 8, 0)));
        assert!(!req.matches(&semver::Version::new(2, 6, 99)));
    }

    #[test]
    fn version_req_exact() {
        let req = parse_version_req("v2.7.8").unwrap();
        assert!(req.matches(&semver::Version::new(2, 7, 8)));
        assert!(!req.matches(&semver::Version::new(2, 7, 7)));
        assert!(!req.matches(&semver::Version::new(2, 7, 9)));
    }

    #[test]
    fn version_req_no_v_prefix() {
        let req = parse_version_req("4").unwrap();
        assert!(req.matches(&semver::Version::new(4, 2, 2)));
        assert!(!req.matches(&semver::Version::new(5, 0, 0)));
    }

    // is_semver_like tests

    #[test]
    fn semver_like_detection() {
        assert!(is_semver_like("v2"));
        assert!(is_semver_like("v2.7"));
        assert!(is_semver_like("v2.7.8"));
        assert!(is_semver_like("2"));
        assert!(!is_semver_like("main"));
        assert!(!is_semver_like("feature-branch"));
    }

    // resolve_compatible tests

    #[test]
    fn compatible_major_resolves_highest() {
        let mock = mock_checkout();
        let req = parse_version_req("v4").unwrap();
        let result = resolve_compatible(&mock, "actions", "checkout", &req).unwrap();
        assert_eq!(result.version, "v4.2.2");
        assert_eq!(result.sha, "11bd71901bbe5b1630ceea73d27597364c9af683");
    }

    #[test]
    fn compatible_major_minor_resolves_highest_patch() {
        let mock = mock_rust_cache();
        let req = parse_version_req("v2.7").unwrap();
        let result = resolve_compatible(&mock, "Swatinem", "rust-cache", &req).unwrap();
        assert_eq!(result.version, "v2.7.8");
        assert_eq!(result.sha, "9d47c6ad4b02e050fd481d890b2ea34778fd09d6");
    }

    #[test]
    fn compatible_exact_version() {
        let mock = mock_rust_cache();
        let req = parse_version_req("v2.7.7").unwrap();
        let result = resolve_compatible(&mock, "Swatinem", "rust-cache", &req).unwrap();
        assert_eq!(result.version, "v2.7.7");
        assert_eq!(result.sha, "ad47c6ad4b02e050fd481d890b2ea34778fd09d6");
    }

    #[test]
    fn compatible_different_major() {
        let mock = mock_rust_cache();
        let req = parse_version_req("v1").unwrap();
        let result = resolve_compatible(&mock, "Swatinem", "rust-cache", &req).unwrap();
        assert_eq!(result.version, "v1.4.0");
        assert_eq!(result.sha, "cd47c6ad4b02e050fd481d890b2ea34778fd09d6");
    }

    #[test]
    fn compatible_no_match() {
        let mock = mock_checkout();
        let req = parse_version_req("v99").unwrap();
        let result = resolve_compatible(&mock, "actions", "checkout", &req);
        assert!(result.is_err());
    }

    #[test]
    fn compatible_major_skips_other_major() {
        let mock = mock_checkout();
        // v3 should only match v3.6.0, not v4.x
        let req = parse_version_req("v3").unwrap();
        let result = resolve_compatible(&mock, "actions", "checkout", &req).unwrap();
        assert_eq!(result.version, "v3.6.0");
        assert_eq!(result.sha, "f43a0e5ff2bd294095638e18286ca9a3d1956744");
    }

    // resolve_action dispatch with semver

    #[test]
    fn resolve_action_semver_major() {
        let mock = mock_checkout();
        let action_ref = parse_action_ref("actions/checkout@v4").unwrap();
        let result = resolve_action(&mock, &action_ref).unwrap();
        assert_eq!(result.version, "v4.2.2");
        assert_eq!(result.sha, "11bd71901bbe5b1630ceea73d27597364c9af683");
    }

    #[test]
    fn resolve_action_semver_major_minor() {
        let mock = mock_checkout();
        let action_ref = parse_action_ref("actions/checkout@v4.1").unwrap();
        let result = resolve_action(&mock, &action_ref).unwrap();
        assert_eq!(result.version, "v4.1.0");
        assert_eq!(result.sha, "8ade135a41bc03ea155e62e844d188df1ea18608");
    }

    #[test]
    fn resolve_action_semver_exact() {
        let mock = mock_checkout();
        let action_ref = parse_action_ref("actions/checkout@v4.2.2").unwrap();
        let result = resolve_action(&mock, &action_ref).unwrap();
        assert_eq!(result.version, "v4.2.2");
        assert_eq!(result.sha, "11bd71901bbe5b1630ceea73d27597364c9af683");
    }

    #[test]
    fn resolve_action_branch_ref_not_semver() {
        // Non-semver tags should still go through resolve_tag directly
        let mut mock = MockGitHubApi::new();
        mock.refs
            .insert("owner/repo/tags/main".into(), commit_ref("abc123"));
        let action_ref = parse_action_ref("owner/repo@main").unwrap();
        let result = resolve_action(&mock, &action_ref).unwrap();
        assert_eq!(result.version, "main");
        assert_eq!(result.sha, "abc123");
    }
}
