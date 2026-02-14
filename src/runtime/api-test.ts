// ---- WorkflowJobsContext: inputs are typed and available ----

workflow("Input test", {
  on: triggers({
    workflow_dispatch: {
      inputs: {
        env: input("choice", {
          options: ["staging", "production"] as const,
          required: true,
        }),
        dry_run: input("boolean"),
      },
    },
    push: ["main"],
  }),

  jobs(ctx) {
    // inputs are available on the jobs context
    const _env: string = ctx.inputs.env;

    // required input is not optional
    // @ts-expect-error - env is required, should not accept undefined
    const _envUndef: undefined = ctx.inputs.env;

    // optional input may be undefined
    const _dry: "true" | "false" | undefined = ctx.inputs.dry_run;

    // choice input is narrowed to its options
    // @ts-expect-error - env is "staging" | "production", not arbitrary string
    const _envWrong: "other" = ctx.inputs.env;

    // github context is available
    const _repo: string = ctx.github.repository;

    // vars context is available
    const _v: string = ctx.vars["SOME_VAR"];

    // job returns a JobRef
    const build = ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps(ctx) { },
    });

    // JobRef id is normalized
    const _id: "build" = build.id;
    // @ts-expect-error - id is "build", not arbitrary string
    const _idWrong: "Build" = build.id;
  },
});

// ---- Workflow without inputs: inputs is empty ----

workflow("No inputs", {
  on: triggers({
    push: ["main"],
  }),

  jobs(ctx) {
    // @ts-expect-error - no workflow_dispatch, so no inputs
    ctx.inputs.anything;
  },
});

// ---- StepsContext: contexts are propagated into steps ----

workflow("Steps context test", {
  on: triggers({
    workflow_dispatch: {
      inputs: {
        target: input("string", { required: true }),
      },
    },
    push: ["main"],
  }),

  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",

      steps(ctx) {
        // github context available in steps
        const _sha: string = ctx.github.sha;
        const _repo: string = ctx.github.repository;

        // inputs propagated from workflow into steps
        const _target: string = ctx.inputs.target;

        // vars and secrets available
        const _v: string = ctx.vars["MY_VAR"];
        const _s: string = ctx.secrets["MY_SECRET"];

        // strategy context available
        const _idx: number = ctx.strategy.job_index;
        const _total: number = ctx.strategy.job_total;
        const _ff: boolean = ctx.strategy.fail_fast;

        // runner context available with typed fields
        const _os: "Linux" | "Windows" | "macOS" = ctx.runner.os;
        const _arch: "X86" | "X64" | "ARM" | "ARM64" = ctx.runner.arch;
        const _temp: string = ctx.runner.temp;
        const _runnerEnv: "github-hosted" | "self-hosted" = ctx.runner.environment;

        // job context available
        const _status: "success" | "failure" | "cancelled" = ctx.job.status;
        const _containerId: string = ctx.job.container.id;

        // env context available
        const _e: string = ctx.env["MY_ENV"];
      },
    });
  },
});

// ---- StepRef: outputs are typed ----

{
  const ref: StepRef<{ artifact_id: string }> = { outputs: { artifact_id: "123" } };
  const _id: string = ref.outputs.artifact_id;

  // @ts-expect-error - nonexistent output key
  ref.outputs.nonexistent;

  const emptyRef: StepRef = { outputs: {} };
  // @ts-expect-error - no outputs defined
  emptyRef.outputs.anything;
}

// ---- matrix(): type inference ----

{
  // regular keys: infers { foo: string, test: string }
  const m1 = matrix({ foo: ["a", "b"], test: ["x", "y"] });
  const _foo: string = m1.foo;
  const _test: string = m1.test;
  // @ts-expect-error - nonexistent key
  m1.nonexistent;

  // include: infers { color: string, foo: string, baz: string }
  const m2 = matrix({
    include: [{ color: "blue" }, { color: "red" }],
    foo: ["bar"],
    baz: ["qux"],
  });
  const _color: string = m2.color;
  const _foo2: string = m2.foo;
  const _baz: string = m2.baz;
  // @ts-expect-error - nonexistent key
  m2.nonexistent;

  // include-only: infers { key: string, foo: string }
  const m3 = matrix({
    include: [{ key: "value", foo: "bar" }, { key: "other", foo: "baz" }],
  });
  const _key: string = m3.key;
  const _foo3: string = m3.foo;
  // @ts-expect-error - nonexistent key
  m3.nonexistent;

  // object values: infers { node: { version: string, env: string } }
  const m4 = matrix({
    node: [{ version: 14, env: "staging" }, { version: 20, env: "production" }],
  });
  const _version: string = m4.node.version;
  const _nodeEnv: string = m4.node.env;
  // @ts-expect-error - nonexistent nested key
  m4.node.nonexistent;

  // mixed: string keys + object keys
  const m5 = matrix({
    os: ["ubuntu", "windows"],
    config: [{ opt: "fast", debug: false }],
  });
  const _os: string = m5.os;
  const _opt: string = m5.config.opt;
  const _debug: string = m5.config.debug;

  // deeply nested objects
  const m6 = matrix({
    target: [{ platform: "linux", opts: { arch: "x64", features: { sse: true } } }],
  });
  const _arch: string = m6.target.opts.arch;
  const _sse: string = m6.target.opts.features.sse;
  // @ts-expect-error - nonexistent deep key
  m6.target.opts.features.avx;

  // include with nested objects
  const m7 = matrix({
    include: [
      { target: { os: "linux", arch: "x64" }, features: { sse: true } },
      { target: { os: "macos", arch: "arm64" }, features: { sse: false } },
    ],
  });
  const _os2: string = m7.target.os;
  const _arch2: string = m7.target.arch;
  const _sse2: string = m7.features.sse;
  // @ts-expect-error - nonexistent nested include key
  m7.target.nonexistent;
}

// ---- matrix propagates into steps context ----

workflow("Matrix test", {
  on: triggers({ push: ["main"] }),

  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      strategy: { matrix: matrix({ os: ["ubuntu-latest", "windows-latest"], node: ["18", "20"] }), fail_fast: true, max_parallel: 10 },

      steps(ctx) {
        const _os: string = ctx.matrix.os;
        const _node: string = ctx.matrix.node;
        // @ts-expect-error - nonexistent matrix key
        ctx.matrix.nonexistent;
      },
    });

    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      strategy: matrix({ os: ["ubuntu-latest", "windows-latest"], node: ["18", "20"] }),

      steps(ctx) {
        const _os: string = ctx.matrix.os;
        const _node: string = ctx.matrix.node;
        // @ts-expect-error - nonexistent matrix key
        ctx.matrix.nonexistent;
      },
    });
  },
});

// ---- NormalizeId: dashes and spaces become underscores ----

{
  const _dash: "my_job" = "" as NormalizeId<"My-Job">;
  const _space: "my_job" = "" as NormalizeId<"My Job">;
  const _mixed: "build_and_test" = "" as NormalizeId<"Build and Test">;
  // @ts-expect-error - dashes become underscores, not kept
  const _wrong: "my-job" = "" as NormalizeId<"My-Job">;
}

// ---- needs: job outputs propagate to dependent jobs ----

workflow("Needs test", {
  on: triggers({ push: ["main"] }),

  jobs(ctx) {
    const build = ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps(ctx) {
        return { artifact_id: "some-id", version: "1.0.0" };
      },
    });

    // build.id is normalized
    const _id: "build" = build.id;

    ctx.job("Deploy", {
      runs_on: "ubuntu-latest",
      needs: [build],

      steps(ctx) {
        // needs outputs are typed
        const _art: string = ctx.needs.build.outputs.artifact_id;
        const _ver: string = ctx.needs.build.outputs.version;

        // @ts-expect-error - nonexistent output key
        ctx.needs.build.outputs.nonexistent;

        // @ts-expect-error - nonexistent job in needs
        ctx.needs.nonexistent;
      },
    });
  },
});

// ---- needs: multiple dependencies ----

workflow("Multi needs", {
  on: triggers({ push: ["main"] }),

  jobs(ctx) {
    const lint = ctx.job("Lint", {
      runs_on: "ubuntu-latest",
      steps(ctx) {
        return { passed: "true" };
      },
    });

    const build = ctx.job("Build Artifacts", {
      runs_on: "ubuntu-latest",
      steps(ctx) {
        return { path: "/tmp/build" };
      },
    });

    ctx.job("Deploy", {
      runs_on: "ubuntu-latest",
      needs: [lint, build],

      steps(ctx) {
        const _passed: string = ctx.needs.lint.outputs.passed;
        const _path: string = ctx.needs.build_artifacts.outputs.path;

        // @ts-expect-error - wrong job id (not normalized)
        ctx.needs.build_artifact;
      },
    });
  },
});

// ---- ValueOrFactory: job fields accept plain values or callbacks ----

workflow("ValueOrFactory test", {
  on: triggers({
    workflow_dispatch: {
      inputs: {
        runner: input("string", { required: true }),
        environment: input("choice", {
          options: ["staging", "production"] as const,
          required: true,
        }),
      },
    },
    push: ["main"],
  }),

  // workflow-level fields
  run_name: (ctx) => `Deploy to ${ctx.inputs.environment}`,
  concurrency: (ctx) => ({ group: `deploy-${ctx.github.ref}`, cancel_in_progress: true }),
  env: (ctx) => ({ DEPLOY_ENV: ctx.inputs.environment, TOKEN: ctx.secrets.GITHUB_TOKEN }),

  jobs(ctx) {
    const build = ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps(ctx) {
        return { artifact: "build.tar.gz" };
      },
    });

    ctx.job("Deploy", {
      needs: [build],

      runs_on: (ctx) => {
        // runs_on callback: github, needs, strategy, matrix, vars, inputs
        const _: string = ctx.github.sha;
        const _2: string = ctx.needs.build.outputs.artifact;
        const _3: number = ctx.strategy.job_index;
        const _4: string = ctx.vars.MY_VAR;
        const _5: string = ctx.inputs.runner;
        return ctx.inputs.runner;
      },

      strategy: (ctx) => {
        // strategy callback: github, needs, vars, inputs
        const _: string = ctx.github.sha;
        const _2: string = ctx.needs.build.outputs.artifact;
        const _3: string = ctx.vars.MY_VAR;
        const _4: string = ctx.inputs.runner;
        // @ts-expect-error - no strategy in strategy callback
        ctx.strategy;
        // @ts-expect-error - no matrix in strategy callback
        ctx.matrix;
        // @ts-expect-error - no secrets in strategy callback
        ctx.secrets;
        return matrix({ target: [ctx.inputs.environment] });
      },

      if: (ctx) => {
        // if callback: github, needs, vars, inputs
        const _: string = ctx.github.sha;
        const _2: string = ctx.needs.build.outputs.artifact;
        const _3: string = ctx.vars.MY_VAR;
        const _4: string = ctx.inputs.runner;
        // @ts-expect-error - no strategy in if callback
        ctx.strategy;
        // @ts-expect-error - no matrix in if callback
        ctx.matrix;
        // @ts-expect-error - no secrets in if callback
        ctx.secrets;
        return "always()";
      },

      env: (ctx) => {
        // env callback: github, needs, strategy, matrix, vars, secrets, inputs
        const _: string = ctx.github.sha;
        const _2: string = ctx.needs.build.outputs.artifact;
        const _3: number = ctx.strategy.job_index;
        const _4: string = ctx.vars.MY_VAR;
        const _5: string = ctx.secrets.DEPLOY_KEY;
        const _6: string = ctx.inputs.runner;
        return { CI: "true" };
      },

      steps(ctx) { },
    });

    // plain values also work
    ctx.job("Lint", {
      runs_on: "ubuntu-latest",
      if: "always()",
      env: { CI: "true" },
      steps(ctx) { },
    });
  },
});

// ---- run: script with options ----

workflow("Run test", {
  on: triggers({ push: ["main"] }),

  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",

      steps(ctx) {
        // basic run
        run("echo hello");

        // run with all options
        run("cargo test", {
          shell: "bash",
          working_directory: "./crates",
          env: { RUST_LOG: "debug" },
          timeout_minutes: 30,
          continue_on_error: true,
          name: "Run tests",
          if: "always()",
        });

        // run returns a StepRef (with empty outputs)
        const step = run("echo done");
        const _outputs: {} = step.outputs;
        // @ts-expect-error - run steps have no typed outputs
        step.outputs.anything;

        // multiline script
        run(`
          echo "line 1"
          echo "line 2"
        `);
      },
    });
  },
});

// ---- uses: per-action overloads ----

workflow("Uses test", {
  on: triggers({ push: ["main"] }),

  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",

      steps(ctx) {
        // basic usage, no inputs
        const co = uses("actions/checkout");

        // outputs are typed
        const _ref: string = co.outputs.ref;
        const _commit: string = co.outputs.commit;
        // @ts-expect-error - nonexistent output
        co.outputs.nonexistent;

        // with typed inputs
        uses("actions/checkout", {
          with: { fetch_depth: 0, submodules: "recursive" },
        });

        // @ts-expect-error - invalid input key
        uses("actions/checkout", { with: { invalid_key: true } });

        // @ts-expect-error - invalid input type
        uses("actions/checkout", { with: { fetch_depth: "not a number" } });

        // @ts-expect-error - unregistered action shows `ghat add` hint
        uses("some/unregistered-action");

        // step options work alongside with
        uses("actions/checkout", {
          with: { ref: "main" },
          env: { GIT_TOKEN: "abc" },
          timeout_minutes: 5,
          continue_on_error: true,
          name: "Checkout code",
          if: "always()",
        });

        // second action: org/foo
        const deploy = uses("org/foo", {
          with: { target: "production" },
        });
        const _url: string = deploy.outputs.url;
        const _deployId: string = deploy.outputs.deploy_id;
        // @ts-expect-error - nonexistent output on org/foo
        deploy.outputs.nonexistent;

        // @ts-expect-error - options required when action has required inputs
        uses("org/foo");

        // @ts-expect-error - with required when action has required inputs
        uses("org/foo", { env: { FOO: "bar" } });

        // @ts-expect-error - target is required within with
        uses("org/foo", { with: { dry_run: true } });

        // optional inputs work
        uses("org/foo", {
          with: { target: "staging", dry_run: true, retries: 3 },
        });

        // @ts-expect-error - wrong type for org/foo input
        uses("org/foo", { with: { target: "prod", retries: "three" } });
      },
    });
  },
});
