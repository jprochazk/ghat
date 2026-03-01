use std::path::Path;

use miette::{Context, IntoDiagnostic};

const API_DTS: &str = include_str!("../runtime/api.d.ts");
const WORKFLOW_DTS: &str = include_str!("../runtime/workflow.d.ts");

const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "isolatedModules": true,
    "isolatedDeclarations": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["workflows/**/*.ts", "types/**/*.d.ts", "actions/**/*.d.ts"]
}
"#;

pub fn run() -> miette::Result<()> {
    let base = Path::new(super::common::BASE_DIR);

    if base.exists() {
        log::info!("{} already exists", base.display());
    }

    let dirs = [
        base.join("types"),
        base.join("workflows"),
        base.join("actions"),
    ];
    for dir in &dirs {
        std::fs::create_dir_all(dir)
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to create directory: {}", dir.display()))?;
    }

    write_if_changed(&base.join("types/api.d.ts"), API_DTS)?;
    write_if_changed(&base.join("types/workflow.d.ts"), WORKFLOW_DTS)?;
    write_if_changed(&base.join("tsconfig.json"), TSCONFIG)?;

    let lockfile = base.join("ghat.lock");
    if !lockfile.exists() {
        std::fs::write(&lockfile, "")
            .into_diagnostic()
            .wrap_err("failed to create lockfile")?;
        log::info!("created {}", lockfile.display());
    }

    eprintln!("initialized ghat project in {}", base.display());
    Ok(())
}

/// Write a file only if the contents have changed (or it doesn't exist yet).
fn write_if_changed(path: &Path, contents: &str) -> miette::Result<()> {
    if path.exists() {
        let existing = std::fs::read_to_string(path)
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to read {}", path.display()))?;
        if existing == contents {
            log::debug!("{} is up to date", path.display());
            return Ok(());
        }
        log::info!("updating {}", path.display());
    } else {
        log::info!("creating {}", path.display());
    }

    std::fs::write(path, contents)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to write {}", path.display()))?;
    Ok(())
}
