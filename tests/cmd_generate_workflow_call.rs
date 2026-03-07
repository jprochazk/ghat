mod support;

use support::TestProject;

#[test]
fn workflow_call_with_string_and_number_inputs() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/build.ts",
            r#"const build = workflow("Build", {
  on: triggers({
    workflow_call: {
      inputs: {
        version: input("string", { required: true }),
        retries: input("number", { default: 3 }),
      },
    },
  }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() {
        run(`echo version=${ctx.inputs.version} retries=${ctx.inputs.retries}`)
      }
    })
  }
})

workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.uses("Run Build", build, {
      with: { version: "1.0.0", retries: 5 },
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
fn workflow_call_explicit_secrets() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/deploy.ts",
            r#"const deploy = workflow("Deploy", {
  on: triggers({
    workflow_call: {
      secrets: {
        deploy_token: { description: "Deployment token", required: true },
        api_key: { description: "API key" },
      },
    },
  }),
  jobs(ctx) {
    ctx.job("Deploy", {
      runs_on: "ubuntu-latest",
      steps() { run("echo deploying") }
    })
  }
})

workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.uses("Run Deploy", deploy, {
      secrets: {
        deploy_token: "${{ secrets.DEPLOY_TOKEN }}",
        api_key: "${{ secrets.API_KEY }}",
      },
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
fn workflow_call_no_inputs() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/build.ts",
            r#"const build = workflow("Build", {
  on: triggers({ workflow_call: {} }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() { run("cargo build") }
    })
  }
})

workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.uses("Run Build", build)
  }
})
"#,
        )
        .build();

    let output = p.ghat(&["generate"]).run();
    snapshot!(p.generate_snapshot(&output));
}

#[test]
fn multiple_workflow_call_consumers() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/_shared_build.ts",
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
            r#"import { build } from "./_shared_build.ts"

workflow("CI", {
  on: triggers({ pull_request: ["main"] }),
  jobs(ctx) {
    ctx.uses("Build Linux", build, {
      with: { target: "x86_64-unknown-linux-gnu" },
    })
    ctx.uses("Build Mac", build, {
      with: { target: "x86_64-apple-darwin" },
    })
  }
})
"#,
        )
        .build();

    let output = p.ghat(&["generate"]).run();
    snapshot!(p.generate_snapshot(&output));
}
