declare global {
  /**
   * Install the Rust toolchain
   * @see https://github.com/dtolnay/rust-toolchain
   */
  function uses(
    action: "dtolnay/rust-toolchain",
    options: UsesOptionsRequired<{
    /** Rust toolchain specification -- see https://rust-lang.github.io/rustup/concepts/toolchains.html#toolchain-specification */
    toolchain: string;
    /** Comma-separated list of target triples to install for this toolchain */
    targets?: string;
    /** Alias for `targets` */
    target?: string;
    /** Comma-separated list of components to be additionally installed */
    components?: string;
    }>,
  ): StepRef<{
    /** A short hash of the rustc version, appropriate for use as a cache key. "20220627a831" */
    cachekey: string;
    /** Rustup's name for the selected version of the toolchain. "1.62.0" */
    name: string;
  }>;
}
export {};
