//! Growable runtime-array candidate analysis (throw-fallout Stage 4).
//!
//! THE single choke-point predicate for the growable-array lane:
//! [`growable_array_candidates`] decides, purely syntactically, which
//! `const`/`let` array-literal bindings of one function are ALLOWED to be
//! promoted to codegen's growable (push-accumulated) tagged-handle lane.
//! Promotion is an ALLOWLIST of safe positions — the program-history lesson
//! (`kali-forin-spec4a`): for "an internal repr must not escape", allowlist
//! the safe positions at one choke point; never denylist sinks.
//!
//! A binding is a candidate iff:
//! - it is declared EXACTLY ONCE in the function (params and nested
//!   declarations of the same name count against it — shadowing would make
//!   the name-based occurrence scan unsound), by a `const` or `let`
//!   declarator whose init is an array literal of scalar-shaped seeds
//!   (numeric/boolean literals and arithmetic over them — Task 2's i64 lane;
//!   Task 3 relaxes seeds/pushes to strings),
//! - it has at least one `.push` occurrence, and
//! - EVERY occurrence of its name anywhere in the function body — including
//!   inside nested functions/closures, where NO position is safe (a capture
//!   is an escape) — is one of the safe growable positions:
//!   * the declarator init itself,
//!   * `x.push(v)` receiver (exactly one scalar-shaped argument),
//!   * `x.length` read,
//!   * `x[i]` index READ (never an index write),
//!   * `for..of` RHS iterable (rejected fail-closed E5506 until Task 4),
//!   * `x.join(sep)` receiver, 0/1 args (rejected fail-closed E5506 until
//!     Task 5).
//!
//! ANY other occurrence — a bare read, call argument, `return x`, a store
//! into an object/array, reassignment or compound assignment of the binding,
//! an index write `x[i] = v`, a non-push/join method receiver, `delete`,
//!  `for..in` RHS, `++`/`--` — disqualifies the name: NO promotion, and the
//! binding keeps the pre-existing plain lane byte-identically (the fail-open
//! push no-op it had before this stage; Task 6 sweeps those to E5506).
//!
//! The scanner is an EXHAUSTIVE match over `kali_ast`'s `Statement` and
//! `Expression` enums with no wildcard arm, so adding a new AST variant
//! forces a decision here (default-deny by construction). Statement/
//! expression kinds this analysis cannot cheaply see through (classes, JSX
//! trees, `with`, enums, import/export) POISON the whole function — no
//! candidate is promoted in it (sound: promotion misses only ever keep the
//! old behavior).
//!
//! The candidate set is SYNTACTIC (pre-repr): `kali_types::repr_infer`
//! intersects it with the solved repr axes at `emit_table` time (every
//! pushed value and the element axis must prove plain i64, never
//! float/string/object/array/function) before recording the promotion into
//! [`kali_common::ReprTable::set_growable_array_binding`].

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::{
    Expression, ExpressionOrSpread, ForInLefthand, ForInit, ForOfLefthand, LiteralValue, Statement,
};

/// One recognized `candidate.push(<scalar-shaped arg>)` occurrence, in
/// source order. `repr_infer` uses these to find the pushed-value repr nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GrowablePushSite {
    /// Receiver binding name.
    pub(crate) name: String,
    /// The pushed argument, when it is a bare identifier — checked against
    /// object/array/function bindings at `emit_table` time (a raw handle
    /// stored as an element would read back as a number: a miscompile).
    pub(crate) arg_identifier: Option<String>,
}

#[derive(Debug, Default)]
struct DeclInfo {
    /// How many declarations of this name the function contains (params,
    /// declarators, catch params, nested functions'/closures' declarations
    /// and params all count — any count > 1 disqualifies).
    count: usize,
    /// True when the single declaration is a `const`/`let` declarator whose
    /// init is an array literal of scalar-shaped seeds.
    growable_shape: bool,
}

#[derive(Debug, Default)]
struct Scan {
    decls: BTreeMap<String, DeclInfo>,
    /// Names with at least one occurrence outside the safe-position allowlist.
    unsafe_names: BTreeSet<String>,
    /// Clean `.push` receivers, in source order.
    pushes: Vec<GrowablePushSite>,
    /// Every identifier that appears as the base of a `.push`/`["push"]`/
    /// `?.push` CALL, in ANY position or nesting and regardless of argument
    /// shape/arity (Task 6). A growable-shape binding that is a push receiver
    /// here but fails promotion (not a candidate) is the silent-poison class:
    /// it must fail closed (E5506), never keep the pre-existing push-no-op.
    push_receiver_mentions: BTreeSet<String>,
    /// A construct the scanner cannot see through appeared: no candidates.
    poisoned: bool,
}

/// Names of `func_params` + statements' bindings eligible for growable
/// promotion, plus their recognized push sites. See the module doc — this is
/// the choke-point predicate Tasks 3–6 extend.
pub(crate) fn growable_array_candidates(
    func_params: &[String],
    body: &[Statement],
) -> (BTreeSet<String>, Vec<GrowablePushSite>, BTreeSet<String>) {
    let mut scan = Scan::default();
    for param in func_params {
        scan.declare(param, false);
    }
    for stmt in body {
        scan.stmt(stmt, false);
    }
    if scan.poisoned {
        return (BTreeSet::new(), Vec::new(), BTreeSet::new());
    }
    let candidates: BTreeSet<String> = scan
        .decls
        .iter()
        .filter(|(name, info)| {
            info.count == 1
                && info.growable_shape
                && !scan.unsafe_names.contains(*name)
                && scan.pushes.iter().any(|push| &push.name == *name)
        })
        .map(|(name, _)| name.clone())
        .collect();
    // Task 6 fail-closed reject set: a binding with the growable SHAPE
    // (declared exactly once as a `const`/`let` array-literal of scalar seeds)
    // that is a `.push` RECEIVER but is NOT a promotable candidate — because
    // some occurrence sits outside the safe-position allowlist (escaping via
    // `return`/alias, a computed `["push"]`/optional-chain call, closure
    // capture, a non-`push` mutator like `.pop()`, or a wrong-arity push). The
    // pre-existing plain lane silently no-ops the pushes, so `length`/`x[i]`
    // reads diverge from node: this must fail closed (E5506), not run.
    let rejects: BTreeSet<String> = scan
        .push_receiver_mentions
        .iter()
        .filter(|name| {
            !candidates.contains(*name)
                && scan
                    .decls
                    .get(*name)
                    .is_some_and(|info| info.count == 1 && info.growable_shape)
        })
        .cloned()
        .collect();
    let pushes = scan
        .pushes
        .into_iter()
        .filter(|push| candidates.contains(&push.name))
        .collect();
    (candidates, pushes, rejects)
}

/// True when `stmt` (a `for..of` BODY) contains any `<name>.push(...)` call —
/// the resolve-phase for..of gate's same-binding self-push reject (Stage 4
/// Task 4 review fix): a push into the array being iterated would grow the
/// iteration under node but not under the counted loop's once-snapshotted
/// length — a silent node-divergent miscompile. Purely syntactic and
/// name-based: a shadowing redeclaration of `name` in an inner scope still
/// rejects (conservative, per review guidance), and constructs the walk cannot
/// see through (`with`, class bodies, JSX, import/export/enum) return `true`
/// (reject) rather than risk missing an occurrence — belt-and-braces, since
/// the promotion scanner already poisons any function containing them. Nested
/// function/arrow bodies ARE walked (conservative; a closure capturing a
/// growable never promotes anyway). A push on a DIFFERENT binding never
/// matches — the target fixture's `out.push(v)` inside `for (const v of o)`
/// stays admitted.
pub(crate) fn statement_contains_push_on(stmt: &Statement, name: &str) -> bool {
    push_scan_stmt(stmt, name)
}

fn push_scan_block(block: &kali_ast::BlockStatement, name: &str) -> bool {
    block.body.iter().any(|stmt| push_scan_stmt(stmt, name))
}

fn push_scan_stmt(stmt: &Statement, name: &str) -> bool {
    match stmt {
        Statement::ExpressionStatement(s) => push_scan_expr(&s.expression, name),
        Statement::BreakStatement(_)
        | Statement::ContinueStatement(_)
        | Statement::DebuggerStatement(_)
        | Statement::TypeAliasDeclaration(_)
        | Statement::InterfaceDeclaration(_) => false,
        // Cannot see through these — conservative TRUE (reject).
        Statement::WithStatement(_)
        | Statement::ClassDeclaration(_)
        | Statement::ImportDeclaration(_)
        | Statement::ExportAll(_)
        | Statement::ExportNamed(_)
        | Statement::ExportDefault(_)
        | Statement::EnumDeclaration(_) => true,
        Statement::ReturnStatement(s) => s
            .argument
            .as_ref()
            .is_some_and(|arg| push_scan_expr(arg, name)),
        Statement::LabeledStatement(s) => push_scan_stmt(&s.body, name),
        Statement::IfStatement(s) => {
            push_scan_expr(&s.test, name)
                || push_scan_block(&s.consequent, name)
                || s.alternate
                    .as_ref()
                    .is_some_and(|alt| push_scan_block(alt, name))
        }
        Statement::SwitchStatement(s) => {
            push_scan_expr(&s.discriminant, name)
                || s.cases.iter().any(|case| {
                    case.test
                        .as_ref()
                        .is_some_and(|test| push_scan_expr(test, name))
                        || case
                            .consequent
                            .iter()
                            .any(|stmt| push_scan_stmt(stmt, name))
                })
        }
        Statement::ThrowStatement(s) => push_scan_expr(&s.argument, name),
        Statement::TryStatement(s) => {
            push_scan_block(&s.block, name)
                || s.handler
                    .as_ref()
                    .is_some_and(|handler| push_scan_block(&handler.body, name))
                || s.finalizer
                    .as_ref()
                    .is_some_and(|finalizer| push_scan_block(finalizer, name))
        }
        Statement::BlockStatement(s) => push_scan_block(s, name),
        Statement::ForStatement(s) => {
            (match &s.init {
                Some(ForInit::VariableDeclaration(decl)) => push_scan_var_decl(decl, name),
                Some(ForInit::Expression(expr)) => push_scan_expr(expr, name),
                None => false,
            }) || s
                .test
                .as_ref()
                .is_some_and(|test| push_scan_expr(test, name))
                || s.update
                    .as_ref()
                    .is_some_and(|update| push_scan_expr(update, name))
                || push_scan_block(&s.body, name)
        }
        Statement::ForInStatement(s) => {
            (match &s.left {
                ForInLefthand::VariableDeclaration(decl) => push_scan_var_decl(decl, name),
                ForInLefthand::Expression(expr) => push_scan_expr(expr, name),
            }) || push_scan_expr(&s.right, name)
                || push_scan_stmt(&s.body, name)
        }
        Statement::ForOfStatement(s) => {
            (match &s.left {
                ForOfLefthand::VariableDeclaration(decl) => push_scan_var_decl(decl, name),
                ForOfLefthand::Expression(expr) => push_scan_expr(expr, name),
            }) || push_scan_expr(&s.right, name)
                || push_scan_stmt(&s.body, name)
        }
        Statement::WhileStatement(s) => {
            push_scan_expr(&s.test, name) || push_scan_block(&s.body, name)
        }
        Statement::DoWhileStatement(s) => {
            push_scan_block(&s.body, name) || push_scan_expr(&s.test, name)
        }
        // Nested function bodies ARE walked (conservative).
        Statement::FunctionDeclaration(decl) => push_scan_block(&decl.body, name),
        Statement::VariableDeclaration(decl) => push_scan_var_decl(decl, name),
    }
}

fn push_scan_var_decl(decl: &kali_ast::VariableDeclaration, name: &str) -> bool {
    decl.declarations.iter().any(|declarator| {
        declarator
            .init
            .as_ref()
            .is_some_and(|init| push_scan_expr(init, name))
    })
}

fn push_scan_expr(expr: &Expression, name: &str) -> bool {
    match expr {
        Expression::Identifier(_)
        | Expression::Literal(_)
        | Expression::BigIntLiteral(_)
        | Expression::MetaProperty(_)
        | Expression::JsxEmptyExpression
        | Expression::ThisExpression
        | Expression::SuperExpression
        | Expression::PrivateIdentifier(_) => false,
        Expression::BinaryExpression(e) => {
            push_scan_expr(&e.left, name) || push_scan_expr(&e.right, name)
        }
        Expression::UnaryExpression(e) => push_scan_expr(&e.argument, name),
        Expression::CallExpression(call) => {
            // THE match: `<name>.push(...)` (paren-stripped receiver).
            if let Expression::MemberExpression(member) = strip_parens(&call.callee) {
                if member.computed_index.is_none()
                    && member.property == "push"
                    && matches!(strip_parens(&member.object),
                        Expression::Identifier(object) if object == name)
                {
                    return true;
                }
            }
            push_scan_expr(&call.callee, name)
                || call.args.iter().any(|arg| push_scan_expr(arg, name))
        }
        Expression::MemberExpression(member) => {
            push_scan_expr(&member.object, name)
                || member
                    .computed_index
                    .as_ref()
                    .is_some_and(|index| push_scan_expr(index, name))
        }
        Expression::ArrayExpression(array) => array.elements.iter().any(|element| match element {
            Some(ExpressionOrSpread::Expression(expr)) => push_scan_expr(expr, name),
            Some(ExpressionOrSpread::Spread(spread)) => push_scan_expr(&spread.argument, name),
            Some(ExpressionOrSpread::Empty) | None => false,
        }),
        Expression::ObjectExpression(object) => object
            .properties
            .iter()
            .any(|property| push_scan_expr(&property.value, name)),
        // Nested closure bodies ARE walked (conservative).
        Expression::FunctionExpression(func) => func
            .body
            .as_ref()
            .is_some_and(|body| push_scan_block(body, name)),
        Expression::ArrowFunctionExpression(arrow) => push_scan_expr(&arrow.body, name),
        // Cannot see through — conservative TRUE (reject).
        Expression::ClassExpression(_) | Expression::JsxElement(_) | Expression::JsxFragment(_) => {
            true
        }
        Expression::NewExpression(e) => {
            push_scan_expr(&e.callee, name) || e.args.iter().any(|arg| push_scan_expr(arg, name))
        }
        Expression::TemplateLiteral(template) => template
            .expressions
            .iter()
            .any(|expr| push_scan_expr(expr, name)),
        Expression::TaggedTemplateExpression(e) => {
            push_scan_expr(&e.tag, name)
                || e.template
                    .expressions
                    .iter()
                    .any(|expr| push_scan_expr(expr, name))
        }
        Expression::UpdateExpression(e) => push_scan_expr(&e.argument, name),
        Expression::AssignmentExpression(e) => {
            push_scan_expr(&e.left, name) || push_scan_expr(&e.right, name)
        }
        Expression::LogicalExpression(e) => {
            push_scan_expr(&e.left, name) || push_scan_expr(&e.right, name)
        }
        Expression::ConditionalExpression(e) => {
            push_scan_expr(&e.test, name)
                || push_scan_expr(&e.consequent, name)
                || push_scan_expr(&e.alternate, name)
        }
        Expression::SequenceExpression(e) => {
            e.expressions.iter().any(|expr| push_scan_expr(expr, name))
        }
        Expression::ParenthesizedExpression(e) => push_scan_expr(&e.expression, name),
        Expression::YieldExpression(e) => e
            .argument
            .as_ref()
            .is_some_and(|arg| push_scan_expr(arg, name)),
        Expression::AwaitExpression(e) => push_scan_expr(&e.argument, name),
        Expression::OptionalChainExpression(e) => match e.inner.as_ref() {
            kali_ast::OptionalChainInner::NonNull { object, .. } => push_scan_expr(object, name),
        },
        Expression::ChainExpression(e) => push_scan_expr(&e.expression, name),
        Expression::SpreadElement(e) => push_scan_expr(&e.argument, name),
        Expression::RestElement(e) => push_scan_expr(&e.argument, name),
        Expression::ImportExpression(e) => push_scan_expr(&e.source, name),
        Expression::DecoratedExpression(e) => push_scan_expr(&e.expression, name),
        Expression::TypeAssertion(e) => push_scan_expr(&e.expression, name),
        Expression::SatisfiesExpression(e) => push_scan_expr(&e.expression, name),
    }
}

/// True for the scalar-shaped expressions admitted as push arguments, array
/// seeds, and computed indices in Task 2's i64 lane: numeric literals, bare
/// identifiers (`allow_identifiers` — repr-checked later at `emit_table`),
/// and unary/binary arithmetic over such. Everything else (calls, members,
/// objects, arrays, strings, booleans, templates, …) is out — it could
/// deliver a value this lane would store raw and read back wrong.
fn scalar_value_shape_ok(expr: &Expression, allow_identifiers: bool) -> bool {
    match expr {
        Expression::Literal(LiteralValue::Number(_)) => true,
        Expression::Identifier(_) => allow_identifiers,
        Expression::ParenthesizedExpression(inner) => {
            scalar_value_shape_ok(&inner.expression, allow_identifiers)
        }
        Expression::UnaryExpression(unary) => {
            matches!(unary.operator.as_str(), "-" | "+" | "~")
                && scalar_value_shape_ok(&unary.argument, allow_identifiers)
        }
        Expression::BinaryExpression(binary) => {
            matches!(
                binary.operator.as_str(),
                "+" | "-" | "*" | "/" | "%" | "**" | "&" | "|" | "^" | "<<" | ">>" | ">>>"
            ) && scalar_value_shape_ok(&binary.left, allow_identifiers)
                && scalar_value_shape_ok(&binary.right, allow_identifiers)
        }
        _ => false,
    }
}

/// Task 3: shape admitted specifically as a `.push` ARGUMENT — everything
/// `scalar_value_shape_ok` admits, PLUS a bare string literal. Deliberately
/// NOT folded into `scalar_value_shape_ok` itself: that function is also
/// used for array-literal SEEDS and computed INDICES, neither of which this
/// task relaxes (a string seed/element-repr-union for seeded literals and a
/// string computed index are both out of scope here — indices must stay
/// numeric, and seeded-string-literal declarators are not part of this
/// task's target fixture). A string identifier push is already admitted via
/// `scalar_value_shape_ok`'s `allow_identifiers` arm (repr-checked later).
fn push_argument_shape_ok(expr: &Expression, allow_identifiers: bool) -> bool {
    matches!(
        strip_parens(expr),
        Expression::Literal(LiteralValue::String(_))
    ) || scalar_value_shape_ok(expr, allow_identifiers)
}

fn strip_parens(expr: &Expression) -> &Expression {
    let mut current = expr;
    while let Expression::ParenthesizedExpression(inner) = current {
        current = &inner.expression;
    }
    current
}

/// Strips parentheses AND optional-chain (`?.`) `NonNull` wrappers to reach the
/// underlying expression — so `(o)`, `o?.` and `(o)?.` all resolve to `o`.
fn strip_parens_and_optional(expr: &Expression) -> &Expression {
    let mut current = expr;
    loop {
        match current {
            Expression::ParenthesizedExpression(inner) => current = &inner.expression,
            Expression::OptionalChainExpression(chain) => match chain.inner.as_ref() {
                kali_ast::OptionalChainInner::NonNull { object, .. } => current = object,
            },
            _ => return current,
        }
    }
}

/// The base identifier receiving a `.push` call in ANY recognized form —
/// `o.push(..)`, `o["push"](..)` (computed member, `property` normalized to
/// `"push"`), or `o?.push(..)` (optional-chain object) — regardless of the
/// argument shape or arity. Task 6 records these as push-receiver MENTIONS: a
/// growable-shape binding mentioned here that does not promote is a silent
/// miscompile to fail closed. A non-identifier base (`a.b.push(..)`,
/// `f().push(..)`) is not a bare-binding push and returns `None`.
fn push_receiver_base(call: &kali_ast::CallExpression) -> Option<&str> {
    if let Expression::MemberExpression(member) = strip_parens(&call.callee) {
        if member.property == "push" {
            if let Expression::Identifier(name) = strip_parens_and_optional(&member.object) {
                return Some(name);
            }
        }
    }
    None
}

impl Scan {
    fn declare(&mut self, name: &str, growable_shape: bool) {
        let info = self.decls.entry(name.to_string()).or_default();
        info.count += 1;
        info.growable_shape = growable_shape && info.count == 1;
    }

    fn mark_unsafe(&mut self, name: &str) {
        self.unsafe_names.insert(name.to_string());
    }

    fn block(&mut self, block: &kali_ast::BlockStatement, nested: bool) {
        for stmt in &block.body {
            self.stmt(stmt, nested);
        }
    }

    fn stmt(&mut self, stmt: &Statement, nested: bool) {
        match stmt {
            Statement::ExpressionStatement(s) => self.expr(&s.expression, nested),
            Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::DebuggerStatement(_) => {}
            // `with` rebinds free identifiers dynamically — the name-based
            // occurrence scan is unsound under it. Poison.
            Statement::WithStatement(_) => self.poisoned = true,
            Statement::ReturnStatement(s) => {
                if let Some(arg) = &s.argument {
                    self.expr(arg, nested);
                }
            }
            Statement::LabeledStatement(s) => self.stmt(&s.body, nested),
            Statement::IfStatement(s) => {
                self.expr(&s.test, nested);
                self.block(&s.consequent, nested);
                if let Some(alt) = &s.alternate {
                    self.block(alt, nested);
                }
            }
            Statement::SwitchStatement(s) => {
                self.expr(&s.discriminant, nested);
                for case in &s.cases {
                    if let Some(test) = &case.test {
                        self.expr(test, nested);
                    }
                    for stmt in &case.consequent {
                        self.stmt(stmt, nested);
                    }
                }
            }
            Statement::ThrowStatement(s) => self.expr(&s.argument, nested),
            Statement::TryStatement(s) => {
                self.block(&s.block, nested);
                if let Some(handler) = &s.handler {
                    // The catch param is a declaration (it shadows within the
                    // handler; count it so a same-named candidate disqualifies).
                    self.declare(&handler.param, false);
                    self.block(&handler.body, nested);
                }
                if let Some(finalizer) = &s.finalizer {
                    self.block(finalizer, nested);
                }
            }
            Statement::BlockStatement(s) => self.block(s, nested),
            Statement::ForStatement(s) => {
                match &s.init {
                    Some(ForInit::VariableDeclaration(decl)) => {
                        self.variable_declaration(decl, nested)
                    }
                    Some(ForInit::Expression(expr)) => self.expr(expr, nested),
                    None => {}
                }
                if let Some(test) = &s.test {
                    self.expr(test, nested);
                }
                if let Some(update) = &s.update {
                    self.expr(update, nested);
                }
                self.block(&s.body, nested);
            }
            Statement::ForInStatement(s) => {
                match &s.left {
                    ForInLefthand::VariableDeclaration(decl) => {
                        self.variable_declaration(decl, nested)
                    }
                    // Bare-identifier key: a WRITE to that name each
                    // iteration — the plain expr scan marks it unsafe.
                    ForInLefthand::Expression(expr) => self.expr(expr, nested),
                }
                // `for..in` over a growable array is NOT in the allowlist
                // (only for..of is): the plain expr scan marks an identifier
                // RHS unsafe.
                self.expr(&s.right, nested);
                self.stmt(&s.body, nested);
            }
            Statement::ForOfStatement(s) => {
                match &s.left {
                    ForOfLefthand::VariableDeclaration(decl) => {
                        self.variable_declaration(decl, nested)
                    }
                    ForOfLefthand::Expression(expr) => self.expr(expr, nested),
                }
                // for..of RHS is a SAFE position for a bare identifier (the
                // full stage surface; fail-closed E5506 until Task 4 lowers
                // it — see the resolve-phase for..of gate).
                if nested || !matches!(strip_parens(&s.right), Expression::Identifier(_)) {
                    self.expr(&s.right, nested);
                }
                self.stmt(&s.body, nested);
            }
            Statement::WhileStatement(s) => {
                self.expr(&s.test, nested);
                self.block(&s.body, nested);
            }
            Statement::DoWhileStatement(s) => {
                self.block(&s.body, nested);
                self.expr(&s.test, nested);
            }
            Statement::FunctionDeclaration(decl) => {
                // The nested function's name shadows/collides at function
                // scope; its params + body occurrences are all scanned in
                // nested mode (no safe positions inside a closure — a
                // captured growable handle is an escape).
                self.declare(&decl.name, false);
                for param in &decl.params {
                    self.declare(param, false);
                }
                self.block(&decl.body, true);
            }
            // Class bodies (methods, field initializers) are not walked by
            // this analysis: poison rather than risk missing an occurrence.
            Statement::ClassDeclaration(_) => self.poisoned = true,
            Statement::VariableDeclaration(decl) => self.variable_declaration(decl, nested),
            Statement::ImportDeclaration(_)
            | Statement::ExportAll(_)
            | Statement::ExportNamed(_)
            | Statement::ExportDefault(_)
            | Statement::EnumDeclaration(_) => self.poisoned = true,
            // Type-only declarations carry no runtime identifier references.
            Statement::TypeAliasDeclaration(_) | Statement::InterfaceDeclaration(_) => {}
        }
    }

    fn variable_declaration(&mut self, decl: &kali_ast::VariableDeclaration, nested: bool) {
        for declarator in &decl.declarations {
            let growable_shape = !nested
                && matches!(decl.kind.as_str(), "const" | "let")
                && declarator.init.as_ref().is_some_and(|init| {
                    matches!(init, Expression::ArrayExpression(array)
                    if array.elements.iter().all(|element| matches!(
                        element,
                        Some(ExpressionOrSpread::Expression(expr))
                            // Seeds admit NO identifiers: an identifier
                            // seed could deliver an object/array handle
                            // the emit-time repr gate cannot see on the
                            // element axis.
                            if scalar_value_shape_ok(expr, false)
                    )))
                });
            self.declare(&declarator.id, growable_shape);
            if let Some(init) = &declarator.init {
                self.expr(init, nested);
            }
        }
    }

    /// Marks the write-target base of an assignment/update: a bare
    /// identifier target is a reassignment; a member target (`x[i] = v`,
    /// `x.f = v`, `x[i]++`) mutates its base outside the allowlist (index
    /// WRITES are not lowered by this lane). Subexpressions (computed
    /// indices, nested bases) are scanned normally.
    fn write_target(&mut self, target: &Expression, nested: bool) {
        match strip_parens(target) {
            Expression::Identifier(name) => self.mark_unsafe(name),
            Expression::MemberExpression(member) => {
                match strip_parens(&member.object) {
                    Expression::Identifier(name) => self.mark_unsafe(name),
                    other => self.expr(other, nested),
                }
                if let Some(index) = &member.computed_index {
                    self.expr(index, nested);
                }
            }
            // Any other target shape: scan it plainly — every identifier
            // inside marks unsafe.
            other => self.expr(other, nested),
        }
    }

    fn expr(&mut self, expr: &Expression, nested: bool) {
        match expr {
            // THE identifier choke point: any bare occurrence that was not
            // consumed by a safe-position arm above lands here → unsafe.
            Expression::Identifier(name) => self.mark_unsafe(name),
            Expression::Literal(_) | Expression::BigIntLiteral(_) => {}
            Expression::BinaryExpression(e) => {
                self.expr(&e.left, nested);
                self.expr(&e.right, nested);
            }
            Expression::UnaryExpression(e) => {
                if e.operator == "delete" {
                    // `delete x[i]` / `delete x.f` mutates the base.
                    self.write_target(&e.argument, nested);
                } else {
                    self.expr(&e.argument, nested);
                }
            }
            Expression::CallExpression(call) => {
                // Task 6: record EVERY push-receiver mention (any form/nesting)
                // so a growable-shape binding that is a push receiver but does
                // not promote fails closed rather than silently no-opping.
                if let Some(base) = push_receiver_base(call) {
                    self.push_receiver_mentions.insert(base.to_string());
                }
                if !nested {
                    if let Expression::MemberExpression(member) = strip_parens(&call.callee) {
                        if member.computed_index.is_none() {
                            if let Expression::Identifier(name) = strip_parens(&member.object) {
                                match member.property.as_str() {
                                    // `x.push(v)` — safe receiver iff exactly
                                    // one scalar- or string-literal-shaped
                                    // argument (Task 3: string elements).
                                    "push"
                                        if call.args.len() == 1
                                            && push_argument_shape_ok(&call.args[0], true) =>
                                    {
                                        let arg = strip_parens(&call.args[0]);
                                        self.pushes.push(GrowablePushSite {
                                            name: name.clone(),
                                            arg_identifier: match arg {
                                                Expression::Identifier(id) => Some(id.clone()),
                                                _ => None,
                                            },
                                        });
                                        // Occurrences INSIDE the argument are
                                        // classified normally (`x.push(x)`
                                        // still marks `x` unsafe).
                                        self.expr(&call.args[0], nested);
                                        return;
                                    }
                                    // `x.join(sep)` — safe receiver (Task 5
                                    // lowers; E5506 until then).
                                    "join" if call.args.len() <= 1 => {
                                        for arg in &call.args {
                                            self.expr(arg, nested);
                                        }
                                        return;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                self.expr(&call.callee, nested);
                for arg in &call.args {
                    self.expr(arg, nested);
                }
            }
            Expression::MemberExpression(member) => {
                if !nested {
                    if let Expression::Identifier(name) = strip_parens(&member.object) {
                        if let Some(index) = &member.computed_index {
                            // `x[i]` index READ — safe base; the index must
                            // itself be scalar-shaped (a string index like
                            // `x["length"]` has no growable lowering).
                            if scalar_value_shape_ok(index, true) {
                                self.expr(index, nested);
                                return;
                            }
                            self.mark_unsafe(name);
                            self.expr(index, nested);
                            return;
                        }
                        // `x.length` read, or `x[0]` parsed with the index
                        // stringified into `property`.
                        if member.property == "length" || member.property.parse::<u64>().is_ok() {
                            return;
                        }
                        // Any other dot member (`x.pop`, `x.map`, …) — not in
                        // the allowlist.
                        self.mark_unsafe(name);
                        return;
                    }
                }
                self.expr(&member.object, nested);
                if let Some(index) = &member.computed_index {
                    self.expr(index, nested);
                }
            }
            Expression::ArrayExpression(array) => {
                for element in &array.elements {
                    match element {
                        Some(ExpressionOrSpread::Expression(expr)) => self.expr(expr, nested),
                        Some(ExpressionOrSpread::Spread(spread)) => {
                            self.expr(&spread.argument, nested)
                        }
                        Some(ExpressionOrSpread::Empty) | None => {}
                    }
                }
            }
            Expression::ObjectExpression(object) => {
                for property in &object.properties {
                    self.expr(&property.value, nested);
                }
            }
            Expression::FunctionExpression(func) => {
                if let Some(id) = &func.id {
                    self.declare(id, false);
                }
                for param in &func.params {
                    self.declare(&param.name, false);
                }
                if let Some(body) = &func.body {
                    self.block(body, true);
                }
            }
            Expression::ArrowFunctionExpression(arrow) => {
                for param in &arrow.params {
                    self.declare(&param.name, false);
                }
                self.expr(&arrow.body, true);
            }
            Expression::ClassExpression(_) => self.poisoned = true,
            Expression::NewExpression(e) => {
                self.expr(&e.callee, nested);
                for arg in &e.args {
                    self.expr(arg, nested);
                }
            }
            Expression::MetaProperty(_) => {}
            Expression::TemplateLiteral(template) => {
                for expr in &template.expressions {
                    self.expr(expr, nested);
                }
            }
            Expression::TaggedTemplateExpression(e) => {
                self.expr(&e.tag, nested);
                for expr in &e.template.expressions {
                    self.expr(expr, nested);
                }
            }
            Expression::UpdateExpression(e) => self.write_target(&e.argument, nested),
            Expression::AssignmentExpression(e) => {
                self.write_target(&e.left, nested);
                self.expr(&e.right, nested);
            }
            Expression::LogicalExpression(e) => {
                self.expr(&e.left, nested);
                self.expr(&e.right, nested);
            }
            Expression::ConditionalExpression(e) => {
                self.expr(&e.test, nested);
                self.expr(&e.consequent, nested);
                self.expr(&e.alternate, nested);
            }
            Expression::SequenceExpression(e) => {
                for expr in &e.expressions {
                    self.expr(expr, nested);
                }
            }
            Expression::ParenthesizedExpression(e) => self.expr(&e.expression, nested),
            Expression::YieldExpression(e) => {
                if let Some(arg) = &e.argument {
                    self.expr(arg, nested);
                }
            }
            Expression::AwaitExpression(e) => self.expr(&e.argument, nested),
            Expression::OptionalChainExpression(e) => match e.inner.as_ref() {
                kali_ast::OptionalChainInner::NonNull { object, .. } => self.expr(object, nested),
            },
            Expression::ChainExpression(e) => self.expr(&e.expression, nested),
            Expression::SpreadElement(e) => self.expr(&e.argument, nested),
            Expression::RestElement(e) => self.expr(&e.argument, nested),
            Expression::ImportExpression(e) => self.expr(&e.source, nested),
            Expression::DecoratedExpression(e) => self.expr(&e.expression, nested),
            // JSX trees embed expressions this analysis does not walk:
            // poison rather than risk missing an occurrence.
            Expression::JsxElement(_) | Expression::JsxFragment(_) => self.poisoned = true,
            Expression::JsxEmptyExpression => {}
            Expression::TypeAssertion(e) => self.expr(&e.expression, nested),
            Expression::SatisfiesExpression(e) => self.expr(&e.expression, nested),
            Expression::ThisExpression
            | Expression::SuperExpression
            | Expression::PrivateIdentifier(_) => {}
        }
    }
}

#[cfg(test)]
#[path = "growable_tests.rs"]
mod growable_tests;
