declare global {
  /**
   * Setup a Go environment and add it to the PATH
   * @see https://github.com/actions/setup-go
   */
  function uses(
    action: "actions/setup-go",
    options?: UsesOptions<{
    /** The Go version to download (if necessary) and use. Supports semver spec and ranges. Be sure to enclose this option in single quotation marks. */
    go_version?: string;
    /** Path to the go.mod, go.work, .go-version, or .tool-versions file. */
    go_version_file?: string;
    /** Set this option to true if you want the action to always check for the latest available version that satisfies the version spec */
    check_latest?: string;
    /** Used to pull Go distributions from go-versions. Since there's a default, this is typically not supplied by the user. When running this action on github.com, the default value is sufficient. When running on GHES, you can pass a personal access token for github.com if you are experiencing rate limiting. */
    token?: string;
    /** Used to specify whether caching is needed. Set to true, if you'd like to enable caching. */
    cache?: string;
    /** Used to specify the path to a dependency file (e.g., go.mod, go.sum) */
    cache_dependency_path?: string;
    /** Target architecture for Go to use. Examples: x86, x64. Will use system architecture by default. */
    architecture?: string;
    }>,
  ): StepRef<{
    /** The installed Go version. Useful when given a version range as input. */
    go_version: string;
    /** A boolean value to indicate if a cache was hit */
    cache_hit: string;
  }>;
}
export {};
