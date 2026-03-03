mod support;

use support::mock_github::{MockGitHubServer, mock_checkout, mock_rust_cache, mock_rust_toolchain};
use support::project::TestProject;

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
fn update_within_major() {
    let server = MockGitHubServer::new().add(mock_checkout()).start();
    let p = project_with_lockfile(
        "actions/checkout tag:v4.1.0 8ade135a41bc03ea155e62e844d188df1ea18608\n",
    );
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = server_env(&p, &["update", "actions/checkout"], &server);
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    snapshot!("diff", before.diff(&after));
    snapshot!(
        "dts",
        p.read_file(".github/ghat/actions/actions__checkout.d.ts")
    );
}

#[test]
fn update_already_latest() {
    let server = MockGitHubServer::new().add(mock_checkout()).start();
    let p = project_with_lockfile(
        "actions/checkout tag:v4.2.2 11bd71901bbe5b1630ceea73d27597364c9af683\n",
    );
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = server_env(&p, &["update", "actions/checkout"], &server);
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    let diff = before.diff(&after);
    assert!(diff.is_empty(), "expected no diff, got:\n{diff}");
}

#[test]
fn update_breaking() {
    let server = MockGitHubServer::new().add(mock_rust_cache()).start();
    let p = project_with_lockfile(
        "Swatinem/rust-cache tag:v1.4.0 cd47c6ad4b02e050fd481d890b2ea34778fd09d6\n",
    );
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = server_env(
        &p,
        &["update", "--breaking", "Swatinem/rust-cache"],
        &server,
    );
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    snapshot!("diff", before.diff(&after));
}

#[test]
fn update_all() {
    let server = MockGitHubServer::new()
        .add(mock_checkout())
        .add(mock_rust_cache())
        .start();
    let p = project_with_lockfile(
        "Swatinem/rust-cache tag:v2.7.0 bd47c6ad4b02e050fd481d890b2ea34778fd09d6\n\
         actions/checkout tag:v4.1.0 8ade135a41bc03ea155e62e844d188df1ea18608\n",
    );
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = server_env(&p, &["update"], &server);
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    snapshot!("diff", before.diff(&after));
}

#[test]
fn update_specific() {
    let server = MockGitHubServer::new()
        .add(mock_checkout())
        .add(mock_rust_cache())
        .start();
    let p = project_with_lockfile(
        "Swatinem/rust-cache tag:v2.7.0 bd47c6ad4b02e050fd481d890b2ea34778fd09d6\n\
         actions/checkout tag:v4.1.0 8ade135a41bc03ea155e62e844d188df1ea18608\n",
    );
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = server_env(&p, &["update", "actions/checkout"], &server);
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    snapshot!("diff", before.diff(&after));
}

#[test]
fn update_dry_run() {
    let server = MockGitHubServer::new()
        .add(mock_checkout())
        .add(mock_rust_cache())
        .start();
    let p = project_with_lockfile(
        "Swatinem/rust-cache tag:v2.7.0 bd47c6ad4b02e050fd481d890b2ea34778fd09d6\n\
         actions/checkout tag:v4.2.2 11bd71901bbe5b1630ceea73d27597364c9af683\n",
    );
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = server_env(&p, &["update", "--dry-run"], &server);
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    let diff = before.diff(&after);
    assert!(
        diff.is_empty(),
        "expected no changes to lockfile, got:\n{diff}"
    );
}

#[test]
fn update_empty_lockfile() {
    let server = MockGitHubServer::new().start();
    let p = project_with_lockfile("");

    let output = server_env(&p, &["update"], &server);
    snapshot!(output);
}

#[test]
fn update_not_found() {
    let server = MockGitHubServer::new().add(mock_checkout()).start();
    let p = project_with_lockfile(
        "actions/checkout tag:v4.2.2 11bd71901bbe5b1630ceea73d27597364c9af683\n",
    );

    let output = server_env(&p, &["update", "nonexistent/action"], &server);
    snapshot!(output);
}

#[test]
fn update_branch_ref() {
    let server = MockGitHubServer::new().add(mock_rust_toolchain()).start();
    let p = project_with_lockfile(
        "dtolnay/rust-toolchain branch:stable 0000000000000000000000000000000000000000\n",
    );
    let before = p.snapshot_glob(".github/ghat/ghat.lock");

    let output = server_env(&p, &["update", "dtolnay/rust-toolchain"], &server);
    let after = p.snapshot_glob(".github/ghat/ghat.lock");

    snapshot!("output", output);
    snapshot!("diff", before.diff(&after));
}
