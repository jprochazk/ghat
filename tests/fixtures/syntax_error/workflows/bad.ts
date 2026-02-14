workflow("Bad", {
  on: triggers({ push: ["main"] }),
  jobs(ctx) {
    ctx.job("Broken", {
      runs_on: "ubuntu-latest",
      steps() {
        this is not valid javascript at all !!!
      }
    })
  }
})
