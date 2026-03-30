mod support;

use support::mock_github::{MockGitHubServer, mock_default_actions};
use support::TestProject;

fn init_with_server(
    p: &TestProject,
    server: &support::mock_github::RunningMockServer,
) -> support::project::CommandOutput {
    p.ghat(&["init"])
        .env("GITHUB_TOKEN", "fake")
        .env("__GHAT_TEST_GITHUB_API_URL", server.url())
        .env("__GHAT_TEST_GITHUB_CONTENT_URL", server.url())
        .run()
}

#[test]
fn fresh_project() {
    let server = MockGitHubServer::from_actions(mock_default_actions()).start();
    let p = TestProject::new().build();
    let output = init_with_server(&p, &server);

    snapshot!("output", output);
    snapshot!("project", p.snapshot_full());
}

#[test]
fn idempotent() {
    let server = MockGitHubServer::from_actions(mock_default_actions()).start();
    let p = TestProject::new().build();

    let out1 = init_with_server(&p, &server);
    assert_eq!(out1.exit_code, 0);
    let first = p.snapshot_full();

    let out2 = init_with_server(&p, &server);
    assert_eq!(out2.exit_code, 0);
    let second = p.snapshot_full();

    // No diff between first and second init
    let diff = first.diff(&second);
    assert!(diff.is_empty(), "expected no diff, got:\n{diff}");
}

#[test]
fn updates_type_defs() {
    let server = MockGitHubServer::from_actions(mock_default_actions()).start();
    let p = TestProject::new().build();

    let out1 = init_with_server(&p, &server);
    assert_eq!(out1.exit_code, 0);
    let before = p.snapshot_full();

    // Corrupt a type definition file
    std::fs::write(p.path().join(".github/ghat/types/api.d.ts"), "// modified").unwrap();

    // Re-running init should restore it
    let out2 = init_with_server(&p, &server);
    assert_eq!(out2.exit_code, 0);
    let after = p.snapshot_full();

    // Should be identical to original
    let diff = before.diff(&after);
    assert!(diff.is_empty(), "expected no diff, got:\n{diff}");
}

#[test]
fn bare_skips_defaults() {
    let p = TestProject::new().build();
    let output = p.ghat(&["init", "--bare"]).run();
    assert_eq!(output.exit_code, 0);

    // No ghat_check.ts created
    assert!(!p.file_exists(".github/ghat/workflows/ghat_check.ts"));
    // Empty lockfile
    assert_eq!(p.read_file(".github/ghat/ghat.lock"), "");
}

// --- Migration scenarios ---

/// Old-style project with hardcoded ghat_check.yaml gets it removed and replaced
/// with the new ghat_check.ts workflow definition.
#[test]
fn migration_removes_old_check_workflow() {
    let server = MockGitHubServer::from_actions(mock_default_actions()).start();
    let p = TestProject::new()
        .file(
            ".github/workflows/ghat_check.yaml",
            "# old hardcoded workflow\n",
        )
        .build();

    let output = init_with_server(&p, &server);
    assert_eq!(output.exit_code, 0);

    assert!(!p.file_exists(".github/workflows/ghat_check.yaml"));
    assert!(p.file_exists(".github/ghat/workflows/ghat_check.ts"));
    snapshot!(output);
}

/// Lockfile already has some default actions (e.g. checkout), init adds only the missing ones.
#[test]
fn migration_adds_missing_default_actions() {
    let server = MockGitHubServer::from_actions(mock_default_actions()).start();
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/ghat.lock",
            "actions/checkout tag:v4.2.2 11bd71901bbe5b1630ceea73d27597364c9af683\n",
        )
        .build();

    let output = init_with_server(&p, &server);
    assert_eq!(output.exit_code, 0);

    let lockfile = p.read_file(".github/ghat/ghat.lock");
    assert!(lockfile.contains("actions/checkout"));
    assert!(lockfile.contains("actions/cache"));
    assert!(lockfile.contains("actions/upload-artifact"));
    assert!(lockfile.contains("actions/download-artifact"));
    snapshot!(output);
}

/// Lockfile already has all default actions, init skips them all.
#[test]
fn migration_all_defaults_already_present() {
    let server = MockGitHubServer::from_actions(mock_default_actions()).start();
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/ghat.lock",
            "actions/checkout tag:v4.2.2 11bd71901bbe5b1630ceea73d27597364c9af683\n\
             actions/cache tag:v4.2.0 ab5e6d0c87105b4c9c2047343972218f562e4319\n\
             actions/upload-artifact tag:v4.6.0 65c4c4a1ddee5b72f698fdd19549f0f0fb45cf08\n\
             actions/download-artifact tag:v4.2.0 fa0a91b85d4f404e444e00e005971372dc801d16\n",
        )
        .build();

    let output = init_with_server(&p, &server);
    assert_eq!(output.exit_code, 0);
    snapshot!(output);
}

/// User-customized ghat_check.ts is not overwritten by init.
#[test]
fn migration_preserves_custom_check_workflow() {
    let server = MockGitHubServer::from_actions(mock_default_actions()).start();
    let custom = r#"workflow("ghat check", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Custom", {
      runs_on: "ubuntu-latest",
      steps() {
        uses("actions/checkout")
        run("echo custom check")
      },
    })
  },
})
"#;
    let p = TestProject::new()
        .init()
        .file(".github/ghat/workflows/ghat_check.ts", custom)
        .build();

    let output = init_with_server(&p, &server);
    assert_eq!(output.exit_code, 0);

    // Custom workflow is preserved
    assert_eq!(p.read_file(".github/ghat/workflows/ghat_check.ts"), custom);
}

/// Full migration + generate: old project gets migrated, then generate produces
/// the new generated_ghat_check.yaml.
#[test]
fn migration_then_generate() {
    let server = MockGitHubServer::from_actions(mock_default_actions()).start();
    let p = TestProject::new()
        .init()
        .file(
            ".github/workflows/ghat_check.yaml",
            "# old hardcoded workflow\n",
        )
        .file(
            ".github/ghat/ghat.lock",
            "actions/checkout tag:v4.2.2 11bd71901bbe5b1630ceea73d27597364c9af683\n",
        )
        .build();

    // Migrate
    let init_output = init_with_server(&p, &server);
    assert_eq!(init_output.exit_code, 0);
    assert!(!p.file_exists(".github/workflows/ghat_check.yaml"));

    // Generate
    let gen_output = p.ghat(&["generate", "--no-check"]).run();
    assert_eq!(gen_output.exit_code, 0);
    assert!(p.file_exists(".github/workflows/generated_ghat_check.yaml"));
    snapshot!(p.generate_snapshot(&gen_output));
}
