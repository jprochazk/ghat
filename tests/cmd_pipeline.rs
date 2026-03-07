mod support;

use support::mock_github::{
    MockGitHubServer, mock_checkout, mock_rust_cache, mock_rust_toolchain,
};
use support::project::TestProject;

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

/// Full pipeline: add action, write workflow using it, generate YAML.
#[test]
fn add_then_generate() {
    let server = MockGitHubServer::new().add(mock_checkout()).start();
    let p = TestProject::new().init().build();

    let add_output = server_env(&p, &["add", "actions/checkout@v4"], &server);
    assert_eq!(add_output.exit_code, 0);

    std::fs::write(
        p.path().join(".github/ghat/workflows/ci.ts"),
        r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() {
        uses("actions/checkout")
        run("cargo build")
      }
    })
  }
})
"#,
    )
    .unwrap();

    let gen_output = p.ghat(&["generate", "--no-check"]).run();
    snapshot!(p.generate_snapshot(&gen_output));
}

/// Add action, use it with `with` params, verify YAML has correct inputs.
#[test]
fn add_then_uses_with_inputs() {
    let server = MockGitHubServer::new().add(mock_checkout()).start();
    let p = TestProject::new().init().build();

    let add_output = server_env(&p, &["add", "actions/checkout@v4.2.2"], &server);
    assert_eq!(add_output.exit_code, 0);

    std::fs::write(
        p.path().join(".github/ghat/workflows/ci.ts"),
        r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() {
        uses("actions/checkout", {
          with: { ref: "develop", fetch_depth: "0" },
        })
        run("cargo build")
      }
    })
  }
})
"#,
    )
    .unwrap();

    let gen_output = p.ghat(&["generate", "--no-check"]).run();
    snapshot!(p.generate_snapshot(&gen_output));
}

/// Add two actions, write workflow using both, verify both are pinned.
#[test]
fn add_multiple_then_generate() {
    let server = MockGitHubServer::new()
        .add(mock_checkout())
        .add(mock_rust_cache())
        .start();
    let p = TestProject::new().init().build();

    let add_output = server_env(
        &p,
        &["add", "actions/checkout@v4", "Swatinem/rust-cache@v2"],
        &server,
    );
    assert_eq!(add_output.exit_code, 0);

    std::fs::write(
        p.path().join(".github/ghat/workflows/ci.ts"),
        r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() {
        uses("actions/checkout")
        uses("Swatinem/rust-cache")
        run("cargo build")
      }
    })
  }
})
"#,
    )
    .unwrap();

    let gen_output = p.ghat(&["generate", "--no-check"]).run();
    snapshot!(p.generate_snapshot(&gen_output));
}

/// Add branch ref action, use in workflow, verify SHA pinning.
#[test]
fn add_branch_then_generate() {
    let server = MockGitHubServer::new()
        .add(mock_rust_toolchain())
        .start();
    let p = TestProject::new().init().build();

    let add_output = server_env(&p, &["add", "dtolnay/rust-toolchain@stable"], &server);
    assert_eq!(add_output.exit_code, 0);

    std::fs::write(
        p.path().join(".github/ghat/workflows/ci.ts"),
        r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() {
        uses("dtolnay/rust-toolchain")
        run("cargo build")
      }
    })
  }
})
"#,
    )
    .unwrap();

    let gen_output = p.ghat(&["generate", "--no-check"]).run();
    snapshot!(p.generate_snapshot(&gen_output));
}

/// Update action version, regenerate, verify new SHA in YAML.
#[test]
fn update_then_generate() {
    let server = MockGitHubServer::new().add(mock_checkout()).start();
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/ghat.lock",
            "actions/checkout tag:v4.1.0 8ade135a41bc03ea155e62e844d188df1ea18608\n",
        )
        .build();

    std::fs::write(
        p.path().join(".github/ghat/workflows/ci.ts"),
        r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() {
        uses("actions/checkout")
        run("cargo build")
      }
    })
  }
})
"#,
    )
    .unwrap();

    // Generate with old version
    let gen_before = p.ghat(&["generate", "--no-check"]).run();
    assert_eq!(gen_before.exit_code, 0);
    let before = p.snapshot_glob(".github/workflows/generated_*");

    // Update
    let update_output = server_env(&p, &["update"], &server);
    assert_eq!(update_output.exit_code, 0);
    snapshot!("update_output", update_output);

    // Regenerate
    let gen_after = p.ghat(&["generate", "--no-check"]).run();
    assert_eq!(gen_after.exit_code, 0);
    let after = p.snapshot_glob(".github/workflows/generated_*");

    snapshot!("diff", before.diff(&after));
}

/// Remove an action that is still used by a workflow, generate should fail.
#[test]
fn rm_then_generate_fails() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/ghat.lock",
            "actions/checkout tag:v4.2.2 11bd71901bbe5b1630ceea73d27597364c9af683\n",
        )
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() {
        uses("actions/checkout")
        run("cargo build")
      }
    })
  }
})
"#,
        )
        .build();

    // Remove the action
    let rm_output = p.ghat(&["rm", "actions/checkout"]).run();
    assert_eq!(rm_output.exit_code, 0);

    // Generate should fail because workflow still uses it
    let gen_output = p.ghat(&["generate", "--no-check"]).run();
    assert_ne!(gen_output.exit_code, 0);
    snapshot!(gen_output);
}

/// Remove an unused action, generate should succeed.
#[test]
fn rm_unused_then_generate() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/ghat.lock",
            "actions/checkout tag:v4.2.2 11bd71901bbe5b1630ceea73d27597364c9af683\nSwatinem/rust-cache tag:v2.7.8 9d47c6ad4b02e050fd481d890b2ea34778fd09d6\n",
        )
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() {
        uses("actions/checkout")
        run("cargo build")
      }
    })
  }
})
"#,
        )
        .build();

    // Remove unused action
    let rm_output = p.ghat(&["rm", "Swatinem/rust-cache"]).run();
    assert_eq!(rm_output.exit_code, 0);

    // Generate should still work
    let gen_output = p.ghat(&["generate", "--no-check"]).run();
    assert_eq!(gen_output.exit_code, 0);
    snapshot!(p.generate_snapshot(&gen_output));
}
