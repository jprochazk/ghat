globalThis.__GHAT_ACTION_MAPPINGS = {
  "Swatinem/rust-cache": {
    inputs: {
      "prefix_key": "prefix-key",
      "shared_key": "shared-key",
      "add_job_id_key": "add-job-id-key",
      "add_rust_environment_hash_key": "add-rust-environment-hash-key",
      "env_vars": "env-vars",
      "cache_directories": "cache-directories",
      "cache_targets": "cache-targets",
      "cache_on_failure": "cache-on-failure",
      "cache_all_crates": "cache-all-crates",
      "cache_workspace_crates": "cache-workspace-crates",
      "save_if": "save-if",
      "cache_provider": "cache-provider",
      "cache_bin": "cache-bin",
      "lookup_only": "lookup-only"
    },
    outputs: {
      "cache_hit": "cache-hit"
    },
  },
  "actions/checkout": {
    inputs: {
      "ssh_key": "ssh-key",
      "ssh_known_hosts": "ssh-known-hosts",
      "ssh_strict": "ssh-strict",
      "ssh_user": "ssh-user",
      "persist_credentials": "persist-credentials",
      "sparse_checkout": "sparse-checkout",
      "sparse_checkout_cone_mode": "sparse-checkout-cone-mode",
      "fetch_depth": "fetch-depth",
      "fetch_tags": "fetch-tags",
      "show_progress": "show-progress",
      "set_safe_directory": "set-safe-directory",
      "github_server_url": "github-server-url"
    },
  },
};
