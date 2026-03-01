use std::path::PathBuf;

use miette::{Context, IntoDiagnostic};

use crate::runtime::Runtime;

pub fn run() -> miette::Result<()> {
    let workflows_dir = PathBuf::from(".github/ghat/workflows");
    miette::ensure!(
        std::fs::exists(&workflows_dir).is_ok_and(|success| success),
        "workflows directory not found: {}",
        workflows_dir.display()
    );

    let mut builder = Runtime::builder();

    let mappings_path = super::common::base_dir().join("actions/mappings.js");
    match std::fs::read_to_string(&mappings_path) {
        Ok(s) => builder = builder.mappings(&s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            return Err(e)
                .into_diagnostic()
                .wrap_err("failed to read mappings.js")
        }
    };

    let rt = builder.build()?;

    let entries = std::fs::read_dir(&workflows_dir)
        .into_diagnostic()
        .wrap_err("failed to load directory")?;
    for entry in entries {
        let entry = entry
            .into_diagnostic()
            .wrap_err("failed to load workflow file")?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        if file_name.starts_with('_') {
            continue;
        }

        if file_name.ends_with(".ts") || file_name.ends_with(".js") {
            log::info!("evaluating workflow: {file_name}");
            rt.eval_workflow_definition(&entry.path())?;
        }
    }

    let workflows = rt.finish();
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
