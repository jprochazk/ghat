use std::collections::HashMap;

use miette::{Context, Diagnostic, IntoDiagnostic};

/// Run the type-checker on the project's tsconfig.json.
///
/// Returns `Err` if any diagnostics have category `Error`.
pub fn typecheck() -> miette::Result<()> {
    let tsconfig = super::common::base_dir().join("tsconfig.json");
    miette::ensure!(
        tsconfig.exists(),
        "tsconfig.json not found (run `ghat init` first)"
    );

    let diagnostics = tsgo::check_project(&tsconfig)
        .into_diagnostic()
        .wrap_err("type-check failed")?;

    if diagnostics.is_empty() {
        return Ok(());
    }

    let has_errors = diagnostics
        .iter()
        .any(|d| matches!(d.category, tsgo::DiagnosticCategory::Error));

    let errors = TypeCheckErrors::from_diagnostics(diagnostics);

    if has_errors {
        Err(miette::Error::new(errors))
    } else {
        // Print warnings/suggestions without failing
        let handler = miette::GraphicalReportHandler::new();
        let mut buf = String::new();
        handler
            .render_report(&mut buf, &errors)
            .expect("BUG: failed to render diagnostics");
        eprint!("{buf}");
        Ok(())
    }
}

#[derive(Debug)]
struct TypeCheckErrors {
    errors: Vec<TypeCheckError>,
}

impl TypeCheckErrors {
    fn from_diagnostics(diagnostics: Vec<tsgo::Diagnostic>) -> Self {
        // Read each source file once
        let mut sources: HashMap<String, Option<miette::NamedSource<String>>> = HashMap::new();

        let errors = diagnostics
            .into_iter()
            .map(|d| {
                let source = d.file.as_deref().and_then(|path| {
                    sources
                        .entry(path.to_owned())
                        .or_insert_with(|| {
                            let content = std::fs::read_to_string(path).ok()?;
                            Some(miette::NamedSource::new(path, content))
                        })
                        .clone()
                });

                let span = source.as_ref().map(|src| {
                    let content = src.inner();
                    line_col_to_span(content, d.line, d.column, d.end_line, d.end_column)
                });

                TypeCheckError {
                    message: d.message,
                    code: d.code,
                    severity: match d.category {
                        tsgo::DiagnosticCategory::Error => miette::Severity::Error,
                        tsgo::DiagnosticCategory::Warning => miette::Severity::Warning,
                        tsgo::DiagnosticCategory::Suggestion | tsgo::DiagnosticCategory::Message => {
                            miette::Severity::Advice
                        }
                    },
                    source,
                    span,
                }
            })
            .collect();

        Self { errors }
    }
}

impl std::fmt::Display for TypeCheckErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type-check failed with {} error(s)", self.errors.len())
    }
}

impl std::error::Error for TypeCheckErrors {}

impl Diagnostic for TypeCheckErrors {
    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        Some(Box::new(self.errors.iter().map(|e| e as &dyn Diagnostic)))
    }
}

#[derive(Debug)]
struct TypeCheckError {
    message: String,
    code: i32,
    severity: miette::Severity,
    source: Option<miette::NamedSource<String>>,
    span: Option<miette::SourceSpan>,
}

impl std::fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TypeCheckError {}

impl Diagnostic for TypeCheckError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(format!("TS{}", self.code)))
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(self.severity)
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.source.as_ref().map(|s| s as &dyn miette::SourceCode)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.span
            .map(|span| -> Box<dyn Iterator<Item = miette::LabeledSpan>> {
                Box::new(std::iter::once(miette::LabeledSpan::underline(span)))
            })
    }
}

/// Convert 0-indexed line/column to a byte offset span.
fn line_col_to_span(
    source: &str,
    line: u32,
    col: u32,
    end_line: u32,
    end_col: u32,
) -> miette::SourceSpan {
    let offset = line_col_to_offset(source, line, col);
    let end = line_col_to_offset(source, end_line, end_col);
    miette::SourceSpan::new(offset.into(), (end.saturating_sub(offset)).max(1).into())
}

fn line_col_to_offset(source: &str, line: u32, col: u32) -> usize {
    let mut offset = 0;
    for (i, l) in source.lines().enumerate() {
        if i == line as usize {
            return offset + (col as usize).min(l.len());
        }
        offset += l.len() + 1; // +1 for newline
    }
    source.len()
}

/// CLI entry point for `ghat check`.
pub fn run() -> miette::Result<()> {
    typecheck()?;
    let _workflows = super::common::eval_workflow_definitions()?;
    eprintln!("check passed");
    Ok(())
}
