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
    CallExpression, Expression, ForInit, FunctionDeclaration, ObjectExpression, ObjectPropertyKind,
    PropertyName, Statement,
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

    /// The fresh clone name for `spec` of `func`, or `None` if `func` is not
    /// specialized or `spec` is not one of its keys. The index is `spec`'s
    /// position in the (sorted) key vector, so it is stable and unique.
    pub fn clone_name(&self, func: &str, spec: &SpecKey) -> Option<String> {
        let keys = self.specializations.get(func)?;
        let idx = keys.iter().position(|k| k == spec)?;
        Some(mangled_name(func, idx))
    }
}

/// The fresh name of the `idx`-th specialization of `func`. The `{` / `}` are
/// **not** valid identifier characters in the lexer (which accepts only
/// alphanumerics, `_` and `$`), so this can never collide with any user-written
/// identifier — the parser cannot produce this name. Downstream it is only ever
/// used as an opaque map key / wasm export name (never re-lexed), so the shape
/// is free.
fn mangled_name(func: &str, idx: usize) -> String {
    format!("{func}${{{idx}}}")
}

/// Rewrite the callee identifier of selected call sites in one function body,
/// keyed by the pre-order call ordinal from [`compute_mono_plan`]. This is the
/// **only** mutation Task 2 performs on the AST; it drives the *mutable* arm of
/// the shared [`define_call_walk!`] traversal, so its ordinals match the plan's
/// by construction (see the macro's docs).
pub fn rewrite_callees_in_body(body: &mut [Statement], renames: &BTreeMap<usize, String>) {
    if renames.is_empty() {
        return;
    }
    let mut ordinal = 0usize;
    walk_calls_mut(body, &mut ordinal, &mut |ord, call| {
        if let Some(name) = renames.get(&ord) {
            call.callee = Expression::Identifier(name.clone());
        }
    });
}

/// Apply object-shape monomorphization to a parsed program in place.
///
/// Computes the [`MonoPlan`] and, if non-empty, clones every function reached by
/// ≥2 distinct object-param shape tuples once per tuple under a fresh
/// [`mangled_name`], rewrites every specialized call site (including nested
/// transitive calls inside a clone) to its matching clone, and drops the now-dead
/// specialized originals. An **empty plan is a hard no-op** — the statements are
/// left byte-identical — which is the case for all current fixtures/tests.
///
/// After this returns the untouched resolver → repr_infer → codegen pipeline sees
/// each clone as a separate, monomorphic function (every shape decision keys off
/// the function name), so no downstream stage needs any change.
pub fn monomorphize_statements(statements: &mut Vec<Statement>) {
    let plan = compute_mono_plan(statements);
    if plan.is_empty() {
        return;
    }
    apply_plan(statements, &plan);
}

/// Materialize the plan: build clones, rewrite call sites, drop dead originals.
fn apply_plan(statements: &mut Vec<Statement>, plan: &MonoPlan) {
    // (caller, caller_spec) -> (ordinal -> fresh callee name).
    let mut renames: BTreeMap<(String, SpecKey), BTreeMap<usize, String>> = BTreeMap::new();
    for b in plan.call_bindings() {
        if let Some(name) = plan.clone_name(&b.callee, &b.callee_spec) {
            renames
                .entry((b.caller.clone(), b.caller_spec.clone()))
                .or_default()
                .insert(b.ordinal, name);
        }
    }

    // Snapshot the original declaration of every specialized function so we can
    // clone from it after the tree is mutated.
    let mut originals: BTreeMap<String, FunctionDeclaration> = BTreeMap::new();
    snapshot_specialized_decls(statements, &plan.specializations, &mut originals);

    // Build the specialized clones, grouped by original function name. Each
    // clone's body is rewritten for the transitive calls it makes under its own
    // specialization context.
    let mut clones_by_func: BTreeMap<String, Vec<Statement>> = BTreeMap::new();
    for (func, keys) in &plan.specializations {
        let Some(orig) = originals.get(func) else {
            continue;
        };
        for (idx, key) in keys.iter().enumerate() {
            let mut cloned = orig.clone();
            cloned.name = mangled_name(func, idx);
            if let Some(rmap) = renames.get(&(func.clone(), key.clone())) {
                rewrite_callees_in_body(&mut cloned.body.body, rmap);
            }
            clones_by_func
                .entry(func.clone())
                .or_default()
                .push(Statement::FunctionDeclaration(cloned));
        }
    }

    // Rewrite every surviving caller body (module scope + each function's own
    // unspecialized body) for its calls into specialized callees.
    rewrite_tree(statements, TOP_LEVEL, &renames);
    // Replace each now-dead specialized original *in place* with its clones, so
    // the clones keep the original's declaration position (a declaration-before-
    // use resolver still sees them ahead of the rewritten call sites).
    splice_clones(statements, &mut clones_by_func);
}

/// Recursively rewrite call sites in a caller body and every nested function
/// body. `caller` names the enclosing function (`_start` at module scope); a
/// surviving body is always the *unspecialized* context (empty `SpecKey`) — a
/// specialized function's real bodies are its clones, rewritten separately.
fn rewrite_tree(
    body: &mut [Statement],
    caller: &str,
    renames: &BTreeMap<(String, SpecKey), BTreeMap<usize, String>>,
) {
    if let Some(rmap) = renames.get(&(caller.to_string(), SpecKey::default())) {
        rewrite_callees_in_body(body, rmap);
    }
    for stmt in body.iter_mut() {
        rewrite_nested_fns(stmt, renames);
    }
}

/// Descend a statement's child blocks to find nested function declarations and
/// rewrite each as its own caller context. Control-flow bodies themselves are
/// already handled by the enclosing body's [`rewrite_callees_in_body`] walk;
/// this only re-roots the ordinal counter at each nested *function* boundary
/// (which the call walk deliberately does not cross).
fn rewrite_nested_fns(
    stmt: &mut Statement,
    renames: &BTreeMap<(String, SpecKey), BTreeMap<usize, String>>,
) {
    match stmt {
        Statement::FunctionDeclaration(f) => {
            let name = f.name.clone();
            rewrite_tree(&mut f.body.body, &name, renames);
        }
        Statement::IfStatement(s) => {
            for st in &mut s.consequent.body {
                rewrite_nested_fns(st, renames);
            }
            if let Some(alt) = &mut s.alternate {
                for st in &mut alt.body {
                    rewrite_nested_fns(st, renames);
                }
            }
        }
        Statement::ForStatement(s) => descend_block(&mut s.body.body, renames),
        Statement::ForInStatement(s) => rewrite_nested_fns(&mut s.body, renames),
        Statement::ForOfStatement(s) => rewrite_nested_fns(&mut s.body, renames),
        Statement::WhileStatement(s) => descend_block(&mut s.body.body, renames),
        Statement::DoWhileStatement(s) => descend_block(&mut s.body.body, renames),
        Statement::BlockStatement(b) => descend_block(&mut b.body, renames),
        Statement::LabeledStatement(s) => rewrite_nested_fns(&mut s.body, renames),
        Statement::TryStatement(s) => {
            descend_block(&mut s.block.body, renames);
            if let Some(h) = &mut s.handler {
                descend_block(&mut h.body.body, renames);
            }
            if let Some(f) = &mut s.finalizer {
                descend_block(&mut f.body, renames);
            }
        }
        Statement::SwitchStatement(s) => {
            for case in &mut s.cases {
                descend_block(&mut case.consequent, renames);
            }
        }
        _ => {}
    }
}

fn descend_block(
    body: &mut [Statement],
    renames: &BTreeMap<(String, SpecKey), BTreeMap<usize, String>>,
) {
    for st in body.iter_mut() {
        rewrite_nested_fns(st, renames);
    }
}

/// Clone the declaration of every specialized function, by name (recursively).
fn snapshot_specialized_decls(
    stmts: &[Statement],
    specs: &BTreeMap<String, Vec<SpecKey>>,
    out: &mut BTreeMap<String, FunctionDeclaration>,
) {
    for stmt in stmts {
        match stmt {
            Statement::FunctionDeclaration(f) => {
                if specs.contains_key(&f.name) {
                    out.entry(f.name.clone()).or_insert_with(|| f.clone());
                }
                snapshot_specialized_decls(&f.body.body, specs, out);
            }
            Statement::IfStatement(s) => {
                snapshot_specialized_decls(&s.consequent.body, specs, out);
                if let Some(alt) = &s.alternate {
                    snapshot_specialized_decls(&alt.body, specs, out);
                }
            }
            Statement::ForStatement(s) => snapshot_specialized_decls(&s.body.body, specs, out),
            Statement::ForInStatement(s) => {
                snapshot_specialized_decls(std::slice::from_ref(&s.body), specs, out)
            }
            Statement::ForOfStatement(s) => {
                snapshot_specialized_decls(std::slice::from_ref(&s.body), specs, out)
            }
            Statement::WhileStatement(s) => snapshot_specialized_decls(&s.body.body, specs, out),
            Statement::DoWhileStatement(s) => snapshot_specialized_decls(&s.body.body, specs, out),
            Statement::BlockStatement(b) => snapshot_specialized_decls(&b.body, specs, out),
            Statement::LabeledStatement(s) => {
                snapshot_specialized_decls(std::slice::from_ref(&s.body), specs, out)
            }
            Statement::TryStatement(s) => {
                snapshot_specialized_decls(&s.block.body, specs, out);
                if let Some(h) = &s.handler {
                    snapshot_specialized_decls(&h.body.body, specs, out);
                }
                if let Some(f) = &s.finalizer {
                    snapshot_specialized_decls(&f.body, specs, out);
                }
            }
            Statement::SwitchStatement(s) => {
                for case in &s.cases {
                    snapshot_specialized_decls(&case.consequent, specs, out);
                }
            }
            _ => {}
        }
    }
}

/// Replace every specialized function's (now-dead) original declaration with its
/// clones, in place, preserving declaration order. Their call sites have all
/// been rewritten to clones, so the original is unreachable; leaving it would
/// give repr_infer a shape-less param and re-trigger E5506. Recurses through
/// every block so a nested declaration is handled too. `clones` is drained as
/// each function's declaration site is found (each is declared once).
fn splice_clones(body: &mut Vec<Statement>, clones: &mut BTreeMap<String, Vec<Statement>>) {
    let mut rebuilt: Vec<Statement> = Vec::with_capacity(body.len());
    for mut stmt in body.drain(..) {
        if let Statement::FunctionDeclaration(f) = &stmt {
            if let Some(fn_clones) = clones.remove(&f.name) {
                rebuilt.extend(fn_clones);
                continue; // drop the original; its clones take its place
            }
        }
        splice_clones_in_stmt(&mut stmt, clones);
        rebuilt.push(stmt);
    }
    *body = rebuilt;
}

fn splice_clones_in_stmt(stmt: &mut Statement, clones: &mut BTreeMap<String, Vec<Statement>>) {
    match stmt {
        Statement::FunctionDeclaration(f) => splice_clones(&mut f.body.body, clones),
        Statement::IfStatement(s) => {
            splice_clones(&mut s.consequent.body, clones);
            if let Some(alt) = &mut s.alternate {
                splice_clones(&mut alt.body, clones);
            }
        }
        Statement::ForStatement(s) => splice_clones(&mut s.body.body, clones),
        Statement::ForInStatement(s) => splice_clones_in_stmt(&mut s.body, clones),
        Statement::ForOfStatement(s) => splice_clones_in_stmt(&mut s.body, clones),
        Statement::WhileStatement(s) => splice_clones(&mut s.body.body, clones),
        Statement::DoWhileStatement(s) => splice_clones(&mut s.body.body, clones),
        Statement::BlockStatement(b) => splice_clones(&mut b.body, clones),
        Statement::LabeledStatement(s) => splice_clones_in_stmt(&mut s.body, clones),
        Statement::TryStatement(s) => {
            splice_clones(&mut s.block.body, clones);
            if let Some(h) = &mut s.handler {
                splice_clones(&mut h.body.body, clones);
            }
            if let Some(f) = &mut s.finalizer {
                splice_clones(&mut f.body, clones);
            }
        }
        Statement::SwitchStatement(s) => {
            for case in &mut s.cases {
                splice_clones(&mut case.consequent, clones);
            }
        }
        _ => {}
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
///
/// The argument expressions are owned clones so the enumeration (built by the
/// shared [`walk_calls`] visitor) carries no borrow of the body — the mutable
/// rewrite pass ([`rewrite_callees_in_body`]) shares the *same* walk over a
/// `&mut` body, and decoupling the two lifetimes lets one macro generate both.
struct CallInfo {
    ordinal: usize,
    callee: Option<String>,
    args: Vec<Expression>,
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
                env.entry(name.clone()).or_default().insert(tuple.clone());
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
    fn collect_env(
        &self,
        stmts: &[Statement],
        env: &mut Env,
        returns: &BTreeMap<Instance, ShapeVal>,
    ) {
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

    fn collect_stmt(
        &self,
        stmt: &Statement,
        env: &mut Env,
        returns: &BTreeMap<Instance, ShapeVal>,
    ) {
        self.collect_env(std::slice::from_ref(stmt), env, returns);
    }

    /// Evaluate the object shape(s) an expression can carry in `env`.
    fn eval(
        &self,
        expr: &Expression,
        env: &Env,
        returns: &BTreeMap<Instance, ShapeVal>,
    ) -> ShapeVal {
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
                match self.callee_ctx(&call.args, env, returns) {
                    CalleeCtx::Definite(ck) => returns
                        .get(&(callee.clone(), ck))
                        .cloned()
                        .unwrap_or_default(),
                    _ => ShapeVal::new(),
                }
            }
            _ => ShapeVal::new(),
        }
    }

    /// The specialization context a call selects from its argument shapes.
    fn callee_ctx(
        &self,
        args: &[Expression],
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
    fn return_shape(
        &self,
        func: &str,
        env: &Env,
        returns: &BTreeMap<Instance, ShapeVal>,
    ) -> ShapeVal {
        let mut out = ShapeVal::new();
        collect_returns(self.body_of(func), &mut |expr| {
            union_into(&mut out, &self.eval(expr, env, returns));
        });
        out
    }

    /// Enumerate every recognized call in a function body, pre-order, without
    /// descending into nested function declarations. Uses the shared
    /// [`walk_calls`] traversal — the *same* one the mutable rewrite pass uses —
    /// so call-site ordinals are identical between analysis and rewrite.
    fn calls_of(&self, func: &str) -> Vec<CallInfo> {
        let mut calls = Vec::new();
        let mut ordinal = 0usize;
        walk_calls(self.body_of(func), &mut ordinal, &mut |ordinal, call| {
            let callee = match &call.callee {
                Expression::Identifier(name) => Some(name.clone()),
                _ => None,
            };
            calls.push(CallInfo {
                ordinal,
                callee,
                args: call.args.clone(),
            });
        });
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
            // Fail-closed guard (Task 7a-2 follow-up): a function whose body
            // contains a nested `function` declaration cannot be safely cloned
            // per shape. Codegen exports every nested function declaration by
            // name (`kali_codegen::lower`), so cloning the enclosing function N
            // times would duplicate the nested declaration into N same-named
            // wasm exports — wasm validation rejects that with an opaque
            // duplicate-export error rather than a clean diagnostic. Drop the
            // function from `specializations` here (before the transitive-bail
            // pass below) instead: this is a pure narrowing — it only removes a
            // would-be specialization, never adds one — so if `func` was only
            // multi-shape because of it, the existing repr_infer E5506
            // conflicting-object-shapes diagnostic fires downstream instead;
            // and any callee that was only specialized because of `func`
            // routing distinct shapes to it gets cleaned up by the transitive
            // fail-closed post-pass immediately below, which already handles
            // exactly this "un-specialized multi-shape caller" shape.
            if self
                .funcs
                .get(func.as_str())
                .map(|f| body_has_nested_fn_decl(&f.body.body))
                .unwrap_or(false)
            {
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
                let poly = keys_by_func
                    .get(func)
                    .map(|k| k.len() >= 2)
                    .unwrap_or(false);
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

/// True if `stmts` contains a `function` declaration nested anywhere inside —
/// directly, or inside a `block`/`if`/`for`/`for-in`/`for-of`/`while`/
/// `do-while`/`labeled`/`try`/`switch` body. Used by [`Analyzer::build_plan`]
/// to fail-closed-drop a would-be-specialized function whose clones would
/// otherwise duplicate a nested declaration into multiple same-named wasm
/// exports (see the call site's doc comment). One hit is enough, so this does
/// not need to descend into a found `FunctionDeclaration`'s own body.
fn body_has_nested_fn_decl(stmts: &[Statement]) -> bool {
    stmts.iter().any(stmt_has_nested_fn_decl)
}

fn stmt_has_nested_fn_decl(stmt: &Statement) -> bool {
    match stmt {
        Statement::FunctionDeclaration(_) => true,
        Statement::IfStatement(s) => {
            body_has_nested_fn_decl(&s.consequent.body)
                || s.alternate
                    .as_ref()
                    .is_some_and(|alt| body_has_nested_fn_decl(&alt.body))
        }
        Statement::ForStatement(s) => body_has_nested_fn_decl(&s.body.body),
        Statement::ForInStatement(s) => stmt_has_nested_fn_decl(&s.body),
        Statement::ForOfStatement(s) => stmt_has_nested_fn_decl(&s.body),
        Statement::WhileStatement(s) => body_has_nested_fn_decl(&s.body.body),
        Statement::DoWhileStatement(s) => body_has_nested_fn_decl(&s.body.body),
        Statement::BlockStatement(b) => body_has_nested_fn_decl(&b.body),
        Statement::LabeledStatement(s) => stmt_has_nested_fn_decl(&s.body),
        Statement::TryStatement(s) => {
            body_has_nested_fn_decl(&s.block.body)
                || s.handler
                    .as_ref()
                    .is_some_and(|h| body_has_nested_fn_decl(&h.body.body))
                || s.finalizer
                    .as_ref()
                    .is_some_and(|f| body_has_nested_fn_decl(&f.body))
        }
        Statement::SwitchStatement(s) => s
            .cases
            .iter()
            .any(|case| body_has_nested_fn_decl(&case.consequent)),
        _ => false,
    }
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

/// Pre-order `CallExpression` walk, generated **once** for both an immutable
/// visitor (the plan analyzer's [`Analyzer::calls_of`]) and a mutable visitor
/// (Task 2's [`rewrite_callees_in_body`]).
///
/// Generating both from a single macro body is the lockstep guarantee the whole
/// monomorphization contract rests on: the recursion shape and — critically —
/// the `*ordinal += 1` placement are *literally the same source text* for the
/// analysis walk and the rewrite walk, so a call site's pre-order ordinal is
/// identical in both. `CallExpression` carries no node id, so this ordinal is
/// the *only* thing tying a plan `CallBinding` to the physical node the rewrite
/// mutates; a divergent second walk here would silently route a call site to the
/// wrong clone (the [[kali-forin-spec4a]] two-walks-in-lockstep fail-open class).
///
/// The two arms differ only in the reference kind (`&` vs `&mut`) and what the
/// visitor is handed; ordinal assignment is on entry to each call, before its
/// callee and arguments are visited.
macro_rules! define_call_walk {
    ($walk_stmts:ident, $walk_stmt:ident, $walk_expr:ident, $($mut_:tt)?) => {
        fn $walk_stmts(
            stmts: & $($mut_)? [Statement],
            ordinal: &mut usize,
            visit: &mut dyn FnMut(usize, & $($mut_)? CallExpression),
        ) {
            for stmt in stmts {
                $walk_stmt(stmt, ordinal, visit);
            }
        }

        fn $walk_stmt(
            stmt: & $($mut_)? Statement,
            ordinal: &mut usize,
            visit: &mut dyn FnMut(usize, & $($mut_)? CallExpression),
        ) {
            match stmt {
                Statement::ExpressionStatement(es) => {
                    $walk_expr(& $($mut_)? es.expression, ordinal, visit)
                }
                Statement::ReturnStatement(r) => {
                    if let Some(arg) = & $($mut_)? r.argument {
                        $walk_expr(arg, ordinal, visit);
                    }
                }
                Statement::VariableDeclaration(decl) => {
                    for d in & $($mut_)? decl.declarations {
                        if let Some(init) = & $($mut_)? d.init {
                            $walk_expr(init, ordinal, visit);
                        }
                    }
                }
                Statement::IfStatement(s) => {
                    $walk_expr(& $($mut_)? s.test, ordinal, visit);
                    $walk_stmts(& $($mut_)? s.consequent.body, ordinal, visit);
                    if let Some(alt) = & $($mut_)? s.alternate {
                        $walk_stmts(& $($mut_)? alt.body, ordinal, visit);
                    }
                }
                Statement::ForStatement(s) => {
                    if let Some(ForInit::VariableDeclaration(decl)) = & $($mut_)? s.init {
                        for d in & $($mut_)? decl.declarations {
                            if let Some(init) = & $($mut_)? d.init {
                                $walk_expr(init, ordinal, visit);
                            }
                        }
                    } else if let Some(ForInit::Expression(e)) = & $($mut_)? s.init {
                        $walk_expr(e, ordinal, visit);
                    }
                    if let Some(test) = & $($mut_)? s.test {
                        $walk_expr(test, ordinal, visit);
                    }
                    if let Some(update) = & $($mut_)? s.update {
                        $walk_expr(update, ordinal, visit);
                    }
                    $walk_stmts(& $($mut_)? s.body.body, ordinal, visit);
                }
                Statement::ForInStatement(s) => {
                    $walk_expr(& $($mut_)? s.right, ordinal, visit);
                    $walk_stmt(& $($mut_)? s.body, ordinal, visit);
                }
                Statement::ForOfStatement(s) => {
                    $walk_expr(& $($mut_)? s.right, ordinal, visit);
                    $walk_stmt(& $($mut_)? s.body, ordinal, visit);
                }
                Statement::WhileStatement(s) => {
                    $walk_expr(& $($mut_)? s.test, ordinal, visit);
                    $walk_stmts(& $($mut_)? s.body.body, ordinal, visit);
                }
                Statement::DoWhileStatement(s) => {
                    $walk_stmts(& $($mut_)? s.body.body, ordinal, visit);
                    $walk_expr(& $($mut_)? s.test, ordinal, visit);
                }
                Statement::BlockStatement(b) => $walk_stmts(& $($mut_)? b.body, ordinal, visit),
                Statement::LabeledStatement(s) => $walk_stmt(& $($mut_)? s.body, ordinal, visit),
                Statement::TryStatement(s) => {
                    $walk_stmts(& $($mut_)? s.block.body, ordinal, visit);
                    if let Some(h) = & $($mut_)? s.handler {
                        $walk_stmts(& $($mut_)? h.body.body, ordinal, visit);
                    }
                    if let Some(f) = & $($mut_)? s.finalizer {
                        $walk_stmts(& $($mut_)? f.body, ordinal, visit);
                    }
                }
                Statement::SwitchStatement(s) => {
                    $walk_expr(& $($mut_)? s.discriminant, ordinal, visit);
                    for case in & $($mut_)? s.cases {
                        if let Some(test) = & $($mut_)? case.test {
                            $walk_expr(test, ordinal, visit);
                        }
                        $walk_stmts(& $($mut_)? case.consequent, ordinal, visit);
                    }
                }
                Statement::ThrowStatement(s) => $walk_expr(& $($mut_)? s.argument, ordinal, visit),
                _ => {}
            }
        }

        fn $walk_expr(
            expr: & $($mut_)? Expression,
            ordinal: &mut usize,
            visit: &mut dyn FnMut(usize, & $($mut_)? CallExpression),
        ) {
            match expr {
                Expression::CallExpression(call) => {
                    let this = *ordinal;
                    *ordinal += 1;
                    visit(this, & $($mut_)? **call);
                    $walk_expr(& $($mut_)? call.callee, ordinal, visit);
                    for arg in & $($mut_)? call.args {
                        $walk_expr(arg, ordinal, visit);
                    }
                }
                Expression::BinaryExpression(b) => {
                    $walk_expr(& $($mut_)? b.left, ordinal, visit);
                    $walk_expr(& $($mut_)? b.right, ordinal, visit);
                }
                Expression::LogicalExpression(l) => {
                    $walk_expr(& $($mut_)? l.left, ordinal, visit);
                    $walk_expr(& $($mut_)? l.right, ordinal, visit);
                }
                Expression::UnaryExpression(u) => $walk_expr(& $($mut_)? u.argument, ordinal, visit),
                Expression::ConditionalExpression(c) => {
                    $walk_expr(& $($mut_)? c.test, ordinal, visit);
                    $walk_expr(& $($mut_)? c.consequent, ordinal, visit);
                    $walk_expr(& $($mut_)? c.alternate, ordinal, visit);
                }
                Expression::AssignmentExpression(a) => {
                    $walk_expr(& $($mut_)? a.left, ordinal, visit);
                    $walk_expr(& $($mut_)? a.right, ordinal, visit);
                }
                Expression::ParenthesizedExpression(p) => {
                    $walk_expr(& $($mut_)? p.expression, ordinal, visit)
                }
                Expression::MemberExpression(m) => {
                    $walk_expr(& $($mut_)? m.object, ordinal, visit);
                    if let Some(idx) = & $($mut_)? m.computed_index {
                        $walk_expr(idx, ordinal, visit);
                    }
                }
                Expression::SequenceExpression(s) => {
                    for e in & $($mut_)? s.expressions {
                        $walk_expr(e, ordinal, visit);
                    }
                }
                _ => {}
            }
        }
    };
}

define_call_walk!(walk_calls, walk_call_stmt, walk_call_expr,);
define_call_walk!(walk_calls_mut, walk_call_stmt_mut, walk_call_expr_mut, mut);
