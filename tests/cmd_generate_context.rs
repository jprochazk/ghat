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

#[test]
fn expr_can_interpolate_multiple_context_values() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    const build = ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() {
        return { changed: "true" }
      }
    })

    ctx.job("Deploy", {
      runs_on: "ubuntu-latest",
      needs: [build],
      if: (ctx) => expr`${ctx.github.ref} == 'refs/heads/main' && ${ctx.needs.build.outputs.changed} == 'true'`,
      steps() { run("echo deploy") }
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
fn pull_request_event_fields_work_without_narrowing() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/pr.ts",
            r#"workflow("PR", {
  on: triggers({
    pull_request: { types: ["opened", "synchronize"] }
  }),
  jobs(ctx) {
    ctx.job("Check", {
      runs_on: "ubuntu-latest",
      if: (ctx) => expr`${ctx.github.event.pull_request.head.ref} == 'main'`,
      steps(ctx) {
        run("echo checking", {
          env: {
            PR_REF: `${ctx.github.event.pull_request.head.ref}`,
            PR_NUMBER: `${ctx.github.event.pull_request.number}`,
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
fn pull_request_target_uses_pull_request_payload_shape() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/pr_target.ts",
            r#"workflow("PR Target", {
  on: triggers({
    pull_request_target: { types: ["opened"] }
  }),
  jobs(ctx) {
    ctx.job("Check", {
      runs_on: "ubuntu-latest",
      if: (ctx) => expr`${ctx.github.event.pull_request.head.ref} != ''`,
      steps(ctx) {
        run("echo target", {
          env: {
            HEAD_REF: `${ctx.github.event.pull_request.head.ref}`,
            BASE_REF: `${ctx.github.event.pull_request.base.ref}`,
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
fn merged_event_context_allows_multi_trigger_access_without_narrowing() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/mixed.ts",
            r#"workflow("Mixed", {
  on: triggers({
    pull_request: { types: ["opened"] },
    issue_comment: { types: ["created"] },
  }),
  jobs(ctx) {
    ctx.job("Check", {
      runs_on: "ubuntu-latest",
      if: (ctx) => expr`${ctx.github.event.pull_request.head.ref} != '' && ${ctx.github.event.comment.body} != ''`,
      steps(ctx) {
        run("echo mixed", {
          env: {
            PR_REF: `${ctx.github.event.pull_request.head.ref}`,
            COMMENT_BODY: `${ctx.github.event.comment.body}`,
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
