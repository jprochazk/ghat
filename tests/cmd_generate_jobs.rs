mod support;

use support::TestProject;

#[test]
fn job_runs_on_labels_array() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: ["self-hosted", "linux", "x64"],
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
fn workflow_defaults() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  defaults: { run: { shell: "pwsh" } },
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "windows-latest",
      steps() { run("Write-Host hello") }
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
fn job_env_literal() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      env: { RUST_LOG: "debug", CI: "true" },
      steps() { run("cargo test") }
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
fn job_if_literal() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Deploy", {
      runs_on: "ubuntu-latest",
      if: "github.ref == 'refs/heads/main'",
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
fn many_jobs_chain() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/pipeline.ts",
            r#"workflow("Pipeline", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    const lint = ctx.job("Lint", {
      runs_on: "ubuntu-latest",
      steps() { run("cargo clippy") }
    })
    const build = ctx.job("Build", {
      runs_on: "ubuntu-latest",
      needs: [lint],
      steps() {
        run("cargo build --release")
        return { artifact: "target/release/app" }
      }
    })
    const test = ctx.job("Test", {
      runs_on: "ubuntu-latest",
      needs: [build],
      steps(ctx) {
        run(`echo testing ${ctx.needs.build.outputs.artifact}`)
      }
    })
    ctx.job("Deploy", {
      runs_on: "ubuntu-latest",
      needs: [test],
      steps() { run("echo deploying") }
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
fn job_outputs_consumed_by_multiple() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/fan.ts",
            r#"workflow("Fan Out", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    const build = ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() {
        run("cargo build")
        return { version: "1.0.0", sha: "abc123" }
      }
    })
    ctx.job("Deploy Staging", {
      runs_on: "ubuntu-latest",
      needs: [build],
      steps(ctx) {
        run(`echo deploying ${ctx.needs.build.outputs.version} to staging`)
      }
    })
    ctx.job("Deploy Production", {
      runs_on: "ubuntu-latest",
      needs: [build],
      steps(ctx) {
        run(`echo deploying ${ctx.needs.build.outputs.sha} to prod`)
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
