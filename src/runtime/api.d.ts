
declare global {
  // ---- GitHub Context Types ----
  // Generated from https://docs.github.com/en/actions/reference/workflows-and-actions/contexts
  // and https://github.com/octokit/webhooks payload schemas.

  // ---- Shared types ----


  /** A GitHub user account. */
  interface GitHubUser {
    /** The username of the user. */
    login: string;
    /** Unique identifier of the user. */
    id: number;
    /** The GraphQL identifier of the user. */
    node_id: string;
    /** Display name of the user. */
    name?: string;
    /** Email address of the user. */
    email?: string | null;
    /** URL of the user's avatar image. */
    avatar_url: string;
    gravatar_id: string;
    /** API URL for the user. */
    url: string;
    /** The URL to the user's profile on GitHub.com. */
    html_url: string;
    followers_url: string;
    following_url: string;
    gists_url: string;
    starred_url: string;
    subscriptions_url: string;
    organizations_url: string;
    repos_url: string;
    events_url: string;
    received_events_url: string;
    /** The type of the account. */
    type: "Bot" | "User" | "Organization";
    /** Whether the user is a site administrator. */
    site_admin: boolean;
  }

  /** Metaproperties for Git author/committer information. */
  interface GitHubCommitter {
    /** The git author's name. */
    name: string;
    /** The git author's email address. */
    email: string | null;
    date?: string;
    username?: string;
  }

  /** A commit in a push event. */
  interface GitHubCommit {
    /** The SHA of the commit. */
    id: string;
    /** The SHA of the tree for the commit. */
    tree_id: string;
    /** Whether this commit is distinct from any that have been pushed before. */
    distinct: boolean;
    /** The commit message. */
    message: string;
    /** The ISO 8601 timestamp of the commit. */
    timestamp: string;
    /** URL that points to the commit API resource. */
    url: string;
    /** The author of the commit. */
    author: GitHubCommitter;
    /** The committer of the commit. */
    committer: GitHubCommitter;
    /** An array of files added in the commit. */
    added: string[];
    /** An array of files modified by the commit. */
    modified: string[];
    /** An array of files removed in the commit. */
    removed: string[];
  }

  /** A label on an issue or pull request. */
  interface GitHubLabel {
    id: number;
    node_id: string;
    url: string;
    /** The name of the label. */
    name: string;
    /** The description of the label. */
    description: string | null;
    /** 6-character hex code (without leading `#`) identifying the color of the label. */
    color: string;
    default: boolean;
  }

  /** A git repository. */
  interface GitHubRepository {
    /** Unique identifier of the repository. */
    id: number;
    /** The GraphQL identifier of the repository. */
    node_id: string;
    /** The name of the repository. */
    name: string;
    /** The full, globally unique name of the repository (e.g. `octocat/Hello-World`). */
    full_name: string;
    /** Whether the repository is private or public. */
    private: boolean;
    /** The owner of the repository. */
    owner: GitHubUser;
    /** The URL to view the repository on GitHub.com. */
    html_url: string;
    /** The repository description. */
    description: string | null;
    /** Whether the repository is a fork. */
    fork: boolean;
    /** The API URL of the repository. */
    url: string;
    forks_url: string;
    keys_url: string;
    collaborators_url: string;
    teams_url: string;
    hooks_url: string;
    issue_events_url: string;
    events_url: string;
    assignees_url: string;
    branches_url: string;
    tags_url: string;
    blobs_url: string;
    git_tags_url: string;
    git_refs_url: string;
    trees_url: string;
    statuses_url: string;
    languages_url: string;
    stargazers_url: string;
    contributors_url: string;
    subscribers_url: string;
    subscription_url: string;
    commits_url: string;
    git_commits_url: string;
    comments_url: string;
    issue_comment_url: string;
    contents_url: string;
    compare_url: string;
    merges_url: string;
    archive_url: string;
    downloads_url: string;
    issues_url: string;
    pulls_url: string;
    milestones_url: string;
    notifications_url: string;
    labels_url: string;
    releases_url: string;
    deployments_url: string;
    created_at: string | number;
    updated_at: string;
    pushed_at: string | number | null;
    git_url: string;
    ssh_url: string;
    clone_url: string;
    svn_url: string;
    homepage: string | null;
    size: number;
    stargazers_count: number;
    watchers_count: number;
    /** The primary language of the repository. */
    language: string | null;
    /** Whether issues are enabled. */
    has_issues: boolean;
    /** Whether projects are enabled. */
    has_projects: boolean;
    /** Whether downloads are enabled. */
    has_downloads: boolean;
    /** Whether the wiki is enabled. */
    has_wiki: boolean;
    has_pages: boolean;
    /** Whether discussions are enabled. */
    has_discussions?: boolean;
    forks_count: number;
    mirror_url: string | null;
    /** Whether the repository is archived. */
    archived: boolean;
    /** Whether the repository is disabled. */
    disabled?: boolean;
    open_issues_count: number;
    license: { key: string; name: string; spdx_id: string; url: string | null; node_id: string } | null;
    forks: number;
    open_issues: number;
    watchers: number;
    stargazers?: number;
    /** The default branch of the repository. */
    default_branch: string;
    /** Whether to allow squash merges for pull requests. */
    allow_squash_merge?: boolean;
    /** Whether to allow merge commits for pull requests. */
    allow_merge_commit?: boolean;
    /** Whether to allow rebase merges for pull requests. */
    allow_rebase_merge?: boolean;
    /** Whether to allow auto-merge for pull requests. */
    allow_auto_merge?: boolean;
    /** Whether to allow private forks. */
    allow_forking?: boolean;
    allow_update_branch?: boolean;
    use_squash_pr_title_as_default?: boolean;
    squash_merge_commit_message?: string;
    squash_merge_commit_title?: string;
    merge_commit_message?: string;
    merge_commit_title?: string;
    is_template: boolean;
    web_commit_signoff_required: boolean;
    topics: string[];
    /** The visibility of the repository. */
    visibility: "public" | "private" | "internal";
    /** Whether to delete head branches when pull requests are merged. */
    delete_branch_on_merge?: boolean;
    master_branch?: string;
    permissions?: {
      pull: boolean;
      push: boolean;
      admin: boolean;
      maintain?: boolean;
      triage?: boolean;
    };
    public?: boolean;
    organization?: string;
    custom_properties: Record<string, string | string[] | null>;
  }

  type AuthorAssociation =
    | "COLLABORATOR"
    | "CONTRIBUTOR"
    | "FIRST_TIMER"
    | "FIRST_TIME_CONTRIBUTOR"
    | "MANNEQUIN"
    | "MEMBER"
    | "NONE"
    | "OWNER";

  /** Reactions on an issue or comment. */
  interface GitHubReactions {
    url: string;
    total_count: number;
    "+1": number;
    "-1": number;
    laugh: number;
    hooray: number;
    confused: number;
    heart: number;
    rocket: number;
    eyes: number;
  }

  /** A milestone on a repository. */
  interface GitHubMilestone {
    url: string;
    html_url: string;
    labels_url: string;
    id: number;
    node_id: string;
    /** The number of the milestone. */
    number: number;
    /** The title of the milestone. */
    title: string;
    description: string | null;
    creator: GitHubUser;
    open_issues: number;
    closed_issues: number;
    state: "open" | "closed";
    created_at: string;
    updated_at: string;
    due_on: string | null;
    closed_at: string | null;
  }

  /** A team in a GitHub organization. */
  interface GitHubTeam {
    id: number;
    node_id: string;
    name: string;
    slug: string;
    description: string | null;
    privacy: string;
    url: string;
    html_url: string;
    members_url: string;
    repositories_url: string;
    permission: string;
  }

  /** A pull request ref (head or base). */
  interface PullRequestRef {
    label: string;
    ref: string;
    sha: string;
    user: GitHubUser;
    repo: GitHubRepository | null;
  }

  /** A link object. */
  interface GitHubLink {
    href: string;
  }

  /** Auto-merge configuration on a pull request. */
  interface GitHubAutoMerge {
    enabled_by: GitHubUser;
    merge_method: "merge" | "squash" | "rebase";
    commit_title: string;
    commit_message: string;
  }

  // ---- Pull Request ----

  /** A pull request. */
  interface GitHubPullRequest {
    url: string;
    /** Unique identifier of the pull request. */
    id: number;
    node_id: string;
    /** The URL to view the pull request on GitHub.com. */
    html_url: string;
    diff_url: string;
    patch_url: string;
    issue_url: string;
    /** Number uniquely identifying the pull request within its repository. */
    number: number;
    /** State of this pull request. Either `open` or `closed`. */
    state: "open" | "closed";
    locked: boolean;
    /** The title of the pull request. */
    title: string;
    /** The user who opened the pull request. */
    user: GitHubUser;
    /** The body (description) of the pull request. */
    body: string | null;
    created_at: string;
    updated_at: string;
    closed_at: string | null;
    merged_at: string | null;
    merge_commit_sha: string | null;
    assignee: GitHubUser | null;
    assignees: GitHubUser[];
    requested_reviewers: (GitHubUser | GitHubTeam)[];
    requested_teams: GitHubTeam[];
    labels: GitHubLabel[];
    milestone: GitHubMilestone | null;
    commits_url: string;
    review_comments_url: string;
    review_comment_url: string;
    comments_url: string;
    statuses_url: string;
    /** The head (source) branch of the pull request. */
    head: PullRequestRef;
    /** The base (target) branch of the pull request. */
    base: PullRequestRef;
    _links: {
      self: GitHubLink;
      html: GitHubLink;
      issue: GitHubLink;
      comments: GitHubLink;
      review_comments: GitHubLink;
      review_comment: GitHubLink;
      commits: GitHubLink;
      statuses: GitHubLink;
    };
    author_association: AuthorAssociation;
    auto_merge: GitHubAutoMerge | null;
    active_lock_reason: "resolved" | "off-topic" | "too heated" | "spam" | null;
    /** Indicates whether or not the pull request is a draft. */
    draft: boolean;
    merged: boolean | null;
    mergeable: boolean | null;
    rebaseable: boolean | null;
    mergeable_state: string;
    merged_by: GitHubUser | null;
    comments: number;
    review_comments: number;
    /** Indicates whether maintainers can modify the pull request. */
    maintainer_can_modify: boolean;
    commits: number;
    additions: number;
    deletions: number;
    changed_files: number;
  }

  // ---- Issue ----

  /** An issue. */
  interface GitHubIssue {
    /** URL for the issue. */
    url: string;
    repository_url: string;
    labels_url: string;
    comments_url: string;
    events_url: string;
    html_url: string;
    id: number;
    node_id: string;
    /** Number uniquely identifying the issue within its repository. */
    number: number;
    /** Title of the issue. */
    title: string;
    user: GitHubUser;
    labels: GitHubLabel[];
    /** State of the issue; either `open` or `closed`. */
    state: "open" | "closed";
    locked: boolean;
    assignee: GitHubUser | null;
    assignees: GitHubUser[];
    milestone: GitHubMilestone | null;
    comments: number;
    created_at: string;
    updated_at: string;
    closed_at: string | null;
    author_association: AuthorAssociation;
    active_lock_reason: "resolved" | "off-topic" | "too heated" | "spam" | null;
    draft?: boolean;
    /** Contents of the issue. */
    body: string | null;
    reactions: GitHubReactions;
    timeline_url?: string;
    /** The reason for the current state. */
    state_reason?: string | null;
    /** Present when the issue is also a pull request. */
    pull_request?: {
      url: string;
      html_url: string;
      diff_url: string;
      patch_url: string;
      merged_at?: string | null;
    };
  }

  // ---- Issue Comment ----

  /** A comment on an issue or pull request. */
  interface GitHubIssueComment {
    /** URL for the issue comment. */
    url: string;
    html_url: string;
    issue_url: string;
    /** Unique identifier of the issue comment. */
    id: number;
    node_id: string;
    user: GitHubUser;
    created_at: string;
    updated_at: string;
    author_association: AuthorAssociation;
    /** Contents of the issue comment. */
    body: string;
    reactions: GitHubReactions;
  }

  // ---- Event Payloads ----

  /** Webhook payload for the `push` event. */
  interface PushEventPayload {
    /** The full git ref that was pushed. Example: `refs/heads/main` or `refs/tags/v3.14.1`. */
    ref: string;
    /** The SHA of the most recent commit on `ref` before the push. */
    before: string;
    /** The SHA of the most recent commit on `ref` after the push. */
    after: string;
    /** Whether this push created the `ref`. */
    created: boolean;
    /** Whether this push deleted the `ref`. */
    deleted: boolean;
    /** Whether this push was a force push of the `ref`. */
    forced: boolean;
    base_ref: string | null;
    /** URL that shows the changes in this `ref` update, from the `before` commit to the `after` commit. */
    compare: string;
    /** An array of commit objects describing the pushed commits. Maximum of 2048 commits. */
    commits: GitHubCommit[];
    /** The most recent commit on `ref` after the push (or `null` for tag deletions). */
    head_commit: GitHubCommit | null;
    /** The repository where the push occurred. */
    repository: GitHubRepository;
    /** Metaproperties for the pusher. */
    pusher: GitHubCommitter;
    /** The user that triggered the push. */
    sender: GitHubUser;
  }

  /** Webhook payload for the `pull_request` and `pull_request_target` events. */
  interface PullRequestEventPayload {
    action: string;
    /** The pull request number. */
    number: number;
    /** The pull request itself. */
    pull_request: GitHubPullRequest;
    /** The repository where the event occurred. */
    repository: GitHubRepository;
    /** The user that triggered the event. */
    sender: GitHubUser;
    assignee?: GitHubUser | null;
  }

  /** Webhook payload for the `issue_comment` event. */
  interface IssueCommentEventPayload {
    action: "created" | "deleted" | "edited" | "pinned" | "unpinned";
    /** The comment itself. */
    comment: GitHubIssueComment;
    /** The issue the comment belongs to. */
    issue: GitHubIssue;
    /** The repository where the event occurred. */
    repository: GitHubRepository;
    /** The user that triggered the event. */
    sender: GitHubUser;
  }

  /** Webhook payload for the `workflow_dispatch` event. */
  interface WorkflowDispatchEventPayload {
    /** The branch ref from which the workflow was run. */
    ref: string;
    /** The repository where the event occurred. */
    repository: GitHubRepository;
    /** The user that triggered the event. */
    sender: GitHubUser;
    /** Relative path to the workflow file which contains the workflow. */
    workflow: string;
  }

  /** Webhook payload for the `schedule` event. The schedule event has no additional payload properties beyond the common ones. */
  interface ScheduleEventPayload {
    /** The cron schedule that triggered the workflow. */
    schedule: string;
    /** The repository where the event occurred. */
    repository: GitHubRepository;
    /** The user that triggered the event (the repository owner for scheduled runs). */
    sender: GitHubUser;
  }

  interface GithubEventPayloadTypeMap {
    push: PushEventPayload,
    pull_request: PullRequestEventPayload,
    pull_request_target: PullRequestEventPayload,
    issue_comment: IssueCommentEventPayload,
    schedule: ScheduleEventPayload,
    workflow_dispatch: WorkflowDispatchEventPayload,
    workflow_call: {},
  }

  // ---- GitHub Context ----

  /**
   * The `github` context contains information about the workflow run and the event that triggered the run.
   *
   * @see https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#github-context
   */
  interface GitHubContextData<Triggers> {
    /**
     * The name of the action currently running, or the `id` of a step.
     * GitHub removes special characters, and uses the name `__run` when the current step runs a script without an `id`.
     * If you use the same action more than once in the same job, the name will include a suffix with the sequence number
     * with underscore before it (e.g. `actionscheckout2`).
     */
    action: string;

    /**
     * The path where an action is located. This property is only supported in composite actions.
     * You can use this path to access files located in the same repository as the action.
     */
    action_path: string;

    /**
     * For a step executing an action, this is the ref of the action being executed. For example, `v2`.
     */
    action_ref: string;

    /**
     * For a step executing an action, this is the owner and repository name of the action. For example, `actions/checkout`.
     */
    action_repository: string;

    /**
     * For a composite action, the current result of the composite action.
     */
    action_status: string;

    /**
     * The username of the user that triggered the initial workflow run.
     * If the workflow run is a re-run, this value may differ from `github.triggering_actor`.
     */
    actor: string;

    /**
     * The account ID of the person or app that triggered the initial workflow run. For example, `1234567`.
     */
    actor_id: string;

    /**
     * The URL of the GitHub REST API.
     */
    api_url: string;

    /**
     * The `base_ref` or target branch of the pull request in a workflow run.
     * This property is only available when the event that triggers a workflow run is either `pull_request` or `pull_request_target`.
     */
    base_ref: string;

    /**
     * Path on the runner to the file that sets environment variables from workflow commands.
     * This file is unique to the current step and is a different file for each step in a job.
     */
    env: string;

    /**
     * The full event webhook payload. This object is identical to the webhook payload of the event
     * that triggered the workflow run, and is different for each event.
     */
    event: {
      [K in keyof GithubEventPayloadTypeMap as K extends keyof Triggers ? K : never]:
      GithubEventPayloadTypeMap[K]
    };

    /**
     * The name of the event that triggered the workflow run.
     */
    event_name: string;

    /**
     * The path to the file on the runner that contains the full event webhook payload.
     */
    event_path: string;

    /**
     * The URL of the GitHub GraphQL API.
     */
    graphql_url: string;

    /**
     * The `head_ref` or source branch of the pull request in a workflow run.
     * This property is only available when the event that triggers a workflow run is either `pull_request` or `pull_request_target`.
     */
    head_ref: string;

    /**
     * The `job_id` of the current job.
     * Note: This context property is set by the Actions runner, and is only available within the execution `steps` of a job.
     */
    job: string;

    /**
     * Path on the runner to the file that sets system `PATH` variables from workflow commands.
     * This file is unique to the current step and is a different file for each step in a job.
     */
    path: string;

    /**
     * The fully-formed ref of the branch or tag that triggered the workflow run.
     * For workflows triggered by `push`, this is the branch or tag ref that was pushed.
     * For workflows triggered by `pull_request`, this is the pull request merge branch.
     * For example, `refs/heads/feature-branch-1`.
     */
    ref: string;

    /**
     * The short ref name of the branch or tag that triggered the workflow run.
     * This value matches the branch or tag name shown on GitHub. For example, `feature-branch-1`.
     */
    ref_name: string;

    /**
     * `true` if branch protections or rulesets are configured for the ref that triggered the workflow run.
     */
    ref_protected: boolean;

    /**
     * The type of ref that triggered the workflow run. Valid values are `branch` or `tag`.
     */
    ref_type: string;

    /**
     * The owner and repository name. For example, `octocat/Hello-World`.
     */
    repository: string;

    /**
     * The ID of the repository. For example, `123456789`.
     */
    repository_id: string;

    /**
     * The repository owner's username. For example, `octocat`.
     */
    repository_owner: string;

    /**
     * The repository owner's account ID. For example, `1234567`.
     */
    repository_owner_id: string;

    /**
     * The Git URL to the repository. For example, `git://github.com/octocat/hello-world.git`.
     */
    repositoryUrl: string;

    /**
     * The number of days that workflow run logs and artifacts are kept.
     */
    retention_days: string;

    /**
     * A unique number for each workflow run within a repository. This number does not change if you re-run the workflow run.
     */
    run_id: string;

    /**
     * A unique number for each run of a particular workflow in a repository.
     * This number begins at 1 for the workflow's first run, and increments with each new run.
     */
    run_number: string;

    /**
     * A unique number for each attempt of a particular workflow run in a repository.
     * This number begins at 1 for the workflow run's first attempt, and increments with each re-run.
     */
    run_attempt: string;

    /**
     * The source of a secret used in a workflow. Possible values are `None`, `Actions`, `Codespaces`, or `Dependabot`.
     */
    secret_source: string;

    /**
     * The URL of the GitHub server. For example: `https://github.com`.
     */
    server_url: string;

    /**
     * The commit SHA that triggered the workflow. For example, `ffac537e6cbbf934b08745a378932722df287a53`.
     */
    sha: string;

    /**
     * A token to authenticate on behalf of the GitHub App installed on your repository.
     * This is functionally equivalent to the `GITHUB_TOKEN` secret.
     * Note: This context property is set by the Actions runner, and is only available within the execution `steps` of a job.
     */
    token: string;

    /**
     * The username of the user that initiated the workflow run.
     * If the workflow run is a re-run, this value may differ from `github.actor`.
     */
    triggering_actor: string;

    /**
     * The name of the workflow. If the workflow file doesn't specify a `name`,
     * the value of this property is the full path of the workflow file in the repository.
     */
    workflow: string;

    /**
     * The ref path to the workflow. For example, `octocat/hello-world/.github/workflows/my-workflow.yml@refs/heads/my_branch`.
     */
    workflow_ref: string;

    /**
     * The commit SHA for the workflow file.
     */
    workflow_sha: string;

    /**
     * The default working directory on the runner for steps, and the default location of your repository
     * when using the `checkout` action.
     */
    workspace: string;
  }

  // ---- Workflow Context ----

  interface GitHubContext<Triggers> {
    github: GitHubContextData<Triggers>;
  }

  interface VarsContext {
    vars: Record<string, string>;
  }

  interface SecretsContext {
    secrets: Record<string, string>;
  }

  interface EnvContext {
    env: Record<string, string>;
  }

  interface RunnerContext {
    runner: {
      /** The name of the runner executing the job. */
      name: string;

      /** The operating system of the runner executing the job. Possible values are `Linux`, `Windows`, or `macOS`. */
      os: "Linux" | "Windows" | "macOS";

      /** The architecture of the runner executing the job. Possible values are `X86`, `X64`, `ARM`, or `ARM64`. */
      arch: "X86" | "X64" | "ARM" | "ARM64";

      /** The path to a temporary directory on the runner. This directory is emptied at the beginning and end of each job. */
      temp: string;

      /** The path to the directory containing preinstalled tools for GitHub-hosted runners. */
      tool_cache: string;

      /** Set only if debug logging is enabled. Always has the value of `1`. */
      debug: string;

      /** The environment of the runner. Possible values are `github-hosted` or `self-hosted`. */
      environment: "github-hosted" | "self-hosted";
    };
  }

  interface JobContext {
    job: {
      /** The check run ID of the current job. */
      check_run_id: number;

      /** Information about the job's container. */
      container: {
        /** The ID of the container. */
        id: string;
        /** The ID of the container network. */
        network: string;
      };

      /** The service containers created for a job. */
      services: Record<string, {
        /** The ID of the service container. */
        id: string;
        /** The ID of the service container network. */
        network: string;
        /** The exposed ports of the service container. */
        ports: Record<string, string>;
      }>;

      /** The current status of the job. Possible values are `success`, `failure`, or `cancelled`. */
      status: "success" | "failure" | "cancelled";
    };
  }

  interface StrategyContext {
    strategy: {
      /** When this evaluates to `true`, all in-progress jobs are canceled if any job in a matrix fails. */
      fail_fast: boolean,

      /** The index of the current job in the matrix. **Note:** This number is a zero-based number. The first job's index in the matrix is `0`. */
      job_index: number,

      /** The total number of jobs in the matrix. */
      job_total: number,

      /** The maximum number of jobs that can run simultaneously when using a `matrix` job strategy. */
      max_parallel: number,
    }
  }

  interface MatrixContext<Matrix> {
    matrix: Matrix,
  }

  /** Recursively converts leaf values to `string`, preserving object structure. */
  type Stringify<T> =
    T extends Record<string, unknown>
    ? { [K in Extract<keyof T, string>]: Stringify<T[K]> }
    : string;

  /** Extracts the combined key set from a matrix definition (regular keys + include keys). */
  type MatrixTypeKeys<T> =
    | Exclude<Extract<keyof T, string>, "include">
    | (T extends { include: (infer E)[] } ? Extract<keyof E, string> : never);

  /** Resolves the value type for a single matrix key. */
  type MatrixValueForKey<T, K extends string> =
    K extends Exclude<keyof T, "include">
    ? T[K] extends readonly (infer E)[] ? Stringify<E> : string
    : T extends { include: readonly (infer E)[] }
    ? E extends Record<string, unknown>
    ? K extends keyof E ? Stringify<E[K]> : string
    : string
    : string;

  /** Maps a matrix definition to a record keyed by the combined keys, with structured value types. */
  type MatrixType<T> = { [K in MatrixTypeKeys<T>]: MatrixValueForKey<T, K> };


  /** Extract the inputs record from either workflow_dispatch or workflow_call triggers. */
  type ExtractInputs<Triggers> =
    Triggers extends { workflow_dispatch: { inputs: infer I } } ? I :
    Triggers extends { workflow_call: { inputs: infer I } } ? I :
    never;

  /** Map an inputs record to the typed inputs context. */
  type MapInputs<Inputs> = {
    [K in keyof Inputs as Inputs[K] extends BaseInput<true> ? K : never]-?:
    Inputs[K] extends ChoiceInput<infer O, true> ? Required<O[number]> :
    Inputs[K] extends StringInput<true> ? string :
    Inputs[K] extends NumberInput<true> ? string :
    Inputs[K] extends BooleanInput<true> ? "true" | "false" : never;
  } & {
    [K in keyof Inputs as Inputs[K] extends BaseInput<false> ? K : never]?:
    Inputs[K] extends ChoiceInput<infer O, false> ? O[number] :
    Inputs[K] extends StringInput<false> ? string :
    Inputs[K] extends NumberInput<false> ? string :
    Inputs[K] extends BooleanInput<false> ? "true" | "false" : never;
  };

  interface InputsContext<Triggers> {
    /** The inputs provided to the workflow. */
    inputs: ExtractInputs<Triggers> extends never ? {} : MapInputs<ExtractInputs<Triggers>>;
  }

  type ValueOrFactory<Result, Context> =
    Result | ((ctx: Context) => Result);

  type Digit = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9";
  type LowerAlpha =
    | "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m"
    | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z";
  type Allowed = LowerAlpha | Digit | "_";

  /** Helper to convert any string to an id consisting of `[a-z0-9_]`. Dashes become underscores, other non-allowed characters are stripped. */
  type NormalizeId<S extends string> =
    S extends `${infer C}${infer Rest}`
    ? C extends "-" | " "
    ? `_${NormalizeId<Rest>}`
    : `${Lowercase<C> extends Allowed ? Lowercase<C> : ""}${NormalizeId<Rest>}`
    : "";

  /** Return type of `jobs()`: void normally, required output map when `workflow_call.outputs` is declared. */
  type WorkflowJobsReturn<Triggers> =
    Triggers extends { workflow_call: { outputs: infer O } }
    ? keyof O extends never ? void : { [K in Extract<keyof O, string>]: string }
    : void;

  type WorkflowRunNameContext<Triggers> = GitHubContext<Triggers> & InputsContext<Triggers> & VarsContext;
  type WorkflowConcurrencyContext<Triggers> = GitHubContext<Triggers> & InputsContext<Triggers> & VarsContext;
  type WorkflowEnvContext<Triggers> = GitHubContext<Triggers> & SecretsContext & InputsContext<Triggers> & VarsContext;

  interface Workflow<Triggers> {
    /** Determines when and how the workflow will be triggered. */
    on: Triggers;

    /** Used as the name in the GitHub Actions run list UI. */
    run_name?: ValueOrFactory<string, WorkflowRunNameContext<Triggers>>;

    /** Defaults for different kinds of steps. */
    defaults?: Defaults;

    /** Concurrency settings. */
    concurrency?: ValueOrFactory<Concurrency, WorkflowConcurrencyContext<Triggers>>;

    /** Additional environment variables for all jobs in this workflow. */
    env?: ValueOrFactory<EnvMap, WorkflowEnvContext<Triggers>>;

    /** Permissions for the generated GitHub token. */
    permissions?: "write-all" | "read-all" | ScopedPermissions;

    jobs(ctx: WorkflowJobsContext<Triggers>): WorkflowJobsReturn<Triggers>;
  }

  interface ScopedPermissions {
    contents?: PermissionLevel;
    pull_requests?: PermissionLevel;
    packages?: PermissionLevel;
    id_token?: PermissionLevel;
    deployments?: PermissionLevel;
    actions?: PermissionLevel;
    attestations?: PermissionLevel;
  }

  type PermissionLevel = "read" | "write" | "none";

  type Concurrency = string | {
    group: string;
    cancel_in_progress?: boolean;
  };

  type EnvMap = Record<string, string>;

  interface Defaults {
    run?: RunDefaults;
  }

  interface RunDefaults {
    shell?: string;
    working_directory?: string;
  }

  type Branches = string[];

  interface FilteredTrigger {
    /**
     * Branch patterns to filter for.
     *
     * Accepts glob patterns (`*`, `**`, `+`, `?`, `!`). Examples:
     * - `main`: matches exactly `main`
     * - `releases/**`: matches anything starting with `releases/`
     * - `!test/**`: anything starting with `test/` will _not_ trigger the workflow
     *
     * May be combined with `tags` and `paths`.
     */
    branches?: string[];

    /**
     * Tag patterns to filter for.
     *
     * Accepts glob patterns (`*`, `**`, `+`, `?`, `!`). Examples:
     * - `v*`: anything starting with `v` will trigger the workflow (`v0.1.0`, `v100.0.0`, etc.)
     * - `!sample-tag/**`: anything starting with `sample-tag/` will _not_ trigger the workflow
     *
     * May be combined with `branches` and `paths`.
     */
    tags?: string[];

    /**
     * Path patterns to filter for.
     *
     * Accepts glob patterns (`*`, `**`, `+`, `?`, `!`). Examples:
     * - `crates/**`: anything starting with `crates/` will trigger the workflow (`crates/a`, `crates/b/d`, etc.)
     * - `!tests/**`: anything starting with `tests/` will _not_ trigger the workflow
     *
     * May be combined with `branches` and `tags`.
     */
    paths?: string[];
  }

  type BaseInput<Required extends boolean> = {
    required?: Required;
    description?: string;
  };

  type BooleanInput<Required extends boolean> = BaseInput<Required> & { type: "boolean"; default?: boolean };
  type StringInput<Required extends boolean> = BaseInput<Required> & { type: "string"; default?: string };
  type NumberInput<Required extends boolean> = BaseInput<Required> & { type: "number"; default?: number };
  type ChoiceInput<O extends readonly [string, ...string[]], Required extends boolean> = BaseInput<Required> & {
    type: "choice";
    options: O;
    default?: O[number];
  };

  interface Trigger<Inputs = {}> {
    push?: string[] | FilteredTrigger;
    pull_request?: string[] | Omit<FilteredTrigger, "tags">;
    pull_request_target?: string[] | Omit<FilteredTrigger, "tags">;
    issue_comment?: {
      types?: ("created" | "deleted" | "edited" | "pinned" | "unpinned")[];
    };
    schedule?: {
      /** A cron expression, like `0 0 * * *`. */
      cron: string;
    }[];
    workflow_dispatch?: {
      /** Workflow inputs. Maximum number of inputs is 25. */
      inputs?: Inputs;
    };
    workflow_call?: {
      /** Inputs that the called workflow receives. */
      inputs?: Inputs;
      /** Outputs that the reusable workflow makes available to the caller. Values are derived from the `jobs()` return. */
      outputs?: Record<string, { description?: string }>;
      /** Secrets that the caller must or may pass. */
      secrets?: Record<string, { description?: string; required?: boolean }>;
    };
  }

  interface JobStrategy<Matrix> {
    matrix: Matrix,
    fail_fast?: boolean,
    max_parallel?: number,
  }

  // TODO: typed expressions
  type Expression = string;

  interface StepRef<Outputs> {
    outputs: Outputs;
  }

  type JobStrategyContext<Triggers, Needs> = GitHubContext<Triggers> & NeedsContext<Needs> & VarsContext & InputsContext<Triggers>;

  type JobIfContext<Triggers, Needs> = GitHubContext<Triggers> & NeedsContext<Needs> & VarsContext & InputsContext<Triggers>;

  type JobRunsOnContext<Triggers, Needs, Matrix> = GitHubContext<Triggers> & NeedsContext<Needs> & StrategyContext & MatrixContext<Matrix> & VarsContext & InputsContext<Triggers>;

  type JobEnvContext<Triggers, Needs, Matrix> = JobRunsOnContext<Triggers, Needs, Matrix> & SecretsContext;

  // TODO: steps can reference other steps
  type StepsContext<Triggers, Needs, Matrix> =
    GitHubContext<Triggers>
    & NeedsContext<Needs>
    & StrategyContext
    & MatrixContext<Matrix>
    & JobContext
    & RunnerContext
    & EnvContext
    & InputsContext<Triggers>
    & VarsContext
    & SecretsContext;

  /** Extract the output keys from a `workflow_call` trigger as `{ key: string }`. */
  type ExtractOutputs<Triggers> =
    Triggers extends { workflow_call: { outputs: infer O } }
    ? { [K in Extract<keyof O, string>]: string }
    : {};

  type ExtractRequiredInputs<Triggers> =
    Triggers extends { workflow_call: { inputs: infer Inputs } }
    ? { [K in keyof Inputs as Inputs[K] extends BaseInput<true> ? K : never]: any }
    : {};

  /** A reference to a workflow that has `workflow_call` trigger, usable with `ctx.uses()`. */
  interface WorkflowRef<Triggers> {
    /** @internal */ readonly __brand: unique symbol;
  }

  /** Extract the `with` type for calling a reusable workflow. */
  type WorkflowCallWith<Inputs> = {
    [K in keyof Inputs as Inputs[K] extends BaseInput<true> ? K : never]-?:
    Inputs[K] extends BooleanInput<any> ? boolean :
    Inputs[K] extends NumberInput<any> ? number :
    Inputs[K] extends StringInput<any> ? string :
    Inputs[K] extends ChoiceInput<infer O, any> ? O[number] : unknown;
  } & {
    [K in keyof Inputs as Inputs[K] extends BaseInput<false> ? K : never]?:
    Inputs[K] extends BooleanInput<any> ? boolean :
    Inputs[K] extends NumberInput<any> ? number :
    Inputs[K] extends StringInput<any> ? string :
    Inputs[K] extends ChoiceInput<infer O, any> ? O[number] : unknown;
  };

  interface Job<
    Triggers, // propagates workflow.workflow_dispatch.inputs
    Needs, // inferred to type of `needs: [...]`, which is a tuple of job refs
    Outputs, // the output type of this job, comes from `steps`
    Matrix // inferred type of matrix strategy
  > {
    runs_on: ValueOrFactory<string | string[], JobRunsOnContext<Triggers, Needs, Matrix>>;
    needs?: Needs;
    strategy?: ValueOrFactory<Matrix | JobStrategy<Matrix>, JobStrategyContext<Triggers, Needs>>;
    if?: ValueOrFactory<Expression, JobIfContext<Triggers, Needs>>;
    env?: ValueOrFactory<Record<string, string>, JobEnvContext<Triggers, Needs, Matrix>>;
    timeout_minutes?: number;

    steps(ctx: StepsContext<Triggers, Needs, Matrix>): Outputs | void;
  }

  interface JobRef<Id extends string, Outputs> {
    id: Id;
    outputs: { [K in Extract<keyof Outputs, string>]: string };
  }

  /** Maps a tuple of JobRefs into a `{ needs: { <id>: { outputs: <Outputs> } } }` context. */
  type NeedsContext<T> = T extends JobRef<string, unknown>[]
    ? {
      needs: {
        [R in T[number]as R["id"]]: {
          outputs: R extends JobRef<string, infer O> ? O : never;
        };
      };
    }
    : {};

  function triggers<T extends Trigger<Inputs>, Inputs extends Record<string, any>>(trigger: T): T;

  function input<const Req extends boolean = false>(type: "boolean", options?: Omit<BooleanInput<Req>, "type">): BooleanInput<Req>;
  function input<const Req extends boolean = false>(type: "string", options?: Omit<StringInput<Req>, "type">): StringInput<Req>;
  function input<const Req extends boolean = false>(type: "number", options?: Omit<NumberInput<Req>, "type">): NumberInput<Req>;
  function input<
    const O extends readonly [string, ...string[]],
    const Req extends boolean = false,
  >(
    type: "choice", options: Omit<ChoiceInput<O, Req>, "type">
  ): ChoiceInput<O, Req>;

  type MatrixInput = { include?: Record<string, unknown>[] } & Record<string, unknown[]>;

  function matrix<const T extends MatrixInput>(def: T): MatrixType<T>;

  interface WorkflowCallJobOptions<Triggers, UsesTriggers, Needs> {
    needs?: Needs;
    if?: ValueOrFactory<Expression, JobIfContext<Triggers, Needs>>;
    with?: WorkflowCallWith<ExtractInputs<UsesTriggers>>;
    secrets?: Record<string, string> | "inherit";
  }

  interface WorkflowJobsContext<Triggers> extends GitHubContext<Triggers>, InputsContext<Triggers>, VarsContext {
    // returns a `JobRef`, which can be used as the input to `needs` in other jobs, allowing them to use its outputs.
    job<
      const Name extends string,
      const Needs extends JobRef<string, unknown>[],
      const Outputs,
      const Matrix,
    >(name: Name, job: Job<Triggers, Needs, Outputs, Matrix>): JobRef<NormalizeId<Name>, Outputs>;

    /** Call a reusable workflow as a job. */
    uses<
      const Name extends string,
      const Needs extends JobRef<string, unknown>[],
      const UsesTriggers,
    >(
      name: Name,
      ref: WorkflowRef<UsesTriggers>,
      ...options: (keyof ExtractRequiredInputs<UsesTriggers>) extends never
        ? [WorkflowCallJobOptions<Triggers, UsesTriggers, Needs>?]
        : [WorkflowCallJobOptions<Triggers, UsesTriggers, Needs>],
    ): JobRef<NormalizeId<Name>, ExtractOutputs<UsesTriggers>>;
  }

  function workflow<const Triggers>(
    name: string,
    definition: Workflow<Triggers>,
  ): WorkflowRef<Triggers>;

  // ---- Step builtins ----

  interface StepOptions {
    name?: string;
    if?: Expression;
    env?: Record<string, string>;
    timeout_minutes?: number;
    continue_on_error?: boolean;
  }

  interface RunOptions extends StepOptions {
    shell?: string;
    working_directory?: string;
  }

  // We can't return typed outputs from `run`, so return a generic record.
  function run(script: string, options?: RunOptions): StepRef<Record<string, string>>;

  // ---- Step options ----

  interface UsesOptions<Inputs> extends StepOptions {
    with?: Inputs;
  }

  interface UsesOptionsRequired<Inputs> extends StepOptions {
    with: Inputs;
  }

  // ---- uses: per-action overloads (generated) ----

  /**
   * Action not found. Run `ghat add <action>` to add it to the lockfile
   * and generate its type definitions.
   */
  function uses(action: never): never;
}

export { };
