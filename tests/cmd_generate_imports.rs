mod support;

use support::TestProject;

#[test]
fn import_shared_triggers() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/_triggers.ts",
            r#"export const ci_triggers = triggers({
  push: ["main"],
  pull_request: { branches: ["main"] },
})
"#,
        )
        .file(
            ".github/ghat/workflows/build.ts",
            r#"import { ci_triggers } from "./_triggers.ts"

workflow("Build", {
  on: ci_triggers,
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() { run("cargo build") }
    })
  }
})
"#,
        )
        .file(
            ".github/ghat/workflows/test.ts",
            r#"import { ci_triggers } from "./_triggers.ts"

workflow("Test", {
  on: ci_triggers,
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
    snapshot!(p.generate_snapshot(&output));
}

#[test]
fn import_shared_job_factory() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/_jobs.ts",
            r#"export function addCIJobs(ctx: any) {
  ctx.job("Lint", {
    runs_on: "ubuntu-latest",
    steps() { run("cargo clippy") }
  })
  ctx.job("Format", {
    runs_on: "ubuntu-latest",
    steps() { run("cargo fmt --check") }
  })
}
"#,
        )
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"import { addCIJobs } from "./_jobs.ts"

workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    addCIJobs(ctx)
  }
})
"#,
        )
        .build();

    let output = p.ghat(&["generate"]).run();
    snapshot!(p.generate_snapshot(&output));
}

#[test]
fn import_chain() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/_constants.ts",
            r#"export const RUNNER = "ubuntu-latest"
export const MAIN_BRANCH = "main"
"#,
        )
        .file(
            ".github/ghat/workflows/_helpers.ts",
            r#"import { RUNNER } from "./_constants.ts"

export function buildJob(ctx: any) {
  ctx.job("Build", {
    runs_on: RUNNER,
    steps() { run("cargo build") }
  })
}
"#,
        )
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"import { MAIN_BRANCH } from "./_constants.ts"
import { buildJob } from "./_helpers.ts"

workflow("CI", {
  on: triggers({ push: [MAIN_BRANCH] }),
  jobs(ctx) {
    buildJob(ctx)
  }
})
"#,
        )
        .build();

    let output = p.ghat(&["generate"]).run();
    snapshot!(p.generate_snapshot(&output));
}

/// A reusable workflow defined in a shared module, imported by two separate
/// workflow files. The module should only be evaluated once (module caching).
#[test]
fn reusable_workflow_imported_by_two_files() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/_shared.ts",
            r#"export const build = workflow("Shared Build", {
  on: triggers({
    workflow_call: {
      inputs: {
        target: input("string", { required: true }),
      },
      outputs: {
        artifact: { description: "Build artifact" },
      },
    },
  }),
  jobs(ctx) {
    const b = ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() {
        run(`cargo build --target ${ctx.inputs.target}`)
        return { artifact: "build.tar.gz" }
      }
    })
    return { artifact: b.outputs.artifact }
  }
})
"#,
        )
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"import { build } from "./_shared.ts"

workflow("CI", {
  on: triggers({ pull_request: ["main"] }),
  jobs(ctx) {
    const b = ctx.uses("Build", build, {
      with: { target: "x86_64-unknown-linux-gnu" },
    })
    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      needs: [b],
      steps(ctx) {
        run(`echo testing ${ctx.needs.build.outputs.artifact}`)
      }
    })
  }
})
"#,
        )
        .file(
            ".github/ghat/workflows/release.ts",
            r#"import { build } from "./_shared.ts"

workflow("Release", {
  on: triggers({ push: { tags: ["v*"] } }),
  jobs(ctx) {
    const b = ctx.uses("Build", build, {
      with: { target: "x86_64-unknown-linux-gnu" },
    })
    ctx.job("Publish", {
      runs_on: "ubuntu-latest",
      needs: [b],
      steps(ctx) {
        run(`echo publishing ${ctx.needs.build.outputs.artifact}`)
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
