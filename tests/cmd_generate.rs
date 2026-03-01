mod support;

use support::TestProject;

#[test]
fn no_workflows_dir() {
    let p = TestProject::new().build();
    let output = p.ghat(&["generate"]).run();
    snapshot!(output);
}

#[test]
fn empty_workflows_dir() {
    let p = TestProject::new().init().build();
    let before = p.snapshot_full();

    let output = p.ghat(&["generate"]).run();
    assert_eq!(output.exit_code, 0);

    let after = p.snapshot_full();
    snapshot!("diff", before.diff(&after));
}

#[test]
fn simple_workflow() {
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
        run("echo hello")
      }
    })
  }
})
"#,
        )
        .build();

    let output = p.ghat(&["generate"]).run();
    snapshot!("output", output);
    snapshot!("generated", p.snapshot_glob(".github/workflows/**/*"));
}

#[test]
fn skips_underscore_files() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/_helpers.ts",
            r#"export const foo = "bar""#,
        )
        .build();

    let before = p.snapshot_full();
    let output = p.ghat(&["generate"]).run();
    assert_eq!(output.exit_code, 0);

    let after = p.snapshot_full();
    snapshot!("diff", before.diff(&after));
}

#[test]
fn multi_file_import() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/_utils.ts",
            r#"export function cargo_test(): void {
  run("cargo test --all-features")
}
"#,
        )
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"import { cargo_test } from "./_utils.ts"

workflow("CI", {
  on: triggers({ push: ["main"], pull_request: ["main"] }),
  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      steps() {
        cargo_test()
      }
    })
  }
})
"#,
        )
        .build();

    let output = p.ghat(&["generate"]).run();
    snapshot!("output", output);
    snapshot!("generated", p.snapshot_glob(".github/workflows/**/*"));
}

#[test]
fn pins_actions_to_lockfile_sha() {
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
        run("echo hello")
      }
    })
  }
})
"#,
        )
        .build();

    let output = p.ghat(&["generate", "--no-check"]).run();
    snapshot!("output", output);
    snapshot!("generated", p.snapshot_glob(".github/workflows/**/*"));
}

#[test]
fn unlocked_action_fails() {
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
fn dedents_multiline_run() {
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
        run(`
          echo "step 1"
          if true; then
            echo "step 2"
          fi
        `)
      }
    })
  }
})
"#,
        )
        .build();

    let output = p.ghat(&["generate"]).run();
    snapshot!("output", output);
    snapshot!("generated", p.snapshot_glob(".github/workflows/**/*"));
}

#[test]
fn syntax_error() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/bad.ts",
            r#"workflow("Bad", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Broken", {
      runs_on: "ubuntu-latest",
      steps() {
        this is not valid javascript at all !!!
      }
    })
  }
})
"#,
        )
        .build();

    let output = p.ghat(&["generate"]).run();
    snapshot!(output);
}
