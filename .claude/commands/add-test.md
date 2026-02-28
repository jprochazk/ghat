Create one or more e2e test fixtures for the ghat CLI.

## How e2e tests work

Tests live in `tests/fixtures/`. Each fixture is a directory containing:
- `workflows/` — directory with `.ts` and/or `.js` workflow definition files

Files starting with `_` (e.g. `_utils.ts`) are reusable modules — they won't be evaluated directly by `ghat generate` but can be imported by other workflow files.

The test harness (`tests/e2e.rs`) does the following for each fixture:
1. Copies `workflows/` into a temp dir at `.github/ghat/workflows/`
2. Runs `ghat generate`
3. Snapshots exit code, stderr, and all generated YAML files
4. Compares against `tests/snapshots/{fixture_name}.snap`

## Your task

Based on this description: $ARGUMENTS

The description may contain multiple test cases (comma-separated, numbered, or otherwise listed). Create a separate fixture for each one.

For each fixture:
1. Pick a descriptive snake_case name
2. Create `tests/fixtures/{name}` (empty dir)
3. Create the `.ts`/`.js` files under `tests/fixtures/{name}/workflows/`

After creating all fixtures:
4. Run `cargo test --test e2e` once to generate all snapshots
5. Review each snapshot in the output — if they look correct, run `cargo insta accept`
6. Run `cargo test --test e2e` again to confirm everything passes

Look at existing fixtures in `tests/fixtures/` and the builder API in `src/runtime/api-impl.ts` for reference on what's available (workflow, triggers, job, run, uses, etc).
