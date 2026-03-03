use std::path::PathBuf;

use miette::{Context, IntoDiagnostic};

const TEMPLATE: &str = r#"workflow("<WORKFLOW_NAME>", {
  on: triggers({
    push: ["main"],
  }),

  jobs(ctx) {
    ctx.job("Build", {
      runs_on: "ubuntu-latest",

      steps() {
        run("echo hello")
      },
    })
  },
})
"#;

fn prompt_name() -> miette::Result<String> {
    let name: String = dialoguer::Input::new()
        .with_prompt("Workflow name")
        .default("ci".into())
        .validate_with(|input: &String| validate_name(input))
        .interact_text()
        .into_diagnostic()?;
    Ok(name)
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("name cannot be empty".into());
    }
    if name.starts_with('_') {
        return Err("names starting with _ are reserved for utility files".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("name must contain only letters, digits, hyphens, and underscores".into());
    }
    Ok(())
}

pub fn run(name: Option<String>) -> miette::Result<()> {
    let name = match name {
        Some(name) => {
            validate_name(&name).map_err(|e| miette::miette!("{e}"))?;
            name
        }
        None => prompt_name()?,
    };

    let workflows_dir = PathBuf::from(super::common::BASE_DIR).join("workflows");
    miette::ensure!(
        workflows_dir.exists(),
        "workflows directory not found: {}\n\nRun `ghat init` first.",
        workflows_dir.display()
    );

    let file_name = format!("{name}.ts");
    let output_path = workflows_dir.join(&file_name);

    if output_path.exists() {
        return Err(miette::miette!("{} already exists", output_path.display()));
    }

    let workflow_name = name
        .split(['_', '-'])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    format!("{upper}{}", chars.as_str())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let contents = TEMPLATE.replace("<WORKFLOW_NAME>", &workflow_name);

    std::fs::write(&output_path, &contents)
        .into_diagnostic()
        .wrap_err("failed to write workflow file")?;

    super::style::status("Created", output_path.display());

    Ok(())
}
