mod support;

use support::TestProject;

#[test]
fn no_tsconfig() {
    let p = TestProject::new().build();
    let output = p.ghat(&["check"]).run();
    snapshot!(output);
}

#[test]
fn ok_empty() {
    let p = TestProject::new().init().build();
    let output = p.ghat(&["check"]).run();
    snapshot!(output);
}

#[test]
fn ok_with_workflow() {
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

    let output = p.ghat(&["check"]).run();
    snapshot!(output);
}

#[test]
fn type_error() {
    let p = TestProject::new()
        .init()
        .file(".github/ghat/workflows/bad.ts", "const x: string = 42;\n")
        .build();

    let output = p.ghat(&["check"]).run();
    assert_ne!(output.exit_code, 0);
    snapshot!(output);
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

    let output = p.ghat(&["check"]).run();
    assert_ne!(output.exit_code, 0);
    snapshot!(output);
}
