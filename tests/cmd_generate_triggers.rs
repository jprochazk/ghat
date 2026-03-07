mod support;

use support::TestProject;

#[test]
fn schedule_trigger() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/nightly.ts",
            r#"workflow("Nightly", {
  on: triggers({ schedule: [{ cron: "0 0 * * *" }] }),
  jobs(ctx) {
    ctx.job("Cleanup", {
      runs_on: "ubuntu-latest",
      steps() { run("echo nightly cleanup") }
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
fn multiple_schedule_triggers() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/crons.ts",
            r#"workflow("Crons", {
  on: triggers({
    schedule: [
      { cron: "0 0 * * *" },
      { cron: "30 12 * * 1-5" },
    ]
  }),
  jobs(ctx) {
    ctx.job("Run", {
      runs_on: "ubuntu-latest",
      steps() { run("echo scheduled") }
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
fn push_with_tags_and_paths() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({
    push: {
      branches: ["main", "release/*"],
      tags: ["v*"],
      paths: ["src/**", "Cargo.toml"],
    }
  }),
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

    let output = p.ghat(&["generate"]).run();
    snapshot!(p.generate_snapshot(&output));
}

#[test]
fn pull_request_with_types() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/pr.ts",
            r#"workflow("PR", {
  on: triggers({
    pull_request: {
      branches: ["main"],
      types: ["opened", "synchronize", "reopened"],
    }
  }),
  jobs(ctx) {
    ctx.job("Check", {
      runs_on: "ubuntu-latest",
      steps() { run("cargo check") }
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
fn pull_request_target_trigger() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/pr_target.ts",
            r#"workflow("PR Target", {
  on: triggers({
    pull_request_target: {
      branches: ["main"],
      types: ["opened", "labeled"],
    }
  }),
  jobs(ctx) {
    ctx.job("Label Check", {
      runs_on: "ubuntu-latest",
      steps() { run("echo checking labels") }
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
fn issue_comment_trigger() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/comment.ts",
            r#"workflow("Comment Bot", {
  on: triggers({
    issue_comment: { types: ["created"] }
  }),
  jobs(ctx) {
    ctx.job("Respond", {
      runs_on: "ubuntu-latest",
      steps() { run("echo comment received") }
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
fn bare_push_trigger() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: {} }),
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

    let output = p.ghat(&["generate"]).run();
    snapshot!(p.generate_snapshot(&output));
}

// This combines push + pull_request + workflow_dispatch (no schedule, since that's broken).
#[test]
fn multiple_triggers_combined() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({
    push: ["main"],
    pull_request: { branches: ["main"] },
    workflow_dispatch: {
      inputs: {
        debug: input("boolean", { default: false }),
      }
    },
  }),
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

    let output = p.ghat(&["generate"]).run();
    snapshot!(p.generate_snapshot(&output));
}
