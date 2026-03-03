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

#[test]
fn matrix_strategy() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      strategy: matrix({
        os: ["ubuntu-latest", "macos-latest"],
        node: ["18", "20"],
      }),
      steps(ctx) {
        run(`echo ${ctx.matrix.os} ${ctx.matrix.node}`)
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
fn workflow_dispatch_inputs() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/deploy.ts",
            r#"workflow("Deploy", {
  on: triggers({
    workflow_dispatch: {
      inputs: {
        environment: input("choice", {
          description: "Target environment",
          required: true,
          options: ["staging", "production"],
          default: "staging",
        }),
        dry_run: input("boolean", {
          description: "Perform a dry run",
          default: false,
        }),
      }
    }
  }),
  jobs(ctx) {
    ctx.job("Deploy", {
      runs_on: "ubuntu-latest",
      steps() {
        run(`echo ${ctx.inputs.environment} ${ctx.inputs.dry_run}`)
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

/// Tests: job needs/outputs, job name normalization, job if/env/timeout,
/// step context proxies (github, runner, secrets), step output proxies,
/// run step options (name, shell, working_directory, env, if, timeout, continue_on_error),
/// and workflow-level context functions (run_name, concurrency, env).
#[test]
fn job_and_step_features() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/ghat.lock",
            "actions/checkout tag:v4.2.2 11bd71901bbe5b1630ceea73d27597364c9af683\n",
        )
        .file(
            ".github/ghat/workflows/features.ts",
            r#"
// --- job needs, outputs, name normalization, if, env, timeout ---
workflow("Needs And Outputs", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    const build = ctx.job("Build Artifacts", {
      runs_on: "ubuntu-latest",
      steps() {
        run("cargo build --release")
        return { artifact: "build.tar.gz", version: "1.0.0" }
      }
    })

    ctx.job("Deploy To Production", {
      runs_on: "ubuntu-latest",
      needs: [build],
      if: (ctx) => `\${{ needs.build_artifacts.result == 'success' }}`,
      env: (ctx) => ({ VERSION: `${ctx.needs.build_artifacts.outputs.version}` }),
      timeout_minutes: 30,
      steps(ctx) {
        run(`echo deploying ${ctx.needs.build_artifacts.outputs.artifact}`)
      }
    })
  }
})

// --- step context proxies (github, runner, secrets) ---
workflow("Context Proxies", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      steps(ctx) {
        run(`echo sha=${ctx.github.sha} runner=${ctx.runner.os} secret=${ctx.secrets.TOKEN}`)
      }
    })
  }
})

// --- step output proxy (uses return -> steps.X.outputs.Y) ---
workflow("Step Outputs", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      steps() {
        const co = uses("actions/checkout")
        run(`echo checked out ${co.outputs.ref}`)
      }
    })
  }
})

// --- run step options ---
workflow("Run Options", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      steps() {
        run("echo hello")
        run("cargo test", {
          name: "Run tests",
          shell: "bash",
          working_directory: "./crates",
          env: { RUST_LOG: "debug" },
          timeout_minutes: 30,
          continue_on_error: true,
          if: "always()",
        })
      }
    })
  }
})

// --- workflow-level context functions (run_name, concurrency, env) ---
workflow("Deploy Context", {
  on: triggers({
    workflow_dispatch: {
      inputs: {
        env: input("choice", {
          options: ["staging", "production"],
          required: true,
        }),
      }
    }
  }),
  run_name: (ctx) => `Deploy to ${ctx.inputs.env}`,
  concurrency: (ctx) => ({ group: `deploy-${ctx.github.ref}`, cancel_in_progress: true }),
  env: (ctx) => ({ TOKEN: `${ctx.secrets.GITHUB_TOKEN}` }),
  jobs(ctx) {
    ctx.job("Deploy", {
      runs_on: "ubuntu-latest",
      steps() { run("echo deploy") }
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
fn run_name_rejects_secrets() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/bad.ts",
            r#"workflow("Bad", {
  on: triggers({ push: ["main"] }),
  run_name: (ctx) => `${ctx.secrets.TOKEN}`,
  jobs(ctx) {}
})
"#,
        )
        .build();
    let output = p.ghat(&["generate", "--no-check"]).run();
    assert_ne!(output.exit_code, 0);
    snapshot!(output);
}

#[test]
fn run_name_rejects_strategy() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/bad.ts",
            r#"workflow("Bad", {
  on: triggers({ push: ["main"] }),
  run_name: (ctx) => `${ctx.strategy.fail_fast}`,
  jobs(ctx) {}
})
"#,
        )
        .build();
    let output = p.ghat(&["generate", "--no-check"]).run();
    assert_ne!(output.exit_code, 0);
    snapshot!(output);
}

#[test]
fn concurrency_rejects_needs() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/bad.ts",
            r#"workflow("Bad", {
  on: triggers({ push: ["main"] }),
  concurrency: (ctx) => ({ group: `${ctx.needs.build}` }),
  jobs(ctx) {}
})
"#,
        )
        .build();
    let output = p.ghat(&["generate", "--no-check"]).run();
    assert_ne!(output.exit_code, 0);
    snapshot!(output);
}

#[test]
fn env_rejects_matrix() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/bad.ts",
            r#"workflow("Bad", {
  on: triggers({ push: ["main"] }),
  env: (ctx) => ({ FOO: `${ctx.matrix.os}` }),
  jobs(ctx) {}
})
"#,
        )
        .build();
    let output = p.ghat(&["generate", "--no-check"]).run();
    assert_ne!(output.exit_code, 0);
    snapshot!(output);
}

#[test]
fn unknown_context_typo() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/bad.ts",
            r#"workflow("Bad", {
  on: triggers({ push: ["main"] }),
  run_name: (ctx) => `${ctx.secerts.TOKEN}`,
  jobs(ctx) {}
})
"#,
        )
        .build();
    let output = p.ghat(&["generate", "--no-check"]).run();
    assert_ne!(output.exit_code, 0);
    snapshot!(output);
}

#[test]
fn subdirectory_and_parent_imports() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/shared/helpers.ts",
            r#"export function greeting(): string { return "hello"; }"#,
        )
        .file(
            ".github/ghat/workflows/lib/steps.ts",
            r#"export function checkStep() { return "cargo check"; }"#,
        )
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"import { greeting } from "../shared/helpers.ts"
import { checkStep } from "./lib/steps.ts"

workflow("Subdir Import", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      steps() {
        run(checkStep())
        run(greeting())
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
fn multiple_workflows_one_file() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/multi.ts",
            r#"workflow("Build", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() { run("cargo build") }
    })
  }
})

workflow("Test", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      steps() { run("cargo test") }
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
fn missing_import() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/bad.ts",
            r#"import { foo } from "./_nonexistent.ts"
foo()
"#,
        )
        .build();

    let output = p.ghat(&["generate"]).run();
    assert_ne!(output.exit_code, 0);
    snapshot!(output);
}
