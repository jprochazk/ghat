/** TypeScript type definitions for GitHub Actions workflow schema (mirrors src/workflow.rs) */

export interface Workflow {
  name: string;
  "run-name"?: string;
  on: Triggers;
  permissions?: Permissions;
  env?: Record<string, string>;
  defaults?: Defaults;
  concurrency?: Concurrency;
  jobs: Record<string, Job>;
}

export interface Triggers {
  push?: Push;
  pull_request?: PullRequest;
  pull_request_target?: PullRequest;
  issue_comment?: IssueComment;
  schedule?: Schedule[];
  workflow_dispatch?: WorkflowDispatch;
  repository_dispatch?: RepositoryDispatch;
}

export interface Push {
  branches?: string[];
  tags?: string[];
  paths?: string[];
}

export interface PullRequest {
  types?: string[];
  branches?: string[];
  paths?: string[];
}

export interface IssueComment {
  types?: string[];
}

export interface Schedule {
  /** Cron expression string, e.g. "0 0 * * *" */
  cron: string;
}

export interface WorkflowDispatch {
  inputs?: Record<string, DispatchInput>;
}

export interface DispatchInput {
  description?: string;
  type?: string;
  required?: boolean;
  default?: string;
  options?: string[];
}

export interface RepositoryDispatch {
  types?: string[];
}

export type Permissions = "write-all" | "read-all" | ScopedPermissions;

export interface ScopedPermissions {
  contents?: PermissionLevel;
  "pull-requests"?: PermissionLevel;
  packages?: PermissionLevel;
  "id-token"?: PermissionLevel;
  deployments?: PermissionLevel;
  actions?: PermissionLevel;
  attestations?: PermissionLevel;
}

export type PermissionLevel = "read" | "write" | "none";

/** A string group name, or an object with group + cancel-in-progress. */
export type Concurrency = string | {
  group: string;
  "cancel-in-progress"?: boolean;
};

export interface Defaults {
  run?: RunDefaults;
}

export interface RunDefaults {
  shell?: string;
  "working-directory"?: string;
}

export interface Job {
  name?: string;
  "runs-on"?: RunsOn;
  needs?: string | string[];
  if?: string;
  permissions?: Permissions;
  env?: Record<string, string>;
  defaults?: Defaults;
  concurrency?: Concurrency;
  "timeout-minutes"?: number;
  strategy?: Strategy;
  outputs?: Record<string, string>;
  steps?: Step[];
  /** Reusable workflow reference */
  uses?: string;
  with?: Record<string, unknown>;
  secrets?: unknown;
}

export type RunsOn = string | string[];

export interface Strategy {
  matrix: unknown;
  "fail-fast"?: boolean;
  "max-parallel"?: number;
}

export interface Step {
  id?: string;
  name?: string;
  uses?: string;
  run?: string;
  shell?: string;
  "working-directory"?: string;
  with?: Record<string, unknown>;
  env?: Record<string, string>;
  if?: string;
  "timeout-minutes"?: number;
  "continue-on-error"?: boolean;
}
