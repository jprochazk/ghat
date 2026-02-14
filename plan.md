## GHAT

GitHub Actions Templating system and runtime.

Features:

- Workflow definitions are written in JS/TS
- Help maintain a lockfile of commit-pinned actions
- Input/output types and docs generated from `action.yml` files
- Outputs human-readable workflow files

Main goal for this is to improve the ergonomics of authoring workflow files, and allow users of GitHub Actions to retain some of their sanity.

### Example

```
ghat init
```

Initializes project structure:
- `.github/ghat/ghat.lock` - a lockfile
- `.github/ghat/types/*.d.ts` - automatically generated type definitions, and types of builtins
- `.github/ghat/tsconfig.json` - TS config file to pick up `types.d.ts` automatically

`ghat` is itself a JS runtime; it actually executes the workflow files to produce the workflow definitions.
It has the ability to execute both regular JS and also TS (with `isolatedModules` and `isolatedDeclarations`).

Workflow definitions use builder globals (`workflow`, `job`, `run`, `uses`, `step`, `checkout`)
to declare structure imperatively. These globals are only available at definition time; they do not
exist at CI runtime.

Here is an example workflow definition - a realistic `Rust` workflow with caching, including setup code factored out.

Files beginning with `_` are ignored by the runtime and not treated as workflow definitions, though they may still be imported by other files.

```ts
//_rust.ts

export const setup_rust = () => {
  // note: smart defaults, `run` uses `shell: bash --noprofile --norc -euo pipefail {0}`,
  // but this can easily be overridden with `.shell(...)`
  run("rustup install")

  // `step` helper can be used to add `name` to steps, otherwise the name is left blank,
  // and inferred by the gha runner based on `uses` or `runs`.
  step(
    "Setup Rust cache",
    uses("Swatinem/rust-cache"),
  )
}
```

```ts
//ci-rust.ts

import { setup_rust } from "_rust.ts";

// There may be more than one workflow per file.
// The only constraint is that their normalized names are unique.

workflow("Rust CI", {
  on: {
    push: ["main"],
    pull_request: []
  },

  jobs(ctx) {
    job("Lint", {
      runs_on: "ubuntu-latest-large",

      steps() {
        checkout()
        setup_rust()
        run("cargo clippy --all-features --all-targets")
      }
    })

    job("Test", {
      runs_on: "ubuntu-latest-large",

      strategy: matrix({
        include: [
          { key: "value", foo: "bar" },
          { key: "other", foo: "baz" },
        ],
      }),

      steps() {
        checkout()
        setup_rust()
        uses("taiki-e/install-action", {
          tool: "cargo-nextest"
        })
        run("cargo nextest run --all-features --all-targets")
      }
    })
  }
})
```

### Generated output

Before generation, the used actions must be pinned and vendored.

This is done using package-manager-like commands:

```
ghat add Swatinem/rust-cache taiki-e/install-action
```

> Note that all official `actions/*` are added by default.

The `add` command will add the given version (or latest, if none specified) to the
lockfile, which will later be used to pin the action to a commit sha, for maximum
protection against supply chain attacks.

`ghat add --auto` scans workflow definitions and automatically adds all referenced
actions that are not yet in the lockfile. This is mutually exclusive with specifying
actions explicitly.

`ghat rm` without arguments brings up an interactive list of all locked actions,
showing whether each action is currently used in any workflow definition, and lets
the user select which ones to remove. Actions can also be specified explicitly:
`ghat rm Swatinem/rust-cache`.

`ghat update` updates actions to the latest version according to their GitHub
releases page. Without arguments it updates all locked actions.

Note that this is not an actual package manager; it's just a convenience for version pinning.
It doesn't care which other actions each action depends on, it does not maintain a dependency
graph - it's fully flat.

Finally, once all dependencies are locked, the output may be generated:

```console
ghat generate
```

This will:
- Check workflow definition files
  - Semantics checks:
    - Are all actions available?
    - Are any pinned actions unused?
  - Lint pass on inline JS callbacks (scope safety checks)
  - Type checking via tsgo against generated `.d.ts` files
- Strip types using oxc (`isolatedModules`/`isolatedDeclarations`)
- Run them

Each call to the built-in `workflow` function results in one workflow file being generated.
The name is `generated_${normalized_name}`.

The above example produces the following file:

```yaml
#.github/workflows/generated_rust_ci.yaml
# Automatically generated from `Rust CI` in .github/ghat/workflows/ci-rust.ts
name: "Rust CI"

on:
  push:
    branches: ["main"]
  pull_request:

defaults:
  run:
    shell: bash --noprofile --norc -euo pipefail {0}

jobs:
  lint:
    name: "Lint"
    runs-on: "ubuntu-latest-large"
    steps:
      - uses: "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd"
        id: "step_0"

      - run: "rustup install"
        id: "step_1"

      - name: "Setup Rust cache"
        uses: "Swatinem/rust-cache@779680da715d629ac1d338a641029a2f4372abb5"
        id: "step_2"

      - run: "cargo clippy --all-features --all-targets"
        id: "step_3"
  test:
    name: "Test"
    runs-on: "ubuntu-latest-large"
    steps:
      - uses: "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd"
        id: "step_0"

      - run: "rustup install"
        id: "step_1"

      - name: "Setup Rust cache"
        uses: "Swatinem/rust-cache@779680da715d629ac1d338a641029a2f4372abb5"
        id: "step_2"

      - uses: "taiki-e/install-action@288875dd3d64326724fa6d9593062d9f8ba0b131"
        with:
          tool: "cargo-nextest"
        id: "step_3"

      - run: "cargo nextest run --all-features --all-targets"
        id: "step_4"
```


On CI, one job is always generated:

```yaml
#.github/workflows/generated_ghat_codegen_check.yaml
# Automatically generated by `ghat`
name: "Workflow codegen check"

on:
  pull_request:

jobs:
  codegen-check:
    runs-on: ubuntu-latest
    steps:
      - uses: "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd"
      - uses: "taiki-e/install-action@288875dd3d64326724fa6d9593062d9f8ba0b131"
        with:
          tool: "ghat"
      - run: "ghat check"
        shell: bash --noprofile --norc -euo pipefail {0}
```

The CLI only ever touches or modifies the `generated_*` files.
If there are other workflow files already present, they are ignored.

## Job/Step dependencies

Dependencies between jobs and steps are explicit, and use JS variables:

```ts
jobs() {
  // Under the hood, each job is always assigned a unique ID. These IDs are normalized job names.
  // If the job `steps` function returns an object, that becomes its `outputs:` section.
  let build = job("Build", {
    steps() {
      checkout()
      setup()
      build()

      // Just like jobs, every step is assigned a unique ID, and they can produce outputs.
      // For `uses`, the outputs are typed based on `action.yml`, just like inputs (`with`).

      let { artifact_id } = uses("actions/upload-artifact", {
        path: "build"
      })

      // The outputs section is populated based on the returned values.
      // Only plain string outputs are allowed as outputs.
      return { artifact_id }
    }
  })

  job("Deploy", {
    needs: [build],

    steps() {
      uses("actions/download-artifact", {
        artifact_ids: this.needs.build.artifact_id,
        path: "build",
      })

      deploy("./build")
    }
  })
}
```

### Generated output

```yaml
#.github/workflows/generated_rust_ci.yaml
# Automatically generated from `Rust CI` in .github/ghat/workflows/ci-rust.ts
name: "Rust CI"

on:
  push:
    branches: ["main"]
  pull_request:

defaults:
  run:
    shell: bash --noprofile --norc -euo pipefail {0}

jobs:
  build:
    name: "Build"
    runs-on: "ubuntu-latest"
    outputs:
      artifact_id: ${{ steps.step_3.outputs.artifact_id }}
    steps:
      - uses: "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd"
        id: "step_0"

      - run: "...some setup code..."
        id: "step_1"

      - run: "...build..."
        id: "step_2"

      - uses: "actions/upload-artifact@b7c566a772e6b6bfb58ed0dc250532a479d7789f"
        id: "step_3"
        with:
          path: "build"
          
  deploy:
    name: "Deploy"
    runs-on: "ubuntu-latest"
    needs: ["build"]
    steps:
      - uses: "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd"

      - uses: "actions/download-artifact@37930b1c2abaa49bbe596cd826c3c89aef350131"
        with:
          artifact_ids: "${{ needs.build.outputs.artifact_id }}"
          path: "build"

      - run: "...deployment..."

```


## Inline JS snippets

The `run` builtin accepts a callback for inline JS, similar in spirit to `actions/github-script`
but without the heavy bundling (no webpack'd node builtins, no full Octokit SDK).

```ts
steps() {
  run(async (ctx) => {
    await ctx.gh.api.create_comment({
      issue_number: ctx.run.pull_request.number,
      body: "Hello from inline JS snippet"
    })
  })
}
```

### Codegen

The callback body is extracted via AST (using oxc) and emitted as a `run:` step invoking
`ghat exec` with the script inlined as a heredoc via stdin. `ghat exec` accepts a
script from `-e` (inline argument) or from stdin (heredoc / pipe); if neither is
provided it exits with an error. The single-quoted heredoc delimiter prevents shell
variable expansion, mitigating script injection. Workflow inputs/outputs
from other jobs/steps are passed as explicit dependencies via `env:`, which become
variables on `ctx`.

```yaml
- run: |
    ghat exec <<'GHAT_SCRIPT'
    await ctx.gh.api.create_comment({
      issue_number: ctx.run.pull_request.number,
      body: "Hello from inline JS snippet"
    })
    GHAT_SCRIPT
  env:
    GHAT_PULL_REQUEST_NUMBER: "${{ github.event.pull_request.number }}"
    GITHUB_TOKEN: "${{ secrets.GITHUB_TOKEN }}"
```

### Available APIs

- `ctx` runtime APIs:
  - `ctx.gh.api` - thin GitHub REST client (authenticated via `GITHUB_TOKEN`)
  - `ctx.gh.event` - event context
- `ctx` environment:
  - `ctx.env`
  - `ctx.needs.*`
  - `ctx.steps.*`
- `ctx` standard libraries:
  - `ctx.fs` (cwd, read/write entire files, byte or string, read dir, glob)
  - `ctx.path` (path manipulation)
  - `ctx.os` (os-specific info, e.g. path separator)
  - `ctx.crypto` (hashing)
  - `ctx.buffer` (binary data)
  - `ctx.base64` (base64 encode/decode, url-safe, pad or no pad, etc.)
  - `ctx.serde` (json, toml, yaml)
  - `ctx.url` (url parsing and manipulation)
  - `ctx.exec(cmd)` - subprocess execution
  - `ctx.http` - http requests (simplified compared to node/web fetch)
  - `ctx.zlib` - compression
  - `ctx.tar` - tarball processing
  - `ctx.semver` - semantic version string handling

All APIs are built into the `ghat` binary itself — no webpack, no bundled node_modules.
Type definitions for `ctx` and available APIs are generated alongside other `.d.ts` files
during `ghat init`.

### Scope safety

The callback body is extracted via AST and stringified, but outer-scope references survive
as free identifiers in the extracted source. A lint pass during `ghat generate` checks
scans the extracted function body's AST and emits hard errors for any references that aren't:
- The callback parameter (`ctx`)
- Locally-declared variables within the callback
- Known allowed globals (`fs`, `path`, `os`, `crypto`, `glob`, `fetch`, `Buffer`, `URL`, etc.)

References to builder globals (`workflow`, `job`, `uses`, `run`, `step`, `checkout`) or
user-defined outer-scope variables are rejected before the workflow file is ever generated.

As a secondary safety net, `ghat exec` at runtime evaluates in a context where only the
documented APIs exist, so stray references would be `ReferenceError`s anyway.


## Implementation stack

- **Rust** — CLI, codegen, lockfile management, YAML generation. Statically linked (musl)
  single binary. Builtin assets (type definitions, etc.) embedded via `include_bytes!`/`include_str!`.
- **QuickJS** (via [rquickjs](https://github.com/nickel-org/rquickjs)) — JS evaluation for
  workflow definitions (`ghat generate`) and inline script execution (`ghat exec`). Sub-millisecond
  startup, ~300KB footprint, trivial to statically link. No JIT, but the scripts being evaluated
  are short CI glue code where JIT is irrelevant.
- **oxc** — Rust-native TS/JS parser. Used for type stripping (`isolatedModules`/`isolatedDeclarations`),
  AST extraction of inline `run` callback bodies, and the lint pass that checks for outer-scope
  references.
- **tsgo** — Native TypeScript type checker (Microsoft's Go port). Used by `ghat check`/`ghat generate`
  for full type checking of workflow definitions against generated `.d.ts` files.

### tsgo distribution

Each `ghat` GitHub Release includes a `tsgo` binary for every target platform, unpacked from
the corresponding `@typescript/native-preview-$target` npm package before publishing. This means
`ghat` releases are fully self-contained — no runtime downloads, no npm dependency.

For local development:
- `tsgo-version` — a file in the repo root containing the npm version string
  (e.g. `7.0.0-dev.20260214.1`)
- `scripts/download-tsgo` — dev-only script that downloads the `tsgo` binary for the current
  platform from `@typescript/native-preview-$target` at the version specified in `tsgo-version`
- `scripts/update-tsgo` — dev-only script that updates `tsgo-version` to the latest published
  version and runs `download-tsgo`

This guarantees reproducible builds for any given version of `ghat`.
