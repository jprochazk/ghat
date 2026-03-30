
// Feel free to edit this file, subsequent runs of `ghat init` won't regenerate it.

// For convenience, here are some configuration options:
/** The runner used for the check job. */
const RUNNER = "ubuntu-latest";
/** The download URL, in case you want to use a mirror instead. */
const DOWNLOAD_URL = "https://github.com/jprochazk/ghat/releases/latest/download/ghat-installer.sh"

workflow("ghat check", {
  on: triggers({
    push: [],
  }),

  jobs(ctx) {
    ctx.job("Check", {
      runs_on: RUNNER,

      steps() {
        uses("actions/checkout", {
            // We only need to look at the latest commit
            with: { fetch_depth: "1" }
        })
        run(`curl --proto '=https' --tlsv1.2 -LsSf ${DOWNLOAD_URL} | sh`)
        run("ghat generate")
        run("git diff --exit-code")
      },
    })
  },
})
