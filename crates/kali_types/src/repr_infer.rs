//! Interprocedural int-vs-float representation inference.
//!
//! Float is a **forward-flow** property of scalar program points: a node is
//! `f64` iff a float *seed* reaches it along directed "if source is float then
//! target is float" edges (`operand -> result`, `rhs -> lhs`, `expr -> return`,
//! `arg -> param`, `return -> callsite`, `value -> array_element`,
//! `array_element -> read_result`). Float seeds are division results,
//! non-integer literals, `Math.sqrt`/`Math.cbrt` results, `.toFixed` receivers,
//! and `/=` targets. The property is solved by BFS reachability from the seed
//! set — it is deliberately **directional** so that an integer operand binding
//! that merely feeds a float result (e.g. `i` in `(i + j) / 2`) is NOT floated;
//! codegen converts each i64 operand inline where a float is needed.
//!
//! Array *element storage*, by contrast, is a single shared property of an
//! array's memory. Element identity across aliases (interprocedural array
//! arg↔param passing) is modelled with a **bidirectional** [`UnionFind`]: the
//! two arrays share ONE element node. Stores and reads are still directed edges
//! into / out of that shared node (an int stored into a float array stays int
//! and is converted at the store).
//!
//! The solved decisions populate a [`ReprTable`]; only float decisions are
//! recorded, so an all-integer program yields an empty table. Two axes are
//! tracked per program point, both defaulting to `I64`: a scalar repr (per
//! binding/param/return) and an array *element* repr. Array handles themselves
//! are always `i64`; only their elements can become `f64`.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use kali_ast::{AssignmentOperator, BlockStatement, Expression, ForInit, LiteralValue, Statement};
use kali_common::{Repr, ReprTable, UnionFind};

/// Synthetic function name for top-level statements, matching codegen's entry.
const TOP_LEVEL: &str = "_start";

/// A deferred interprocedural call constraint, resolved after every function
/// body has been walked (so all param/return/element nodes already exist).
struct CallEdge {
    /// Bare-identifier callee name.
    callee: String,
    /// Result node of each positional argument (scalar flow).
    arg_nodes: Vec<usize>,
    /// For each positional argument, `Some((caller_func, name))` when the
    /// argument is a bare identifier (candidate array binding), else `None`.
    arg_array_names: Vec<Option<(String, String)>>,
    /// Result node of the call expression itself (target of the callee's
    /// return-flow edge).
    result_node: usize,
}

#[derive(Default)]
struct ReprInfer {
    /// Bidirectional union-find, used ONLY for array-element storage aliasing
    /// (interprocedural array arg↔param). Node ids are allocated here so the
    /// directed reachability graph and the array-aliasing forest share one id
    /// space; scalar/param/return/result nodes are never unioned, so their
    /// representative is always themselves.
    uf: UnionFind,
    /// Number of allocated nodes (== next node id). Tracked because
    /// [`UnionFind`] exposes no length accessor.
    node_count: usize,
    /// Directed float-flow edges `from -> to` (float(from) ⇒ float(to)).
    /// Endpoints are canonicalised through `uf` at solve time so edges touching
    /// aliased array-element nodes follow the shared representative.
    edges: Vec<(usize, usize)>,
    /// Directly-float nodes (division results, float literals, `Math.sqrt`, …).
    seeds: Vec<usize>,
    /// One node per scalar binding/param/local: `(func, name) -> node`.
    scalar_node: BTreeMap<(String, String), usize>,
    /// One node per array binding/param element repr: `(func, name) -> node`.
    array_elem_node: BTreeMap<(String, String), usize>,
    /// One node per function's return value: `func -> node`.
    return_node: BTreeMap<String, usize>,
    /// Ordered parameter names of every user `FunctionDeclaration`.
    functions: BTreeMap<String, Vec<String>>,
    /// Names locally bound within a given scope: a function's own parameters
    /// plus every `let`/`const`/`var` declarator reachable from its body
    /// without descending into a nested function (module scope uses the
    /// `TOP_LEVEL` key). Lets identifier resolution tell a local read from a
    /// module-scope read regardless of source order, mirroring codegen's
    /// `self.locals`/`self.bindings` shadow precedence (see
    /// `kali_codegen::emit::control_flow`'s identifier fallback and
    /// `kali_codegen::lower`'s `module_binding_names`).
    local_names: BTreeMap<String, BTreeSet<String>>,
    /// Deferred interprocedural call constraints.
    calls: Vec<CallEdge>,
    /// Ordered field names of each slot directly initialized by an object literal.
    obj_literal_fields: BTreeMap<ObjSlot, Vec<String>>,
    /// Bidirectional object-aliasing flows (assignment, array element,
    /// arg↔param, return↔call-site). Harmless for scalar slots: flows only
    /// take effect for slots proven to hold object literals.
    obj_flows: Vec<(ObjSlot, ObjSlot)>,
    /// Deferred member accesses (wired in `resolve_objects`).
    obj_accesses: Vec<ObjAccess>,
    /// Per-(slot, field) storage node, unioned across aliased slots.
    obj_field_node: BTreeMap<(ObjSlot, String), usize>,
    /// Slots that must lower as runtime heap objects (any write, any flow).
    obj_materialized: BTreeSet<ObjSlot>,
    /// Object slots with their propagated field lists (set by `resolve_objects`).
    obj_fields_of: BTreeMap<ObjSlot, Vec<String>>,
    /// Deferred *structural* gate messages, keyed by the slot whose literal is
    /// unsupported on the runtime object lane (non-identifier property name,
    /// getter/setter, nested object). Emitted ONLY if that slot later
    /// materializes; a read-only fold-lane literal (e.g. a string-keyed object
    /// consumed only by `Object.keys`) never materializes and so keeps today's
    /// fold-lane behavior byte-identically (fold-first invariant).
    obj_pending_conflicts: BTreeMap<ObjSlot, String>,
    /// Gate messages (unsupported or contradictory object usage).
    obj_conflicts: Vec<String>,
}

/// Identity of an object-holding slot for shape/aliasing purposes.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum ObjSlot {
    /// `(func, binding)` — a named binding or parameter.
    Binding(String, String),
    /// `(func, array_binding)` — every element of the named array (elements
    /// of one array share one shape and one per-field storage cluster).
    ArrayElem(String, String),
    /// `func` — the function's return value.
    Return(String),
}

/// A recorded `<base>.field` access: read (`other` = the result node) or
/// write (`other` = the stored-value node). Wired to shared field storage
/// after object propagation (`resolve_objects`).
struct ObjAccess {
    base: ObjSlot,
    field: String,
    other: usize,
    is_write: bool,
}

/// Whole-program pass: allocate nodes, add seeds + intra/inter-procedural
/// directed float-flow edges (plus array-element aliasing unions), solve by
/// reachability, and emit the [`ReprTable`].
pub fn infer_reprs(statements: &[Statement]) -> ReprTable {
    let mut infer = ReprInfer::default();

    // Phase A: collect every function signature (recursively) and eagerly
    // create a scalar node per parameter so interprocedural edges have a
    // stable target even for params never mentioned in the body.
    infer.collect_functions(statements);

    // Phase A2: collect every locally-declared name per scope (module scope
    // plus each function's own params/declarators), so identifier resolution
    // in Phase B can distinguish a local read from a module-scope read
    // regardless of source order.
    infer.collect_local_names(TOP_LEVEL, statements);

    // Phase B: walk bodies. Top-level non-function statements run under the
    // synthetic `_start`; each `FunctionDeclaration` runs under its own name.
    for stmt in statements {
        infer.visit_stmt(TOP_LEVEL, stmt);
    }

    // Phase C: resolve deferred call edges (transitive array-param fixpoint +
    // directed scalar/return edges + bidirectional array-element unions).
    infer.resolve_calls();

    // Phase C2: object-shape propagation (field lists across flows, shared
    // field storage unions, deferred member-access wiring, materialization).
    infer.resolve_objects();

    // Phase D: solve → table.
    infer.emit_table()
}

impl ReprInfer {
    // ---- node / edge / seed constructors -------------------------------

    /// Allocate a fresh node id (kept in the `uf` id space).
    fn new_node(&mut self) -> usize {
        let n = self.uf.fresh();
        self.node_count = n + 1;
        n
    }

    /// Record a directed float-flow edge `from -> to`.
    fn add_edge(&mut self, from: usize, to: usize) {
        self.edges.push((from, to));
    }

    /// Mark `node` as a direct float seed.
    fn add_seed(&mut self, node: usize) {
        self.seeds.push(node);
    }

    // ---- node accessors (get-or-create) --------------------------------

    fn scalar_node_for(&mut self, func: &str, name: &str) -> usize {
        let key = (func.to_string(), name.to_string());
        if let Some(&n) = self.scalar_node.get(&key) {
            return n;
        }
        let n = self.new_node();
        self.scalar_node.insert(key, n);
        n
    }

    fn array_elem_node_for(&mut self, func: &str, name: &str) -> usize {
        let key = (func.to_string(), name.to_string());
        if let Some(&n) = self.array_elem_node.get(&key) {
            return n;
        }
        let n = self.new_node();
        self.array_elem_node.insert(key, n);
        n
    }

    fn return_node_for(&mut self, func: &str) -> usize {
        if let Some(&n) = self.return_node.get(func) {
            return n;
        }
        let n = self.new_node();
        self.return_node.insert(func.to_string(), n);
        n
    }

    fn obj_field_node_for(&mut self, slot: &ObjSlot, field: &str) -> usize {
        let key = (slot.clone(), field.to_string());
        if let Some(&n) = self.obj_field_node.get(&key) {
            return n;
        }
        let n = self.new_node();
        self.obj_field_node.insert(key, n);
        n
    }

    /// Record an object literal initializing `slot`: remember its ordered
    /// field names, visit each value, and wire `value -> field storage`
    /// float edges. Unsupported property forms (non-identifier key,
    /// getter/setter, nested object) record a *deferred* structural conflict
    /// keyed by `slot` and return WITHOUT a field list — the slot then never
    /// materializes on its own, so a read-only fold-lane literal keeps today's
    /// behavior. The deferred message is promoted to a real gate conflict only
    /// if the slot is later forced onto the object lane (`resolve_objects`).
    fn record_object_literal(
        &mut self,
        func: &str,
        slot: ObjSlot,
        obj: &kali_ast::ObjectExpression,
    ) {
        let mut names = Vec::new();
        for prop in &obj.properties {
            let kali_ast::PropertyName::Identifier(key) = &prop.key else {
                self.obj_pending_conflicts.insert(
                    slot.clone(),
                    format!(
                        "object literal for {slot:?} uses a non-identifier property name, which is unavailable in the current phase"
                    ),
                );
                return;
            };
            if !matches!(prop.kind, kali_ast::ObjectPropertyKind::Init) {
                self.obj_pending_conflicts.insert(
                    slot.clone(),
                    format!(
                        "object literal for {slot:?} uses a getter/setter, which is unavailable in the current phase"
                    ),
                );
                return;
            }
            if matches!(prop.value, Expression::ObjectExpression(_)) {
                self.obj_pending_conflicts.insert(
                    slot.clone(),
                    format!("nested object field '{key}' is unavailable in the current phase"),
                );
                return;
            }
            let value_node = self.visit_expr(func, &prop.value);
            let field_node = self.obj_field_node_for(&slot, key);
            self.add_edge(value_node, field_node);
            names.push(key.clone());
        }
        match self.obj_literal_fields.entry(slot.clone()) {
            std::collections::btree_map::Entry::Occupied(existing) => {
                if *existing.get() != names {
                    self.obj_conflicts
                        .push(format!("conflicting object shapes assigned to {slot:?}"));
                }
            }
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(names);
            }
        }
    }

    /// Record an aliasing flow `dst ~ <expr>` when the expression can carry an
    /// object reference: identifier, `arr[i]`, or bare-identifier call.
    fn record_object_flow_from_expr(&mut self, func: &str, dst: ObjSlot, expr: &Expression) {
        match expr {
            Expression::Identifier(name) => self
                .obj_flows
                .push((dst, ObjSlot::Binding(func.to_string(), name.clone()))),
            Expression::MemberExpression(member) if member.computed_index.is_some() => {
                if let Expression::Identifier(array) = &member.object {
                    self.obj_flows
                        .push((dst, ObjSlot::ArrayElem(func.to_string(), array.clone())));
                }
            }
            Expression::CallExpression(call) => {
                if let Expression::Identifier(callee) = &call.callee {
                    self.obj_flows.push((dst, ObjSlot::Return(callee.clone())));
                }
            }
            Expression::ParenthesizedExpression(inner) => {
                self.record_object_flow_from_expr(func, dst, &inner.expression)
            }
            _ => {}
        }
    }

    /// Slot for a member-access base: a bare identifier (binding) or a
    /// subscript of a bare identifier (array element). Registers the array's
    /// element node in the subscript case (the base is an array) and visits
    /// the index for its own edges.
    fn member_base_slot(&mut self, func: &str, base: &Expression) -> Option<ObjSlot> {
        match base {
            Expression::Identifier(name) => Some(ObjSlot::Binding(func.to_string(), name.clone())),
            Expression::MemberExpression(member) if member.computed_index.is_some() => {
                if let Some(index) = &member.computed_index {
                    self.visit_expr(func, index);
                }
                match &member.object {
                    Expression::Identifier(array) => {
                        self.array_elem_node_for(func, array);
                        Some(ObjSlot::ArrayElem(func.to_string(), array.clone()))
                    }
                    _ => None,
                }
            }
            Expression::ParenthesizedExpression(inner) => {
                self.member_base_slot(func, &inner.expression)
            }
            _ => None,
        }
    }

    // ---- Phase A: signature collection ---------------------------------

    fn collect_functions(&mut self, statements: &[Statement]) {
        for stmt in statements {
            self.collect_functions_in_stmt(stmt);
        }
    }

    fn collect_functions_in_stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::FunctionDeclaration(func) => {
                self.functions
                    .insert(func.name.clone(), func.params.clone());
                for param in &func.params {
                    // Eagerly allocate the param's scalar node.
                    self.scalar_node_for(&func.name, param);
                }
                self.collect_functions(&func.body.body);
            }
            Statement::BlockStatement(block) => self.collect_functions(&block.body),
            Statement::IfStatement(node) => {
                self.collect_functions(&node.consequent.body);
                if let Some(alt) = &node.alternate {
                    self.collect_functions(&alt.body);
                }
            }
            Statement::ForStatement(node) => self.collect_functions(&node.body.body),
            Statement::ForInStatement(node) => self.collect_functions_in_stmt(&node.body),
            Statement::ForOfStatement(node) => self.collect_functions_in_stmt(&node.body),
            Statement::WhileStatement(node) => self.collect_functions(&node.body.body),
            Statement::DoWhileStatement(node) => self.collect_functions(&node.body.body),
            Statement::LabeledStatement(node) => self.collect_functions_in_stmt(&node.body),
            Statement::TryStatement(node) => {
                self.collect_functions(&node.block.body);
                if let Some(handler) = &node.handler {
                    self.collect_functions(&handler.body.body);
                }
                if let Some(finalizer) = &node.finalizer {
                    self.collect_functions(&finalizer.body);
                }
            }
            _ => {}
        }
    }

    // ---- Phase A2: local-name collection --------------------------------

    /// Populate `local_names` for `func`'s own scope (module scope when
    /// `func == TOP_LEVEL`): every `let`/`const`/`var` declarator reachable
    /// from `statements` without descending into a nested function, plus
    /// (via the `FunctionDeclaration` arm below) each nested function's own
    /// parameters.
    fn collect_local_names(&mut self, func: &str, statements: &[Statement]) {
        for stmt in statements {
            self.collect_local_names_in_stmt(func, stmt);
        }
    }

    fn collect_local_names_in_stmt(&mut self, func: &str, stmt: &Statement) {
        match stmt {
            Statement::FunctionDeclaration(decl) => {
                let entry = self.local_names.entry(decl.name.clone()).or_default();
                for param in &decl.params {
                    entry.insert(param.clone());
                }
                self.collect_local_names(&decl.name, &decl.body.body);
            }
            Statement::VariableDeclaration(decl) => {
                let entry = self.local_names.entry(func.to_string()).or_default();
                for d in &decl.declarations {
                    entry.insert(d.id.clone());
                }
            }
            Statement::BlockStatement(block) => self.collect_local_names(func, &block.body),
            Statement::IfStatement(node) => {
                self.collect_local_names(func, &node.consequent.body);
                if let Some(alt) = &node.alternate {
                    self.collect_local_names(func, &alt.body);
                }
            }
            Statement::ForStatement(node) => {
                if let Some(ForInit::VariableDeclaration(decl)) = &node.init {
                    let entry = self.local_names.entry(func.to_string()).or_default();
                    for d in &decl.declarations {
                        entry.insert(d.id.clone());
                    }
                }
                self.collect_local_names(func, &node.body.body);
            }
            Statement::ForInStatement(node) => self.collect_local_names_in_stmt(func, &node.body),
            Statement::ForOfStatement(node) => self.collect_local_names_in_stmt(func, &node.body),
            Statement::WhileStatement(node) => self.collect_local_names(func, &node.body.body),
            Statement::DoWhileStatement(node) => self.collect_local_names(func, &node.body.body),
            Statement::LabeledStatement(node) => self.collect_local_names_in_stmt(func, &node.body),
            Statement::TryStatement(node) => {
                self.collect_local_names(func, &node.block.body);
                if let Some(handler) = &node.handler {
                    self.collect_local_names(func, &handler.body.body);
                }
                if let Some(finalizer) = &node.finalizer {
                    self.collect_local_names(func, &finalizer.body);
                }
            }
            _ => {}
        }
    }

    /// True when `name` is locally bound in `func`'s own scope (a parameter
    /// or a `let`/`const`/`var` declarator anywhere in its body, ignoring
    /// nested functions) — mirrors codegen's local/binding precedence.
    fn is_locally_declared(&self, func: &str, name: &str) -> bool {
        self.local_names
            .get(func)
            .is_some_and(|names| names.contains(name))
    }

    // ---- Phase B: statement walk ---------------------------------------

    fn visit_block(&mut self, func: &str, block: &BlockStatement) {
        for stmt in &block.body {
            self.visit_stmt(func, stmt);
        }
    }

    fn visit_stmt(&mut self, func: &str, stmt: &Statement) {
        match stmt {
            Statement::FunctionDeclaration(decl) => {
                // Walk the body under the function's own name.
                self.visit_block(&decl.name, &decl.body);
            }
            Statement::VariableDeclaration(decl) => {
                for d in &decl.declarations {
                    if let Some(init) = &d.init {
                        self.visit_declarator_init(func, &d.id, init);
                    }
                }
            }
            Statement::ExpressionStatement(stmt) => {
                // A bare, top-level member-read statement (`p.field;`) never
                // observes its result — the value is unconditionally
                // discarded. This is also, incidentally, the exact AST shape
                // the parser currently produces for `delete p.field;` (the
                // parser silently drops the `delete` keyword rather than
                // building a `UnaryExpression` for it) — routing it through
                // the same "unobserved" helper fixes that case too, without
                // requiring dedicated `delete` parser support. See
                // `visit_unobserved_member_target`.
                if matches!(stmt.expression.as_ref(), Expression::MemberExpression(_)) {
                    self.visit_unobserved_member_target(func, &stmt.expression);
                    return;
                }
                self.visit_expr(func, &stmt.expression);
            }
            Statement::ReturnStatement(stmt) => {
                if let Some(arg) = &stmt.argument {
                    if let Expression::ObjectExpression(obj) = arg {
                        self.record_object_literal(func, ObjSlot::Return(func.to_string()), obj);
                    } else {
                        self.record_object_flow_from_expr(
                            func,
                            ObjSlot::Return(func.to_string()),
                            arg,
                        );
                        let rn = self.visit_expr(func, arg);
                        let ret = self.return_node_for(func);
                        self.add_edge(rn, ret);
                    }
                }
            }
            Statement::IfStatement(stmt) => {
                self.visit_expr(func, &stmt.test);
                self.visit_block(func, &stmt.consequent);
                if let Some(alt) = &stmt.alternate {
                    self.visit_block(func, alt);
                }
            }
            Statement::ForStatement(stmt) => {
                if let Some(init) = &stmt.init {
                    match init {
                        ForInit::VariableDeclaration(decl) => {
                            for d in &decl.declarations {
                                if let Some(i) = &d.init {
                                    self.visit_declarator_init(func, &d.id, i);
                                }
                            }
                        }
                        ForInit::Expression(expr) => {
                            self.visit_expr(func, expr);
                        }
                    }
                }
                if let Some(test) = &stmt.test {
                    self.visit_expr(func, test);
                }
                if let Some(update) = &stmt.update {
                    self.visit_expr(func, update);
                }
                self.visit_block(func, &stmt.body);
            }
            Statement::ForInStatement(stmt) => {
                self.visit_expr(func, &stmt.right);
                self.visit_stmt(func, &stmt.body);
            }
            Statement::ForOfStatement(stmt) => {
                self.visit_expr(func, &stmt.right);
                self.visit_stmt(func, &stmt.body);
            }
            Statement::WhileStatement(stmt) => {
                self.visit_expr(func, &stmt.test);
                self.visit_block(func, &stmt.body);
            }
            Statement::DoWhileStatement(stmt) => {
                self.visit_block(func, &stmt.body);
                self.visit_expr(func, &stmt.test);
            }
            Statement::BlockStatement(block) => self.visit_block(func, block),
            Statement::LabeledStatement(stmt) => self.visit_stmt(func, &stmt.body),
            Statement::SwitchStatement(stmt) => {
                self.visit_expr(func, &stmt.discriminant);
                for case in &stmt.cases {
                    if let Some(test) = &case.test {
                        self.visit_expr(func, test);
                    }
                    for s in &case.consequent {
                        self.visit_stmt(func, s);
                    }
                }
            }
            Statement::TryStatement(stmt) => {
                self.visit_block(func, &stmt.block);
                if let Some(handler) = &stmt.handler {
                    self.visit_block(func, &handler.body);
                }
                if let Some(finalizer) = &stmt.finalizer {
                    self.visit_block(func, finalizer);
                }
            }
            Statement::ThrowStatement(stmt) => {
                self.visit_expr(func, &stmt.argument);
            }
            _ => {}
        }
    }

    /// `let/const/var id = init` — array-producing inits create an element
    /// node for `id`; everything else flows the init into `id`'s scalar node
    /// (`init -> id`).
    fn visit_declarator_init(&mut self, func: &str, id: &str, init: &Expression) {
        if let Expression::ObjectExpression(obj) = init {
            self.record_object_literal(
                func,
                ObjSlot::Binding(func.to_string(), id.to_string()),
                obj,
            );
            return;
        }
        self.record_object_flow_from_expr(
            func,
            ObjSlot::Binding(func.to_string(), id.to_string()),
            init,
        );
        if self.init_is_array(init) {
            let elem = self.array_elem_node_for(func, id);
            // Array-literal elements flow (store direction) into the element.
            if let Expression::ArrayExpression(arr) = init {
                for element in arr.elements.iter().flatten() {
                    if let kali_ast::ExpressionOrSpread::Expression(expr) = element {
                        if let Expression::ObjectExpression(obj) = expr {
                            self.record_object_literal(
                                func,
                                ObjSlot::ArrayElem(func.to_string(), id.to_string()),
                                obj,
                            );
                            continue;
                        }
                        self.record_object_flow_from_expr(
                            func,
                            ObjSlot::ArrayElem(func.to_string(), id.to_string()),
                            expr,
                        );
                        let en = self.visit_expr(func, expr);
                        self.add_edge(en, elem);
                    }
                }
            }
            return;
        }
        let rn = self.visit_expr(func, init);
        let sn = self.scalar_node_for(func, id);
        // init -> id.
        self.add_edge(rn, sn);
    }

    /// True when `expr` produces a fresh array binding (`new Array(...)` or an
    /// array literal).
    fn init_is_array(&self, expr: &Expression) -> bool {
        match expr {
            Expression::ArrayExpression(_) => true,
            Expression::NewExpression(new_expr) => {
                constructor_name(&new_expr.callee).as_deref() == Some("Array")
            }
            _ => false,
        }
    }

    // ---- Phase B: expression walk --------------------------------------

    /// Walk `expr`, wiring seeds/edges, and return its result node.
    fn visit_expr(&mut self, func: &str, expr: &Expression) -> usize {
        match expr {
            Expression::Identifier(name) => {
                // A read of a name not locally bound in `func`'s own scope
                // but declared at module scope is a module-const/binding
                // read (see `kali_codegen`'s matching identifier fallback):
                // route it to the SAME node as the module declaration so its
                // float-ness (e.g. `const DPY = 365.24;`) reaches every
                // reader, instead of a fresh, permanently-unseeded node
                // private to this function.
                let scope = if func != TOP_LEVEL
                    && !self.is_locally_declared(func, name)
                    && self.is_locally_declared(TOP_LEVEL, name)
                {
                    TOP_LEVEL
                } else {
                    func
                };
                self.scalar_node_for(scope, name)
            }

            Expression::Literal(LiteralValue::Number(n)) => {
                let node = self.new_node();
                if is_float_literal(*n) {
                    self.add_seed(node);
                }
                node
            }
            Expression::Literal(_) => self.new_node(),

            Expression::ParenthesizedExpression(inner) => self.visit_expr(func, &inner.expression),

            Expression::BinaryExpression(bin) => {
                let left = self.visit_expr(func, &bin.left);
                let right = self.visit_expr(func, &bin.right);
                let result = self.new_node();
                match bin.operator.as_str() {
                    "/" => {
                        // Division always yields a float: SEED the result. Its
                        // operands are NOT floated (edges run operand ->
                        // result only), so integer operands stay integer.
                        self.add_seed(result);
                        self.add_edge(left, result);
                        self.add_edge(right, result);
                    }
                    "+" | "-" | "*" | "%" | "**" => {
                        // Forward-flow: a float operand floats the result; the
                        // result does NOT flow back to the operands.
                        self.add_edge(left, result);
                        self.add_edge(right, result);
                    }
                    // Comparisons / bitwise / shift ops yield i64 (boolean or
                    // int32); operands are visited for their own edges but no
                    // edge runs into the result.
                    _ => {}
                }
                result
            }

            Expression::UnaryExpression(unary) => {
                if unary.operator == "delete" {
                    // `delete <base>.field` / `delete <base>[i]` never OBSERVES
                    // the deleted slot's value (codegen lowers `delete` through
                    // its own dedicated path, independent of this object axis).
                    // Visiting it like an ordinary member read would wrongly
                    // register a deferred field-*read* access, which the
                    // pending-conflict promotion below treats as evidence that
                    // the slot's value is actually consumed — miscounting a
                    // `delete` as a read would over-promote a structural
                    // literal that is only ever deleted-from and reinserted
                    // into (never truly read through this axis), rejecting a
                    // program that today runs correctly. Still visit the base
                    // (and computed index) for their own housekeeping, just
                    // without creating the ObjAccess.
                    self.visit_unobserved_member_target(func, &unary.argument);
                    return self.new_node();
                }
                let arg = self.visit_expr(func, &unary.argument);
                let result = self.new_node();
                if matches!(unary.operator.as_str(), "-" | "+") {
                    self.add_edge(arg, result);
                }
                result
            }

            Expression::UpdateExpression(update) => {
                // `x++` / `--x`: numeric identity on the operand.
                self.visit_expr(func, &update.argument)
            }

            Expression::AssignmentExpression(assign) => self.visit_assignment(func, assign),

            Expression::ConditionalExpression(cond) => {
                self.visit_expr(func, &cond.test);
                let cons = self.visit_expr(func, &cond.consequent);
                let alt = self.visit_expr(func, &cond.alternate);
                let result = self.new_node();
                // Either branch floats the result.
                self.add_edge(cons, result);
                self.add_edge(alt, result);
                result
            }

            Expression::LogicalExpression(logical) => {
                let left = self.visit_expr(func, &logical.left);
                let right = self.visit_expr(func, &logical.right);
                let result = self.new_node();
                self.add_edge(left, result);
                self.add_edge(right, result);
                result
            }

            Expression::SequenceExpression(seq) => {
                let mut last = self.new_node();
                for e in &seq.expressions {
                    last = self.visit_expr(func, e);
                }
                last
            }

            Expression::MemberExpression(member) => self.visit_member(func, member),

            Expression::CallExpression(call) => self.visit_call(func, call),

            Expression::NewExpression(new_expr) => {
                // Constructor arguments are visited for edges (e.g. `new Array`
                // length is an int). The handle itself is i64.
                for arg in &new_expr.args {
                    self.visit_expr(func, arg);
                }
                self.new_node()
            }

            // Any other expression kind is a fresh (int) node.
            _ => self.new_node(),
        }
    }

    fn visit_assignment(&mut self, func: &str, assign: &kali_ast::AssignmentExpression) -> usize {
        // Whole-object (re)assignment through a plain identifier target.
        if let Expression::Identifier(name) = &assign.left {
            if matches!(assign.operator, AssignmentOperator::Assign) {
                if let Expression::ObjectExpression(obj) = &assign.right {
                    let slot = ObjSlot::Binding(func.to_string(), name.clone());
                    self.record_object_literal(func, slot.clone(), obj);
                    // A reassigned literal is observable through the binding:
                    // the fold lane cannot represent it, so materialize.
                    self.obj_materialized.insert(slot);
                    return self.scalar_node_for(func, name);
                }
                self.record_object_flow_from_expr(
                    func,
                    ObjSlot::Binding(func.to_string(), name.clone()),
                    &assign.right,
                );
            }
        }

        // Array element store: `a[i] = v`.
        if let Expression::MemberExpression(member) = &assign.left {
            if let Some(index) = &member.computed_index {
                self.visit_expr(func, index); // index stays i64 (untouched).
                let rn = self.visit_expr(func, &assign.right);
                if let Expression::Identifier(name) = &member.object {
                    let elem = self.array_elem_node_for(func, name);
                    // Store is directed: value -> element (a float value floats
                    // the array; an int value into a float array stays int).
                    self.add_edge(rn, elem);
                } else {
                    self.visit_expr(func, &member.object);
                }
                return rn;
            }
            // Non-computed member store: `<base>.field = v` — deferred object
            // field access, wired after object propagation.
            let rn = self.visit_expr(func, &assign.right);
            if let Some(base) = self.member_base_slot(func, &member.object) {
                self.obj_accesses.push(ObjAccess {
                    base,
                    field: member.property.clone(),
                    other: rn,
                    is_write: true,
                });
            } else {
                self.visit_expr(func, &member.object);
            }
            return rn;
        }

        // Scalar assignment: `x = v`, `x += v`, `x /= v`, ...
        let rn = self.visit_expr(func, &assign.right);
        if let Expression::Identifier(name) = &assign.left {
            let sn = self.scalar_node_for(func, name);
            match assign.operator {
                AssignmentOperator::Assign
                | AssignmentOperator::AddAssign
                | AssignmentOperator::SubtractAssign
                | AssignmentOperator::MultiplyAssign
                | AssignmentOperator::ModuloAssign
                | AssignmentOperator::ExponentAssign => {
                    // rhs -> x.
                    self.add_edge(rn, sn);
                }
                AssignmentOperator::DivideAssign => {
                    // `x /= v` ⇒ x is float; the divisor keeps its own repr, so
                    // no `rhs -> x` edge.
                    self.add_seed(sn);
                }
                // Bitwise/logical compound assigns keep i64.
                _ => {}
            }
            return sn;
        }
        rn
    }

    fn visit_member(&mut self, func: &str, member: &kali_ast::MemberExpression) -> usize {
        // Computed access `a[i]` → array element read.
        if let Some(index) = &member.computed_index {
            self.visit_expr(func, index); // index untouched (i64).
            if let Expression::Identifier(name) = &member.object {
                let elem = self.array_elem_node_for(func, name);
                let result = self.new_node();
                // Read is directed: element -> read result.
                self.add_edge(elem, result);
                return result;
            }
            self.visit_expr(func, &member.object);
            return self.new_node();
        }

        // `.length` on a bare identifier is an array-length access: register the
        // receiver as an array binding so its i64 handle is treated as a
        // linear-memory array (matching codegen's `.length` header-load path),
        // even when the identifier is never subscripted directly. This mirrors
        // the subscript/`.fill`/`new Array` seeds and lets interprocedural
        // propagation link pass-through callers. Other dot access carries no
        // array signal. Arrays that are also subscripted are already seeded, so
        // integer programs (whose arrays are always indexed) are unaffected.
        if member.property.as_str() == "length" {
            if let Expression::Identifier(name) = &member.object {
                self.array_elem_node_for(func, name);
            }
        }

        // Non-computed member read `<base>.field` — deferred object access.
        if let Some(base) = self.member_base_slot(func, &member.object) {
            let result = self.new_node();
            self.obj_accesses.push(ObjAccess {
                base,
                field: member.property.as_str().to_string(),
                other: result,
                is_write: false,
            });
            return result;
        }

        // `.length` and other dot access → i64 result.
        self.visit_expr(func, &member.object);
        self.new_node()
    }

    /// Visit `expr` WITHOUT recording a deferred object-field READ access,
    /// for positions whose member-access result is provably never observed:
    /// the target of `delete <expr>` (which never observes the deleted
    /// slot's value — codegen lowers `delete` through its own dedicated
    /// path), and a bare top-level `<expr>;` expression-statement (whose
    /// value is unconditionally discarded — see the `ExpressionStatement`
    /// arm of `visit_stmt`, which is *also* the exact AST shape the parser
    /// currently produces for `delete <expr>;`, since it drops the `delete`
    /// keyword rather than building a `UnaryExpression` for it; routing both
    /// spellings through this helper keeps them consistent regardless of
    /// which one the parser happens to use). Mirrors `visit_member`'s
    /// housekeeping (index visiting, array-binding registration, `.length`
    /// array signal) so array/float bookkeeping is unaffected; only the
    /// terminal `ObjAccess` push is skipped. Not observing this read matters
    /// for `resolve_objects`'s pending-conflict promotion: an unobserved
    /// read must NOT count as evidence that a structurally-unsupported
    /// literal's value is consumed (that would over-promote a literal that
    /// is only ever deleted-from/reinserted-into and enumerated via
    /// `Object.keys`-style builtins, rejecting a program that runs correctly
    /// today — see `pending_slots_reached_by_a_read`).
    fn visit_unobserved_member_target(&mut self, func: &str, expr: &Expression) {
        let Expression::MemberExpression(member) = expr else {
            self.visit_expr(func, expr);
            return;
        };
        if let Some(index) = &member.computed_index {
            self.visit_expr(func, index);
            if let Expression::Identifier(name) = &member.object {
                self.array_elem_node_for(func, name);
            } else {
                self.visit_expr(func, &member.object);
            }
            return;
        }
        if member.property.as_str() == "length" {
            if let Expression::Identifier(name) = &member.object {
                self.array_elem_node_for(func, name);
                return;
            }
        }
        // Still visit the base for its own housekeeping (e.g. a nested
        // member/array access reached along the way), but never create a
        // deferred field-read ObjAccess for the terminal `.field`.
        self.visit_expr(func, &member.object);
    }

    fn visit_call(&mut self, func: &str, call: &kali_ast::CallExpression) -> usize {
        match &call.callee {
            // Method call: `obj.method(args)`.
            Expression::MemberExpression(member) if member.computed_index.is_none() => {
                let method = member.property.as_str();
                match method {
                    "sqrt" | "cbrt" if is_math_object(&member.object) => {
                        for arg in &call.args {
                            self.visit_expr(func, arg);
                        }
                        let result = self.new_node();
                        self.add_seed(result);
                        result
                    }
                    "toFixed" => {
                        // The receiver is a float.
                        let recv = self.visit_expr(func, &member.object);
                        self.add_seed(recv);
                        for arg in &call.args {
                            self.visit_expr(func, arg);
                        }
                        // `.toFixed` returns a string; result is a fresh i64.
                        self.new_node()
                    }
                    "fill" => {
                        // `a.fill(v)` is a store: value -> receiver element.
                        let vnode = call
                            .args
                            .first()
                            .map(|arg| self.visit_expr(func, arg))
                            .unwrap_or_else(|| self.new_node());
                        for arg in call.args.iter().skip(1) {
                            self.visit_expr(func, arg);
                        }
                        if let Expression::Identifier(name) = &member.object {
                            let elem = self.array_elem_node_for(func, name);
                            self.add_edge(vnode, elem);
                        } else {
                            self.visit_expr(func, &member.object);
                        }
                        // `.fill` returns the array handle (i64).
                        self.new_node()
                    }
                    _ => {
                        self.visit_expr(func, &member.object);
                        for arg in &call.args {
                            self.visit_expr(func, arg);
                        }
                        self.new_node()
                    }
                }
            }

            // Bare-identifier call: candidate user-function call. Record an
            // interprocedural edge (resolved after all bodies are walked).
            Expression::Identifier(callee) => {
                let mut arg_nodes = Vec::with_capacity(call.args.len());
                let mut arg_array_names = Vec::with_capacity(call.args.len());
                for arg in &call.args {
                    if matches!(arg, Expression::ObjectExpression(_)) {
                        self.obj_conflicts.push(
                            "an object literal passed directly as a call argument is unavailable in the current phase; bind it to a const first"
                                .to_string(),
                        );
                    }
                    arg_nodes.push(self.visit_expr(func, arg));
                    arg_array_names.push(match arg {
                        Expression::Identifier(name) => Some((func.to_string(), name.clone())),
                        _ => None,
                    });
                }
                let result_node = self.new_node();
                self.calls.push(CallEdge {
                    callee: callee.clone(),
                    arg_nodes,
                    arg_array_names,
                    result_node,
                });
                result_node
            }

            other => {
                self.visit_expr(func, other);
                for arg in &call.args {
                    self.visit_expr(func, arg);
                }
                self.new_node()
            }
        }
    }

    // ---- Phase C: interprocedural resolution ---------------------------

    fn resolve_calls(&mut self) {
        // Step 1: compute the transitive "is an array param" set to a fixpoint.
        // Seed with every binding directly used as an array (subscript, read,
        // write, `.fill`, `new Array`, array literal — anything that created an
        // `array_elem_node`). Then propagate: a caller's identifier argument
        // that is passed at position `k` to a callee whose param `k` is an
        // array binding is itself an array binding. This links pass-through
        // params (e.g. `w` in `AtAu(u,v,w){ Au(u,w); Atu(w,v); }`) that never
        // subscript their param directly.
        let mut array_bindings: BTreeSet<(String, String)> =
            self.array_elem_node.keys().cloned().collect();
        loop {
            let mut changed = false;
            for edge in &self.calls {
                let Some(params) = self.functions.get(&edge.callee) else {
                    continue;
                };
                for (k, arg) in edge.arg_array_names.iter().enumerate() {
                    let Some((caller, argname)) = arg else {
                        continue;
                    };
                    let Some(param_name) = params.get(k) else {
                        continue;
                    };
                    if array_bindings.contains(&(edge.callee.clone(), param_name.clone()))
                        && array_bindings.insert((caller.clone(), argname.clone()))
                    {
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }

        // Step 2: drain call edges and wire the interprocedural constraints.
        let calls = std::mem::take(&mut self.calls);
        for edge in calls {
            let Some(params) = self.functions.get(&edge.callee).cloned() else {
                continue; // Not a user function (builtin / undefined) — skip.
            };

            for (k, param_name) in params.iter().enumerate() {
                let is_array_param =
                    array_bindings.contains(&(edge.callee.clone(), param_name.clone()));
                if is_array_param {
                    // Array element flow is bidirectional shared storage: union
                    // the caller argument's element node with the param's. Only
                    // meaningful when the argument is a bare identifier.
                    if let Some(Some((caller, name))) = edge.arg_array_names.get(k) {
                        let caller_elem = self.array_elem_node_for(caller, name);
                        let param_elem = self.array_elem_node_for(&edge.callee, param_name);
                        self.uf.union(caller_elem, param_elem);
                        // Elements of the two arrays are the same objects.
                        self.obj_flows.push((
                            ObjSlot::ArrayElem(caller.clone(), name.clone()),
                            ObjSlot::ArrayElem(edge.callee.clone(), param_name.clone()),
                        ));
                    }
                } else if let Some(&arg_node) = edge.arg_nodes.get(k) {
                    // Scalar arg flow is directional: arg -> param.
                    let pnode = self.scalar_node_for(&edge.callee, param_name);
                    self.add_edge(arg_node, pnode);
                    // Object aliasing arg ~ param (no-op unless proven object).
                    if let Some(Some((caller, name))) = edge.arg_array_names.get(k) {
                        self.obj_flows.push((
                            ObjSlot::Binding(caller.clone(), name.clone()),
                            ObjSlot::Binding(edge.callee.clone(), param_name.clone()),
                        ));
                    }
                }
            }

            // Return flow is directional: callee return -> call-site result.
            let ret = self.return_node_for(&edge.callee);
            self.add_edge(ret, edge.result_node);
        }
    }

    // ---- Phase C2: object-shape propagation -----------------------------

    fn resolve_objects(&mut self) {
        // 1. Propagate field lists across flows to a fixpoint (copy into
        //    unknown sides only; mismatches are flagged once, afterwards).
        let mut fields_of: BTreeMap<ObjSlot, Vec<String>> = self.obj_literal_fields.clone();
        loop {
            let mut changed = false;
            for (a, b) in &self.obj_flows {
                match (fields_of.contains_key(a), fields_of.get(b).cloned()) {
                    (false, Some(fields)) => {
                        fields_of.insert(a.clone(), fields);
                        changed = true;
                    }
                    (true, None) => {
                        fields_of.insert(b.clone(), fields_of[a].clone());
                        changed = true;
                    }
                    _ => {}
                }
            }
            if !changed {
                break;
            }
        }
        for (a, b) in &self.obj_flows {
            if let (Some(fa), Some(fb)) = (fields_of.get(a), fields_of.get(b)) {
                if fa != fb {
                    self.obj_conflicts.push(format!(
                        "conflicting object shapes flow between {a:?} and {b:?}"
                    ));
                }
            }
        }

        // 2. Union per-field storage across flows between object slots; both
        //    endpoints of an object flow are observable through an alias, so
        //    they materialize.
        let flows = self.obj_flows.clone();
        for (a, b) in &flows {
            let Some(names) = fields_of.get(a).cloned() else {
                continue;
            };
            if !fields_of.contains_key(b) {
                continue;
            }
            for name in &names {
                let x = self.obj_field_node_for(a, name);
                let y = self.obj_field_node_for(b, name);
                self.uf.union(x, y);
            }
            self.obj_materialized.insert(a.clone());
            self.obj_materialized.insert(b.clone());
        }

        // 2.5. Compute which *pending*-conflict slots (structurally-unsupported
        //      literals — non-identifier key / getter-setter / nested object;
        //      see `record_object_literal`) are observably READ somewhere they
        //      can be reached, and so must be promoted below: a purely
        //      write-only or purely aliased-but-never-read pending literal
        //      stays on the fold lane untouched (fold-first). See the doc
        //      comment on `pending_slots_reached_by_a_read` for the exact
        //      rule and why it is shaped this way.
        let promote_via_read = self.pending_slots_reached_by_a_read();

        // 2.6. Pre-mark materialization for every WRITE access on a
        //      known-shape base, BEFORE gating individual accesses in step 3
        //      below. Materialization is a whole-program property: a write
        //      appearing later in the source must still be visible to an
        //      earlier unknown-field READ's materialized-gate (step 3), not
        //      just to writes processed after it in this same pass.
        for access in &self.obj_accesses {
            if access.is_write {
                if let Some(names) = fields_of.get(&access.base) {
                    if names.contains(&access.field) {
                        self.obj_materialized.insert(access.base.clone());
                    }
                }
            }
        }

        // 3. Wire deferred member accesses through canonical field storage.
        let accesses = std::mem::take(&mut self.obj_accesses);
        for access in accesses {
            let Some(names) = fields_of.get(&access.base) else {
                continue; // not an object: fold lane / existing behavior
            };
            if !names.contains(&access.field) {
                // Unknown-field access on a KNOWN shape: a real conflict only
                // when the base is observable outside the fold lane — a
                // write, or a base materialized elsewhere. A read-only
                // unknown-field access on a literal that never materializes
                // matches JS (`undefined`) and must NOT reject (fold-first).
                if access.is_write || self.obj_materialized.contains(&access.base) {
                    self.obj_conflicts.push(format!(
                        "unknown field '{}' on fixed-shape object {:?}",
                        access.field, access.base
                    ));
                }
                continue;
            }
            let field_node = self.obj_field_node_for(&access.base, &access.field);
            if access.is_write {
                self.add_edge(access.other, field_node);
                self.obj_materialized.insert(access.base.clone());
            } else {
                self.add_edge(field_node, access.other);
            }
        }

        // 4. Promote deferred structural conflicts for any slot that was
        //    forced onto the object lane: reassigned (already recorded in
        //    `obj_materialized` eagerly by `visit_assignment`'s whole-object
        //    reassignment branch), or reached by an observable read per 2.5
        //    above. A structurally-unsupported literal that stayed read-only
        //    and non-aliased, or that is only written-to and consumed via
        //    generic enumeration (`Object.keys`-style builtins, which never
        //    create a deferred field access here), never materializes and so
        //    keeps its fold-lane behavior byte-identically (fold-first
        //    invariant) — this is the whole point of the deferral.
        let pending = std::mem::take(&mut self.obj_pending_conflicts);
        for (slot, message) in pending {
            if self.obj_materialized.contains(&slot) || promote_via_read.contains(&slot) {
                self.obj_conflicts.push(message);
            }
        }

        self.obj_fields_of = fields_of;
    }

    /// Which `obj_pending_conflicts` slots are observably READ somewhere
    /// reachable from them, and so must be promoted to a real (rejected)
    /// conflict rather than staying deferred.
    ///
    /// A pending (structurally-unsupported) literal hits the buggy fold-lane
    /// `.field` read codegen this gate exists to guard against precisely when
    /// its value can be read from somewhere OTHER than a compile-time fold of
    /// its own literal text:
    ///
    /// - **Same-slot write + read**: the slot is both the base of a WRITE
    ///   `ObjAccess` and a READ `ObjAccess` — the write makes a same-slot
    ///   fold-to-literal-text read stale (`const p = {"a-b":1}; p.c = 2;
    ///   console.log(p.c);` must print `2`, not the folded/default value).
    /// - **Cross-slot via any flow**: the slot participates in ANY
    ///   object-aliasing flow (assignment, array-element sharing,
    ///   arg↔param, return↔call-site — `obj_flows`, regardless of whether
    ///   either endpoint has a known field list) AND *some* slot in that
    ///   flow-connected component has a READ `ObjAccess` anywhere. Crossing a
    ///   flow means the reading site has no visibility into the original
    ///   literal text to fold from at all (`function f(o){return o.c;}
    ///   f(structuralLiteral)` must reject, not silently fold to `0`) — a
    ///   write on that same component is not required.
    ///
    /// A literal that is only ever read locally with no write (the classic
    /// read-only fold-lane case), or only ever written-to/aliased but never
    /// read through this axis (e.g. deleted-from and reinserted, then only
    /// consumed via `Object.keys`/`Object.values`/`Object.entries`/
    /// `Reflect.ownKeys`, none of which create a deferred `ObjAccess`), is
    /// NOT promoted — that would re-break fold-first, the whole point of the
    /// pending-conflict deferral. `delete <base>.field` (and any bare,
    /// top-level `<base>.field;` expression statement, whose result is
    /// unconditionally discarded either way) is deliberately excluded from
    /// counting as a read — see `visit_unobserved_member_target`.
    fn pending_slots_reached_by_a_read(&self) -> BTreeSet<ObjSlot> {
        // Union-find over `ObjSlot`s via `obj_flows` (undirected aliasing).
        // `ObjSlot`s are never merged in the *field-storage* union-find
        // (`self.uf`, which only unions field nodes of slots that already
        // have BOTH field lists known — step 2 above); this is a separate,
        // slot-identity-level grouping used only to decide promotion.
        fn find(parent: &mut BTreeMap<ObjSlot, ObjSlot>, x: &ObjSlot) -> ObjSlot {
            let p = parent.entry(x.clone()).or_insert_with(|| x.clone()).clone();
            if &p == x {
                return p;
            }
            let root = find(parent, &p);
            parent.insert(x.clone(), root.clone());
            root
        }

        let mut parent: BTreeMap<ObjSlot, ObjSlot> = BTreeMap::new();
        for (a, b) in &self.obj_flows {
            let ra = find(&mut parent, a);
            let rb = find(&mut parent, b);
            if ra != rb {
                parent.insert(ra, rb);
            }
        }

        let mut members: BTreeMap<ObjSlot, BTreeSet<ObjSlot>> = BTreeMap::new();
        for (a, b) in &self.obj_flows {
            let root = find(&mut parent, a);
            members.entry(root.clone()).or_default().insert(a.clone());
            members.entry(root).or_default().insert(b.clone());
        }

        let mut comp_has_read: BTreeSet<ObjSlot> = BTreeSet::new();
        let mut comp_has_write: BTreeSet<ObjSlot> = BTreeSet::new();
        for access in &self.obj_accesses {
            let root = find(&mut parent, &access.base);
            if access.is_write {
                comp_has_write.insert(root);
            } else {
                comp_has_read.insert(root);
            }
        }

        let mut promoted = BTreeSet::new();
        for slot in self.obj_pending_conflicts.keys() {
            let root = find(&mut parent, slot);
            let component_size = members.get(&root).map(BTreeSet::len).unwrap_or(1);
            let has_read = comp_has_read.contains(&root);
            let has_write = comp_has_write.contains(&root);
            if has_read && (has_write || component_size > 1) {
                promoted.insert(slot.clone());
            }
        }
        promoted
    }

    // ---- Phase D: solve → table ----------------------------------------

    /// BFS float reachability over the directed edge graph, with endpoints
    /// canonicalised through the array-element union-find. Returns a per-node
    /// float flag indexed by node id; array-element clusters are read via their
    /// representative.
    fn solve_float(&mut self) -> Vec<bool> {
        let n = self.node_count;
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        let edges = std::mem::take(&mut self.edges);
        for (from, to) in edges {
            let f = self.uf.find(from);
            let t = self.uf.find(to);
            adj[f].push(t);
        }

        let mut float = vec![false; n];
        let mut queue: VecDeque<usize> = VecDeque::new();
        let seeds = std::mem::take(&mut self.seeds);
        for s in seeds {
            let r = self.uf.find(s);
            if !float[r] {
                float[r] = true;
                queue.push_back(r);
            }
        }
        while let Some(u) = queue.pop_front() {
            let mut i = 0;
            while i < adj[u].len() {
                let v = adj[u][i];
                if !float[v] {
                    float[v] = true;
                    queue.push_back(v);
                }
                i += 1;
            }
        }
        float
    }

    fn emit_table(mut self) -> ReprTable {
        let float = self.solve_float();
        let mut table = ReprTable::default();

        // Scalars (BTreeMap ⇒ deterministic order). Scalar nodes are never
        // unioned, so the flag can be read directly.
        let scalars: Vec<((String, String), usize)> = self
            .scalar_node
            .iter()
            .map(|(k, &v)| (k.clone(), v))
            .collect();
        for ((func, name), node) in scalars {
            if float[node] {
                table.set_scalar(&func, &name, Repr::F64);
            }
        }

        // Array elements — read via the union-find representative.
        let elems: Vec<((String, String), usize)> = self
            .array_elem_node
            .iter()
            .map(|(k, &v)| (k.clone(), v))
            .collect();
        for ((func, name), node) in elems {
            // Every binding/param with an element node is an array (i64 or f64),
            // including subscripted params and pass-through array params linked
            // only via the transitive array-param fixpoint (which unions their
            // element node into the callee's in `resolve_calls`).
            table.set_array_binding(&func, &name);
            let rep = self.uf.find(node);
            if float[rep] {
                table.set_array_element(&func, &name, Repr::F64);
            }
        }

        // Returns.
        let returns: Vec<(String, usize)> = self
            .return_node
            .iter()
            .map(|(k, &v)| (k.clone(), v))
            .collect();
        for (func, node) in returns {
            if float[node] {
                table.set_return(&func, Repr::F64);
            }
        }

        // Params (aliased to each param's scalar node, indexed by position).
        let functions: Vec<(String, Vec<String>)> = self
            .functions
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (func, params) in functions {
            for (index, name) in params.iter().enumerate() {
                if let Some(&node) = self.scalar_node.get(&(func.clone(), name.clone())) {
                    if float[node] {
                        table.set_param(&func, index, Repr::F64);
                    }
                }
            }
        }

        // Object shapes: one interned Shape per materialized object slot.
        // Unmaterialized (write-free, non-flowing) literals get NO entry —
        // codegen keeps its compile-time fold lane for them.
        let fields_of = std::mem::take(&mut self.obj_fields_of);
        let materialized = std::mem::take(&mut self.obj_materialized);
        for (slot, names) in &fields_of {
            if !materialized.contains(slot) {
                continue;
            }
            let fields: Vec<(String, Repr)> = names
                .iter()
                .map(|name| {
                    let node = self.obj_field_node_for(slot, name);
                    let rep = self.uf.find(node);
                    let repr = if float[rep] { Repr::F64 } else { Repr::I64 };
                    (name.clone(), repr)
                })
                .collect();
            let shape = table.intern_shape(fields);
            match slot {
                ObjSlot::Binding(func, name) => {
                    // A binding both object and float-unified is contradictory.
                    if let Some(&node) = self.scalar_node.get(&(func.clone(), name.clone())) {
                        if float[node] {
                            self.obj_conflicts.push(format!(
                                "binding '{name}' in '{func}' is used both as an object and as a number"
                            ));
                            continue;
                        }
                    }
                    table.set_scalar(func, name, Repr::Object(shape));
                }
                ObjSlot::ArrayElem(func, name) => {
                    table.set_array_binding(func, name);
                    table.set_array_element(func, name, Repr::Object(shape));
                }
                ObjSlot::Return(func) => table.set_return(func, Repr::Object(shape)),
            }
        }
        // Object params mirror the binding entry positionally.
        let functions: Vec<(String, Vec<String>)> = self
            .functions
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (func, params) in functions {
            for (index, name) in params.iter().enumerate() {
                if let Repr::Object(shape) = table.scalar(&func, name) {
                    table.set_param(&func, index, Repr::Object(shape));
                } else if table.is_array_binding(&func, name) {
                    // Array-of-objects param: the param itself has no scalar
                    // Object entry (only its elements do), but codegen still
                    // needs the shape at the param position to lower element
                    // accesses without re-deriving it from the array binding.
                    if let Repr::Object(shape) = table.array_element(&func, name) {
                        table.set_param(&func, index, Repr::Object(shape));
                    }
                }
            }
        }
        for message in std::mem::take(&mut self.obj_conflicts) {
            table.add_shape_conflict(message);
        }

        table
    }
}

/// Classify a numeric literal value as a float seed. The AST stores literals as
/// `f64` (the raw token text is not retained), so a literal seeds float iff it
/// is not an exact finite integer (has a fractional part, or is non-finite).
fn is_float_literal(n: f64) -> bool {
    !(n.is_finite() && n.fract() == 0.0)
}

/// True when `expr` is the `Math` object (`Math` identifier).
fn is_math_object(expr: &Expression) -> bool {
    matches!(expr, Expression::Identifier(name) if name == "Math")
}

/// Extract the constructor identifier name from a `new`-expression callee,
/// which the parser may shape as `Identifier("Array")` or
/// `CallExpression { callee: Identifier("Array"), .. }` (`new Array(n)`).
fn constructor_name(callee: &Expression) -> Option<String> {
    match callee {
        Expression::Identifier(name) => Some(name.clone()),
        Expression::CallExpression(call) => match &call.callee {
            Expression::Identifier(name) => Some(name.clone()),
            _ => None,
        },
        _ => None,
    }
}
