pub mod cli;
pub mod codegen;
pub mod github;
pub mod lockfile;
pub mod oxc;
pub mod runtime;
pub mod workflow;

fn miette_hook() {
    miette::set_hook(Box::new(|_| {
        if std::env::var("__GHAT_TEST").ok().is_some() {
            Box::new(miette::GraphicalReportHandler::new_themed(
                miette::GraphicalTheme::none(),
            ))
        } else {
            Box::new(miette::GraphicalReportHandler::new_themed(
                miette::GraphicalTheme::unicode(),
            ))
        }
    }))
    .expect("failed to set miette hook");
}

fn main() -> miette::Result<()> {
    miette_hook();

    cli::entrypoint()
}
