use super::*;

#[test]
fn parse_scheduled_workflow() {
    let yaml = r#"
name: Nightly Build

on:
  workflow_dispatch:
  schedule:
    - cron: "15 3 * * *"

defaults:
  run:
    shell: bash -euo pipefail {0}
    working-directory: app

permissions:
  contents: write
  id-token: write
  deployments: write
  pull-requests: write
  packages: read

jobs:
  lint:
    name: Lint
    uses: ./.github/workflows/reusable_lint.yml
    with:
      CHANNEL: nightly
    secrets: inherit

  build:
    timeout-minutes: 60
    name: Build
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: 1.80.0
      - run: cargo build
        working-directory: app

  deploy:
    timeout-minutes: 60
    name: Deploy
    concurrency: nightly
    needs:
      [lint, build]
    runs-on: ubuntu-latest
    steps:
      - name: Set short SHA
        run: echo "SHORT_SHA=$(echo ${{ github.sha }} | cut -c1-7)" >> $GITHUB_ENV
        working-directory: .
      - uses: some/deploy-action@v1
        with:
          prerelease: true
          tag: prerelease
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
"#;
    let wf: Workflow = serde_yaml_ng::from_str(yaml).unwrap();
    assert_eq!(wf.name, "Nightly Build");
    assert!(wf.on.workflow_dispatch.is_some());
    assert_eq!(wf.on.schedule.len(), 1);
    assert!(wf.defaults.is_some());
    let run = wf.defaults.as_ref().unwrap().run.as_ref().unwrap();
    assert_eq!(run.shell.as_deref(), Some("bash -euo pipefail {0}"));
    assert_eq!(run.working_directory.as_deref(), Some("app"));

    if let Some(Permissions::Scoped(s)) = &wf.permissions {
        assert_eq!(s.contents, Some(PermissionLevel::Write));
        assert_eq!(s.packages, Some(PermissionLevel::Read));
    } else {
        panic!("expected scoped permissions");
    }

    assert_eq!(wf.jobs.len(), 3);

    let lint = &wf.jobs["lint"];
    assert_eq!(
        lint.uses.as_deref(),
        Some("./.github/workflows/reusable_lint.yml")
    );
    assert!(!lint.with.is_empty());

    let build = &wf.jobs["build"];
    assert!(build.strategy.is_some());
    assert!(build.runs_on.is_some());
    assert_eq!(build.timeout_minutes, Some(60));
    assert_eq!(build.steps.len(), 3);

    let deploy = &wf.jobs["deploy"];
    assert_eq!(
        deploy.concurrency,
        Some(Concurrency {
            group: "nightly".into(),
            cancel_in_progress: false
        })
    );
    assert_eq!(deploy.needs, vec!["lint", "build"]);
    assert!(!deploy.steps[1].env.is_empty());
}

#[test]
fn parse_pull_request_workflow() {
    let yaml = r#"
name: PR Checks

on:
  pull_request:
    types:
      - opened
      - synchronize
    paths:
      - "src/**"
      - "tests/**"
      - ".github/workflows/*.yml"

permissions: write-all

jobs:
  detect-changes:
    if: github.event.pull_request.head.repo.owner.login == 'my-org'
    runs-on: ubuntu-latest
    outputs:
      src_changes: ${{ steps.filter.outputs.src_changes }}
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.event.pull_request.head.ref }}
      - uses: dorny/paths-filter@v3
        id: filter
        with:
          filters: |
            src_changes:
              - 'src/**'

  test:
    name: Tests
    needs: [detect-changes]
    if: needs.detect-changes.outputs.src_changes == 'true'
    uses: ./.github/workflows/reusable_test.yml
    with:
      CONCURRENCY: pr-${{ github.event.pull_request.number }}
    secrets: inherit
"#;
    let wf: Workflow = serde_yaml_ng::from_str(yaml).unwrap();
    assert_eq!(wf.name, "PR Checks");

    let pr = wf.on.pull_request.as_ref().unwrap();
    assert_eq!(pr.types, vec!["opened", "synchronize"]);
    assert_eq!(pr.paths.len(), 3);

    assert_eq!(wf.permissions, Some(Permissions::WriteAll));

    let detect = &wf.jobs["detect-changes"];
    assert!(detect.if_condition.is_some());
    assert_eq!(detect.runs_on, Some(RunsOn::Label("ubuntu-latest".into())));
    assert!(detect.outputs.contains_key("src_changes"));
    assert_eq!(detect.steps.len(), 2);
    assert_eq!(detect.steps[1].id.as_deref(), Some("filter"));

    let test = &wf.jobs["test"];
    assert_eq!(test.needs, vec!["detect-changes"]);
    assert!(test.if_condition.is_some());
    assert!(test.uses.is_some());
}

#[test]
fn parse_issue_comment_workflow() {
    let yaml = r#"
name: Bot Commands

on:
  issue_comment:
    types: [created]

defaults:
  run:
    shell: bash -euo pipefail {0}

permissions:
  contents: read
  id-token: write
  pull-requests: write

jobs:
  handle-command:
    if: |
      contains(github.event.comment.body, '@my-bot') &&
      contains(fromJSON('["MEMBER","OWNER"]'), github.event.comment.author_association)
    runs-on: ubuntu-latest
    outputs:
      command: ${{ steps.parse.outputs.command }}
    steps:
      - uses: actions/checkout@v4
      - name: Parse comment
        id: parse
        env:
          COMMENT_BODY: ${{ github.event.comment.body }}
        run: python ./scripts/parse_comment.py
"#;
    let wf: Workflow = serde_yaml_ng::from_str(yaml).unwrap();
    assert_eq!(wf.name, "Bot Commands");

    let ic = wf.on.issue_comment.as_ref().unwrap();
    assert_eq!(ic.types, vec!["created"]);

    let run = wf.defaults.as_ref().unwrap().run.as_ref().unwrap();
    assert!(run.shell.is_some());
    assert!(run.working_directory.is_none());

    if let Some(Permissions::Scoped(s)) = &wf.permissions {
        assert_eq!(s.contents, Some(PermissionLevel::Read));
        assert_eq!(s.id_token, Some(PermissionLevel::Write));
        assert_eq!(s.pull_requests, Some(PermissionLevel::Write));
    } else {
        panic!("expected scoped permissions");
    }

    let job = &wf.jobs["handle-command"];
    assert!(job.if_condition.is_some());
    assert!(job.outputs.contains_key("command"));
    let step = &job.steps[1];
    assert_eq!(step.id.as_deref(), Some("parse"));
    assert!(step.env.contains_key("COMMENT_BODY"));
}

#[test]
fn parse_dual_trigger_workflow() {
    let yaml = r#"
name: Basic Checks

on:
  pull_request:
    types:
      - opened
      - synchronize
  push:
    branches:
      - main

permissions:
  contents: read

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo fmt --check

  build:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: subproject
    steps:
      - uses: actions/checkout@v4
      - run: cargo check --locked
"#;
    let wf: Workflow = serde_yaml_ng::from_str(yaml).unwrap();

    assert!(wf.on.pull_request.is_some());
    assert!(wf.on.push.is_some());
    let push = wf.on.push.as_ref().unwrap();
    assert_eq!(push.branches, vec!["main"]);

    assert_eq!(
        wf.permissions,
        Some(Permissions::Scoped(ScopedPermissions {
            contents: Some(PermissionLevel::Read),
            ..Default::default()
        }))
    );

    let build = &wf.jobs["build"];
    let wd = build.defaults.as_ref().unwrap().run.as_ref().unwrap();
    assert_eq!(wd.working_directory.as_deref(), Some("subproject"));
}

#[test]
fn parse_dispatch_with_inputs_and_matrix() {
    let yaml = r#"
name: Benchmarks

on:
  schedule:
    - cron: '0 5 * * *'
  workflow_dispatch:
    inputs:
      commit:
        description: 'Commit SHA to benchmark'
        required: false
        default: ''
      target:
        description: 'Benchmark target'
        type: string
        required: true

defaults:
  run:
    working-directory: bench

jobs:
  run-bench:
    runs-on: runs-on=${{ github.run_id }}/runner=large
    permissions:
      contents: read
      id-token: write
    strategy:
      matrix:
        provider: [aws, gcp]
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ inputs.commit || github.sha }}
      - name: Run benchmarks
        id: bench
        working-directory: .
        run: |
          BENCH_ID="auto-${{ matrix.provider }}-$(date +%Y%m%d)"
          python scripts/bench.py --provider ${{ matrix.provider }}
"#;
    let wf: Workflow = serde_yaml_ng::from_str(yaml).unwrap();
    assert_eq!(wf.name, "Benchmarks");
    assert_eq!(wf.on.schedule.len(), 1);

    let wd = wf.on.workflow_dispatch.as_ref().unwrap();
    assert!(wd.inputs.contains_key("commit"));
    let commit_input = &wd.inputs["commit"];
    assert_eq!(commit_input.required, Some(false));
    assert_eq!(commit_input.default.as_deref(), Some(""));
    assert!(commit_input.input_type.is_none());

    let target_input = &wd.inputs["target"];
    assert_eq!(target_input.input_type.as_deref(), Some("string"));
    assert_eq!(target_input.required, Some(true));

    let job = &wf.jobs["run-bench"];
    assert!(job.strategy.is_some());
    assert!(job.permissions.is_some());
    assert_eq!(
        job.runs_on,
        Some(RunsOn::Label(
            "runs-on=${{ github.run_id }}/runner=large".into()
        ))
    );

    let step = &job.steps[1];
    assert_eq!(step.id.as_deref(), Some("bench"));
    assert_eq!(step.working_directory.as_deref(), Some("."));
}

#[test]
fn parse_push_with_paths_and_dispatch() {
    let yaml = r#"
name: Push to Main

on:
  push:
    branches:
      - main
    paths:
      - "src/**"
      - "lib/**"
  workflow_dispatch:
    inputs:
      CONCURRENCY:
        required: true
        type: string

permissions: write-all

jobs:
  build:
    name: Build
    uses: ./.github/workflows/reusable_build.yml
    with:
      CONCURRENCY: push-${{ github.ref_name }}-${{ inputs.CONCURRENCY || github.sha }}
    secrets: inherit

  test:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo test
"#;
    let wf: Workflow = serde_yaml_ng::from_str(yaml).unwrap();

    let push = wf.on.push.as_ref().unwrap();
    assert_eq!(push.branches, vec!["main"]);
    assert_eq!(push.paths, vec!["src/**", "lib/**"]);

    let wd = wf.on.workflow_dispatch.as_ref().unwrap();
    assert!(wd.inputs.contains_key("CONCURRENCY"));
    assert_eq!(wd.inputs["CONCURRENCY"].required, Some(true));

    assert_eq!(wf.permissions, Some(Permissions::WriteAll));

    let build = &wf.jobs["build"];
    assert!(build.uses.is_some());

    let test = &wf.jobs["test"];
    assert_eq!(test.needs, vec!["build"]);
}

#[test]
fn parse_concurrency_at_workflow_level() {
    let yaml = r#"
name: Sync

on:
  pull_request:
    types: [opened, synchronize]
    branches:
      - main

concurrency:
  group: sync-${{ github.event.pull_request.number }}
  cancel-in-progress: false

jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: echo "syncing"
"#;
    let wf: Workflow = serde_yaml_ng::from_str(yaml).unwrap();
    let c = wf.concurrency.as_ref().unwrap();
    assert_eq!(c.group, "sync-${{ github.event.pull_request.number }}");
    assert!(!c.cancel_in_progress);
}

#[test]
fn parse_runs_on_label_array() {
    let yaml = r#"
name: Self-hosted

on:
  push:

jobs:
  build:
    runs-on: [self-hosted, linux, x64]
    steps:
      - run: echo "building"
"#;
    let wf: Workflow = serde_yaml_ng::from_str(yaml).unwrap();
    let job = &wf.jobs["build"];
    assert_eq!(
        job.runs_on,
        Some(RunsOn::Labels(vec![
            "self-hosted".into(),
            "linux".into(),
            "x64".into(),
        ]))
    );
}

#[test]
fn parse_step_continue_on_error() {
    let yaml = r#"
name: Resilient

on:
  push:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Flaky test
        run: cargo test --flaky
        continue-on-error: true
        timeout-minutes: 10
"#;
    let wf: Workflow = serde_yaml_ng::from_str(yaml).unwrap();
    let step = &wf.jobs["test"].steps[1];
    assert_eq!(step.continue_on_error, Some(true));
    assert_eq!(step.timeout_minutes, Some(10));
}

#[test]
fn parse_strategy_fail_fast() {
    let yaml = r#"
name: Matrix

on:
  push:

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        version: [stable, nightly]
    steps:
      - run: echo ${{ matrix.version }}
"#;
    let wf: Workflow = serde_yaml_ng::from_str(yaml).unwrap();
    let strategy = wf.jobs["test"].strategy.as_ref().unwrap();
    assert_eq!(strategy.fail_fast, Some(false));
    assert!(strategy.matrix.is_object());
}

#[test]
fn parse_paths_ignore() {
    let yaml = r#"
name: CI

on:
  push:
    branches: [main]
    paths-ignore:
      - "docs/**"
      - "*.md"

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "building"
"#;
    let wf: Workflow = serde_yaml_ng::from_str(yaml).unwrap();
    let push = wf.on.push.as_ref().unwrap();
    assert_eq!(push.paths_ignore, vec!["docs/**", "*.md"]);
}

#[test]
fn parse_run_name() {
    let yaml = r#"
name: Deploy
run-name: Deploy by @${{ github.actor }}

on:
  push:

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "deploying"
"#;
    let wf: Workflow = serde_yaml_ng::from_str(yaml).unwrap();
    assert_eq!(
        wf.run_name.as_deref(),
        Some("Deploy by @${{ github.actor }}")
    );
}

#[test]
fn concurrency_string_roundtrip() {
    let c = Concurrency {
        group: "nightly".into(),
        cancel_in_progress: false,
    };
    let yaml = serde_yaml_ng::to_string(&c).unwrap();
    assert_eq!(yaml.trim(), "nightly");
    let parsed: Concurrency = serde_yaml_ng::from_str(&yaml).unwrap();
    assert_eq!(parsed, c);
}

#[test]
fn concurrency_map_roundtrip() {
    let yaml = "group: my-group\ncancel-in-progress: true\n";
    let c: Concurrency = serde_yaml_ng::from_str(yaml).unwrap();
    assert_eq!(c.group, "my-group");
    assert!(c.cancel_in_progress);
}

#[test]
fn permissions_write_all() {
    let p: Permissions = serde_yaml_ng::from_str("write-all").unwrap();
    assert_eq!(p, Permissions::WriteAll);
}

#[test]
fn permissions_read_all() {
    let p: Permissions = serde_yaml_ng::from_str("read-all").unwrap();
    assert_eq!(p, Permissions::ReadAll);
}

#[test]
fn permissions_scoped() {
    let yaml = "contents: read\nid-token: write\n";
    let p: Permissions = serde_yaml_ng::from_str(yaml).unwrap();
    if let Permissions::Scoped(s) = p {
        assert_eq!(s.contents, Some(PermissionLevel::Read));
        assert_eq!(s.id_token, Some(PermissionLevel::Write));
    } else {
        panic!("expected scoped permissions");
    }
}

#[test]
fn runs_on_string() {
    let r: RunsOn = serde_yaml_ng::from_str("ubuntu-latest").unwrap();
    assert_eq!(r, RunsOn::Label("ubuntu-latest".into()));
}

#[test]
fn runs_on_array() {
    let r: RunsOn = serde_yaml_ng::from_str("[self-hosted, linux]").unwrap();
    assert_eq!(
        r,
        RunsOn::Labels(vec!["self-hosted".into(), "linux".into()])
    );
}

#[test]
fn needs_single_string() {
    #[derive(Deserialize)]
    struct H {
        #[serde(default, deserialize_with = "string_or_vec")]
        needs: Vec<String>,
    }
    let h: H = serde_yaml_ng::from_str("needs: lint").unwrap();
    assert_eq!(h.needs, vec!["lint"]);
}

#[test]
fn needs_array() {
    #[derive(Deserialize)]
    struct H {
        #[serde(default, deserialize_with = "string_or_vec")]
        needs: Vec<String>,
    }
    let h: H = serde_yaml_ng::from_str("needs: [a, b, c]").unwrap();
    assert_eq!(h.needs, vec!["a", "b", "c"]);
}

#[test]
fn workflow_dispatch_null() {
    let yaml = "workflow_dispatch:\n";
    let t: Triggers = serde_yaml_ng::from_str(yaml).unwrap();
    assert!(t.workflow_dispatch.is_some());
    assert!(t.workflow_dispatch.unwrap().inputs.is_empty());
}

#[test]
fn push_trigger_null() {
    let yaml = "push:\n";
    let t: Triggers = serde_yaml_ng::from_str(yaml).unwrap();
    assert!(t.push.is_some());
    assert!(t.push.unwrap().branches.is_empty());
}

#[test]
fn pull_request_trigger_null() {
    let yaml = "pull_request:\n";
    let t: Triggers = serde_yaml_ng::from_str(yaml).unwrap();
    assert!(t.pull_request.is_some());
}

#[test]
fn absent_triggers_are_none() {
    let yaml = "push:\n  branches: [main]\n";
    let t: Triggers = serde_yaml_ng::from_str(yaml).unwrap();
    assert!(t.push.is_some());
    assert!(t.pull_request.is_none());
    assert!(t.workflow_dispatch.is_none());
    assert!(t.schedule.is_empty());
}

fn roundtrip(yaml: &str) {
    let wf: Workflow = serde_yaml_ng::from_str(yaml)
        .unwrap_or_else(|e| panic!("failed to parse input YAML: {e}\n---\n{yaml}"));
    let a = serde_yaml_ng::to_string(&wf).unwrap();
    let reparsed: Workflow = serde_yaml_ng::from_str(&a)
        .unwrap_or_else(|e| panic!("failed to re-parse serialized YAML: {e}\n---\n{a}"));
    let b = serde_yaml_ng::to_string(&reparsed).unwrap();
    assert_eq!(a, b, "serialization not stable across roundtrip");
}

#[test]
fn roundtrip_minimal_workflow() {
    roundtrip(
        r#"
name: Minimal
on:
  push:
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - run: echo ok
"#,
    );
}

#[test]
fn roundtrip_scoped_permissions_and_defaults() {
    roundtrip(
        r#"
name: Scoped
run-name: Deploy by @user

on:
  push:
    branches: [main]
    paths:
      - "src/**"
    paths-ignore:
      - "docs/**"

permissions:
  contents: write
  packages: read
  id-token: write

defaults:
  run:
    shell: bash -e {0}
    working-directory: app

jobs: {}
"#,
    );
}

#[test]
fn roundtrip_write_all_and_concurrency_map() {
    roundtrip(
        r#"
name: PR Gate

on:
  pull_request:
    types: [opened, synchronize]
    paths:
      - "src/**"

permissions: write-all

concurrency:
  group: pr-${{ github.event.pull_request.number }}
  cancel-in-progress: true

jobs: {}
"#,
    );
}

#[test]
fn roundtrip_schedule_and_concurrency_string() {
    roundtrip(
        r#"
name: Nightly

on:
  workflow_dispatch: {}
  schedule:
    - cron: "15 3 * * *"

concurrency: nightly

jobs: {}
"#,
    );
}

#[test]
fn roundtrip_dispatch_inputs() {
    roundtrip(
        r#"
name: Dispatch

on:
  workflow_dispatch:
    inputs:
      commit:
        description: Commit SHA
        required: false
        default: ""
      target:
        description: Target env
        type: string
        required: true

jobs: {}
"#,
    );
}

#[test]
fn roundtrip_jobs_with_steps() {
    roundtrip(
        r#"
name: Full

on:
  push:
    branches: [main]

permissions: read-all

env:
  CI: "true"

jobs:
  lint:
    name: Lint
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v4
      - name: Run lint
        run: cargo clippy

  build:
    name: Build
    runs-on:
      - self-hosted
      - linux
    needs:
      - lint
    if: always()
    permissions:
      contents: read
    env:
      RUST_LOG: info
    defaults:
      run:
        working-directory: app
    concurrency: build
    timeout-minutes: 60
    strategy:
      fail-fast: false
      matrix:
        os:
          - ubuntu-latest
          - macos-latest
    outputs:
      artifact: ${{ steps.upload.outputs.path }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: "1.80.0"
      - name: Build
        run: cargo build --release
        working-directory: .
        env:
          TARGET: ${{ matrix.os }}
        continue-on-error: false
        timeout-minutes: 30
      - name: Upload
        id: upload
        uses: actions/upload-artifact@v4
        if: success()
"#,
    );
}

#[test]
fn roundtrip_reusable_workflow_job() {
    roundtrip(
        r#"
name: Reusable

on:
  push:
    branches: [main]

permissions: write-all

jobs:
  deploy:
    name: Deploy
    needs:
      - build
      - test
    uses: ./.github/workflows/reusable_deploy.yml
    with:
      CONCURRENCY: nightly
      DEPLOY: true
    secrets: inherit
"#,
    );
}

#[test]
fn roundtrip_issue_comment_trigger() {
    roundtrip(
        r#"
name: Bot

on:
  issue_comment:
    types: [created, edited]

permissions:
  contents: read
  pull-requests: write

defaults:
  run:
    shell: bash -e {0}

jobs: {}
"#,
    );
}
