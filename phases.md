## Implementation phases

### Phase 1: Scaffold & minimal JS codegen

Goal: `ghat generate` takes a plain JS workflow definition and produces a YAML file.

- Rust CLI skeleton (clap): `ghat generate` subcommand
- QuickJS integration via rquickjs: embed the JS runtime
- Builder globals: `workflow`, `job`, `run`, `step`, `checkout`, `uses` — registered as
  QuickJS globals that record structure into a workflow model
- YAML emitter: take the recorded workflow model and serialize it to GitHub Actions YAML
- File conventions: scan `.github/ghat/workflows/*.js`, skip `_`-prefixed files, write
  `generated_*.yaml` into `.github/workflows/`
- Smart defaults: `shell: bash --noprofile --norc -euo pipefail {0}`, auto-generated
  `id: step_N` on every step
- End-to-end test: a simple JS workflow definition produces the expected YAML output

No TypeScript, no lockfile, no action pinning yet. `uses(...)` emits the string as-is.

### Phase 2: Lockfile & action pinning

Goal: `ghat add`, `ghat rm`, `ghat update` manage a lockfile; `generate` resolves action
references to pinned commit SHAs.

- Lockfile format: `.github/ghat/ghat.lock` — `owner/repo@version → sha` entries (JSON object)
- `ghat add <owner/repo[@version]>` — resolve latest release (or specified version) via
  GitHub API, fetch the commit SHA for that tag, write to lockfile
- `ghat rm <owner/repo>` — remove entry from lockfile
- `ghat update [<owner/repo>]` — update one or all entries to latest release
- Default actions: `actions/checkout` (and other official `actions/*`) seeded on first `add`
  or `init`
- Integrate with codegen: during `generate`, resolve every `uses(...)` reference against the
  lockfile; error if an action is missing
- Semantic checks: warn on unused pinned actions

### Phase 3: TypeScript support

Goal: workflow definitions can be written in `.ts` files with full type checking.

- oxc integration: strip types from `.ts` files (`isolatedModules`/`isolatedDeclarations`)
  before passing to QuickJS
- `ghat init` command: scaffold `.github/ghat/` directory structure, including
  `tsconfig.json` and `types/` directory
- Type generation for builtins: emit `.d.ts` files for builder globals (`workflow`, `job`,
  `run`, `step`, `checkout`, `uses`) and the `ctx` runtime object
- Type generation for actions: parse `action.yml` from locked actions, generate typed
  `uses(...)` overloads with `with` (inputs) and return type (outputs)
- tsgo integration: run tsgo against workflow definitions as a check step during `generate`
- `tsgo-version` file + `scripts/download-tsgo` + `scripts/update-tsgo` dev scripts

### Phase 4: Job/step dependencies & outputs

Goal: jobs and steps can declare dependencies and pass outputs via JS variables.

- `job(...)` returns an opaque handle with typed `.outputs` — backed by
  `${{ needs.<job_id>.outputs.<name> }}` expressions in YAML
- `uses(...)` returns destructurable outputs — backed by
  `${{ steps.<step_id>.outputs.<name> }}` expressions, typed from `action.yml`
- `needs: [...]` on jobs: translate JS handles to `needs:` array in YAML
- `outputs:` section on jobs: emit mapping from return values to step output expressions
- Update `.d.ts` generation to reflect output types on job/step handles

### Phase 5: Inline JS snippets (`run` callbacks & `ghat exec`)

Goal: `run(async (ctx) => { ... })` extracts the callback body and emits it as a
`ghat exec` heredoc step; `ghat exec` runs the script at CI time.

If an object is returned from the callback, its keys are added to `GITHUB_OUTPUT`.

- oxc AST extraction: given a `run(callback)` call, extract the callback function body as
  source text
- Scope safety lint pass: walk the extracted AST, reject references to builder globals or
  outer-scope user variables; allow `ctx`, local declarations, and known globals
- Codegen: emit `run: | ghat exec <<'GHAT_SCRIPT' ... GHAT_SCRIPT` with explicit `env:`
  mappings for workflow context values
- `ghat exec` subcommand: read script from stdin, evaluate in QuickJS with the `ctx` runtime
  object
- `ctx` runtime APIs

### Phase 6: CI integration & release

Goal: fully self-contained releases; generated codegen-check workflow; `ghat check` command.

- `ghat check` command: run `generate` in dry-run mode, diff against existing files, exit
  non-zero if out of date
- Auto-generated `generated_ghat_codegen_check.yaml` workflow: always emitted alongside user
  workflows
- Release pipeline: build static musl binaries for all targets; bundle tsgo binaries from
  `@typescript/native-preview-$target` npm packages; publish GitHub Release with all
  artifacts
- Documentation and README
