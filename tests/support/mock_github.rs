use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
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
    _handle: thread::JoinHandle<()>,
    shutdown: Arc<Mutex<bool>>,
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
        let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind");
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{port}");
        let shutdown = Arc::new(Mutex::new(false));
        let shutdown_clone = shutdown.clone();

        // Build route table
        let routes = build_routes(&self.actions);

        let handle = thread::spawn(move || {
            listener
                .set_nonblocking(true)
                .expect("failed to set nonblocking");
            loop {
                if *shutdown_clone.lock().unwrap() {
                    break;
                }
                match listener.accept() {
                    Ok((stream, _)) => {
                        let routes = routes.clone();
                        // Handle in the same thread (tests are sequential per-server)
                        handle_connection(stream, &routes);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(std::time::Duration::from_millis(5));
                        continue;
                    }
                    Err(e) => {
                        eprintln!("mock server accept error: {e}");
                        break;
                    }
                }
            }
        });

        RunningMockServer {
            url,
            _handle: handle,
            shutdown,
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
        *self.shutdown.lock().unwrap() = true;
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
            let body = format!(
                r#"{{"object":{{"sha":"{sha}","type":"{object_type}"}}}}"#
            );
            routes.insert(path, (200, body));
        }

        // GET /repos/{owner}/{repo}/git/tags/{sha}
        for (tag_sha, commit_sha) in &action.tags {
            let path = format!("/repos/{owner}/{repo}/git/tags/{tag_sha}");
            let body = format!(
                r#"{{"object":{{"sha":"{commit_sha}","type":"commit"}}}}"#
            );
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

fn handle_connection(
    mut stream: std::net::TcpStream,
    routes: &HashMap<String, (u16, String)>,
) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }

    // Parse "GET /path HTTP/1.1"
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let path = parts[1];

    // Consume remaining headers
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
            break;
        }
    }

    let (status, body) = if let Some((code, body)) = routes.get(path) {
        (*code, body.clone())
    } else {
        (404, format!(r#"{{"message":"Not Found","path":"{path}"}}"#))
    };

    let status_text = match status {
        200 => "OK",
        404 => "Not Found",
        _ => "Unknown",
    };

    let response = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

// Pre-built mock actions matching the test data in src/github.rs::testing

/// actions/checkout — lightweight tags (object type "commit")
pub fn mock_checkout() -> MockAction {
    let mut refs = HashMap::new();
    refs.insert(
        "v4.2.2".into(),
        ("commit".into(), "11bd71901bbe5b1630ceea73d27597364c9af683".into()),
    );
    refs.insert(
        "v4.2.1".into(),
        ("commit".into(), "b4ffde65f46336ab88eb53be808477a3936bae11".into()),
    );
    refs.insert(
        "v4.1.0".into(),
        ("commit".into(), "8ade135a41bc03ea155e62e844d188df1ea18608".into()),
    );
    refs.insert(
        "v3.6.0".into(),
        ("commit".into(), "f43a0e5ff2bd294095638e18286ca9a3d1956744".into()),
    );

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
        manifests: HashMap::new(),
    }
}

/// Swatinem/rust-cache — annotated tags (object type "tag", needs dereferencing)
pub fn mock_rust_cache() -> MockAction {
    let mut refs = HashMap::new();
    let mut tags = HashMap::new();

    refs.insert(
        "v2.7.8".into(),
        ("tag".into(), "aa7c1c80a07a27a84c0aa76d0cef0aad3830e330".into()),
    );
    tags.insert(
        "aa7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
        "9d47c6ad4b02e050fd481d890b2ea34778fd09d6".into(),
    );

    refs.insert(
        "v2.7.7".into(),
        ("tag".into(), "bb7c1c80a07a27a84c0aa76d0cef0aad3830e330".into()),
    );
    tags.insert(
        "bb7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
        "ad47c6ad4b02e050fd481d890b2ea34778fd09d6".into(),
    );

    refs.insert(
        "v2.7.0".into(),
        ("tag".into(), "cc7c1c80a07a27a84c0aa76d0cef0aad3830e330".into()),
    );
    tags.insert(
        "cc7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
        "bd47c6ad4b02e050fd481d890b2ea34778fd09d6".into(),
    );

    refs.insert(
        "v1.4.0".into(),
        ("tag".into(), "dd7c1c80a07a27a84c0aa76d0cef0aad3830e330".into()),
    );
    tags.insert(
        "dd7c1c80a07a27a84c0aa76d0cef0aad3830e330".into(),
        "cd47c6ad4b02e050fd481d890b2ea34778fd09d6".into(),
    );

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
        manifests: HashMap::new(),
    }
}
