import type * as Output from "./workflow";

declare global {
  function __define_workflow(name: string, workflow: Output.Workflow): void;
  function __normalize_id(name: string): string;
  var __GHAT_ACTION_MAPPINGS: Record<string, { inputs?: Record<string, string>; outputs?: Record<string, string> }> | undefined;
}

(function() {
  const define_workflow = __define_workflow;
  const normalize_id = __normalize_id;

  // == Context proxies =============================================

  /** Create a proxy that generates `${{ prefix.key }}` expressions on leaf access. */
  function context_proxy(prefix: string): any {
    return new Proxy(() => { }, {
      get(_target: any, prop: string | symbol): any {
        if (prop === Symbol.toPrimitive) {
          return () => `\${{ ${prefix} }}`;
        }
        if (typeof prop === "symbol") return undefined;
        return context_proxy(`${prefix}.${prop}`);
      },
      apply(): string {
        return `\${{ ${prefix} }}`;
      },
    });
  }

  const ALL_CONTEXTS = new Set([
    "github", "needs", "strategy", "matrix", "job", "runner", "env", "vars", "secrets", "steps", "inputs",
  ]);

  /** Build a context object that allows only the specified context names; any other access throws. */
  function build_context(allowed: string[], location: string): any {
    const entries: Record<string, any> = {};
    for (const name of allowed) {
      entries[name] = context_proxy(name);
    }
    return new Proxy(entries, {
      get(target: Record<string, any>, prop: string | symbol): any {
        if (typeof prop === "symbol") return undefined;
        if (prop in target) return target[prop];
        if (ALL_CONTEXTS.has(prop)) {
          throw new Error(`context '${prop}' is not available in '${location}' (available: ${allowed.join(", ")})`);
        }
        throw new Error(`unknown context '${prop}' in '${location}'`);
      },
    });
  }

  // == Value resolution ============================================

  /** Recursively convert any remaining proxy objects to their expression strings. */
  function coerce(v: any): any {
    if (typeof v === "function") return String(v);
    if (Array.isArray(v)) return v.map(coerce);
    if (typeof v === "object" && v !== null) {
      const out: Record<string, any> = {};
      for (const [k, val] of Object.entries(v)) {
        out[k] = coerce(val);
      }
      return out;
    }
    return v;
  }

  function resolve_value<T>(v: T | ((ctx: any) => T), ctx: Record<string, any>): T {
    if (typeof v === "function") return (v as (ctx: any) => T)(ctx);
    return v;
  }

  // == Triggers ====================================================

  function triggers<T extends Trigger<any>>(trigger: T): T {
    return trigger;
  }

  function input_builtin(type: string, options?: any): any {
    return { type, ...options };
  }

  function matrix(matrix: any): any {
    return matrix;
  }

  function map_trigger(trigger: string[] | FilteredTrigger): Output.Push | Output.PullRequest {
    if (Array.isArray(trigger)) {
      return { branches: trigger };
    }
    return trigger;
  }

  function map_triggers(on: Trigger<any>): Output.Triggers {
    const out: Output.Triggers = {};
    if (on.push != null) out.push = map_trigger(on.push);
    if (on.pull_request != null) out.pull_request = map_trigger(on.pull_request);
    if (on.pull_request_target != null) out.pull_request_target = map_trigger(on.pull_request_target);
    if (on.issue_comment != null) out.issue_comment = on.issue_comment;
    if (on.schedule != null) out.schedule = on.schedule;
    if (on.workflow_dispatch != null) out.workflow_dispatch = map_workflow_dispatch(on.workflow_dispatch);
    if (on.workflow_call != null) out.workflow_call = map_workflow_call(on.workflow_call);
    return out;
  }

  function map_workflow_call(wc: {
    inputs?: Record<string, any>;
    outputs?: Record<string, { description?: string }>;
    secrets?: Record<string, { description?: string; required?: boolean }>;
  }): Output.WorkflowCall {
    const result: Output.WorkflowCall = {};
    if (wc.inputs) {
      const inputs: Record<string, Output.DispatchInput> = {};
      for (const key of Object.keys(wc.inputs)) {
        const inp: BaseInput<boolean> & { type?: string; default?: any } = wc.inputs[key];
        const out: Output.DispatchInput = {};
        if (inp.description != null) out.description = inp.description;
        if (inp.type != null) out.type = inp.type;
        if (inp.required != null) out.required = inp.required;
        if (inp.default != null) out.default = String(inp.default);
        inputs[key] = out;
      }
      result.inputs = inputs;
    }
    if (wc.outputs) {
      const outputs: Record<string, Output.WorkflowCallOutput> = {};
      for (const [key, val] of Object.entries(wc.outputs)) {
        const out: Output.WorkflowCallOutput = {};
        if (val.description != null) out.description = val.description;
        // value is filled in later from the jobs() return
        outputs[key] = out;
      }
      result.outputs = outputs;
    }
    if (wc.secrets) {
      const secrets: Record<string, Output.WorkflowCallSecret> = {};
      for (const [key, val] of Object.entries(wc.secrets)) {
        const out: Output.WorkflowCallSecret = {};
        if (val.description != null) out.description = val.description;
        if (val.required != null) out.required = val.required;
        secrets[key] = out;
      }
      result.secrets = secrets;
    }
    return result;
  }

  function map_workflow_dispatch(wd: { inputs?: Record<string, any> }): Output.WorkflowDispatch {
    if (!wd.inputs) return {};
    const inputs: Record<string, Output.DispatchInput> = {};
    for (const key of Object.keys(wd.inputs)) {
      const inp: BaseInput<boolean> & { type?: string; default?: any; options?: readonly string[] } = wd.inputs[key];
      const out: Output.DispatchInput = {};
      if (inp.description != null) out.description = inp.description;
      if (inp.type != null) out.type = inp.type;
      if (inp.required != null) out.required = inp.required;
      if (inp.default != null) out.default = String(inp.default);
      if (inp.options != null) out.options = [...inp.options];
      inputs[key] = out;
    }
    return { inputs };
  }

  // == Mapping helpers =============================================

  function map_concurrency(c: Concurrency): Output.Concurrency {
    if (typeof c === "string") return c;
    const out: Record<string, any> = { group: c.group };
    if (c.cancel_in_progress != null) out["cancel-in-progress"] = c.cancel_in_progress;
    return out as Output.Concurrency;
  }

  function map_defaults(d: Defaults): Output.Defaults {
    const out: Output.Defaults = {};
    if (d.run) {
      const run: Output.RunDefaults = {};
      if (d.run.shell != null) run.shell = d.run.shell;
      if (d.run.working_directory != null) run["working-directory"] = d.run.working_directory;
      out.run = run;
    } else {
      out.run = { shell: "bash --noprofile --norc -euo pipefail {0}" };
    }
    return out;
  }

  function map_permissions(p: "write-all" | "read-all" | ScopedPermissions): Output.Permissions {
    if (typeof p === "string") return p;
    const out: Output.ScopedPermissions = {};
    if (p.contents != null) out.contents = p.contents;
    if (p.pull_requests != null) out["pull-requests"] = p.pull_requests;
    if (p.packages != null) out.packages = p.packages;
    if (p.id_token != null) out["id-token"] = p.id_token;
    if (p.deployments != null) out.deployments = p.deployments;
    if (p.actions != null) out.actions = p.actions;
    if (p.attestations != null) out.attestations = p.attestations;
    return out;
  }

  // == Name normalization ============================================

  // == Strategy =====================================================

  function map_strategy(s: any): Output.Strategy {
    // s is either a raw matrix value (from `strategy: matrix(...)`)
    // or a JobStrategy object (from `strategy: { matrix: matrix(...), fail_fast: true }`)
    if (s.matrix != null) {
      // JobStrategy form
      const out: Output.Strategy = { matrix: s.matrix };
      if (s.fail_fast != null) out["fail-fast"] = s.fail_fast;
      if (s.max_parallel != null) out["max-parallel"] = s.max_parallel;
      return out;
    }
    // Raw matrix value - the strategy IS the matrix
    return { matrix: s };
  }

  // == Steps =========================================================

  function map_step_options(opts: StepOptions | RunOptions | undefined): Partial<Output.Step> {
    if (!opts) return {};
    const out: Partial<Output.Step> & Record<string, any> = {};
    if (opts.name != null) out.name = opts.name;
    if (opts.if != null) out.if = opts.if;
    if (opts.env != null) out.env = opts.env;
    if (opts.timeout_minutes != null) out["timeout-minutes"] = opts.timeout_minutes;
    if (opts.continue_on_error != null) out["continue-on-error"] = opts.continue_on_error;
    if ("shell" in opts && opts.shell != null) out.shell = opts.shell;
    if ("working_directory" in opts && opts.working_directory != null) out["working-directory"] = opts.working_directory;
    return out;
  }

  /** Per-job step accumulator. Set during steps() callback, null otherwise. */
  let current_steps: Output.Step[] | null = null;
  /** Per-job step id counter for auto-generating unique ids. */
  let step_counter = 0;
  /** Per-job steps proxy entries, built up as steps are added. */
  let steps_entries: Record<string, any> | null = null;

  function next_step_id(): string {
    return `step_${step_counter++}`;
  }

  function register_step_outputs(step_id: string, action?: string): any {
    const output_mapping = action ? globalThis.__GHAT_ACTION_MAPPINGS?.[action]?.outputs : undefined;

    // Build a proxy that maps snake_case access to original-case expressions
    const outputs_proxy = new Proxy({}, {
      get(_target: any, prop: string | symbol): any {
        if (prop === Symbol.toPrimitive) {
          return () => `\${{ steps.${step_id}.outputs }}`;
        }
        if (typeof prop === "symbol") return undefined;
        // Map snake_case prop to original name for the expression
        const original = output_mapping?.[prop] ?? prop;
        return context_proxy(`steps.${step_id}.outputs.${original}`);
      },
    });

    // Register into the steps context so subsequent steps can access it via ctx.steps.<id>
    if (steps_entries != null) {
      steps_entries[step_id] = { outputs: outputs_proxy };
    }
    // Return a StepRef-like object with the same proxy
    return { outputs: outputs_proxy };
  }

  function dedent(text: string): string {
    // Strip leading blank line (from template literals starting with newline)
    let s = text;
    if (s.length > 0 && s[0] === "\n") s = s.slice(1);
    const lines = s.split("\n");
    // Find minimum indentation of non-empty lines
    let min = -1;
    for (const line of lines) {
      if (line.trim().length === 0) continue;
      let indent = 0;
      while (indent < line.length && (line[indent] === " " || line[indent] === "\t")) indent++;
      if (min === -1 || indent < min) min = indent;
    }
    if (min > 0) {
      s = lines.map((line) => line.slice(min)).join("\n");
    }
    // Strip trailing whitespace-only line
    if (s.endsWith("\n")) {
      let i = s.length - 2;
      while (i >= 0 && (s[i] === " " || s[i] === "\t")) i--;
      if (i >= 0 && s[i] === "\n") s = s.slice(0, i + 1) + "\n";
    }
    return s;
  }

  function run_builtin(script: string, options?: RunOptions): StepRef {
    if (current_steps == null) throw new Error("run() can only be called inside steps()");
    const step_id = next_step_id();
    const step: Output.Step = { id: step_id, run: dedent(script), ...map_step_options(options) };
    current_steps.push(step);
    return register_step_outputs(step_id);
  }

  function uses_builtin(action: string, options?: any): StepRef {
    if (current_steps == null) throw new Error("uses() can only be called inside steps()");
    const step_id = next_step_id();
    const step: Output.Step & Record<string, any> = { id: step_id, uses: action, ...map_step_options(options) };
    if (options?.with != null) {
      const mapping = globalThis.__GHAT_ACTION_MAPPINGS?.[action]?.inputs;
      const with_out: Record<string, unknown> = {};
      for (const [key, value] of Object.entries(options.with)) {
        with_out[mapping?.[key] ?? key] = value;
      }
      step.with = with_out;
    }
    current_steps.push(step);
    return register_step_outputs(step_id, action);
  }

  // == Jobs =========================================================

  /** State accumulated for a single job's needs dependencies. */
  interface JobRefInternal {
    id: string;
    outputs: Record<string, string>;
  }

  /** Build a needs context proxy from an array of JobRefInternal. */
  function build_needs_proxy(refs: JobRefInternal[]): any {
    const entries: Record<string, any> = {};
    for (const ref_ of refs) {
      const outputs_proxy = context_proxy(`needs.${ref_.id}.outputs`);
      entries[ref_.id] = { outputs: outputs_proxy };
    }
    return new Proxy(entries, {
      get(target: Record<string, any>, prop: string | symbol): any {
        if (typeof prop === "symbol") return undefined;
        if (prop in target) return target[prop];
        throw new Error(`job '${prop}' is not listed in needs`);
      },
    });
  }

  function map_job(name: string, job_def: Job<any, any, any, any>, jobs: Record<string, Output.Job>): JobRefInternal {
    const id = normalize_id(name);

    // Resolve needs
    const needs_refs: JobRefInternal[] = job_def.needs ?? [];
    const needs_ids = needs_refs.map((r) => r.id);

    // Resolve strategy (needs: github, needs, vars, inputs)
    let strategy: Output.Strategy | undefined;
    let matrix_proxy: any = {};
    if (job_def.strategy != null) {
      const strategy_ctx = build_context(["github", "needs", "vars", "inputs"], `${name} > strategy`);
      strategy_ctx.needs = build_needs_proxy(needs_refs);
      const raw = resolve_value(job_def.strategy, strategy_ctx);
      strategy = map_strategy(raw);
      // Build a matrix proxy for downstream contexts
      matrix_proxy = context_proxy("matrix");
    }

    // Resolve if (needs: github, needs, vars, inputs)
    let if_condition: string | undefined;
    if (job_def.if != null) {
      const if_ctx = build_context(["github", "needs", "vars", "inputs"], `${name} > if`);
      if_ctx.needs = build_needs_proxy(needs_refs);
      if_condition = resolve_value(job_def.if, if_ctx);
    }

    // Resolve runs_on (needs: github, needs, strategy, matrix, vars, inputs)
    const runs_on_ctx = build_context(["github", "needs", "strategy", "matrix", "vars", "inputs"], `${name} > runs_on`);
    runs_on_ctx.needs = build_needs_proxy(needs_refs);
    runs_on_ctx.matrix = matrix_proxy;
    const runs_on = resolve_value(job_def.runs_on, runs_on_ctx);

    // Resolve env (needs: github, needs, strategy, matrix, vars, secrets, inputs)
    let env: Record<string, string> | undefined;
    if (job_def.env != null) {
      const env_ctx = build_context(["github", "needs", "strategy", "matrix", "vars", "secrets", "inputs"], `${name} > env`);
      env_ctx.needs = build_needs_proxy(needs_refs);
      env_ctx.matrix = matrix_proxy;
      env = resolve_value(job_def.env, env_ctx);
    }

    // Build steps context (needs: github, needs, strategy, matrix, job, runner, env, vars, secrets, steps, inputs)
    const steps_ctx = build_context(["github", "needs", "strategy", "matrix", "job", "runner", "env", "vars", "secrets", "steps", "inputs"], `${name} > steps`);
    steps_ctx.needs = build_needs_proxy(needs_refs);
    steps_ctx.matrix = matrix_proxy;

    // Set up step accumulator and steps proxy
    current_steps = [];
    step_counter = 0;
    steps_entries = {};
    // Override the steps context entry with our live proxy that grows as steps are added
    steps_ctx.steps = new Proxy(steps_entries, {
      get(target: Record<string, any>, prop: string | symbol): any {
        if (typeof prop === "symbol") return undefined;
        if (prop in target) return target[prop];
        throw new Error(`step '${prop}' has not been defined yet`);
      },
    });

    const outputs_raw = job_def.steps(steps_ctx);
    const collected_steps = current_steps;
    current_steps = null;
    steps_entries = null;

    // Build output job
    const out_job: Output.Job = {};
    out_job.name = name;
    out_job["runs-on"] = runs_on;
    if (needs_ids.length > 0) out_job.needs = needs_ids;
    if (if_condition != null) out_job.if = if_condition;
    if (env != null) out_job.env = env;
    if (job_def.timeout_minutes != null) out_job["timeout-minutes"] = job_def.timeout_minutes;
    if (strategy != null) out_job.strategy = strategy;

    // Map step return value to job outputs
    if (outputs_raw != null && typeof outputs_raw === "object") {
      const job_outputs: Record<string, string> = {};
      for (const [key, value] of Object.entries(outputs_raw as Record<string, string>)) {
        job_outputs[key] = String(value);
      }
      out_job.outputs = job_outputs;
    }

    if (collected_steps.length > 0) out_job.steps = collected_steps;

    jobs[id] = out_job;

    return { id, outputs: context_proxy(`jobs.${id}.outputs`) };
  }

  // == workflow =====================================================

  function workflow(name: string, definition: Workflow<any>): any {
    const on = map_triggers(definition.on);
    const id = normalize_id(name);

    const wf: Output.Workflow = {
      name: name,
      on: on,
      jobs: {},
    };

    if (definition.run_name != null) {
      const ctx = build_context(["github", "inputs", "vars"], `${name} > run_name`);
      wf["run-name"] = resolve_value(definition.run_name, ctx);
    }

    if (definition.permissions != null) {
      wf.permissions = map_permissions(definition.permissions);
    }

    if (definition.defaults != null) {
      wf.defaults = map_defaults(definition.defaults);
    }

    if (definition.concurrency != null) {
      const ctx = build_context(["github", "inputs", "vars"], `${name} > concurrency`);
      wf.concurrency = map_concurrency(resolve_value(definition.concurrency, ctx));
    }

    if (definition.env != null) {
      const ctx = build_context(["github", "secrets", "inputs", "vars"], `${name} > env`);
      wf.env = resolve_value(definition.env, ctx);
    }

    // Jobs
    const jobs_ctx = build_context(["github", "inputs", "vars"], `${name} > jobs`);
    (jobs_ctx as any).job = function(job_name: string, job_def: Job<any, any, any, any>): JobRefInternal {
      return map_job(job_name, job_def, wf.jobs);
    };
    (jobs_ctx as any).uses = function(
      job_name: string,
      workflow_ref: { __workflow_ref: true; __path: string },
      options?: { needs?: JobRefInternal[]; with?: Record<string, unknown>; secrets?: unknown; if?: any },
    ): JobRefInternal {
      const job_id = normalize_id(job_name);
      const needs_refs: JobRefInternal[] = options?.needs ?? [];
      const needs_ids = needs_refs.map((r) => r.id);

      const out_job: Output.Job = {};
      out_job.name = job_name;
      out_job.uses = workflow_ref.__path;
      if (needs_ids.length > 0) out_job.needs = needs_ids;
      if (options?.if != null) {
        const if_ctx = build_context(["github", "needs", "vars", "inputs"], `${job_name} > if`);
        if_ctx.needs = build_needs_proxy(needs_refs);
        out_job.if = resolve_value(options.if, if_ctx);
      }
      if (options?.with != null) out_job.with = options.with;
      if (options?.secrets != null) out_job.secrets = options.secrets;

      wf.jobs[job_id] = out_job;
      return { id: job_id, outputs: context_proxy(`jobs.${job_id}.outputs`) };
    };

    const jobs_return = definition.jobs(jobs_ctx);

    // Fill in workflow_call output values from the jobs() return
    if (wf.on.workflow_call?.outputs && jobs_return != null && typeof jobs_return === "object") {
      for (const [key, value] of Object.entries(jobs_return as Record<string, unknown>)) {
        if (key in wf.on.workflow_call.outputs) {
          wf.on.workflow_call.outputs[key].value = String(value);
        }
      }
    }

    define_workflow(name, coerce(wf));

    // Return a workflow ref for reusable workflow calls
    return { __workflow_ref: true, __path: `./.github/workflows/generated_${id}.yaml` };
  }

  globalThis.workflow = workflow;
  globalThis.triggers = triggers;
  globalThis.input = input_builtin;
  globalThis.matrix = matrix;
  globalThis.run = run_builtin;
  globalThis.uses = uses_builtin as any;
})();
