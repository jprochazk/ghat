mod support;

use support::TestProject;

#[test]
fn job_if_rejects_matrix() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/bad.ts",
            r#"workflow("Bad", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      if: (ctx) => `${ctx.matrix.os}`,
      steps() { run("echo test") }
    })
  }
})
"#,
        )
        .build();
    let output = p.ghat(&["generate", "--no-check"]).run();
    assert_ne!(output.exit_code, 0);
    snapshot!(output);
}

#[test]
fn job_if_rejects_secrets() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/bad.ts",
            r#"workflow("Bad", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      if: (ctx) => `${ctx.secrets.TOKEN}`,
      steps() { run("echo test") }
    })
  }
})
"#,
        )
        .build();
    let output = p.ghat(&["generate", "--no-check"]).run();
    assert_ne!(output.exit_code, 0);
    snapshot!(output);
}

#[test]
fn job_strategy_rejects_secrets() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/bad.ts",
            r#"workflow("Bad", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      strategy: (ctx) => matrix({ key: [`${ctx.secrets.TOKEN}`] }),
      steps() { run("echo test") }
    })
  }
})
"#,
        )
        .build();
    let output = p.ghat(&["generate", "--no-check"]).run();
    assert_ne!(output.exit_code, 0);
    snapshot!(output);
}

#[test]
fn job_runs_on_rejects_secrets() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/bad.ts",
            r#"workflow("Bad", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Test", {
      runs_on: (ctx) => `${ctx.secrets.TOKEN}`,
      steps() { run("echo test") }
    })
  }
})
"#,
        )
        .build();
    let output = p.ghat(&["generate", "--no-check"]).run();
    assert_ne!(output.exit_code, 0);
    snapshot!(output);
}

#[test]
fn steps_can_use_all_contexts() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      strategy: matrix({ os: ["ubuntu-latest"] }),
      steps(ctx) {
        run(`github=${ctx.github.sha}`)
        run(`runner=${ctx.runner.os}`)
        run(`env=${ctx.env.PATH}`)
        run(`vars=${ctx.vars.MY_VAR}`)
        run(`secrets=${ctx.secrets.TOKEN}`)
        run(`matrix=${ctx.matrix.os}`)
        run(`job=${ctx.job.status}`)
        run(`strategy=${ctx.strategy.fail_fast}`)
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
fn workflow_env_can_use_secrets() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  env: (ctx) => ({
    TOKEN: `${ctx.secrets.GITHUB_TOKEN}`,
    REPO: `${ctx.github.repository}`,
  }),
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
