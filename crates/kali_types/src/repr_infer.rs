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
    /// Deferred interprocedural call constraints.
    calls: Vec<CallEdge>,
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

    // Phase B: walk bodies. Top-level non-function statements run under the
    // synthetic `_start`; each `FunctionDeclaration` runs under its own name.
    for stmt in statements {
        infer.visit_stmt(TOP_LEVEL, stmt);
    }

    // Phase C: resolve deferred call edges (transitive array-param fixpoint +
    // directed scalar/return edges + bidirectional array-element unions).
    infer.resolve_calls();

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
                self.visit_expr(func, &stmt.expression);
            }
            Statement::ReturnStatement(stmt) => {
                if let Some(arg) = &stmt.argument {
                    let rn = self.visit_expr(func, arg);
                    let ret = self.return_node_for(func);
                    // expr -> return.
                    self.add_edge(rn, ret);
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
        if self.init_is_array(init) {
            let elem = self.array_elem_node_for(func, id);
            // Array-literal elements flow (store direction) into the element.
            if let Expression::ArrayExpression(arr) = init {
                for element in arr.elements.iter().flatten() {
                    if let kali_ast::ExpressionOrSpread::Expression(expr) = element {
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
            Expression::Identifier(name) => self.scalar_node_for(func, name),

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
            // `.length`/`.field =` — visit both sides, no numeric edge.
            self.visit_expr(func, &member.object);
            return self.visit_expr(func, &assign.right);
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

        // `.length` and other dot access → i64 result.
        self.visit_expr(func, &member.object);
        self.new_node()
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
                    }
                } else if let Some(&arg_node) = edge.arg_nodes.get(k) {
                    // Scalar arg flow is directional: arg -> param.
                    let pnode = self.scalar_node_for(&edge.callee, param_name);
                    self.add_edge(arg_node, pnode);
                }
            }

            // Return flow is directional: callee return -> call-site result.
            let ret = self.return_node_for(&edge.callee);
            self.add_edge(ret, edge.result_node);
        }
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
