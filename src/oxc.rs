use std::path::Path;

use miette::Diagnostic;
use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_codegen::Codegen;
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::Transformer;

pub fn allocator() -> Allocator {
    Allocator::new()
}

pub fn parse_js<'a>(alloc: &'a Allocator, src: &'a str) -> miette::Result<Program<'a>> {
    parse(alloc, src, SourceType::mjs())
}

pub fn parse_ts<'a>(alloc: &'a Allocator, src: &'a str) -> miette::Result<Program<'a>> {
    parse(alloc, src, SourceType::ts())
}

fn parse<'a>(
    alloc: &'a Allocator,
    src: &'a str,
    src_ty: SourceType,
) -> miette::Result<Program<'a>> {
    let result = Parser::new(alloc, src, src_ty).parse();
    if !result.errors.is_empty() {
        return Err(diagnostics_to_error(src, result.errors));
    }
    Ok(result.program)
}

#[derive(Debug)]
struct ParseErrors {
    src: String,
    errors: Vec<OxcDiagnostic>,
}

impl std::error::Error for ParseErrors {}

impl std::fmt::Display for ParseErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let handler =
            miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode_nocolor());
        for error in &self.errors {
            let report = miette::Error::new(error.clone()).with_source_code(self.src.clone());
            handler
                .render_report(f, report.as_ref())
                .map_err(|_| std::fmt::Error)?;
        }
        Ok(())
    }
}

impl Diagnostic for ParseErrors {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        None
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        None
    }

    fn note<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        None
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        None
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        None
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        Some(Box::new(self.errors.iter().map(|v| v as &dyn Diagnostic)))
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        None
    }
}

fn diagnostics_to_error(src: &str, errors: Vec<OxcDiagnostic>) -> miette::Error {
    miette::Error::new(ParseErrors {
        src: src.to_owned(),
        errors,
    })
}

pub struct StrippedSource {
    pub code: String,
    pub source_map: Option<oxc_sourcemap::SourceMap>,
}

pub fn strip_type_annotations<'a>(
    alloc: &'a Allocator,
    mut program: Program<'a>,
    source_path: &str,
) -> StrippedSource {
    let scopes = SemanticBuilder::new()
        .build(&program)
        .semantic
        .into_scoping();

    Transformer::new(alloc, Path::new("filename"), &Default::default())
        .build_with_scoping(scopes, &mut program);

    // Strip phantom `export {};` injected by oxc when all imports were type-only.
    // Only removes empty named exports - real exports are preserved.
    program.body.retain(|stmt| {
        if let oxc_ast::ast::Statement::ExportNamedDeclaration(decl) = stmt {
            // Remove only empty `export {};` - keep real exports
            !(decl.specifiers.is_empty() && decl.declaration.is_none() && decl.source.is_none())
        } else {
            true
        }
    });

    let options = oxc_codegen::CodegenOptions {
        source_map_path: Some(std::path::PathBuf::from(source_path)),
        ..Default::default()
    };
    let result = Codegen::new()
        .with_options(options)
        .with_source_text(program.source_text)
        .build(&program);
    StrippedSource {
        code: result.code,
        source_map: result.map,
    }
}

#[cfg(test)]
mod tests;
