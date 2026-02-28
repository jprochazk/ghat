pub mod cli;
pub mod github;
pub mod lockfile;
pub mod oxc;
pub mod runtime;
pub mod workflow;

fn main() -> miette::Result<()> {
    cli::entrypoint()
}
