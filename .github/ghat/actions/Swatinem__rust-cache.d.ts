declare global {
  /**
   * A GitHub Action that implements smart caching for rust/cargo projects with sensible defaults.
   * @see https://github.com/Swatinem/rust-cache
   */
  function uses(
    action: "Swatinem/rust-cache",
    options?: UsesOptions<{
    /** The prefix cache key, this can be changed to start a new cache manually. */
    prefix_key?: string;
    /** A cache key that is used instead of the automatic `job`-based key, and is stable over multiple jobs. */
    shared_key?: string;
    /** An additional cache key that is added alongside the automatic `job`-based cache key and can be used to further differentiate jobs. */
    key?: string;
    /** If the automatic `job`-based cache key should include the job id. Defaults to true. */
    add_job_id_key?: string;
    /** Weather the a hash of the rust environment should be included in the cache key. This includes a hash of all Cargo.toml/Cargo.lock files, rust-toolchain files, and .cargo/config.toml files (if present), as well as the specified 'env-vars'. Defaults to true. */
    add_rust_environment_hash_key?: string;
    /** Additional environment variables to include in the cache key, separated by spaces. */
    env_vars?: string;
    /** Paths to multiple Cargo workspaces and their target directories, separated by newlines. */
    workspaces?: string;
    /** Additional non workspace directories to be cached, separated by newlines. */
    cache_directories?: string;
    /** Determines whether workspace targets are cached. If `false`, only the cargo registry will be cached. */
    cache_targets?: string;
    /** Cache even if the build fails. Defaults to false. */
    cache_on_failure?: string;
    /** Determines which crates are cached. If `true` all crates will be cached, otherwise only dependent crates will be cached. */
    cache_all_crates?: string;
    /** Similar to cache-all-crates. If `true` the workspace crates will be cached. */
    cache_workspace_crates?: string;
    /** Determiners whether the cache should be saved. If `false`, the cache is only restored. */
    save_if?: string;
    /** Determines which provider to use for caching. Options are github, buildjet, or warpbuild. Defaults to github. */
    cache_provider?: string;
    /** Determines whether to cache ${CARGO_HOME}/bin. */
    cache_bin?: string;
    /** Check if a cache entry exists without downloading the cache */
    lookup_only?: string;
    }>,
  ): StepRef<{
    /** A boolean value that indicates an exact match was found. */
    cache_hit: string;
  }>;
}
export {};
