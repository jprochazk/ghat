# ghat

Define GitHub Actions workflows in TypeScript. Get type-checked inputs/outputs, autocompletion, and reproducible action pinning.

## Quick start

```bash
# Install
cargo install --git https://github.com/jprochazk/ghat.git

# Set up a project
cd my-repo
ghat init

# Add actions you use
ghat add actions/checkout Swatinem/rust-cache

# Write a workflow
cat > .github/ghat/workflows/ci.ts << 'EOF'
workflow("CI", {
  on: triggers({ push: ["main"], pull_request: ["main"] }),
  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",
      steps() {
        uses("actions/checkout")
        run("cargo build --release")
        run("cargo test")
      }
    })
  }
})
EOF

# Generate YAML
ghat generate
```

This creates `.github/workflows/generated_ci.yaml`.

## Why

GitHub Actions workflows are YAML files with no type safety. Typos in input names, missing required fields, and version drift across actions are common sources of CI failures that only surface at runtime.

ghat lets you write workflows in TypeScript instead. Actions are pinned to commit SHAs in a lockfile, and their inputs/outputs are type-checked. If you misspell an input or pass the wrong type, you get an error before the workflow ever runs.

## Project structure

After `ghat init`, your repo looks like this:

```
.github/ghat/
  workflows/     # Your workflow definitions (.ts)
  types/         # Baseline type definitions
  actions/       # Generated action types
  tsconfig.json
  ghat.lock
```

Files prefixed with `_` (e.g. `_utils.ts`) are not evaluated as workflows, but can be imported by other workflow files.

## API

### `workflow(name, definition)`

Define a workflow. The `definition` includes triggers, permissions, env, etc., and a `jobs` callback that receives a context object.

```typescript
workflow("Deploy", {
  on: triggers({
    push: ["main"],
    workflow_dispatch: {
      inputs: {
        environment: input("choice", {
          options: ["staging", "production"] as const,
          required: true,
        }),
      },
    },
  }),
  jobs(ctx) {
    // ...
  }
})
```

### `ctx.job(name, definition)`

Define a job within a workflow. Returns a `JobRef` that can be passed to `needs` in other jobs.

```typescript
jobs(ctx) {
  const build = ctx.job("Build", {
    runs_on: "ubuntu-latest",
    steps() {
      run("cargo build")
      return { version: "1.0.0" }
    }
  })

  ctx.job("Deploy", {
    runs_on: "ubuntu-latest",
    needs: [build],
    steps(ctx) {
      // outputs are typed based on the return value of the steps callback
      run(`deploy ${ctx.needs.build.outputs.version}`)
    }
  })
}
```

### `run(script, options?)`

Add a shell script step.

```typescript
run("echo hello") // defaults to "shell: bash --noprofile --norc -euo pipefail {0}"
run("cargo test", { shell: "bash" })
```

### `uses(action, options?)`

Use a GitHub Action. The action must first be added to the lockfile with `ghat add`. Inputs and outputs are typed based on the action's `action.yml` manifest.

```typescript
const checkout = uses("actions/checkout", {
  with: { fetch_depth: 0 }
})
const ref: string = checkout.outputs.ref
```

Action input/output names with hyphens are converted to snake_case in TypeScript (`fetch-depth` becomes `fetch_depth`). They are mapped back to their original names in the generated YAML.

### `input(type, options?)`

Define a `workflow_dispatch` input. Types: `"string"`, `"number"`, `"boolean"`, `"choice"`.

```typescript
on: triggers({
  workflow_dispatch: {
    inputs: {
      name: input("string", { required: true }),
      count: input("number", { default: 1 }),
      dry_run: input("boolean"),
      env: input("choice", {
        options: ["staging", "production"] as const,
        required: true,
      }),
    },
  },
}),
```

Inputs are accessible in the `steps` callback via `ctx.inputs`, fully typed based on the trigger definition.

### `matrix(definition)`

Define a build matrix for a job.

```typescript
ctx.job("Test", {
  runs_on: "ubuntu-latest",
  strategy: {
    matrix: matrix({
      os: ["ubuntu-latest", "macos-latest"],
      node: [18, 20],
    }),
  },
  steps(ctx) {
    run(`echo ${ctx.matrix.os} node${ctx.matrix.node}`)
  }
})
```

## Commands

### `ghat init`

Create the `.github/ghat/` directory structure and type definitions.

### `ghat add <actions...>`

Add actions to the lockfile, pinned to a release's commit SHA. Generates typed definitions for each action's inputs and outputs.

```bash
ghat add actions/checkout                # latest release
ghat add Swatinem/rust-cache@v2          # latest v2.x.x
ghat add taiki-e/install-action@v2.44.3  # exact version
```

### `ghat rm <actions...>`

Remove actions from the lockfile.

### `ghat update [actions...]`

Update actions to their latest compatible version (within the same major version). Updates all actions if none are specified.

Use `--breaking` to allow major version updates.

### `ghat check`

Type-check workflow definitions and evaluate them without writing files.

### `ghat generate`

Type-check and generate YAML workflow files from definitions. Output goes to `.github/workflows/generated_<name>.yaml`.

Use `--no-check` to skip type-checking.

## Lockfile

`ghat.lock` pins each action to a specific commit SHA:

```
actions/checkout v4.2.2 11bd71901bbe5b1630ceea73d27597364c9af683
Swatinem/rust-cache v2.7.8 779680da715d629ac1d338a641029a2f4372abb5
```

This ensures reproducible builds. The SHA is used directly in the generated YAML instead of a mutable tag.
