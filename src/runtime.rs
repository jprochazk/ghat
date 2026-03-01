use std::path::{Path, PathBuf};
use std::{cell::RefCell, rc::Rc};

use indexmap::IndexMap;
use miette::{Context, IntoDiagnostic};
use rquickjs::{self as js, CatchResultExt as _};

use crate::workflow::Workflow;

struct TsResolver;

impl js::loader::Resolver for TsResolver {
    fn resolve<'js>(&mut self, _ctx: &js::Ctx<'js>, base: &str, name: &str) -> js::Result<String> {
        let path = if name.starts_with('.') {
            let base_path = PathBuf::from(base);
            let parent = base_path.parent().unwrap_or(Path::new(""));
            parent.join(name)
        } else {
            PathBuf::from(name)
        };

        // Canonicalize to collapse `..` and `.` segments
        let resolved = path
            .canonicalize()
            .map_err(|e| js::Error::new_loading_message(name, &e.to_string()))?;

        Ok(resolved.to_string_lossy().into_owned())
    }
}

struct TsLoader;

impl js::loader::Loader for TsLoader {
    fn load<'js>(
        &mut self,
        ctx: &js::Ctx<'js>,
        name: &str,
    ) -> js::Result<js::Module<'js, js::module::Declared>> {
        let source = std::fs::read_to_string(name)
            .map_err(|e| js::Error::new_loading_message(name, &e.to_string()))?;

        let js_source = if name.ends_with(".ts") {
            use crate::oxc;
            let alloc = oxc::allocator();
            let program = oxc::parse_ts(&alloc, &source)
                .map_err(|e| js::Error::new_loading_message(name, &e.to_string()))?;
            let stripped = oxc::strip_type_annotations(&alloc, program, name);
            stripped.code
        } else {
            source
        };

        js::Module::declare(ctx.clone(), name, js_source)
            .map_err(|e| js::Error::new_loading_message(name, &e.to_string()))
    }
}

/// Convert a caught rquickjs error into a [`miette::Report`].
fn caught_error_to_report(err: js::CaughtError) -> miette::Report {
    match err {
        js::CaughtError::Exception(exc) => {
            let msg = exc
                .message()
                .unwrap_or_else(|| "unknown JS exception".into());
            match exc.stack() {
                Some(stack) => miette::miette!("{msg}\n\nStack trace:\n{stack}"),
                None => miette::miette!("{msg}"),
            }
        }
        js::CaughtError::Value(val) => {
            miette::miette!("JS threw: {val:?}")
        }
        js::CaughtError::Error(err) => miette::miette!("{err}"),
    }
}

/// Extension trait to convert `Result<T, CaughtError>` into `miette::Result<T>`.
trait CaughtResultExt<T> {
    fn into_miette(self) -> miette::Result<T>;
}

impl<T> CaughtResultExt<T> for Result<T, js::CaughtError<'_>> {
    fn into_miette(self) -> miette::Result<T> {
        self.map_err(caught_error_to_report)
    }
}

pub struct RuntimeBuilder {
    mappings_js: Option<String>,
}

impl RuntimeBuilder {
    /// Set the action mappings JS source (contents of `mappings.js`).
    ///
    /// This is evaluated before `api-impl.ts` so that `__GHAT_ACTION_MAPPINGS`
    /// is available to the builtins.
    pub fn mappings(mut self, js: &str) -> Self {
        self.mappings_js = Some(js.to_owned());
        self
    }

    /// Build the runtime, evaluating all setup scripts.
    pub fn build(self) -> miette::Result<Runtime> {
        let rt = {
            let rt = js::Runtime::new().into_diagnostic()?;
            let ctx = js::Context::custom::<(
                js::context::intrinsic::Eval,
                js::context::intrinsic::Promise,
                js::context::intrinsic::Proxy,
                js::context::intrinsic::MapSet,
            )>(&rt)
            .into_diagnostic()?;
            rt.set_loader(TsResolver, TsLoader);
            Runtime {
                ctx,
                _rt: rt,
                generated_workflows: Default::default(),
            }
        };

        register_builtins(&rt);

        if let Some(mappings) = &self.mappings_js {
            rt.eval_script(mappings)
                .wrap_err("failed to evaluate mappings.js")?;
        }

        rt.eval_script(&api_impl_js())
            .wrap_err("failed to evaluate api-impl.ts")?;

        rt.ctx.with(|ctx| {
            ctx.globals()
                .remove("eval")
                .expect("BUG: failed to disable `eval`")
        });

        Ok(rt)
    }
}

pub struct Runtime {
    // Drop order matters: ctx must be dropped before _rt so all JS objects
    // are freed before the QuickJS runtime is torn down.
    ctx: js::Context,
    _rt: js::Runtime,

    generated_workflows: Rc<RefCell<IndexMap<String, Workflow>>>,
}

impl Runtime {
    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder {
            mappings_js: None,
        }
    }

    fn eval_script(&self, source: &str) -> miette::Result<()> {
        self.ctx.with(|ctx| {
            ctx.eval::<(), _>(source)
                .catch(&ctx)
                .into_miette()
        })
    }

    pub fn eval_workflow_definition(&self, path: &Path) -> miette::Result<()> {
        let name = path.to_string_lossy().to_string();
        let source = std::fs::read_to_string(path)
            .into_diagnostic()
            .wrap_err("failed to read source file")?;

        self.ctx.with(|ctx| {
            let promise = js::Module::evaluate(ctx.clone(), name, source)
                .catch(&ctx)
                .into_miette()?;
            promise.finish::<()>().catch(&ctx).into_miette()
        })?;

        Ok(())
    }

    pub fn finish(self) -> Vec<(String, Workflow)> {
        self.generated_workflows.take().into_iter().collect()
    }
}

/// Embedded `api-impl.ts` source, stripped of type annotations.
fn api_impl_js() -> String {
    use crate::oxc;

    const SRC: &str = include_str!("./runtime/api-impl.ts");

    let alloc = oxc::allocator();
    let program = oxc::parse_ts(&alloc, SRC).expect("BUG: failed to parse runtime/api-impl.ts");
    let stripped = oxc::strip_type_annotations(&alloc, program, "api-impl.ts");
    stripped.code
}

fn register_builtins(runtime: &Runtime) {
    let define_workflow = {
        let workflows = Rc::clone(&runtime.generated_workflows);
        move |ctx: js::Ctx, name: String, def: js::Value| -> js::Result<()> {
            match rquickjs_serde::from_value::<Workflow>(def) {
                Ok(def) => {
                    let name = normalize_id(name);

                    log::debug!(
                        "new workflow definition {name:?}:\n{}",
                        serde_yaml_ng::to_string(&def).expect("failed to serialize YAML")
                    );

                    workflows.borrow_mut().insert(name, def);
                    Ok(())
                }
                Err(err) => Err(match err.catch(&ctx) {
                    rquickjs_serde::err::CaughtError::Exception(exc) => {
                        js::CaughtError::Exception(exc).throw(&ctx)
                    }
                    rquickjs_serde::err::CaughtError::Value(value) => {
                        js::CaughtError::Value(value).throw(&ctx)
                    }
                    rquickjs_serde::err::CaughtError::Error(err) => {
                        js::CaughtError::Error(err).throw(&ctx)
                    }
                    rquickjs_serde::err::CaughtError::Message(msg) => {
                        js::CaughtError::Value(js::Value::from_string(
                            js::String::from_str(ctx.clone(), &msg)
                                .expect("BUG: failed to create string"),
                        ))
                        .throw(&ctx)
                    }
                }),
            }
        }
    };

    let normalize_id =
        |_ctx: js::Ctx, name: String| -> js::Result<String> { Ok(normalize_id(name)) };

    runtime.ctx.with(move |ctx| {
        let globals = ctx.globals();
        globals
            .set(
                "__define_workflow",
                js::function::Func::from(define_workflow),
            )
            .catch(&ctx)
            .expect("BUG: failed to set global __define_workflow");
        globals
            .set("__normalize_id", js::function::Func::from(normalize_id))
            .catch(&ctx)
            .expect("BUG: failed to set global __normalize_id");
    });
}

/// Normalize a name to a valid id: `[a-z0-9_]`.
/// Dashes and spaces become underscores, other non-allowed characters are stripped.
/// This must match the `NormalizeId` type helper in `api.d.ts`.
fn normalize_id(name: String) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        match c {
            c if c.is_ascii_alphanumeric() => out.push(c.to_ascii_lowercase()),
            '-' | ' ' => out.push('_'),
            _ => { /* strip */ }
        }
    }
    out
}

#[cfg(test)]
mod tests;
