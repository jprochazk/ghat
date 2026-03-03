use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

/// Mock data for a single GitHub action.
pub struct MockAction {
    pub owner: String,
    pub repo: String,
    pub latest_release: String,
    pub all_releases: Vec<String>,
    /// Map from tag name to (object_type, sha).
    /// object_type is "commit" for lightweight tags or "tag" for annotated.
    pub refs: HashMap<String, (String, String)>,
    /// Map from tag object SHA to commit SHA (for annotated tags).
    pub tags: HashMap<String, String>,
    /// Map from branch name to commit SHA.
    pub branch_refs: HashMap<String, String>,
    /// Optional action.yml content keyed by version.
    pub manifests: HashMap<String, String>,
}

/// A fake GitHub HTTP server for testing.
pub struct MockGitHubServer {
    actions: Vec<MockAction>,
}

/// A running mock server that can be dropped to stop.
pub struct RunningMockServer {
    url: String,
    server: Arc<tiny_http::Server>,
    _handle: thread::JoinHandle<()>,
}

impl MockGitHubServer {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// Add a mock action to the server.
    pub fn add(mut self, action: MockAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Start the server on a random port. Returns a running server with `.url()`.
    pub fn start(self) -> RunningMockServer {
        let server =
            Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("failed to start mock server"));
        let port = server.server_addr().to_ip().unwrap().port();
        let url = format!("http://127.0.0.1:{port}");

        let routes = build_routes(&self.actions);
        let server_clone = server.clone();

        let handle = thread::spawn(move || {
            loop {
                let request = match server_clone.recv() {
                    Ok(req) => req,
                    Err(_) => break, // server was unblocked/shut down
                };

                let path = request.url().to_string();

                let (status_code, body) = if let Some((code, body)) = routes.get(&path) {
                    (*code, body.clone())
                } else {
                    (
                        404,
                        format!(r#"{{"message":"Not Found","path":"{path}"}}"#),
                    )
                };

                let header = tiny_http::Header::from_bytes(
                    &b"Content-Type"[..],
                    &b"application/json"[..],
                )
                .unwrap();

                let response = tiny_http::Response::from_string(body)
                    .with_status_code(status_code)
                    .with_header(header);

                let _ = request.respond(response);
            }
        });

        RunningMockServer {
            url,
            server,
            _handle: handle,
        }
    }
}

impl RunningMockServer {
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Drop for RunningMockServer {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

fn build_routes(actions: &[MockAction]) -> HashMap<String, (u16, String)> {
    let mut routes: HashMap<String, (u16, String)> = HashMap::new();

    for action in actions {
        let owner = &action.owner;
        let repo = &action.repo;

        // GET /repos/{owner}/{repo}/releases/latest
        let path = format!("/repos/{owner}/{repo}/releases/latest");
        let body = format!(r#"{{"tag_name":"{}"}}"#, action.latest_release);
        routes.insert(path, (200, body));

        // GET /repos/{owner}/{repo}/releases?per_page=100
        let path = format!("/repos/{owner}/{repo}/releases?per_page=100");
        let releases: Vec<String> = action
            .all_releases
            .iter()
            .map(|tag| format!(r#"{{"tag_name":"{tag}"}}"#))
            .collect();
        let body = format!("[{}]", releases.join(","));
        routes.insert(path, (200, body));

        // GET /repos/{owner}/{repo}/git/ref/tags/{tag}
        for (tag, (object_type, sha)) in &action.refs {
            let path = format!("/repos/{owner}/{repo}/git/ref/tags/{tag}");
            let body = format!(r#"{{"object":{{"sha":"{sha}","type":"{object_type}"}}}}"#);
            routes.insert(path, (200, body));
        }

        // GET /repos/{owner}/{repo}/git/ref/heads/{branch}
        for (branch, sha) in &action.branch_refs {
            let path = format!("/repos/{owner}/{repo}/git/ref/heads/{branch}");
            let body = format!(r#"{{"object":{{"sha":"{sha}","type":"commit"}}}}"#);
            routes.insert(path, (200, body));
        }

        // GET /repos/{owner}/{repo}/git/tags/{sha}
        for (tag_sha, commit_sha) in &action.tags {
            let path = format!("/repos/{owner}/{repo}/git/tags/{tag_sha}");
            let body = format!(r#"{{"object":{{"sha":"{commit_sha}","type":"commit"}}}}"#);
            routes.insert(path, (200, body));
        }

        // GET /{owner}/{repo}/{version}/action.yml (raw content)
        for (version, yaml) in &action.manifests {
            let path = format!("/{owner}/{repo}/{version}/action.yml");
            routes.insert(path, (200, yaml.clone()));
        }
    }

    routes
}

// Pre-built mock actions matching the test data in src/github.rs::testing

/// actions/checkout — lightweight tags (object type "commit")
pub fn mock_checkout() -> MockAction {
    let mut refs = HashMap::new();
    refs.insert(
        "v4.2.2".into(),
        (
            "commit".into(),
            "11bd71901bbe5b1630ceea73d27597364c9af683".into(),
        ),
    );
    refs.insert(
        "v4.2.1".into(),
        (
            "commit".into(),
            "b4ffde65f46336ab88eb53be808477a3936bae11".into(),
        ),
    );
    refs.insert(
        "v4.1.0".into(),
        (
            "commit".into(),
            "8ade135a41bc03ea155e62e844d188df1ea18608".into(),
        ),
    );
    refs.insert(
        "v3.6.0".into(),
        (
            "commit".into(),
            "f43a0e5ff2bd294095638e18286ca9a3d1956744".into(),
        ),
    );

    let mut manifests = HashMap::new();
    let checkout_manifest = r#"
name: Checkout
description: Check out a Git repository at a particular version.
inputs:
  repository:
    description: Repository name with owner.
    required: false
  ref:
    description: The branch, tag or SHA to checkout.
    required: false
  token:
    description: Personal access token used to fetch the repository.
    required: false
    default: ${{ github.token }}
  fetch-depth:
    description: Number of commits to fetch. 0 indicates all history.
    required: false
    default: "1"
  submodules:
    description: Whether to checkout submodules.
    required: false
    default: "false"
outputs:
  ref:
    description: The branch, tag or SHA that was checked out.
  commit:
    description: The commit SHA that was checked out.
"#;
    for version in &["v4.2.2", "v4.2.1", "v4.1.0", "v3.6.0"] {
        manifests.insert(version.to_string(), checkout_manifest.to_string());
    }

    MockAction {
        owner: "actions".into(),
        repo: "checkout".into(),
        latest_release: "v4.2.2".into(),
        all_releases: vec![
            "v4.2.2".into(),
            "v4.2.1".into(),
            "v4.1.0".into(),
            "v3.6.0".into(),
        ],
        refs,
        tags: HashMap::new(),
        branch_refs: HashMap::new(),
        manifests,
    }
}

/// Swatinem/rust-cache — annotated tags (object type "tag", needs dereferencing)
pub fn mock_rust_cache() -> MockAction {
    let mut refs = HashMap::new();
    let mut tags = HashMap::new();

    refs.insert(
        "v2.7.8".into(),
        (
            "tag".into(),
            "aa7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
        ),
    );
    tags.insert(
        "aa7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
        "9d47c6ad4b02e050fd481d890b2ea34778fd09d6".into(),
    );

    refs.insert(
        "v2.7.7".into(),
        (
            "tag".into(),
            "bb7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
        ),
    );
    tags.insert(
        "bb7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
        "ad47c6ad4b02e050fd481d890b2ea34778fd09d6".into(),
    );

    refs.insert(
        "v2.7.0".into(),
        (
            "tag".into(),
            "cc7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
        ),
    );
    tags.insert(
        "cc7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
        "bd47c6ad4b02e050fd481d890b2ea34778fd09d6".into(),
    );

    refs.insert(
        "v1.4.0".into(),
        (
            "tag".into(),
            "dd7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
        ),
    );
    tags.insert(
        "dd7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
        "cd47c6ad4b02e050fd481d890b2ea34778fd09d6".into(),
    );

    let mut manifests = HashMap::new();
    let rust_cache_manifest = r#"
name: Rust Cache
description: A GitHub Action that implements smart caching for Rust projects.
inputs:
  shared-key:
    description: An additional key for the shared cache.
    required: false
    default: ""
  cache-on-failure:
    description: Cache even if the build fails.
    required: false
    default: "true"
outputs:
  cache-hit:
    description: Whether there was a cache hit.
"#;
    for version in &["v2.7.8", "v2.7.7", "v2.7.0", "v1.4.0"] {
        manifests.insert(version.to_string(), rust_cache_manifest.to_string());
    }

    MockAction {
        owner: "Swatinem".into(),
        repo: "rust-cache".into(),
        latest_release: "v2.7.8".into(),
        all_releases: vec![
            "v2.7.8".into(),
            "v2.7.7".into(),
            "v2.7.0".into(),
            "v1.4.0".into(),
        ],
        refs,
        tags,
        branch_refs: HashMap::new(),
        manifests,
    }
}

/// dtolnay/rust-toolchain — uses branch refs (e.g. "stable", "nightly")
pub fn mock_rust_toolchain() -> MockAction {
    let mut branch_refs = HashMap::new();
    branch_refs.insert(
        "stable".into(),
        "a3b77706cfa4c4bf431a39f7e267c878effdf858".into(),
    );
    branch_refs.insert(
        "nightly".into(),
        "b5e4cbcd8cdd1d1f4ee1efb08a5eb44ee7e2dbc0".into(),
    );

    let mut manifests = HashMap::new();
    let toolchain_manifest = r#"
name: Install Rust Toolchain
description: Install a Rust toolchain.
inputs:
  toolchain:
    description: Rust toolchain specification.
    required: false
  components:
    description: Comma-separated list of components to install.
    required: false
  targets:
    description: Comma-separated list of target triples to install.
    required: false
outputs:
  cachekey:
    description: A short hash of the installed rustc version.
"#;
    manifests.insert("stable".into(), toolchain_manifest.to_string());
    manifests.insert("nightly".into(), toolchain_manifest.to_string());

    MockAction {
        owner: "dtolnay".into(),
        repo: "rust-toolchain".into(),
        latest_release: "".into(),
        all_releases: vec![],
        refs: HashMap::new(),
        tags: HashMap::new(),
        branch_refs,
        manifests,
    }
}
