mod support;

use support::TestProject;

#[test]
fn matrix_with_include() {
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
        include: [
          { os: "windows-latest", node: "20" },
        ],
      }),
      steps(ctx) {
        run(`echo ${ctx.matrix.os} node ${ctx.matrix.node}`)
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
fn matrix_with_strategy_options() {
    let p = TestProject::new()
        .init()
        .file(
            ".github/ghat/workflows/ci.ts",
            r#"workflow("CI", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      strategy: {
        matrix: matrix({ os: ["ubuntu-latest", "macos-latest"] }),
        fail_fast: false,
        max_parallel: 2,
      },
      steps(ctx) {
        run(`echo ${ctx.matrix.os}`)
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
fn matrix_nested_objects() {
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
        config: [
          { version: 14, env: "staging" },
          { version: 20, env: "production" },
        ],
      }),
      steps(ctx) {
        run(`echo ${ctx.matrix.config.version} ${ctx.matrix.config.env}`)
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
fn matrix_single_value() {
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
        run(`echo ${ctx.matrix.os}`)
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
fn matrix_and_needs_combined() {
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
        run("cargo build")
        return { artifact: "build.tar.gz" }
      }
    })

    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      needs: [build],
      strategy: matrix({
        target: ["x86_64", "aarch64"],
      }),
      steps(ctx) {
        run(`echo testing ${ctx.matrix.target} with ${ctx.needs.build.outputs.artifact}`)
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
