mod support;

use support::mock_github::{mock_checkout, mock_rust_cache, MockGitHubServer};
use support::project::TestProject;

fn project_with_empty_lockfile() -> TestProject {
    TestProject::new().init().build()
}

fn project_with_lockfile(content: &str) -> TestProject {
    TestProject::new()
        .init()
        .file(".github/ghat/ghat.lock", content)
        .build()
}

fn server_env(
    p: &TestProject,
    args: &[&str],
    server: &support::mock_github::RunningMockServer,
) -> support::project::CommandOutput {
    p.ghat(args)
        .env("GITHUB_TOKEN", "fake")
        .env("__GHAT_TEST_GITHUB_API_URL", server.url())
        .env("__GHAT_TEST_GITHUB_CONTENT_URL", server.url())
        .run()
}

#[test]
fn add_single_action() {
    let server = MockGitHubServer::new().add(mock_checkout()).start();
    let p = project_with_empty_lockfile();
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = server_env(&p, &["add", "actions/checkout@v4.2.2"], &server);
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    snapshot!("diff", before.diff(&after));
}

#[test]
fn add_multiple_actions() {
    let server = MockGitHubServer::new()
        .add(mock_checkout())
        .add(mock_rust_cache())
        .start();
    let p = project_with_empty_lockfile();
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output =
        server_env(&p, &["add", "actions/checkout@v4.2.2", "Swatinem/rust-cache"], &server);
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    snapshot!("diff", before.diff(&after));
}

#[test]
fn add_with_version_constraint() {
    let server = MockGitHubServer::new().add(mock_checkout()).start();
    let p = project_with_empty_lockfile();
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = server_env(&p, &["add", "actions/checkout@v4"], &server);
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    snapshot!("diff", before.diff(&after));
}

#[test]
fn add_exact_version() {
    let server = MockGitHubServer::new().add(mock_checkout()).start();
    let p = project_with_empty_lockfile();
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = server_env(&p, &["add", "actions/checkout@v4.2.1"], &server);
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    snapshot!("diff", before.diff(&after));
}

#[test]
fn add_duplicate_skipped() {
    let server = MockGitHubServer::new().add(mock_checkout()).start();
    let p = project_with_lockfile(
        "actions/checkout v4.2.2 11bd71901bbe5b1630ceea73d27597364c9af683\n",
    );
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = server_env(&p, &["add", "actions/checkout@v4.2.2"], &server);
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    // No changes to lockfile
    let diff = before.diff(&after);
    assert!(diff.is_empty(), "expected no diff, got:\n{diff}");
}

#[test]
fn add_no_lockfile() {
    let p = TestProject::new().build();
    let output = p.ghat(&["add", "actions/checkout"]).run();
    snapshot!(output);
}

#[test]
fn add_no_actions() {
    let p = project_with_empty_lockfile();
    let output = p.ghat(&["add"]).run();
    snapshot!(output);
}

#[test]
fn add_invalid_ref() {
    let p = project_with_empty_lockfile();
    let output = p.ghat(&["add", "invalid"]).run();
    snapshot!(output);
}
