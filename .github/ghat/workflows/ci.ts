
workflow("CI", {
  on: triggers({
    push: ["main"],
    pull_request: [],
  }),

  permissions: {
    contents: "read"
  },

  jobs(ctx) {
    ctx.job("Test", {
      strategy: {
        fail_fast: false,
        matrix: matrix({
          include: [
            { runner: "ubuntu-24.04", toolchain: "stable" },
            { runner: "ubuntu-24.04-arm", toolchain: "stable" },
            { runner: "macos-26", toolchain: "stable" },
            { runner: "windows-2025", toolchain: "stable-x86_64-pc-windows-gnu" },
          ]
        })
      },

      runs_on: (ctx) => ctx.matrix.runner,

      steps(ctx) {
        uses("actions/checkout")
        uses("actions/setup-go", {
          with: { go_version: "1.26" }
        })
        uses("dtolnay/rust-toolchain", {
          with: { toolchain: ctx.matrix.toolchain }
        })
        const cache_result = uses("Swatinem/rust-cache", {
          with: { cache_on_failure: "true" }
        })
        run("cargo test")

        return { cache_hit: cache_result.outputs.cache_hit }
      }
    })
  }
})

