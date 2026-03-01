
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
      runs_on: "ubuntu-latest",

      steps() {
        uses("actions/checkout")
        uses("Swatinem/rust-cache")
        uses("dtolnay/rust-toolchain", { with: { toolchain: "stable" } })
        run("cargo test")
      }
    })
  }
})

