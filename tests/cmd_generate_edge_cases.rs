mod support;

use support::TestProject;

#[test]
fn special_chars_in_job_name() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    const build = ctx.job("Build & Test (Linux)", {
      runs_on: "ubuntu-latest",
      steps() {
        run("cargo build")
        return { result: "ok" }
      }
    })
    ctx.job("Deploy [Staging]", {
      runs_on: "ubuntu-latest",
      needs: [build],
      steps(ctx) {
        run(`echo ${ctx.needs.build__test_linux.outputs.result}`)
      }
    })
  }
})
"#,
        )
        .build();

    let output = p.ghat(&["generate"]).run();
    snapshot!(p.generate_snapshot(&output));
}

#[test]
fn workflow_no_jobs() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/empty.ts",
            r#"workflow("Empty", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {}
})
"#,
        )
        .build();

    let output = p.ghat(&["generate"]).run();
    snapshot!(output);
}

#[test]
fn generate_overwrites_stale() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() { run("cargo build") }
    })
  }
})
"#,
        )
        .build();

    // First generation
    let output1 = p.ghat(&["generate"]).run();
    assert_eq!(output1.exit_code, 0);
    let before = p.snapshot_glob(".github/workflows/generated_*");

    // Modify the workflow
    std::fs::write(
        p.path().join(".github/ghat/workflows/ci.ts"),
        r#"workflow("CI", {
  on: triggers({ push: ["main", "develop"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() { run("cargo build --release") }
    })
  }
})
"#,
    )
    .unwrap();

    // Second generation should overwrite
    let output2 = p.ghat(&["generate"]).run();
    assert_eq!(output2.exit_code, 0);
    let after = p.snapshot_glob(".github/workflows/generated_*");

    snapshot!("diff", before.diff(&after));
}

#[test]
fn lockfile_malformed() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/ghat.lock",
            "this is not a valid lockfile format\n",
        )
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() { run("echo hello") }
    })
  }
})
"#,
        )
        .build();

    let output = p.ghat(&["generate"]).run();
    assert_ne!(output.exit_code, 0);
    snapshot!(output);
}

#[test]
fn lockfile_with_comments() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/ghat.lock",
            "# This is a comment\nactions/checkout tag:v4.2.2 11bd71901bbe5b1630ceea73d27597364c9af683\n\n# Another comment\n",
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
        run("echo hello")
      }
    })
  }
})
"#,
        )
        .build();

    let output = p.ghat(&["generate", "--no-check"]).run();
    snapshot!(output);
}

#[test]
fn generate_no_check_flag() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() { run("echo hello") }
    })
  }
})
"#,
        )
        .build();

    // With --no-check should succeed
    let output = p.ghat(&["generate", "--no-check"]).run();
    assert_eq!(output.exit_code, 0);
    snapshot!(output);
}
