mod support;

use support::TestProject;

#[test]
fn permissions_write_all() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  permissions: "write-all",
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
    snapshot!(p.generate_snapshot(&output));
}

#[test]
fn permissions_read_all() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  permissions: "read-all",
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
    snapshot!(p.generate_snapshot(&output));
}

#[test]
fn permissions_scoped() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  permissions: {
    contents: "write",
    pull_requests: "read",
    packages: "none",
    id_token: "write",
  },
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
    snapshot!(p.generate_snapshot(&output));
}
