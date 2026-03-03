
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
      strategy: matrix({
        include: [
          { runner: "ubuntu-24.04", target: "" },
          { runner: "ubuntu-24.04-arm", target: "" },
          { runner: "macos-26", target: "" },
          { runner: "windows-2025", target: "x86_64-pc-windows-gnu" },
        ]
      }),

      runs_on: (ctx) => ctx.matrix.runner,

      steps(ctx) {
        uses("actions/checkout")
        uses("Swatinem/rust-cache")
        uses("dtolnay/rust-toolchain", {
          with: { toolchain: "stable", targets: ctx.matrix.target }
        })
        run("cargo test")
      }
    })
  }
})

