mod support;

use support::TestProject;

#[test]
fn uses_with_inputs() {
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
        uses("actions/checkout", {
          with: { ref: "develop", fetch_depth: "0", submodules: "true" },
        })
        run("cargo build")
      }
    })
  }
})
"#,
        )
        .build();

    let output = p.ghat(&["generate", "--no-check"]).run();
    snapshot!(p.generate_snapshot(&output));
}

#[test]
fn uses_name_and_if() {
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
        uses("actions/checkout", {
          name: "Checkout code",
          if: "github.event_name == 'push'",
          continue_on_error: true,
          timeout_minutes: 5,
        })
        run("cargo build")
      }
    })
  }
})
"#,
        )
        .build();

    let output = p.ghat(&["generate", "--no-check"]).run();
    snapshot!(p.generate_snapshot(&output));
}

#[test]
fn step_env_with_context() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps(ctx) {
        run("echo building", {
          env: {
            SHA: `${ctx.github.sha}`,
            RUNNER_OS: `${ctx.runner.os}`,
          },
        })
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
fn multiple_run_steps_with_options() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() {
        run("echo step 1", { name: "First step" })
        run("cargo check", {
          name: "Check",
          working_directory: "./crates/core",
          env: { RUST_BACKTRACE: "1" },
        })
        run("cargo test", {
          name: "Test",
          shell: "bash",
          timeout_minutes: 15,
          continue_on_error: true,
          if: "success()",
        })
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
