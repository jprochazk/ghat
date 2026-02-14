pub mod oxc;
pub mod runtime;
pub mod workflow;

pub mod cli;

fn main() -> miette::Result<()> {
    cli::entrypoint()
}
