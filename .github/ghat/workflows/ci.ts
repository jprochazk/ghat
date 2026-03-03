
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
        uses("Swatinem/rust-cache")
        uses("dtolnay/rust-toolchain", {
          with: { toolchain: ctx.matrix.toolchain }
        })
        run("cargo test")
      }
    })
  }
})

