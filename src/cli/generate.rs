use std::path::PathBuf;
use std::time::Instant;

use miette::{Context, IntoDiagnostic};

use super::style::status;
use crate::lockfile::Lockfile;
use crate::workflow::Workflow;

pub fn run(no_check: bool) -> miette::Result<()> {
    let start = Instant::now();

    if !no_check {
        status("Checking", "workflow definitions");
        super::check::typecheck()?;
    }

    let (_, lockfile) = super::common::load_lockfile()?;

    status("Evaluating", "workflow definitions");
    let mut workflows = super::common::eval_workflow_definitions()?;

    for (_name, workflow) in &mut workflows {
        pin_actions(workflow, &lockfile)?;
    }

    let output_dir = PathBuf::from(".github/workflows");
    std::fs::create_dir_all(&output_dir)
        .into_diagnostic()
        .wrap_err("failed to create directory")?;

    let count = workflows.len();
    for (name, workflow) in workflows {
        let yaml = serde_yaml_ng::to_string(&workflow)
            .into_diagnostic()
            .wrap_err("failed to serialize workflow")?;
        let output_path = output_dir.join(format!("generated_{name}.yaml"));
        std::fs::write(&output_path, &yaml)
            .into_diagnostic()
            .wrap_err("failed to write workflow file")?;
        status("Wrote", output_path.display());
    }

    let elapsed = start.elapsed();
    status(
        "Finished",
        format!(
            "generated {count} workflow{s} in {elapsed:.2?}",
            s = if count == 1 { "" } else { "s" },
        ),
    );

    Ok(())
}

/// Rewrite `uses` fields in all steps to pin actions to their lockfile sha.
///
/// A step with `uses: "actions/checkout"` becomes
/// `uses: "actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683"`.
fn pin_actions(workflow: &mut Workflow, lockfile: &Lockfile) -> miette::Result<()> {
    for (_job_id, job) in &mut workflow.jobs {
        for step in &mut job.steps {
            let Some(action) = &step.uses else {
                continue;
            };

            // Skip actions that are already pinned (contain @)
            if action.contains('@') {
                continue;
            }

            let locked = lockfile.actions.get(action.as_str()).ok_or_else(|| {
                miette::miette!(
                    "action `{action}` is not in the lockfile\n\nRun `ghat add {action}` to add it."
                )
            })?;

            step.uses = Some(format!("{action}@{sha}", sha = locked.sha));
        }
    }
    Ok(())
}
