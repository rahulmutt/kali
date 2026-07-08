//! Object-shape monomorphization analysis (fasta Spec 5, Task 7a-1).
//!
//! Computes a [`MonoPlan`]: for every user function reached by two or more
//! *distinct* object-parameter shape tuples, the set of specializations it
//! needs, plus the per-call-site mapping that a later AST clone+rename pass
//! (Task 2) uses to rewrite each callee identifier. A function reached by only
//! one distinct tuple (or none) is **not** in the plan — it already lowers
//! monomorphically today, matching probe P1/P3.
//!
//! ## Model
//!
//! This mirrors the shape identity `kali_types::repr_infer` already uses — an
//! object's shape is the *ordered field-name list* of the literal that created
//! it (`ObjSlot`/`obj_literal_fields`, repr_infer.rs) — but builds a
//! **direction-aware, call-site-tracking** forward flow graph rather than
//! reusing repr_infer's `obj_flows` directly. repr_infer's `obj_flows` is (a)
//! private, (b) *bidirectional* aliasing whose pair order does not consistently
//! encode source→sink direction, and (c) carries no call-site identity (it keys
//! flows by integer result nodes). This analysis needs all three — forward
//! direction from literal sources, and a stable per-call-site identity to
//! rewrite — so it walks the AST directly, reusing the exact literal-shape and
//! alias-recognition *rules* of repr_infer (`record_object_literal`,
//! `record_object_flow_from_expr`) to stay behavior-compatible. repr_infer is
//! left untouched.
//!
//! ## Fixpoint + termination
//!
//! The analysis is context-sensitive: a *function instance* is `(func,
//! SpecKey)` where the `SpecKey` fixes the function's object params to concrete
//! shape tuples. Processing an instance solves the shapes of its local bindings,
//! then seeds a nested-callee instance from each call's argument shapes (the
//! fasta `fastaRandom(table) → makeCumulative(table)` shape). A chaotic-iteration
//! outer loop runs to a fixpoint.
//!
//! **Termination.** No widening or field-synthesis ever occurs — every shape
//! tuple that flows is verbatim one of the finitely many object literals in the
//! source. So each object param maps to one of finitely many tuples, the
//! instance set is finite (`#functions × #shapes^#object-params`), and every
//! round only adds instances or refines return shapes monotonically over that
//! finite lattice. Two hard caps (`CAP_ROUNDS`, `CAP_INSTANCES`) are a
//! defensive backstop: if either is exceeded the whole plan bails to empty
//! (fail-closed — E5506 preserved).
//!
//! **Fail-closed bails (design §4).** A callee is excluded from the plan
//! entirely whenever it cannot be cleanly partitioned:
//! * an object argument whose slot holds ≥2 distinct shapes at one site (a
//!   `cond ? A : B` merge, or two conflicting assignments) — no per-call-site
//!   partition exists;
//! * inconsistent object-param positions across its call sites;
//! * the defensive caps above.
//!
//! An empty (or partial) plan is always safe: unspecialized functions keep
//! today's single-shape behavior and today's E5506 conflict.

use kali_ast::{
    Expression, ForInit, FunctionDeclaration, ObjectExpression, ObjectPropertyKind, PropertyName,
    Statement,
};
use std::collections::{BTreeMap, BTreeSet};

/// Synthetic name for the module-scope (top-level) "function".
const TOP_LEVEL: &str = "_start";

/// Defensive backstops. The finite-lattice argument (see module docs) proves
/// the fixpoint converges well within these; exceeding either signals an
/// unforeseen expansion and forces a global fail-closed bail.
const CAP_ROUNDS: usize = 256;
const CAP_INSTANCES: usize = 10_000;

/// A shape tuple: the ordered field-name list of an object literal.
pub type ShapeTuple = Vec<String>;

/// The specialization identity of a function instance: its object-typed params
/// (in ascending param-index order) each bound to a concrete shape tuple. An
/// empty `params` is the un-specialized / module-scope context.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct SpecKey {
    /// `(param index, ordered field names)` for each object-typed param.
    pub params: Vec<(usize, ShapeTuple)>,
}

impl SpecKey {
    /// The empty (un-specialized) context.
    pub fn is_context_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// The set of param indices bound to an object shape (the partition axis).
    fn index_set(&self) -> Vec<usize> {
        self.params.iter().map(|(i, _)| *i).collect()
    }
}

/// A single call-site rewrite instruction: within `caller`'s body (in the
/// specialization identified by `caller_spec`, empty for the original body),
/// the call at `ordinal` targets specialization `callee_spec` of `callee`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct CallBinding {
    /// Enclosing function name (or `_start` for module scope).
    pub caller: String,
    /// Which specialization of the caller this binding applies to; empty
    /// `params` means the caller's original (unspecialized) body.
    pub caller_spec: SpecKey,
    /// Pre-order index of the `CallExpression` among all calls in `caller`'s
    /// body (not descending into nested function declarations). Stable for a
    /// later rewrite pass that walks the same body identically.
    pub ordinal: usize,
    /// Callee function name.
    pub callee: String,
    /// The specialization of `callee` this call targets.
    pub callee_spec: SpecKey,
}

/// The computed monomorphization plan.
#[derive(Clone, Debug, Default)]
pub struct MonoPlan {
    /// Functions needing specialization: name → its ≥2 distinct `SpecKey`s,
    /// sorted. Functions reached by ≤1 distinct tuple are absent.
    specializations: BTreeMap<String, Vec<SpecKey>>,
    /// Rewrite instructions for every call whose callee is specialized.
    call_bindings: Vec<CallBinding>,
}

impl MonoPlan {
    /// True when nothing needs specialization (a hard no-op for the pipeline).
    pub fn is_empty(&self) -> bool {
        self.specializations.is_empty()
    }

    /// The distinct specialization keys for `func`, or `None` if `func` is not
    /// specialized (reached by ≤1 distinct object-param tuple, or bailed).
    pub fn specialization_keys(&self, func: &str) -> Option<&[SpecKey]> {
        self.specializations.get(func).map(|v| v.as_slice())
    }

    /// All functions that get specialized, with their keys.
    pub fn specializations(&self) -> &BTreeMap<String, Vec<SpecKey>> {
        &self.specializations
    }

    /// All call-site rewrite instructions.
    pub fn call_bindings(&self) -> &[CallBinding] {
        &self.call_bindings
    }
}

/// Compute the [`MonoPlan`] for a parsed program.
pub fn compute_mono_plan(statements: &[Statement]) -> MonoPlan {
    Analyzer::new(statements).run()
}

// ---------------------------------------------------------------------------

/// A set of shape tuples reaching a slot. Empty = no object shape (scalar or
/// unknown). Size 1 = a definite shape. Size ≥2 = an ambiguous merge.
type ShapeVal = BTreeSet<ShapeTuple>;
/// Per-name shape environment within one function instance.
type Env = BTreeMap<String, ShapeVal>;
/// Instance identity: `(func, SpecKey)`.
type Instance = (String, SpecKey);

/// One recognized call within a function body: its pre-order ordinal, the
/// bare-identifier callee name (if any), and the argument expressions.
struct CallInfo<'a> {
    ordinal: usize,
    callee: Option<String>,
    args: Vec<&'a Expression>,
}

struct Analyzer<'a> {
    /// Every user function declaration, by name (collected recursively).
    funcs: BTreeMap<String, &'a FunctionDeclaration>,
    /// Top-level statements (the `_start` body).
    top: &'a [Statement],
    /// Module-scope object bindings (shapes of top-level object literals /
    /// aliases), visible as a fallback in every function's environment.
    globals: Env,
}

impl<'a> Analyzer<'a> {
    fn new(statements: &'a [Statement]) -> Self {
        let mut funcs = BTreeMap::new();
        collect_functions(statements, &mut funcs);
        let mut analyzer = Analyzer {
            funcs,
            top: statements,
            globals: Env::new(),
        };
        // Module globals: solve the top-level body with no params and no
        // known return shapes (call-return-typed globals stay unknown — safe).
        analyzer.globals = analyzer.solve_env(TOP_LEVEL, &SpecKey::default(), &BTreeMap::new());
        analyzer
    }

    /// The statements forming `func`'s body (`_start` = the whole program).
    fn body_of(&self, func: &str) -> &'a [Statement] {
        if func == TOP_LEVEL {
            self.top
        } else {
            self.funcs
                .get(func)
                .map(|f| f.body.body.as_slice())
                .unwrap_or(&[])
        }
    }

    /// Ordered parameter names of `func` (empty for `_start`/unknown).
    fn params_of(&self, func: &str) -> &'a [String] {
        self.funcs
            .get(func)
            .map(|f| f.params.as_slice())
            .unwrap_or(&[])
    }

    /// Solve the shape environment of one function instance to a fixpoint.
    fn solve_env(&self, func: &str, ctx: &SpecKey, returns: &BTreeMap<Instance, ShapeVal>) -> Env {
        let mut env = Env::new();
        // Seed params from the specialization context.
        let params = self.params_of(func);
        for (idx, tuple) in &ctx.params {
            if let Some(name) = params.get(*idx) {
                env.entry(name.clone())
                    .or_default()
                    .insert(tuple.clone());
            }
        }
        let body = self.body_of(func);
        for _ in 0..CAP_ROUNDS {
            let before = env.clone();
            self.collect_env(body, &mut env, returns);
            if env == before {
                break;
            }
        }
        env
    }

    /// Union the shapes produced by every binding/assignment in `stmts` into
    /// `env` (flow-insensitive; all branches merge). Does not descend into
    /// nested function declarations (separate scopes).
    fn collect_env(&self, stmts: &[Statement], env: &mut Env, returns: &BTreeMap<Instance, ShapeVal>) {
        for stmt in stmts {
            match stmt {
                Statement::VariableDeclaration(decl) => {
                    for d in &decl.declarations {
                        if let Some(init) = &d.init {
                            let sv = self.eval(init, env, returns);
                            union_into(env.entry(d.id.clone()).or_default(), &sv);
                        }
                    }
                }
                Statement::ExpressionStatement(es) => {
                    if let Expression::AssignmentExpression(assign) = &*es.expression {
                        if let Expression::Identifier(name) = &assign.left {
                            let sv = self.eval(&assign.right, env, returns);
                            union_into(env.entry(name.clone()).or_default(), &sv);
                        }
                    }
                }
                Statement::IfStatement(s) => {
                    self.collect_env(&s.consequent.body, env, returns);
                    if let Some(alt) = &s.alternate {
                        self.collect_env(&alt.body, env, returns);
                    }
                }
                Statement::ForStatement(s) => {
                    if let Some(ForInit::VariableDeclaration(decl)) = &s.init {
                        for d in &decl.declarations {
                            if let Some(init) = &d.init {
                                let sv = self.eval(init, env, returns);
                                union_into(env.entry(d.id.clone()).or_default(), &sv);
                            }
                        }
                    }
                    self.collect_env(&s.body.body, env, returns);
                }
                Statement::ForInStatement(s) => self.collect_stmt(&s.body, env, returns),
                Statement::ForOfStatement(s) => self.collect_stmt(&s.body, env, returns),
                Statement::WhileStatement(s) => self.collect_env(&s.body.body, env, returns),
                Statement::DoWhileStatement(s) => self.collect_env(&s.body.body, env, returns),
                Statement::BlockStatement(b) => self.collect_env(&b.body, env, returns),
                Statement::LabeledStatement(s) => self.collect_stmt(&s.body, env, returns),
                Statement::TryStatement(s) => {
                    self.collect_env(&s.block.body, env, returns);
                    if let Some(h) = &s.handler {
                        self.collect_env(&h.body.body, env, returns);
                    }
                    if let Some(f) = &s.finalizer {
                        self.collect_env(&f.body, env, returns);
                    }
                }
                Statement::SwitchStatement(s) => {
                    for case in &s.cases {
                        self.collect_env(&case.consequent, env, returns);
                    }
                }
                // Nested function declarations are a separate scope — skip.
                _ => {}
            }
        }
    }

    fn collect_stmt(&self, stmt: &Statement, env: &mut Env, returns: &BTreeMap<Instance, ShapeVal>) {
        self.collect_env(std::slice::from_ref(stmt), env, returns);
    }

    /// Evaluate the object shape(s) an expression can carry in `env`.
    fn eval(&self, expr: &Expression, env: &Env, returns: &BTreeMap<Instance, ShapeVal>) -> ShapeVal {
        match expr {
            Expression::ObjectExpression(obj) => match clean_shape(obj) {
                Some(t) => BTreeSet::from([t]),
                None => ShapeVal::new(),
            },
            Expression::Identifier(name) => env
                .get(name)
                .or_else(|| self.globals.get(name))
                .cloned()
                .unwrap_or_default(),
            Expression::ParenthesizedExpression(p) => self.eval(&p.expression, env, returns),
            Expression::ConditionalExpression(c) => {
                let mut out = self.eval(&c.consequent, env, returns);
                union_into(&mut out, &self.eval(&c.alternate, env, returns));
                out
            }
            Expression::LogicalExpression(l) => {
                let mut out = self.eval(&l.left, env, returns);
                union_into(&mut out, &self.eval(&l.right, env, returns));
                out
            }
            Expression::CallExpression(call) => {
                let Expression::Identifier(callee) = &call.callee else {
                    return ShapeVal::new();
                };
                if !self.funcs.contains_key(callee) {
                    return ShapeVal::new();
                }
                // Return-shape of the callee under the shapes we pass it.
                match self.callee_ctx(&call.args.iter().collect::<Vec<_>>(), env, returns) {
                    CalleeCtx::Definite(ck) => {
                        returns.get(&(callee.clone(), ck)).cloned().unwrap_or_default()
                    }
                    _ => ShapeVal::new(),
                }
            }
            _ => ShapeVal::new(),
        }
    }

    /// The specialization context a call selects from its argument shapes.
    fn callee_ctx(
        &self,
        args: &[&Expression],
        env: &Env,
        returns: &BTreeMap<Instance, ShapeVal>,
    ) -> CalleeCtx {
        let mut params = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            let sv = self.eval(arg, env, returns);
            match sv.len() {
                0 => {}
                1 => params.push((i, sv.into_iter().next().unwrap())),
                _ => return CalleeCtx::Ambiguous,
            }
        }
        if params.is_empty() {
            CalleeCtx::NoObject
        } else {
            CalleeCtx::Definite(SpecKey { params })
        }
    }

    /// The shape(s) a function instance can return.
    fn return_shape(&self, func: &str, env: &Env, returns: &BTreeMap<Instance, ShapeVal>) -> ShapeVal {
        let mut out = ShapeVal::new();
        collect_returns(self.body_of(func), &mut |expr| {
            union_into(&mut out, &self.eval(expr, env, returns));
        });
        out
    }

    /// Enumerate every recognized call in a function body, pre-order, without
    /// descending into nested function declarations.
    fn calls_of(&self, func: &str) -> Vec<CallInfo<'a>> {
        let mut calls = Vec::new();
        let mut ordinal = 0usize;
        collect_calls(self.body_of(func), &mut ordinal, &mut calls);
        calls
    }

    fn run(&self) -> MonoPlan {
        // Fixpoint state.
        let mut instances: BTreeSet<Instance> =
            BTreeSet::from([(TOP_LEVEL.to_string(), SpecKey::default())]);
        let mut returns: BTreeMap<Instance, ShapeVal> = BTreeMap::new();
        let mut bailed: BTreeSet<String> = BTreeSet::new();

        for _round in 0..CAP_ROUNDS {
            let prev_returns = returns.clone();
            let snapshot: Vec<Instance> = instances.iter().cloned().collect();
            let mut additions: BTreeSet<Instance> = BTreeSet::new();
            let mut changed = false;

            for (func, ctx) in &snapshot {
                let env = self.solve_env(func, ctx, &prev_returns);
                let rs = self.return_shape(func, &env, &prev_returns);
                if returns.get(&(func.clone(), ctx.clone())) != Some(&rs) {
                    returns.insert((func.clone(), ctx.clone()), rs);
                    changed = true;
                }
                for call in self.calls_of(func) {
                    let Some(callee) = call.callee else { continue };
                    if !self.funcs.contains_key(&callee) {
                        continue;
                    }
                    match self.callee_ctx(&call.args, &env, &prev_returns) {
                        CalleeCtx::Ambiguous => {
                            // A single call passing an ambiguous object shape:
                            // this callee cannot be cleanly partitioned. Bail it.
                            if bailed.insert(callee.clone()) {
                                changed = true;
                            }
                        }
                        CalleeCtx::NoObject => {}
                        CalleeCtx::Definite(ck) => {
                            let inst = (callee.clone(), ck);
                            if !instances.contains(&inst) {
                                additions.insert(inst);
                            }
                        }
                    }
                }
            }

            if !additions.is_empty() {
                instances.extend(additions);
                changed = true;
            }
            if instances.len() > CAP_INSTANCES {
                return MonoPlan::default(); // defensive global bail
            }
            if !changed {
                break;
            }
            if _round + 1 == CAP_ROUNDS {
                return MonoPlan::default(); // did not converge — global bail
            }
        }

        self.build_plan(&instances, &returns, &bailed)
    }

    /// Assemble the plan from the converged instance set.
    fn build_plan(
        &self,
        instances: &BTreeSet<Instance>,
        returns: &BTreeMap<Instance, ShapeVal>,
        bailed: &BTreeSet<String>,
    ) -> MonoPlan {
        // Distinct non-empty specialization keys per function.
        let mut keys_by_func: BTreeMap<String, BTreeSet<SpecKey>> = BTreeMap::new();
        for (func, ctx) in instances {
            if func == TOP_LEVEL || ctx.is_context_empty() {
                continue;
            }
            keys_by_func
                .entry(func.clone())
                .or_default()
                .insert(ctx.clone());
        }

        let mut specializations: BTreeMap<String, Vec<SpecKey>> = BTreeMap::new();
        for (func, keys) in &keys_by_func {
            if bailed.contains(func) {
                continue; // fail-closed: ambiguous incoming shape somewhere
            }
            if keys.len() < 2 {
                continue; // one distinct tuple => no clone needed (P1/P3)
            }
            // All specializations must partition on the SAME object-param
            // positions; otherwise the call sites cannot be cleanly separated.
            let index_sets: BTreeSet<Vec<usize>> = keys.iter().map(|k| k.index_set()).collect();
            if index_sets.len() != 1 {
                continue;
            }
            specializations.insert(func.clone(), keys.iter().cloned().collect());
        }

        // Transitive fail-closed bail (root-cause post-pass). A callee reached
        // through a caller that is itself un-specialized but has ≥2 distinct
        // instances is NOT cleanly partitioned: that caller's single un-cloned
        // body hosts ONE call site that its multiple instances would route to
        // ≥2 different callee specializations — a broken `call_site → tuple`
        // mapping. Remove every such callee from the plan.
        //
        // A non-`_start` function's instances all carry a non-empty `SpecKey`
        // (instances are only seeded from `CalleeCtx::Definite`), so
        // `keys_by_func[func].len() ≥ 2` exactly means "≥2 distinct instances".
        // A function with ≥2 keys that is absent from `specializations` is
        // "poly-uncloned" — it bailed on ambiguity, failed the `index_sets`
        // partition check, or was itself bailed by a previous round here.
        //
        // Fixpoint: bailing a callee can itself make that callee poly-uncloned,
        // cascading to ITS callees. Terminates because every round that changes
        // anything strictly shrinks `specializations` (a monotone-decreasing,
        // bounded-below measure).
        loop {
            let mut newly_bailed: BTreeSet<String> = BTreeSet::new();
            for (func, ctx) in instances {
                if func == TOP_LEVEL {
                    continue;
                }
                let poly = keys_by_func.get(func).map(|k| k.len() >= 2).unwrap_or(false);
                if !poly || specializations.contains_key(func) {
                    continue; // not a poly-uncloned caller
                }
                let env = self.solve_env(func, ctx, returns);
                for call in self.calls_of(func) {
                    let Some(callee) = call.callee else { continue };
                    if !specializations.contains_key(&callee) {
                        continue;
                    }
                    if let CalleeCtx::Definite(ck) = self.callee_ctx(&call.args, &env, returns) {
                        let targets_real_spec = specializations
                            .get(&callee)
                            .map(|ks| ks.contains(&ck))
                            .unwrap_or(false);
                        if targets_real_spec {
                            newly_bailed.insert(callee.clone());
                        }
                    }
                }
            }
            if newly_bailed.is_empty() {
                break;
            }
            for callee in newly_bailed {
                specializations.remove(&callee);
            }
        }

        // Call bindings: for every instance, resolve each call whose callee is
        // specialized. The caller_spec is the enclosing context when the caller
        // is itself specialized, else empty (the original body).
        let mut bindings: BTreeSet<CallBinding> = BTreeSet::new();
        for (func, ctx) in instances {
            let env = self.solve_env(func, ctx, returns);
            let caller_spec = if specializations.contains_key(func) {
                ctx.clone()
            } else {
                SpecKey::default()
            };
            for call in self.calls_of(func) {
                let Some(callee) = call.callee else { continue };
                if !specializations.contains_key(&callee) {
                    continue;
                }
                if let CalleeCtx::Definite(ck) = self.callee_ctx(&call.args, &env, returns) {
                    // Only emit for real specializations of the callee.
                    if specializations
                        .get(&callee)
                        .map(|ks| ks.contains(&ck))
                        .unwrap_or(false)
                    {
                        bindings.insert(CallBinding {
                            caller: func.clone(),
                            caller_spec: caller_spec.clone(),
                            ordinal: call.ordinal,
                            callee: callee.clone(),
                            callee_spec: ck,
                        });
                    }
                }
            }
        }

        MonoPlan {
            specializations,
            call_bindings: bindings.into_iter().collect(),
        }
    }
}

/// Outcome of resolving a call's argument shapes to a callee context.
enum CalleeCtx {
    /// No object arguments — irrelevant to specialization.
    NoObject,
    /// A clean, definite specialization context.
    Definite(SpecKey),
    /// An object argument holds ≥2 shapes at this site — unpartitionable.
    Ambiguous,
}

// ---- free helpers ---------------------------------------------------------

/// Union `src` into `dst`.
fn union_into(dst: &mut ShapeVal, src: &ShapeVal) {
    for t in src {
        dst.insert(t.clone());
    }
}

/// The ordered field names of an object literal, using exactly repr_infer's
/// acceptance rule (`record_object_literal`): every property must be an
/// `Init` with an `Identifier` key and a non-nested-object value. Anything
/// else is not a supported fixed-shape object → `None` (treated as no shape,
/// so it never drives a specialization).
fn clean_shape(obj: &ObjectExpression) -> Option<ShapeTuple> {
    let mut names = Vec::with_capacity(obj.properties.len());
    for prop in &obj.properties {
        let PropertyName::Identifier(key) = &prop.key else {
            return None;
        };
        if !matches!(prop.kind, ObjectPropertyKind::Init) {
            return None;
        }
        if matches!(prop.value, Expression::ObjectExpression(_)) {
            return None;
        }
        names.push(key.clone());
    }
    Some(names)
}

/// Collect every user `FunctionDeclaration` (recursively) by name.
fn collect_functions<'a>(
    stmts: &'a [Statement],
    out: &mut BTreeMap<String, &'a FunctionDeclaration>,
) {
    for stmt in stmts {
        match stmt {
            Statement::FunctionDeclaration(f) => {
                out.entry(f.name.clone()).or_insert(f);
                collect_functions(&f.body.body, out);
            }
            Statement::IfStatement(s) => {
                collect_functions(&s.consequent.body, out);
                if let Some(alt) = &s.alternate {
                    collect_functions(&alt.body, out);
                }
            }
            Statement::ForStatement(s) => collect_functions(&s.body.body, out),
            Statement::ForInStatement(s) => collect_functions_in_stmt(&s.body, out),
            Statement::ForOfStatement(s) => collect_functions_in_stmt(&s.body, out),
            Statement::WhileStatement(s) => collect_functions(&s.body.body, out),
            Statement::DoWhileStatement(s) => collect_functions(&s.body.body, out),
            Statement::BlockStatement(b) => collect_functions(&b.body, out),
            Statement::LabeledStatement(s) => collect_functions_in_stmt(&s.body, out),
            Statement::TryStatement(s) => {
                collect_functions(&s.block.body, out);
                if let Some(h) = &s.handler {
                    collect_functions(&h.body.body, out);
                }
                if let Some(f) = &s.finalizer {
                    collect_functions(&f.body, out);
                }
            }
            Statement::SwitchStatement(s) => {
                for case in &s.cases {
                    collect_functions(&case.consequent, out);
                }
            }
            _ => {}
        }
    }
}

fn collect_functions_in_stmt<'a>(
    stmt: &'a Statement,
    out: &mut BTreeMap<String, &'a FunctionDeclaration>,
) {
    collect_functions(std::slice::from_ref(stmt), out);
}

/// Visit every `return <expr>;` in a function body (no nested-fn descent).
fn collect_returns(stmts: &[Statement], f: &mut dyn FnMut(&Expression)) {
    for stmt in stmts {
        match stmt {
            Statement::ReturnStatement(r) => {
                if let Some(arg) = &r.argument {
                    f(arg);
                }
            }
            Statement::IfStatement(s) => {
                collect_returns(&s.consequent.body, f);
                if let Some(alt) = &s.alternate {
                    collect_returns(&alt.body, f);
                }
            }
            Statement::ForStatement(s) => collect_returns(&s.body.body, f),
            Statement::ForInStatement(s) => collect_returns_in_stmt(&s.body, f),
            Statement::ForOfStatement(s) => collect_returns_in_stmt(&s.body, f),
            Statement::WhileStatement(s) => collect_returns(&s.body.body, f),
            Statement::DoWhileStatement(s) => collect_returns(&s.body.body, f),
            Statement::BlockStatement(b) => collect_returns(&b.body, f),
            Statement::LabeledStatement(s) => collect_returns_in_stmt(&s.body, f),
            Statement::TryStatement(s) => {
                collect_returns(&s.block.body, f);
                if let Some(h) = &s.handler {
                    collect_returns(&h.body.body, f);
                }
                if let Some(fin) = &s.finalizer {
                    collect_returns(&fin.body, f);
                }
            }
            Statement::SwitchStatement(s) => {
                for case in &s.cases {
                    collect_returns(&case.consequent, f);
                }
            }
            _ => {}
        }
    }
}

fn collect_returns_in_stmt(stmt: &Statement, f: &mut dyn FnMut(&Expression)) {
    collect_returns(std::slice::from_ref(stmt), f);
}

/// Enumerate calls in statements pre-order (no nested-fn descent).
fn collect_calls<'a>(stmts: &'a [Statement], ordinal: &mut usize, out: &mut Vec<CallInfo<'a>>) {
    for stmt in stmts {
        collect_calls_in_stmt(stmt, ordinal, out);
    }
}

fn collect_calls_in_stmt<'a>(stmt: &'a Statement, ordinal: &mut usize, out: &mut Vec<CallInfo<'a>>) {
    match stmt {
        Statement::ExpressionStatement(es) => calls_in_expr(&es.expression, ordinal, out),
        Statement::ReturnStatement(r) => {
            if let Some(arg) = &r.argument {
                calls_in_expr(arg, ordinal, out);
            }
        }
        Statement::VariableDeclaration(decl) => {
            for d in &decl.declarations {
                if let Some(init) = &d.init {
                    calls_in_expr(init, ordinal, out);
                }
            }
        }
        Statement::IfStatement(s) => {
            calls_in_expr(&s.test, ordinal, out);
            collect_calls(&s.consequent.body, ordinal, out);
            if let Some(alt) = &s.alternate {
                collect_calls(&alt.body, ordinal, out);
            }
        }
        Statement::ForStatement(s) => {
            if let Some(ForInit::VariableDeclaration(decl)) = &s.init {
                for d in &decl.declarations {
                    if let Some(init) = &d.init {
                        calls_in_expr(init, ordinal, out);
                    }
                }
            } else if let Some(ForInit::Expression(e)) = &s.init {
                calls_in_expr(e, ordinal, out);
            }
            if let Some(test) = &s.test {
                calls_in_expr(test, ordinal, out);
            }
            if let Some(update) = &s.update {
                calls_in_expr(update, ordinal, out);
            }
            collect_calls(&s.body.body, ordinal, out);
        }
        Statement::ForInStatement(s) => {
            calls_in_expr(&s.right, ordinal, out);
            collect_calls_in_stmt(&s.body, ordinal, out);
        }
        Statement::ForOfStatement(s) => {
            calls_in_expr(&s.right, ordinal, out);
            collect_calls_in_stmt(&s.body, ordinal, out);
        }
        Statement::WhileStatement(s) => {
            calls_in_expr(&s.test, ordinal, out);
            collect_calls(&s.body.body, ordinal, out);
        }
        Statement::DoWhileStatement(s) => {
            collect_calls(&s.body.body, ordinal, out);
            calls_in_expr(&s.test, ordinal, out);
        }
        Statement::BlockStatement(b) => collect_calls(&b.body, ordinal, out),
        Statement::LabeledStatement(s) => collect_calls_in_stmt(&s.body, ordinal, out),
        Statement::TryStatement(s) => {
            collect_calls(&s.block.body, ordinal, out);
            if let Some(h) = &s.handler {
                collect_calls(&h.body.body, ordinal, out);
            }
            if let Some(f) = &s.finalizer {
                collect_calls(&f.body, ordinal, out);
            }
        }
        Statement::SwitchStatement(s) => {
            calls_in_expr(&s.discriminant, ordinal, out);
            for case in &s.cases {
                if let Some(test) = &case.test {
                    calls_in_expr(test, ordinal, out);
                }
                collect_calls(&case.consequent, ordinal, out);
            }
        }
        Statement::ThrowStatement(s) => calls_in_expr(&s.argument, ordinal, out),
        _ => {}
    }
}

/// Pre-order enumeration of `CallExpression`s within an expression. A call is
/// assigned its ordinal on entry, before its callee and arguments are visited.
fn calls_in_expr<'a>(expr: &'a Expression, ordinal: &mut usize, out: &mut Vec<CallInfo<'a>>) {
    match expr {
        Expression::CallExpression(call) => {
            let this = *ordinal;
            *ordinal += 1;
            let callee = match &call.callee {
                Expression::Identifier(name) => Some(name.clone()),
                _ => None,
            };
            out.push(CallInfo {
                ordinal: this,
                callee,
                args: call.args.iter().collect(),
            });
            calls_in_expr(&call.callee, ordinal, out);
            for arg in &call.args {
                calls_in_expr(arg, ordinal, out);
            }
        }
        Expression::BinaryExpression(b) => {
            calls_in_expr(&b.left, ordinal, out);
            calls_in_expr(&b.right, ordinal, out);
        }
        Expression::LogicalExpression(l) => {
            calls_in_expr(&l.left, ordinal, out);
            calls_in_expr(&l.right, ordinal, out);
        }
        Expression::UnaryExpression(u) => calls_in_expr(&u.argument, ordinal, out),
        Expression::ConditionalExpression(c) => {
            calls_in_expr(&c.test, ordinal, out);
            calls_in_expr(&c.consequent, ordinal, out);
            calls_in_expr(&c.alternate, ordinal, out);
        }
        Expression::AssignmentExpression(a) => {
            calls_in_expr(&a.left, ordinal, out);
            calls_in_expr(&a.right, ordinal, out);
        }
        Expression::ParenthesizedExpression(p) => calls_in_expr(&p.expression, ordinal, out),
        Expression::MemberExpression(m) => {
            calls_in_expr(&m.object, ordinal, out);
            if let Some(idx) = &m.computed_index {
                calls_in_expr(idx, ordinal, out);
            }
        }
        Expression::SequenceExpression(s) => {
            for e in &s.expressions {
                calls_in_expr(e, ordinal, out);
            }
        }
        _ => {}
    }
}
