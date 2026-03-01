use std::path::PathBuf;

use miette::{Context, IntoDiagnostic};

pub fn run(no_check: bool) -> miette::Result<()> {
    if !no_check {
        super::check::typecheck()?;
    }

    let workflows = super::common::eval_workflow_definitions()?;

    let output_dir = PathBuf::from(".github/workflows");
    std::fs::create_dir_all(&output_dir)
        .into_diagnostic()
        .wrap_err("failed to create directory")?;

    for (name, workflow) in workflows {
        let yaml = serde_yaml_ng::to_string(&workflow)
            .into_diagnostic()
            .wrap_err("failed to serialize workflow")?;
        let output_path = output_dir.join(format!("generated_{name}.yaml"));
        std::fs::write(&output_path, &yaml)
            .into_diagnostic()
            .wrap_err("failed to write workflow file")?;
        log::info!("wrote {}", output_path.display());
    }

    Ok(())
}
