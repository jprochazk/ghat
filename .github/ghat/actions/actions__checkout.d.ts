declare global {
  /**
   * Checkout a Git repository at a particular version
   * @see https://github.com/actions/checkout
   */
  function uses(
    action: "actions/checkout",
    options?: UsesOptions<{
    /** Repository name with owner. For example, actions/checkout */
    repository?: string;
    /** The branch, tag or SHA to checkout. When checking out the repository that triggered a workflow, this defaults to the reference or SHA for that event.  Otherwise, uses the default branch. */
    ref?: string;
    /** Personal access token (PAT) used to fetch the repository. The PAT is configured with the local git config, which enables your scripts to run authenticated git commands. The post-job step removes the PAT.

We recommend using a service account with the least permissions necessary. Also when generating a new PAT, select the least scopes necessary.

[Learn more about creating and using encrypted secrets](https://help.github.com/en/actions/automating-your-workflow-with-github-actions/creating-and-using-encrypted-secrets) */
    token?: string;
    /** SSH key used to fetch the repository. The SSH key is configured with the local git config, which enables your scripts to run authenticated git commands. The post-job step removes the SSH key.

We recommend using a service account with the least permissions necessary.

[Learn more about creating and using encrypted secrets](https://help.github.com/en/actions/automating-your-workflow-with-github-actions/creating-and-using-encrypted-secrets) */
    ssh_key?: string;
    /** Known hosts in addition to the user and global host key database. The public SSH keys for a host may be obtained using the utility `ssh-keyscan`. For example, `ssh-keyscan github.com`. The public key for github.com is always implicitly added. */
    ssh_known_hosts?: string;
    /** Whether to perform strict host key checking. When true, adds the options `StrictHostKeyChecking=yes` and `CheckHostIP=no` to the SSH command line. Use the input `ssh-known-hosts` to configure additional hosts. */
    ssh_strict?: string;
    /** The user to use when connecting to the remote SSH host. By default 'git' is used. */
    ssh_user?: string;
    /** Whether to configure the token or SSH key with the local git config */
    persist_credentials?: string;
    /** Relative path under $GITHUB_WORKSPACE to place the repository */
    path?: string;
    /** Whether to execute `git clean -ffdx && git reset --hard HEAD` before fetching */
    clean?: string;
    /** Partially clone against a given filter. Overrides sparse-checkout if set. */
    filter?: string;
    /** Do a sparse checkout on given patterns. Each pattern should be separated with new lines. */
    sparse_checkout?: string;
    /** Specifies whether to use cone-mode when doing a sparse checkout. */
    sparse_checkout_cone_mode?: string;
    /** Number of commits to fetch. 0 indicates all history for all branches and tags. */
    fetch_depth?: string;
    /** Whether to fetch tags, even if fetch-depth > 0. */
    fetch_tags?: string;
    /** Whether to show progress status output when fetching. */
    show_progress?: string;
    /** Whether to download Git-LFS files */
    lfs?: string;
    /** Whether to checkout submodules: `true` to checkout submodules or `recursive` to recursively checkout submodules.

When the `ssh-key` input is not provided, SSH URLs beginning with `git@github.com:` are converted to HTTPS. */
    submodules?: string;
    /** Add repository path as safe.directory for Git global config by running `git config --global --add safe.directory <path>` */
    set_safe_directory?: string;
    /** The base URL for the GitHub instance that you are trying to clone from, will use environment defaults to fetch from the same instance that the workflow is running from unless specified. Example URLs are https://github.com or https://my-ghes-server.example.com */
    github_server_url?: string;
    }>,
  ): StepRef<{
    /** The branch, tag or SHA that was checked out */
    ref: string;
    /** The commit SHA that was checked out */
    commit: string;
  }>;
}
export {};
