import { cargo_test } from "./_utils.ts"

workflow("CI", {
  on: triggers({ push: ["main"], pull_request: ["main"] }),
  jobs(ctx) {
    ctx.job("Test", {
      runs_on: "ubuntu-latest",
      steps() {
        cargo_test()
      }
    })
  }
})
