//! Expression resolution.
use crate::*;

impl TypeContext {
    pub(crate) fn is_simple_for_of_binding_expression(&self, expression: &Expression) -> bool {
        matches!(
            self.unwrap_for_of_wrapper_expression(expression),
            Expression::Identifier(_)
        )
    }

    pub(crate) fn is_simple_update_target_expression(&self, expression: &Expression) -> bool {
        matches!(
            expression,
            Expression::Identifier(_)
                | Expression::ParenthesizedExpression(_)
                | Expression::TypeAssertion(_)
                | Expression::SatisfiesExpression(_)
                | Expression::DecoratedExpression(_)
        )
    }

    pub(crate) fn resolve_update_binding_name(&self, expression: &Expression) -> Option<String> {
        match expression {
            Expression::Identifier(name) => Some(name.clone()),
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_update_binding_name(&expr.expression)
            }
            Expression::TypeAssertion(expr) => self.resolve_update_binding_name(&expr.expression),
            Expression::SatisfiesExpression(expr) => {
                self.resolve_update_binding_name(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_update_binding_name(&expr.expression)
            }
            _ => None,
        }
    }

    /// True when a binding named `name` is known to hold a *string* value
    /// (recorded by `resolve_variable_declaration` when its initializer is
    /// string-typed). Walks the scope chain, then the global scope. Reassignment
    /// clears the flag via `invalidate_static_binding`, so a name that was a string
    /// but has since been reassigned (e.g. to a number) is not reported as a
    /// string here — keeping the check flow-aware and sound.
    pub(crate) fn binding_is_string_typed(&self, name: &str) -> bool {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let Some(scope) = self.scopes.get(&scope_id) else {
                return false;
            };
            if let Some(&value) = scope.static_string_typed.get(name) {
                return value;
            }
            current = scope.parent;
        }
        self.global_scope
            .static_string_typed
            .get(name)
            .copied()
            .unwrap_or(false)
    }

    /// Semantic string-typedness of an expression: does it evaluate to a string at
    /// runtime? Covers string/template literals, `+` expressions with a string
    /// operand (JS `string + any` is a string), and *identifiers bound to a string
    /// value* (transparent wrappers unwrapped). This intentionally recognizes
    /// string-typed variables, which codegen's structural check does not.
    pub(crate) fn expression_is_string_typed(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Literal(LiteralValue::String(_)) => true,
            Expression::TemplateLiteral(_) => true,
            // A `for..in` key (or alias) used as a VALUE is its field-name
            // STRING (Spec 4a Task 5) — but ONLY where `repr_infer` actually
            // lifted its scalar repr to `String` (a string SINK: `return c`,
            // `console.log(c)`, `+`/equality of a bare key). `identifier_repr_is_string`
            // reads the SAME solved `scalar(func,name)==String` that codegen's
            // `emit_value`/`is_string_valued` materialization guard consults, so
            // types and codegen cover EXACTLY the same set. A for-in key in a
            // non-sink position (e.g. `strArr[i] = c`, or a ternary arm) is NOT
            // repr-lifted → false here → codegen emits the raw ordinal and this
            // predicate agrees (fail-closed, never a raw-ordinal-as-string open).
            Expression::Identifier(name) => {
                self.binding_is_string_typed(name) || self.identifier_repr_is_string(name)
            }
            // Computed element read `a[i]` of an array whose element axis is
            // proven `Repr::String` (Spec 3). Mirror of codegen's
            // `is_string_valued` `dynamic_array_read_base` arm — both classify
            // the loaded element as a string so the `+`/`.length` gates and the
            // print/concat lowering agree.
            Expression::MemberExpression(member) if member.computed_index.is_some() => {
                matches!(&member.object, Expression::Identifier(base)
                    if self.string_element_array_binding(base))
            }
            Expression::BinaryExpression(expr) if expr.operator == "+" => {
                self.expression_is_string_typed(&expr.left)
                    || self.expression_is_string_typed(&expr.right)
            }
            // A ternary is string-typed iff EITHER arm is (mirrors codegen's
            // `emit_conditional` `string_result` and `is_string_valued`'s ternary
            // arm at `operators.rs`). Fail-closed: one string arm taints the whole
            // conditional, so the `.length`/store/`+` gates that key on this treat
            // a partially-string ternary as a string receiver.
            Expression::ConditionalExpression(expr) => {
                self.expression_is_string_typed(&expr.consequent)
                    || self.expression_is_string_typed(&expr.alternate)
            }
            Expression::ParenthesizedExpression(expr) => {
                self.expression_is_string_typed(&expr.expression)
            }
            Expression::TypeAssertion(expr) => self.expression_is_string_typed(&expr.expression),
            Expression::SatisfiesExpression(expr) => {
                self.expression_is_string_typed(&expr.expression)
            }
            _ => false,
        }
    }

    /// Mirror of codegen's structural `is_string_valued`
    /// (`kali_codegen/src/emit/operators.rs`): recognizes only string/template
    /// literals and `+` chains rooted in one — NOT a variable that holds a string.
    /// Operands for which this is true are lowered to string concatenation
    /// correctly and must not be rejected.
    fn expression_is_codegen_string_valued(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Literal(LiteralValue::String(_)) => true,
            Expression::TemplateLiteral(_) => true,
            Expression::BinaryExpression(expr) if expr.operator == "+" => {
                self.expression_is_codegen_string_valued(&expr.left)
                    || self.expression_is_codegen_string_valued(&expr.right)
            }
            // Mirror of codegen's `is_string_valued` ternary arm
            // (`operators.rs`, marker text "?", either-arm rule): a ternary whose
            // arm is a codegen-recognized structural string is itself lowered to
            // string concatenation, so it must NOT be rejected by the `+` gate.
            // Kept in lockstep with `expression_is_string_typed`'s ternary arm so
            // a string-armed ternary `+` operand passes the E3200 gate.
            Expression::ConditionalExpression(expr) => {
                self.expression_is_codegen_string_valued(&expr.consequent)
                    || self.expression_is_codegen_string_valued(&expr.alternate)
            }
            Expression::ParenthesizedExpression(expr) => {
                self.expression_is_codegen_string_valued(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                self.expression_is_codegen_string_valued(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.expression_is_codegen_string_valued(&expr.expression)
            }
            _ => false,
        }
    }

    /// True when `name`'s runtime representation at the CURRENT resolution
    /// point is proven `Repr::String` by the repr inference — the SAME signal
    /// codegen's `is_string_valued` identifier arm uses
    /// (`kali_codegen/src/emit/operators.rs`), so the gate and codegen never
    /// disagree.
    ///
    /// Mirrors codegen's local/module-const dichotomy (`self.locals.contains_key
    /// (name) ... else self.repr_table.scalar("_start", name)`), which is a
    /// FLAT per-function-body test — codegen has no lexical/block scoping
    /// inside one wasm function, so any binding declared anywhere in the
    /// current function counts as "local" to it. Concretely: walk the
    /// resolver's scope chain from the current position outward.
    ///
    /// - If `name` is found in a scope at or before the tracked function's own
    ///   `ScopeType::Function` scope (`current_function_scope()`), it is local
    ///   to the SAME function codegen is about to emit: consult
    ///   `scalar(current_function_name(), name)`.
    /// - If we reach module/global scope without finding it (and without
    ///   crossing an untracked boundary), it is a free reference to a
    ///   module-level binding: consult `scalar("_start", name)`, mirroring
    ///   codegen's fallback.
    /// - If, before either of the above, we reach a `ScopeType::Function`
    ///   scope that is NOT `current_function_scope()` — an arrow function,
    ///   function expression, class method, or `export default function`,
    ///   none of which push onto `current_function` (see
    ///   `TypeContext::current_function_scope`'s doc) — `current_function_name()`
    ///   does not actually name the function whose body we are in, so neither
    ///   table lookup above is safe: a same-named module binding or a
    ///   same-named binding in a DIFFERENT enclosing function could wrongly
    ///   suppress the gate. FAIL CLOSED (return `false`) instead of guessing.
    fn identifier_repr_is_string(&self, name: &str) -> bool {
        use kali_common::Repr;
        match self.binding_repr_function_key(name) {
            Some(func) => self.repr_table.scalar(&func, name) == Repr::String,
            None => false,
        }
    }

    /// Resolves the `ReprTable` function-key under which a binding named `name`
    /// is recorded, mirroring codegen's local-vs-module dichotomy EXACTLY as the
    /// (now-thin) `identifier_repr_is_string` scalar lookup does — the single
    /// scope-chain walk shared by every "what codegen thinks this binding's repr
    /// is" query (scalar String, array-element String, element non-ASCII/taint).
    ///
    /// - `Some(current_function_name())` — `name` is declared at or before the
    ///   tracked function's own `Function` scope, so it is local to the wasm
    ///   function codegen is about to emit.
    /// - `Some("_start")` — the walk reached module/global scope (or the tracked
    ///   function's top scope) without finding `name`: a free reference to a
    ///   module-level binding, mirroring codegen's `self.locals`-miss fallback.
    /// - `None` — the walk crossed a `Function` scope that is NOT
    ///   `current_function_scope()` (an arrow/function-expression/method/`export
    ///   default function` body that does not push onto `current_function`):
    ///   neither table lookup is safe, so callers FAIL CLOSED.
    fn binding_repr_function_key(&self, name: &str) -> Option<String> {
        let tracked_scope = self.current_function_scope();
        let mut current = self.current_scope_id();
        loop {
            let Some(scope_id) = current else {
                // Reached module/global scope: free top-level reference.
                return Some("_start".to_string());
            };
            let scope = self.scopes.get(&scope_id)?;
            if scope.scope_type == ScopeType::Function && Some(scope_id) != tracked_scope {
                return None;
            }
            if scope.contains(name) {
                return Some(self.current_function_name().to_string());
            }
            if scope.scope_type == ScopeType::Function {
                // Reached the tracked function's own top scope without finding
                // `name` there: mirror codegen's `self.locals`-miss fallback,
                // which unconditionally consults the module `_start` table
                // regardless of any further-enclosing scope (codegen does not
                // model closures over an outer function's locals).
                return Some("_start".to_string());
            }
            current = scope.parent;
        }
    }

    /// True iff `name` is STRUCTURALLY registered as a codegen runtime array
    /// binding in the CURRENT function — the types-side mirror of codegen's
    /// `array_bindings` membership (see `Scope::runtime_array_bindings`). Walks
    /// the scope chain exactly like `binding_repr_function_key`: it stops at the
    /// tracked function's own boundary, so a binding registered in an OUTER
    /// function (or, from within a named function, at module scope) is NOT
    /// structural here — codegen's emitter for this function would never have
    /// registered it. Only when emitting `_start` (no tracked function) does the
    /// walk reach module/global scope, matching codegen's `_start` locals.
    ///
    /// - crossing an UNTRACKED function-shaped scope (arrow/method/etc.) ⇒
    ///   `false` (fail-closed, same reason as `binding_repr_function_key`),
    /// - reaching the tracked function's own top scope without a hit ⇒ `false`
    ///   (a free module reference codegen does NOT register in this function).
    pub(crate) fn is_structural_runtime_array(&self, name: &str) -> bool {
        let tracked_scope = self.current_function_scope();
        let mut current = self.current_scope_id();
        loop {
            let Some(scope_id) = current else {
                // Reached module/global (only possible under `_start`, whose
                // tracked scope is `None`): module bindings are `_start`-local
                // to codegen, so a top-level `new Array` IS structural here.
                return self.global_scope.runtime_array_bindings.contains_key(name);
            };
            let Some(scope) = self.scopes.get(&scope_id) else {
                return false;
            };
            if scope.scope_type == ScopeType::Function && Some(scope_id) != tracked_scope {
                // Crossed into a function `current_function_name()` does not
                // name — fail closed rather than guess.
                return false;
            }
            if scope.runtime_array_bindings.contains_key(name) {
                return true;
            }
            if scope.scope_type == ScopeType::Function {
                // Tracked function's own top scope, no hit: a free module
                // reference codegen's emitter for this function never registers.
                return false;
            }
            current = scope.parent;
        }
    }

    /// `Some(shape)` iff `name` is a `for..in` key binding over a known
    /// object shape — the Spec 4a Task 2 dormant provenance registry. Walks
    /// the scope chain exactly like `is_structural_runtime_array`: stops at
    /// the tracked function's own boundary (fail-closed; a binding registered
    /// in an outer, untracked-boundary-crossed function is NOT visible here),
    /// and only reaches module/global scope when emitting `_start` (no
    /// tracked function).
    pub(crate) fn for_in_key_shape(&self, name: &str) -> Option<kali_common::ShapeId> {
        let tracked_scope = self.current_function_scope();
        let mut current = self.current_scope_id();
        loop {
            let Some(scope_id) = current else {
                return self.global_scope.for_in_key_bindings.get(name).copied();
            };
            let scope = self.scopes.get(&scope_id)?;
            if scope.scope_type == ScopeType::Function && Some(scope_id) != tracked_scope {
                // Crossed into a function `current_function_name()` does not
                // name — fail closed rather than guess.
                return None;
            }
            if let Some(shape) = scope.for_in_key_bindings.get(name) {
                return Some(*shape);
            }
            if scope.scope_type == ScopeType::Function {
                // Tracked function's own top scope, no hit: a free module
                // reference this function never registered a key for.
                return None;
            }
            current = scope.parent;
        }
    }

    /// Spec 4a Task 3 fail-closed gate for a computed for-in-key object access
    /// `obj[c]`. Fires ONLY when `c` is a proven `for..in` key
    /// (`for_in_key_shape`) — a runtime ordinal over a fixed shape — and the
    /// base `obj` is a KNOWN object (`object_shape_of_expression`). Rejects
    /// (E5506) when the base's shape does not match the key's shape (the
    /// ordinal range would be wrong for this base) or is not uniform-repr (a
    /// runtime ordinal cannot select a per-field type — mixed I64/F64 fields
    /// must fail closed, never miscompile). A non-object base (array / unknown)
    /// keeps its existing behavior: `arr[c]` over an array is a valid element
    /// read. This is the types-side authority the codegen recognizer
    /// (`computed_forin_object_access`) mirrors — both admit exactly the
    /// uniform, shape-matched case; codegen fails closed by falling through to
    /// a static-field read for everything else, which this gate makes
    /// unreachable. Runs for both the RHS read `= obj[c]` and the store target
    /// `obj[c] = v` (the assignment dispatch resolves `expr.left` through
    /// `resolve_member_expression` too).
    pub(crate) fn reject_nonuniform_forin_key_object_access(&mut self, member: &MemberExpression) {
        let Some(index) = member.computed_index.as_deref() else {
            return;
        };
        let Expression::Identifier(key) = index else {
            return;
        };
        let Some(obj_shape) = self.object_shape_of_expression(&member.object) else {
            // Not a known object (an array or an unproven base): leave the
            // existing element/host member behavior untouched.
            return;
        };
        // The base IS a known fixed-shape object. The ONLY dynamic
        // (identifier-indexed) access this direct-runtime path supports is a
        // `for..in` key over the SAME shape whose fields all share ONE NUMERIC
        // repr. Everything else off that lane fails closed here.
        let Some(key_shape) = self.for_in_key_shape(key) else {
            // Row 2: a general dynamic key not derived from `for..in` over this
            // object (a plain param/local index `obj[k]`) — Spec 4b territory.
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "computed key access `obj[k]` where the key is not a `for..in` key over `obj` is unavailable in the current direct-runtime path (general dynamic string-keyed access); use a `for..in` key over the same object".to_string(),
            ));
            return;
        };
        if obj_shape != key_shape {
            // Row 4: the key enumerates a DIFFERENT object than the base — the
            // ordinal range is wrong for this base.
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "computed key access `obj[c]` where the for..in key enumerates a different object shape than the base is unavailable in the current direct-runtime path".to_string(),
            ));
            return;
        }
        match self.repr_table.shape_is_uniform_repr(obj_shape) {
            Some(kali_common::Repr::I64) | Some(kali_common::Repr::F64) => {}
            // Row 3 (string-into-field materializes a uniform-String shape) and
            // any object-repr shape: a runtime ordinal only selects a numeric
            // slot in this lane; a string/object field is out of scope.
            Some(_) => {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "computed key access `obj[c]` over a fixed shape whose fields are strings or objects is unavailable in the current direct-runtime path (a runtime ordinal only selects a numeric slot); use an object whose fields are all numbers".to_string(),
                ));
            }
            // Row 5: mixed-repr shape — a runtime ordinal cannot pick a
            // per-field type.
            None => {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "computed key access `obj[c]` over a mixed-repr fixed shape is unavailable in the current direct-runtime path (a runtime ordinal cannot select a per-field type); use an object whose fields all share one type".to_string(),
                ));
            }
        }
    }

    /// Fail-closed gate: a `for..in` key (or alias, or declarator value-copy)
    /// binding used as an operand of `!`, `&&`, `||`, or `??` would be lowered
    /// with raw integer truthiness (`I64Eqz`/`I64And`/`I64Or`/nullish `Eqz`),
    /// which INVERTS the null-sentinel (`-1`) semantics AND leaks the raw ordinal
    /// (`&&`/`||`/`??` value-select an operand) — `!last` with `last == -1`
    /// (null) must be `true` but `-1` is nonzero, and `d && x` with `d` a key must
    /// yield `x`, not the ordinal. Only an `if` condition (lowered `>= 0`), a
    /// computed index (`obj[c]`), and a MATERIALIZED returned value are safe.
    /// Keys on `is_for_in_key_value` (the full value-provenance predicate — direct
    /// key + assignment alias + declarator value-copy), NOT bare `for_in_key_shape`
    /// which misses declarator aliases (`let d = c; d && x`), closing that leak.
    pub(crate) fn reject_forin_key_boolean_operand(&mut self, expr: &Expression, op: &str) {
        let mut inner = expr;
        while let Expression::ParenthesizedExpression(p) = inner {
            inner = &p.expression;
        }
        if let Expression::Identifier(name) = inner {
            if self.is_for_in_key_value(name) {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "a for..in key or alias binding is only usable in an `if` condition, a computed index, or as a returned value; using it as an operand of `{op}` is unavailable in the current direct-runtime path"
                    ),
                ));
            }
        }
    }

    /// Fail-closed gate (Spec 4a Task 6, controller handoff H2): a `for..in`
    /// key or alias binding used as a `while` / `for` / `do-while` condition or
    /// a ternary TEST is lowered via codegen's DEFAULT `!= 0` truthiness (only
    /// an `if` condition lowers through the `>= 0` null-sentinel path in
    /// `emit_branch`), so the `-1` null sentinel would read TRUTHY — a
    /// fail-OPEN in the same class as the `!`/`&&`/`||`/`??` operand rejects
    /// but in loop/ternary test positions. fasta uses NONE of these forms
    /// (only `if (last)`), so reject fail-closed rather than lower `>= 0` here.
    /// Keyed strictly on a `for_in_key_shape`-carrying identifier: a normal
    /// `while`/`for`/`do-while`/ternary on any other binding is untouched, and
    /// `if (last)` (makeCumulative) stays admitted because the `if` arm never
    /// calls this gate.
    pub(crate) fn reject_forin_key_test_operand(&mut self, expr: &Expression, context: &str) {
        let mut inner = expr;
        while let Expression::ParenthesizedExpression(p) = inner {
            inner = &p.expression;
        }
        if let Expression::Identifier(name) = inner {
            if self.for_in_key_shape(name).is_some() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "a for..in key or alias binding is only usable in an `if` condition, a computed index, or as a returned value; using it as a {context} is unavailable in the current direct-runtime path (its null sentinel would read truthy)"
                    ),
                ));
            }
        }
    }

    /// `true` iff `member` is exactly the accept case of
    /// `reject_nonuniform_forin_key_object_access`: a computed access `obj[c]`
    /// where the index `c` is a proven `for..in` key (`for_in_key_shape`), the
    /// base `obj` is a known object whose shape MATCHES the key's shape, and
    /// that shape is uniform-repr. Used by the assignment dispatch to ADMIT a
    /// compound-assign to such a target (`obj[c] += v`) — codegen decomposes it
    /// to `obj[c] = (obj[c] op v)`, routing both the read and the write through
    /// Task 3's dynamic slot lane. Anything the gate would reject (shape
    /// mismatch, mixed-repr, non-key index, non-object base) is NOT admitted
    /// here and falls through to the fail-closed compound-assign rejection.
    pub(crate) fn forin_key_member_target_is_uniform(&self, member: &MemberExpression) -> bool {
        let Some(Expression::Identifier(key)) = member.computed_index.as_deref() else {
            return false;
        };
        let Some(key_shape) = self.for_in_key_shape(key) else {
            return false;
        };
        let Some(obj_shape) = self.object_shape_of_expression(&member.object) else {
            return false;
        };
        obj_shape == key_shape
            && matches!(
                self.repr_table.shape_is_uniform_repr(obj_shape),
                Some(kali_common::Repr::I64) | Some(kali_common::Repr::F64)
            )
    }

    /// `Some(shape)` iff `expr` is a bare identifier whose `ReprTable` scalar
    /// is proven `Repr::Object(shape)` — used to derive the shape a
    /// `for..in`'s `right` enumerates (Spec 4a Task 2). Reuses the same
    /// `binding_repr_function_key` scope-walk every other repr-table query in
    /// this module keys off of, so this never disagrees with codegen's
    /// per-function `ReprTable` entry. Fail-closed: anything other than a
    /// bare identifier, or an untracked-function-boundary binding, is `None`.
    pub(crate) fn object_shape_of_expression(
        &self,
        expr: &Expression,
    ) -> Option<kali_common::ShapeId> {
        use kali_common::Repr;
        let Expression::Identifier(name) = expr else {
            return None;
        };
        let func = self.binding_repr_function_key(name)?;
        match self.repr_table.scalar(&func, name) {
            Repr::Object(shape) => Some(shape),
            _ => None,
        }
    }

    /// True when `expr` is a DECLARATOR init that codegen registers as a runtime
    /// array binding: `new Array(...)`, the bare `Array(...)` call form (both
    /// funnel through codegen's `resolve_array_alloc_call`), or a `.fill(...)`
    /// over such an allocation / an already-structural array binding (codegen's
    /// `resolve_array_fill_call`, control_flow.rs). NARROWER than
    /// `rhs_is_array_shape`: it deliberately EXCLUDES the bare-identifier copy
    /// (`const c = a`), which codegen's declarator path does NOT register (only
    /// the `=` reassignment arm registers an identifier copy).
    pub(crate) fn declarator_registers_runtime_array(&self, expr: &Expression) -> bool {
        match expr {
            // `new Array(n)`: callee is the `CallExpression` `Array(n)` (see
            // `rhs_is_array_shape`); bare `new Array` is the Identifier form.
            Expression::NewExpression(new_expr) => {
                matches!(&new_expr.callee, Expression::Identifier(name) if name == "Array")
                    || matches!(&new_expr.callee, Expression::CallExpression(call)
                        if matches!(&call.callee, Expression::Identifier(name) if name == "Array"))
            }
            Expression::CallExpression(call) => {
                if matches!(&call.callee, Expression::Identifier(name) if name == "Array") {
                    return true;
                }
                // `<recv>.fill(v)` — recv is a fresh `new Array(n)`/`Array(n)`
                // allocation or an already-structural runtime array binding.
                if let Expression::MemberExpression(member) = &call.callee {
                    if member.computed_index.is_none() && member.property.as_str() == "fill" {
                        return self.declarator_registers_runtime_array(&member.object)
                            || matches!(&member.object, Expression::Identifier(name)
                                if self.is_structural_runtime_array(name));
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Register `name` as a structural runtime array binding in the scope where
    /// it is declared (module/global fallback otherwise). Grow-only, mirroring
    /// codegen's insert-only `array_bindings`. Scope-walk twin of
    /// `mark_binding_string_typed`.
    pub(crate) fn register_runtime_array_binding(&mut self, name: &str) {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            if let Some(scope) = self.scopes.get_mut(&scope_id) {
                if scope.bindings.contains_key(name) {
                    scope.runtime_array_bindings.insert(name.to_string(), true);
                    return;
                }
                current = scope.parent;
            } else {
                return;
            }
        }
        if self.global_scope.bindings.contains_key(name) {
            self.global_scope
                .runtime_array_bindings
                .insert(name.to_string(), true);
        }
    }

    /// Register `name` as a `for..in` key binding over `shape` in the scope
    /// where it is declared (module/global fallback otherwise). Grow-only,
    /// mirroring `register_runtime_array_binding` and codegen's insert-only
    /// registries — Spec 4a Task 2's dormant provenance registry.
    pub(crate) fn register_for_in_key(&mut self, name: &str, shape: kali_common::ShapeId) {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            if let Some(scope) = self.scopes.get_mut(&scope_id) {
                if scope.bindings.contains_key(name) {
                    scope.for_in_key_bindings.insert(name.to_string(), shape);
                    return;
                }
                current = scope.parent;
            } else {
                return;
            }
        }
        if self.global_scope.bindings.contains_key(name) {
            self.global_scope
                .for_in_key_bindings
                .insert(name.to_string(), shape);
        }
    }

    /// Spec 4a Task 5 fail-closed: mark `name` as holding a COPY of a `for..in`
    /// key VALUE (a declarator-init alias `let d = c`, or a chain thereof), for
    /// the value-escape reject gate only. Mirrors `register_for_in_key`'s
    /// declaring-scope walk. NOT registered in `for_in_key_bindings` (see the
    /// `Scope` field doc): this must never admit `table[d]` into the key index
    /// lane that codegen cannot lower for a declarator alias.
    pub(crate) fn register_for_in_key_value(&mut self, name: &str) {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            if let Some(scope) = self.scopes.get_mut(&scope_id) {
                if scope.bindings.contains_key(name) {
                    scope
                        .for_in_key_value_bindings
                        .insert(name.to_string(), true);
                    return;
                }
                current = scope.parent;
            } else {
                return;
            }
        }
        if self.global_scope.bindings.contains_key(name) {
            self.global_scope
                .for_in_key_value_bindings
                .insert(name.to_string(), true);
        }
    }

    /// True iff `name` carries `for..in`-key VALUE provenance — either a full
    /// for-in key/alias (`for_in_key_shape`) or a declarator-init value copy
    /// (`register_for_in_key_value`). Same tracked-function-boundary scope walk
    /// as `for_in_key_shape` (fail-closed across an untracked function boundary).
    pub(crate) fn is_for_in_key_value(&self, name: &str) -> bool {
        if self.for_in_key_shape(name).is_some() {
            return true;
        }
        let tracked_scope = self.current_function_scope();
        let mut current = self.current_scope_id();
        loop {
            let Some(scope_id) = current else {
                return self
                    .global_scope
                    .for_in_key_value_bindings
                    .contains_key(name);
            };
            let Some(scope) = self.scopes.get(&scope_id) else {
                return false;
            };
            if scope.scope_type == ScopeType::Function && Some(scope_id) != tracked_scope {
                return false;
            }
            if scope.for_in_key_value_bindings.contains_key(name) {
                return true;
            }
            if scope.scope_type == ScopeType::Function {
                return false;
            }
            current = scope.parent;
        }
    }

    /// True iff `name` resolves (respecting shadowing) to a `for..in`-key VALUE
    /// binding ANYWHERE in the enclosing scope chain — INCLUDING across a function
    /// boundary (a CLOSURE CAPTURE, `let g = () => c`). Unlike `is_for_in_key_value`
    /// (which fail-closes AT the tracked-function boundary, so a capture reads as
    /// `false`), this walks to the FIRST scope that binds `name` and reports
    /// whether THAT scope registered it as a key/alias/value-copy — so a nested
    /// function's OWN binding named `c` (shadowing) is correctly NOT a key, while a
    /// captured outer key IS. The default-deny reject in `resolve_identifier` keys
    /// on this so a captured for-in-key value read rejects (codegen would leak the
    /// raw ordinal into the closure) rather than escaping every same-function gate.
    pub(crate) fn for_in_key_value_binding_in_chain(&self, name: &str) -> bool {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let Some(scope) = self.scopes.get(&scope_id) else {
                return false;
            };
            if scope.bindings.contains_key(name) {
                // First scope that binds `name` decides — respects shadowing.
                return scope.for_in_key_bindings.contains_key(name)
                    || scope.for_in_key_value_bindings.contains_key(name);
            }
            current = scope.parent;
        }
        self.global_scope.bindings.contains_key(name)
            && (self.global_scope.for_in_key_bindings.contains_key(name)
                || self
                    .global_scope
                    .for_in_key_value_bindings
                    .contains_key(name))
    }

    /// If `name` (a bare identifier) is used as an RHS of an alias copy — a
    /// declarator init `let d = <rhs>` or an assignment `d = <rhs>` — and `<rhs>`
    /// carries for-in-key value provenance, mark the LHS `d` as a value copy too,
    /// so a later string-escape use of `d` is caught by the reject gate. Only the
    /// bare-identifier RHS form propagates (an ordinal-domain copy); anything else
    /// is left alone.
    pub(crate) fn propagate_for_in_key_value(&mut self, lhs: &str, rhs: &Expression) {
        if let Expression::Identifier(rhs_name) = rhs {
            if self.is_for_in_key_value(rhs_name) {
                self.register_for_in_key_value(lhs);
            }
        }
    }

    /// Spec 4a Task 5 structural default-deny: resolve `expr` at a for-in-key
    /// PROVEN-SAFE position (an `if` truthiness test, an alias-copy RHS, or an
    /// assignment/declarator target). If `expr` is EXACTLY a bare for-in-key
    /// value identifier — `if (last)`, `x = c`, `let x = c` — suppress the
    /// `resolve_identifier` value-escape reject (the ordinal is the correct
    /// representation here). Anything more complex (`if (id(c))`, `x = id(c)`,
    /// `if (r < table[c])`) is resolved normally, so a nested value-escape of the
    /// key still rejects and a `table[c]` index still stays accepted (its index
    /// is never resolved as an expression). The suppression is scoped to exactly
    /// this one identifier read via save/restore.
    pub(crate) fn resolve_forin_key_safe_position(&mut self, expr: &Expression) {
        let bare_key = matches!(expr, Expression::Identifier(name)
            if self.is_for_in_key_value(name));
        let previous = self.suppress_forin_key_value_reject;
        if bare_key {
            self.suppress_forin_key_value_reject = true;
        }
        self.resolve_expression(expr);
        self.suppress_forin_key_value_reject = previous;
    }

    /// Spec 4a Task 5 allowlist: resolve a STATEMENT-position expression (value
    /// DISCARDED). If it is an alias-copy `x = <bare for-in-key>` — an identifier
    /// target assigned a bare key, the ordinal-domain copy that propagates key
    /// provenance to `x` — suppress the default-deny value-escape reject for its
    /// bare-key RHS. This is the ONLY position where an assignment's bare-key RHS
    /// is safe: when the assignment's VALUE ESCAPES (`return (x = c)`), the RHS is
    /// resolved WITHOUT this suppression and rejects. A member target
    /// (`obj[c] = last`, a store) or a complex RHS (`x = id(c)`, a nested escape)
    /// is NOT an alias-copy and resolves normally → rejects.
    pub(crate) fn resolve_statement_position_expression(&mut self, expr: &Expression) {
        let alias_copy = matches!(expr, Expression::AssignmentExpression(a)
            if matches!(a.operator, AssignmentOperator::Assign)
                && matches!(&a.left, Expression::Identifier(_))
                && matches!(&a.right, Expression::Identifier(name)
                    if self.is_for_in_key_value(name)));
        let previous = self.suppress_forin_key_value_reject;
        if alias_copy {
            self.suppress_forin_key_value_reject = true;
        }
        self.resolve_expression(expr);
        self.suppress_forin_key_value_reject = previous;
    }

    /// True iff `name` resolves to a linear-memory array binding whose element
    /// repr is proven `Repr::String` by the inference (Spec 3 store/read/join
    /// lane). Reuses the SAME function-key resolution as
    /// `identifier_repr_is_string` (via `binding_repr_function_key`) so the F1
    /// store gate, the read-side mirrors, and codegen's element oracles all key
    /// the same `ReprTable` entry and never disagree. Fail-closed: an
    /// untracked-function boundary (`None` key) reports false.
    pub(crate) fn string_element_array_binding(&self, name: &str) -> bool {
        use kali_common::Repr;
        match self.binding_repr_function_key(name) {
            Some(func) => {
                self.repr_table.is_array_binding(&func, name)
                    && self.repr_table.array_element(&func, name) == Repr::String
            }
            None => false,
        }
    }

    /// True when array binding `name`'s String element axis may hold non-ASCII
    /// text (a `.length` byte count would then disagree with JS's UTF-16 unit
    /// count). Same key resolution as `string_element_array_binding`; fail-closed
    /// (unknown key ⇒ assume non-ASCII ⇒ reject).
    pub(crate) fn array_element_non_ascii(&self, name: &str) -> bool {
        match self.binding_repr_function_key(name) {
            Some(func) => self.repr_table.is_array_element_non_ascii(&func, name),
            None => true,
        }
    }

    /// True when `operand`'s runtime representation is proven `Repr::String` by
    /// the repr inference — the SAME signal codegen's `is_string_valued` uses,
    /// so the gate and codegen never disagree. Covers a string-typed identifier
    /// (variable/param, via `identifier_repr_is_string`) and a call to a
    /// string-returning function.
    pub(crate) fn operand_repr_is_string(&self, operand: &Expression) -> bool {
        use kali_common::Repr;
        match operand {
            // Spec 4a Task 5: a for-in key materialized as a string is repr-lifted
            // to `String`, so it is already covered by `identifier_repr_is_string`
            // (the SAME solved-repr signal codegen consults) — no extra for-in-key
            // disjunct needed. A non-repr-lifted key stays false, mirroring codegen.
            Expression::Identifier(name) => self.identifier_repr_is_string(name),
            // Computed element read `a[i]` of a proven `Repr::String` array
            // (Spec 3) — same signal codegen's `is_string_valued`
            // `dynamic_array_read_base` arm consults.
            Expression::MemberExpression(member) if member.computed_index.is_some() => {
                matches!(&member.object, Expression::Identifier(base)
                    if self.string_element_array_binding(base))
            }
            Expression::CallExpression(call) => match &call.callee {
                Expression::Identifier(callee) => {
                    self.repr_table.return_repr(callee) == Repr::String
                }
                // Runtime `a.join(sep)` over a proven `Repr::String`-element
                // array binding produces a runtime string (Spec 3) — same
                // signal codegen's `is_string_valued` `runtime_join_call_parts`
                // arm consults.
                Expression::MemberExpression(member)
                    if member.computed_index.is_none() && member.property.as_str() == "join" =>
                {
                    matches!(&member.object, Expression::Identifier(base)
                        if self.string_element_array_binding(base))
                }
                _ => false,
            },
            // A ternary whose arm is a `Repr::String` identifier/call is itself
            // a runtime string (codegen's `is_string_valued` ternary arm agrees).
            // Fail-closed: keeps the `.length`/store gates classifying a ternary
            // of string-returning calls as a string even though such arms are not
            // `expression_is_string_typed` (which lacks a call arm).
            Expression::ConditionalExpression(cond) => {
                self.operand_repr_is_string(&cond.consequent)
                    || self.operand_repr_is_string(&cond.alternate)
            }
            Expression::ParenthesizedExpression(inner) => {
                self.operand_repr_is_string(&inner.expression)
            }
            _ => false,
        }
    }

    /// True when `name`'s string value may contain non-ASCII text. Checks BOTH
    /// the current-function and module scopes (over-approximate: either scope
    /// non-ASCII rejects — fail-closed against the scope-resolution ambiguity
    /// `identifier_repr_is_string` handles precisely for the String bit).
    fn identifier_string_may_be_non_ascii(&self, name: &str) -> bool {
        let func = self.current_function_name();
        self.repr_table.is_string_non_ascii(func, name)
            || self.repr_table.is_string_non_ascii("_start", name)
    }

    /// True when `expr` is proven an ASCII-only runtime string: `Repr::String`
    /// via the inference AND never reached by a non-ASCII seed. The receivers
    /// the substring/.length lanes accept. Fail-closed: unknown shapes are false.
    pub(crate) fn expression_repr_is_ascii_string(&self, expr: &Expression) -> bool {
        use kali_common::Repr;
        match expr {
            Expression::Identifier(name) => {
                self.identifier_repr_is_string(name)
                    && !self.identifier_string_may_be_non_ascii(name)
            }
            // Computed element read `a[i]` of a proven `Repr::String` array is
            // ASCII iff the element axis was never reached by a non-ASCII seed
            // (Spec 3). This is what lets `a[i].length`/`a[i].substring(...)`
            // through on ASCII element arrays and rejects them on non-ASCII ones.
            Expression::MemberExpression(member) if member.computed_index.is_some() => {
                matches!(&member.object, Expression::Identifier(base)
                    if self.string_element_array_binding(base)
                        && !self.array_element_non_ascii(base))
            }
            Expression::CallExpression(call) => match &call.callee {
                Expression::Identifier(callee) => {
                    self.repr_table.return_repr(callee) == Repr::String
                        && !self.repr_table.is_string_non_ascii_return(callee)
                }
                // A chained substring: ASCII iff ITS receiver is.
                Expression::MemberExpression(member)
                    if member.computed_index.is_none()
                        && member.property.as_str() == "substring" =>
                {
                    self.expression_repr_is_ascii_string(&member.object)
                }
                _ => false,
            },
            Expression::ParenthesizedExpression(inner) => {
                self.expression_repr_is_ascii_string(&inner.expression)
            }
            _ => false,
        }
    }

    /// True when `arg` is safe as a runtime substring bound: provably integer-
    /// repr at runtime. Float/string/unknown shapes reject (JS ToInteger on
    /// NaN/fractions is unimplemented). Fail-closed.
    pub(crate) fn expression_is_int_repr_bound(&self, arg: &Expression) -> bool {
        use kali_common::Repr;
        match arg {
            Expression::Literal(LiteralValue::Number(n)) => n.is_finite() && n.fract() == 0.0,
            Expression::Identifier(name) => {
                let func = self.current_function_name();
                self.repr_table.scalar(func, name) == Repr::I64
                    && self.repr_table.scalar("_start", name) == Repr::I64
            }
            Expression::BinaryExpression(binary)
                if matches!(binary.operator.as_str(), "+" | "-" | "*" | "%") =>
            {
                self.expression_is_int_repr_bound(&binary.left)
                    && self.expression_is_int_repr_bound(&binary.right)
            }
            Expression::UnaryExpression(unary) if unary.operator == "-" => {
                self.expression_is_int_repr_bound(&unary.argument)
            }
            Expression::ParenthesizedExpression(inner) => {
                self.expression_is_int_repr_bound(&inner.expression)
            }
            Expression::CallExpression(call) => match &call.callee {
                Expression::Identifier(callee) => self.repr_table.return_repr(callee) == Repr::I64,
                _ => false,
            },
            // `.length` is a legitimate int-typed substring bound whenever the
            // codegen `.length` lane would actually accept the receiver: a
            // static-foldable receiver (UTF-16-unit count, correct for any
            // literal) or an ASCII-provable runtime string (byte count, which
            // `reject_unprovable_string_length` already proves agrees with the
            // JS character count). Mirrors that gate's "allowed" condition so
            // this predicate does not fail-close a bound the `.length` access
            // itself is legal to read.
            Expression::MemberExpression(member)
                if member.computed_index.is_none() && member.property.as_str() == "length" =>
            {
                self.expression_is_length_fold_receiver(&member.object)
                    || self.expression_repr_is_ascii_string(&member.object)
            }
            _ => false,
        }
    }

    /// Rejects a `+` whose lowering codegen cannot perform correctly: any operand
    /// that is *string-typed* but is not a codegen-recognized structural string
    /// expression (i.e. a string-typed variable / dynamic value that codegen's
    /// `is_string_valued` will not see) AND not proven `Repr::String` by the
    /// repr inference (`operand_repr_is_string` — the same signal codegen's
    /// runtime identifier/call arms now consult, so an operand this predicate
    /// lets through is one codegen lowers correctly). For any other unsupported
    /// operand codegen either integer-adds two string handles or coerces a
    /// string handle through `int_to_string`, both of which silently produce
    /// garbage. Rejecting with a clear `E3200` diagnostic makes the outcome
    /// sound (a compile error instead of a wrong result) while leaving every
    /// literal-rooted concatenation (e.g. `"x" + 3`, `"P(" + n + ")"`) and every
    /// `Repr::String`-backed variable/param/return compiling and correct.
    fn reject_unsupported_string_variable_addition(&mut self, expr: &BinaryExpression) {
        if expr.operator != "+" || self.suppress_string_addition_rejection {
            return;
        }
        let operand_is_unsupported_string = |operand: &Expression| {
            self.expression_is_string_typed(operand)
                && !self.expression_is_codegen_string_valued(operand)
                && !self.operand_repr_is_string(operand)
        };
        if operand_is_unsupported_string(&expr.left) || operand_is_unsupported_string(&expr.right) {
            self.diagnostics.push(
                Diagnostic::error(
                    e3::TYPE_MISMATCH as u32,
                    "'+' with a string-typed variable operand is unavailable in the current direct-runtime path: only string concatenation rooted in a string or template literal (for example \"x\" + 3) is lowered to runtime concatenation; a variable that holds a string is not recognized and would be miscompiled".to_string(),
                )
                .with_suggestion(
                    "root the concatenation in a string literal (\"\" + value), build the string with literal-rooted concatenation, or use the later compatibility path",
                ),
            );
        }
    }

    /// Operand-position layer for `&&`/`||`: a PROVEN runtime-string operand
    /// (`operand_repr_is_string` — the same signal codegen's `is_string_valued`
    /// consults, narrower than `expression_is_string_typed`) has no correct
    /// runtime lowering here. `&&`/`||` truthiness-test their left operand to
    /// pick a side, exactly the defect `reject_string_condition_expression`
    /// rejects for a ternary test — and unlike the ternary, `&&`/`||` can also
    /// YIELD the proven-string operand itself into a caller expression (a
    /// store, an object-literal value, a call argument): `1 && s` prints the
    /// int `1` instead of the string `s` (probed: silent-wrong on this
    /// branch). No correct runtime case exists for a proven string operand of
    /// `&&`/`||`, so reject outright rather than trying to characterize which
    /// downstream uses would be sound.
    fn reject_logical_operand_runtime_string(&mut self, expr: &BinaryExpression) {
        if !matches!(expr.operator.as_str(), "&&" | "||") {
            return;
        }
        if self.operand_repr_is_string(&expr.left) || self.operand_repr_is_string(&expr.right) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "a runtime string value is unavailable as an operand of '&&'/'||' in the current direct-runtime path; truthiness of a runtime string is not evaluated correctly and the operator may yield the string into an unsupported position".to_string(),
            ));
        }
    }

    /// Reject a string-typed expression used as a ternary condition (fail-closed).
    /// Uses the same string-typedness signal as the `+` gate
    /// (`expression_is_string_typed`), covering string literals/templates, `+`
    /// chains rooted in one, and string-typed variables.
    fn reject_string_condition_expression(&mut self, test: &Expression) {
        if self.expression_is_string_typed(test) {
            self.diagnostics.push(Diagnostic::error(
                e3::TYPE_MISMATCH as u32,
                "a string value is unavailable as a ternary condition in the current direct-runtime path; its truthiness is not evaluated".to_string(),
            ));
        }
    }

    /// True when `expr` is the SAME "compile-time constant" shape codegen's
    /// own `.length` fold lanes recognize: a direct string/template literal
    /// (or a `+`/wrapper chain rooted in one), or an IMMUTABLE (`const`)
    /// alias of such. `resolve_static_string_expression` alone is broader —
    /// its identifier arm resolves through `static_values`, which
    /// `resolve/mod.rs` populates for every non-`var` declarator (`let`
    /// included) — but codegen's fold-alias table (`self.bindings` in
    /// `kali_codegen`) only ever aliases `const` bindings. A `let` receiver
    /// that types-side static analysis can still compute (e.g. `let b = a +
    /// ""`, never reassigned in this snippet) is NOT what codegen treats as
    /// foldable: it materializes as a real runtime string handle, so it must
    /// still clear the ASCII-provable check below, not bypass it.
    fn expression_is_length_fold_receiver(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Identifier(name) => {
                if self.binding_is_mutable(name) {
                    return false;
                }
            }
            // Codegen never const-folds a ternary to a static string: it always
            // emits a runtime select (`emit_conditional`), materializing a real
            // runtime string handle. `resolve_static_string_expression` DOES fold
            // a ternary whose arms resolve to the same static string (e.g.
            // `c > 0 ? t : t`), so without this exclusion such a receiver would
            // wrongly take the fold escape and bypass the `.length`/store gates
            // (fail-OPEN). Reject the escape for any ternary; the gates then apply.
            Expression::ConditionalExpression(_) => return false,
            Expression::ParenthesizedExpression(inner) => {
                return self.expression_is_length_fold_receiver(&inner.expression);
            }
            _ => {}
        }
        self.resolve_static_string_expression(expr).is_some()
    }

    /// `.length` gate: a runtime string receiver must be ASCII-provable
    /// (handle len is a byte count; JS counts UTF-16 units — they agree only
    /// for ASCII). Static-foldable receivers stay on the base fold lane,
    /// which counts UTF-16 units and is correct for ANY literal.
    pub(crate) fn reject_unprovable_string_length(&mut self, expr: &MemberExpression) {
        if expr.computed_index.is_some() || expr.property.as_str() != "length" {
            return;
        }
        if self.expression_is_length_fold_receiver(&expr.object) {
            return;
        }
        let object_is_string = self.expression_is_string_typed(&expr.object)
            || self.operand_repr_is_string(&expr.object);
        if object_is_string && !self.expression_repr_is_ascii_string(&expr.object) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "'.length' on a runtime string value is unavailable unless the string is ASCII-provable in the current direct-runtime path; non-ASCII strings would report a byte count, not a JS character count".to_string(),
            ));
        }
    }

    /// True when `expr` produces a RUNTIME string value — one whose handle
    /// exists only at run time (concat results, string-typed vars/params,
    /// string-returning calls, substring slices). Statically-foldable strings
    /// return false: the const-fold lane (e.g. `const a = ["x","y"]` + static
    /// `join`) must stay green, and interned-literal stores keep base
    /// behavior. The F1 store gate keys on this.
    ///
    /// Uses `expression_is_length_fold_receiver` (not a raw
    /// `resolve_static_string_expression(expr).is_some()` check) for the
    /// static-fold escape: `resolve_static_string_expression`'s identifier arm
    /// resolves through `static_values`, which is populated for every
    /// non-`var` declarator INCLUDING `let` (see `resolve_variable_declaration`
    /// in `resolve/mod.rs`). A `let` binding whose current value happens to be
    /// statically known (e.g. `let t = x + "y"` folded from literals) would
    /// otherwise be misreported as "not runtime" here — but codegen's own
    /// fold-alias table only ever aliases `const` bindings, so such a `let`
    /// still materializes as a real runtime string handle and must still be
    /// gated. `expression_is_length_fold_receiver` already encodes this
    /// const-only distinction (Task 5), so reusing it keeps the two gates
    /// consistent.
    pub(crate) fn expression_is_runtime_string_value(&mut self, expr: &Expression) -> bool {
        // Spec 4a Task 5: a for-in key materialized as a string is repr-lifted to
        // `String`, so the `expression_is_string_typed(expr) || operand_repr_is_string
        // (expr)` check below (both now keyed on `identifier_repr_is_string`, the
        // SAME solved repr codegen's materialization guard reads) already covers a
        // seeded key — and a for-in key is never a fold receiver (mutable per
        // iteration), so the fold-receiver escape does not swallow it. An UNSEEDED
        // key stays false → codegen emits the raw ordinal → both sides agree
        // (fail-closed). No unconditional for-in-key arm here (that was the
        // value-flow fail-open: types must not admit a string where codegen
        // emits an ordinal).
        if self.expression_is_length_fold_receiver(expr) {
            return false;
        }
        if self.expression_is_string_typed(expr) || self.operand_repr_is_string(expr) {
            return true;
        }
        // A ternary is a runtime string iff EITHER arm is — recursing into THIS
        // predicate (not only into the string-typed/repr mirrors above, whose
        // ternary arms never reach the substring member-call fallthrough below)
        // so a ternary whose EVERY arm is a `.substring(...)` call still
        // classifies as a runtime string. Fail-closed mirror of codegen's
        // `is_string_valued` ternary arm, in lockstep with the other
        // ConditionalExpression arms above.
        if let Expression::ConditionalExpression(cond) = expr {
            return self.expression_is_runtime_string_value(&cond.consequent)
                || self.expression_is_runtime_string_value(&cond.alternate);
        }
        // `&&`/`||`/`??` yield one of their two operands at runtime (whichever
        // the short-circuit picks) — a runtime string iff EITHER operand is,
        // same self-recursing shape as the ternary arm above. The parser has
        // NO `LogicalExpression` production (verified Task 7): `a && b` / `a
        // || b` / `a ?? b` all lower to `BinaryExpression` with that operator
        // string, so this arm keys on `BinaryExpression`, not the (dead)
        // `LogicalExpression` AST variant.
        if let Expression::BinaryExpression(binary) = expr {
            if matches!(binary.operator.as_str(), "&&" | "||" | "??") {
                return self.expression_is_runtime_string_value(&binary.left)
                    || self.expression_is_runtime_string_value(&binary.right);
            }
        }
        if let Expression::CallExpression(call) = expr {
            if let Expression::MemberExpression(member) = &call.callee {
                // Runtime `a.join(sep)` over a proven `Repr::String`-element
                // array binding is a runtime string producer (a fresh buffer),
                // alongside the substring fallthrough. Both mirror codegen's
                // `is_string_valued`. Non-identifier receivers fall through to
                // the substring check and then to `false` (fail-closed).
                if member.computed_index.is_none() && member.property.as_str() == "join" {
                    return matches!(&member.object, Expression::Identifier(base)
                        if self.string_element_array_binding(base));
                }
                return member.computed_index.is_none() && member.property.as_str() == "substring";
            }
        }
        // Computed element read `a[i]` of a proven `Repr::String` array is a
        // runtime string value (Spec 3). Subsumed by the `expression_is_string_typed`
        // check above, but kept explicit so this predicate — the one the F1 store
        // gate and the array-literal element gate consult directly — recognizes
        // the shape on its own, in lockstep with the other read-side mirrors.
        if let Expression::MemberExpression(member) = expr {
            if member.computed_index.is_some() {
                if let Expression::Identifier(base) = &member.object {
                    return self.string_element_array_binding(base);
                }
            }
        }
        false
    }

    /// F1: reject storing a runtime string into an array element or object
    /// field. Element/field reads are int-lane (per-edge string-axis
    /// exclusion, Spec 1) — a stored runtime string would read back as a raw
    /// number or compare by meaningless handle identity.
    pub(crate) fn reject_runtime_string_store(&mut self, assign: &AssignmentExpression) {
        let Expression::MemberExpression(member) = &assign.left else {
            return;
        };
        if !self.expression_is_runtime_string_value(&assign.right) {
            return;
        }
        // Spec 3 lane: element stores into arrays with proven String elements
        // are supported — the read side, the oracle arms, and mixed arrays'
        // shape conflicts (repr_infer emits E5506 for a string+number element
        // mix) make this sound. The subscript-target shape MIRRORS the `a[i] = v`
        // recognizer in `repr_infer::visit_assignment` (computed index over a
        // bare-identifier base). Fields and every unproven target keep rejecting.
        // The accept path also requires STRUCTURAL registration (C1): a
        // repr-proven String-element array that codegen never registers in
        // this function's `array_bindings` (a call-result capture
        // `const c = mk(); c[0] = ...`) has no element-store lowering — codegen
        // falls through and the read silently yields `0`. Require
        // `is_structural_runtime_array` so such a target rejects (fail-closed).
        if member.computed_index.is_some() {
            if let Expression::Identifier(base_name) = &member.object {
                // Row 3 (Spec 4a Task 6): a computed store whose base is a
                // KNOWN fixed-shape object is a `for..in`-key FIELD store, NOT
                // a Spec-3 string-element array store. The store wires the
                // value into an array-element node (visit_assignment treats
                // `base[i] = v` as an element store), so `string_element_array_binding`
                // reports true here — but the base is an OBJECT, so that accept
                // path must NOT fire. Storing a runtime string into a numeric
                // object field is out of scope; fall through to reject.
                if self.object_shape_of_expression(&member.object).is_none()
                    && self.string_element_array_binding(base_name)
                    && self.is_structural_runtime_array(base_name)
                {
                    return;
                }
            }
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "storing a runtime string value into this element or field is unavailable in the current direct-runtime path unless the target is an array whose elements are all proven strings; use the later compatibility path".to_string(),
        ));
    }

    /// Literal-array mutation gate: a computed subscript STORE whose base
    /// resolves to a static array-LITERAL binding (`resolve_array_literal_binding_name`,
    /// the Task 7 registry backing `is_static_array_iteration_target`) has no
    /// correct runtime lowering unless the WHOLE access folds statically.
    /// Codegen never linearizes a literal array into a mutable runtime buffer
    /// (the `join`-lane doc comment on `resolve_array_literal_binding_name`
    /// notes the same absence for reads); probing on this branch: a runtime
    /// index (`a[k] = 42` for a parameter `k`) and a STATIC index inside a
    /// named function (`a[1] = 42` inside `function h() {...}`) both compile
    /// and silently print a stale/wrong value instead of the stored one.
    /// Rejects when EITHER:
    ///   (a) the index is not a static numeric literal (no fold target at
    ///       all — mirrors the slice/join gates' `is_static_numeric_literal_expr`
    ///       foldability check), OR
    ///   (b) the store executes inside a named function
    ///       (`current_function_name() != "_start"`) — even a static index
    ///       there is not the SAME top-level fold lane that resolves a
    ///       top-level `var a = [...]; a[1] = 42;` (probed: that top-level
    ///       shape's behavior is unchanged by this gate, silent-wrong residual
    ///       or not — out of scope here, no new green lane).
    /// Same dispatch site as `reject_runtime_string_store` (any assignment
    /// operator; a compound `a[1] += 1` on a literal array is exactly as
    /// unsupported as `a[1] = 42`). `new Array(n)` bindings
    /// (`rhs_is_array_shape`/`string_element_array_binding` lanes) are
    /// untouched — this gate only keys on the `ArrayExpression`-literal
    /// binding shape, a disjoint registry.
    pub(crate) fn reject_literal_array_unfoldable_mutation(
        &mut self,
        assign: &AssignmentExpression,
    ) {
        let Expression::MemberExpression(member) = &assign.left else {
            return;
        };
        let Some(index) = member.computed_index.as_deref() else {
            return;
        };
        let Expression::Identifier(base_name) = &member.object else {
            return;
        };
        if !self.resolve_array_literal_binding_name(base_name) {
            return;
        }
        let index_is_foldable = self.is_static_numeric_literal_expr(index);
        let in_named_function = self.current_function_name() != "_start";
        if !index_is_foldable || in_named_function {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "mutating a literal array is unavailable in the current direct-runtime path unless the whole access folds statically; use new Array(n) for runtime mutation".to_string(),
            ));
        }
    }

    /// True when `expr` is one of the array-producing reassignment shapes
    /// codegen's `"="` arm (Task 5, literal.rs) actually routes through the
    /// allocation/copy path: `new Array(...)`, the bare `Array(...)` call form
    /// (both funnel through codegen's `resolve_array_alloc_call`, which does
    /// not distinguish `new` from a plain call once lowered to LIR), or a bare
    /// identifier that is itself a STRUCTURAL runtime array binding (`a = b`,
    /// copied via `bare_identifier_name` — codegen's `=` arm only registers the
    /// copy when `b` is already in `array_bindings`).
    ///
    /// Deliberately narrower than `repr_infer::init_is_array`'s shape list
    /// (repr_infer.rs:712-720), which ALSO accepts `Expression::ArrayExpression`
    /// — that fn only feeds the ANALYSIS-side element-axis merge (Task 2), not
    /// a codegen guarantee. Probed on this branch: codegen's `"="` arm has NO
    /// routing for an array-literal RHS (`emit_aggregate_literal`'s non-object
    /// branch is a side-effect-only stub that pushes a bogus `I64Const(0)`
    /// handle), so `a = [1, 2]` on an array binding would silently clobber the
    /// base handle with 0 — the exact miscompile this gate exists to prevent.
    /// Accepting `ArrayExpression` here would fail OPEN. If codegen ever grows
    /// real array-literal-reassignment routing, widen this arm to match.
    ///
    /// The Identifier arm keys on the STRUCTURAL registry (C1), not the
    /// repr-table `is_array_binding` proof: repr_infer proves array-ness for a
    /// LITERAL-array binding (`let b = [7, 8]`) too, but codegen cannot copy
    /// one (it never linearizes a literal array into a runtime buffer), so
    /// `a = b` there silently clobbers `a`'s handle with `0` (I1). Requiring
    /// `is_structural_runtime_array` rejects that copy source (fail-closed).
    fn rhs_is_array_shape(&self, expr: &Expression) -> bool {
        match expr {
            // `new Array(n)` parses as a `NewExpression` whose callee is the
            // `CallExpression` `Array(n)` (the parser attaches the args to the
            // inner call, leaving `NewExpression.args` empty) — the second
            // `matches!` is the LOAD-BEARING one for the common form; the bare
            // `new Array` (no args) takes the Identifier form. (T5-Minor-1
            // claimed the CallExpression-callee sub-match was unreachable and
            // asked to delete it; empirically it is the normal shape for
            // `new Array(n)` — deleting it makes `new Array(n)` unrecognized
            // and regresses reassignment + declarator registration, so it is
            // deliberately KEPT. See Final-review fix round notes.)
            Expression::NewExpression(new_expr) => {
                matches!(&new_expr.callee, Expression::Identifier(name) if name == "Array")
                    || matches!(&new_expr.callee, Expression::CallExpression(call)
                        if matches!(&call.callee, Expression::Identifier(name) if name == "Array"))
            }
            Expression::CallExpression(call) => {
                matches!(&call.callee, Expression::Identifier(name) if name == "Array")
            }
            Expression::Identifier(name) => self.is_structural_runtime_array(name),
            _ => false,
        }
    }

    /// True when `expr` is an array *literal* or a binding that resolves to one
    /// (`[1, 2]`, or `b` where `let b = [1, 2]`). Used only to give a
    /// reassignment-to-an-array-literal a message that names the actual defect
    /// (codegen has no runtime lowering for a literal-array RHS) instead of the
    /// misleading "non-array value".
    fn rhs_is_literal_array_shape(&self, expr: &Expression) -> bool {
        match expr {
            Expression::ArrayExpression(_) => true,
            Expression::Identifier(name) => self.resolve_array_literal_binding_name(name),
            Expression::ParenthesizedExpression(inner) => {
                self.rhs_is_literal_array_shape(&inner.expression)
            }
            _ => false,
        }
    }

    /// `a = 5` where `a` is an array binding would clobber the base handle
    /// with an integer — later element reads would dereference address 5.
    /// Fail closed. Array-alloc and array-identifier RHS (`rhs_is_array_shape`)
    /// are the supported reassignment shapes; every compound operator (`+=`
    /// etc.) on an array-binding target rejects outright, since none of them
    /// are a supported reassignment shape.
    ///
    /// Scoped to genuinely linear-memory-backed targets ONLY:
    /// `repr_table.is_array_binding` is broader than codegen's runtime array
    /// lane — it also covers compile-time literal arrays consumed by the
    /// static/for-of iteration lane (`static_analysis::array`'s
    /// `is_static_array_iteration_target`/`static_arrays`), which codegen
    /// never backs with a linear-memory handle at all, so reassigning one has
    /// no handle to clobber. `resolve_static_array_binding_name` is that
    /// lane's OWN "is this binding a proven static/literal array" query;
    /// excluding it here is what keeps this gate from firing on
    /// `let values = [1, 2]; values = [3, 4];` (a real, pre-existing,
    /// non-runtime-array reassignment the for-of static-iteration gate
    /// already polices on its own terms).
    pub(crate) fn reject_array_binding_scalar_reassignment(
        &mut self,
        assign: &AssignmentExpression,
    ) {
        let Expression::Identifier(target) = &assign.left else {
            return;
        };
        let Some(func) = self.binding_repr_function_key(target) else {
            return;
        };
        if !self.repr_table.is_array_binding(&func, target) {
            return;
        }
        if self.resolve_static_array_binding_name(target) {
            return;
        }
        if matches!(assign.operator, AssignmentOperator::Assign)
            && self.rhs_is_array_shape(&assign.right)
        {
            return;
        }
        // Distinguish an array-LITERAL / literal-binding RHS (`a = [1, 2]`,
        // `a = b` where `b` is a literal array) from a genuinely non-array
        // value: the former IS an array value, just one codegen cannot copy
        // into a runtime buffer, so the "non-array value" phrasing would
        // misdescribe it (T7-Minor-3).
        let message = if matches!(assign.operator, AssignmentOperator::Assign)
            && self.rhs_is_literal_array_shape(&assign.right)
        {
            "reassigning an array binding to an array literal is unavailable in the current direct-runtime path (codegen does not linearize a literal array into a runtime buffer); use new Array(n) for a runtime-mutable array".to_string()
        } else {
            "reassigning an array binding to a non-array value is unavailable in the current direct-runtime path".to_string()
        };
        self.diagnostics
            .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
    }

    /// Resolves `expression` in a position where codegen folds a string-typed `+`
    /// to a static string (a for-of iterable, a dynamic-import specifier). Such a
    /// `+` never reaches the buggy runtime `+` path, so the string-typed-variable
    /// rejection is suppressed for its duration.
    pub(crate) fn resolve_static_string_fold_position(&mut self, expression: &Expression) {
        let previous = self.suppress_string_addition_rejection;
        self.suppress_string_addition_rejection = true;
        self.resolve_expression(expression);
        self.suppress_string_addition_rejection = previous;
    }

    pub(crate) fn resolve_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Identifier(name) => self.resolve_identifier(name),
            Expression::Literal(_) => {}
            Expression::BinaryExpression(expr) => {
                self.resolve_expression(&expr.left);
                self.resolve_expression(&expr.right);
                self.reject_unsupported_string_variable_addition(expr);
                self.reject_logical_operand_runtime_string(expr);
                // `&&`/`||`/`??` parse to a BinaryExpression here (not a
                // LogicalExpression). A for-in-key alias operand of these is a
                // fail-closed reject (raw integer truthiness inverts the `-1`
                // null sentinel).
                if matches!(expr.operator.as_str(), "&&" | "||" | "??") {
                    self.reject_forin_key_boolean_operand(&expr.left, &expr.operator);
                    self.reject_forin_key_boolean_operand(&expr.right, &expr.operator);
                }
                // NOTE: a non-materialized for-in-key value as a `+`/equality (or
                // any other) operand is rejected structurally by the default-deny
                // in `resolve_identifier` — the operands are resolved as values
                // above. A materialized direct seeded key (`c + "!"`, `c == "a"`,
                // repr `String`) is NOT rejected and materializes correctly.
            }
            Expression::UnaryExpression(expr) => {
                if expr.operator == "delete" {
                    if let Expression::MemberExpression(member) = &expr.argument {
                        if self.resolve_late_process_env_mutation_member(member) {
                            return;
                        }
                    }
                }
                if expr.operator == "!" {
                    self.reject_forin_key_boolean_operand(&expr.argument, "!");
                }
                self.resolve_expression(&expr.argument)
            }
            Expression::CallExpression(expr) => self.resolve_call_expression(expr),
            Expression::MemberExpression(expr) => self.resolve_member_expression(expr),
            Expression::ArrayExpression(ArrayExpression { elements }) => {
                for element in elements.iter().flatten() {
                    match element {
                        ExpressionOrSpread::Expression(expr) => self.resolve_expression(expr),
                        ExpressionOrSpread::Spread(spread) => {
                            self.resolve_expression(&spread.argument)
                        }
                        ExpressionOrSpread::Empty => {}
                    }
                }
                for element in elements.iter().flatten() {
                    if let ExpressionOrSpread::Expression(element_expr) = element {
                        if self.expression_is_runtime_string_value(element_expr) {
                            self.diagnostics.push(Diagnostic::error(
                                e5::FEATURE_UNAVAILABLE as u32,
                                "a runtime string value is unavailable as an array element in the current direct-runtime path; element reads have no string lane yet".to_string(),
                            ));
                        }
                        // A non-materialized for-in-key value as an array element
                        // is a value escape — rejected structurally by the
                        // default-deny in `resolve_identifier` (elements are
                        // resolved as values above).
                    }
                }
            }
            Expression::ObjectExpression(ObjectExpression { properties }) => {
                for property in properties {
                    self.resolve_object_property(property);
                }
            }
            Expression::FunctionExpression(expr) => self.resolve_function_expression(expr),
            Expression::ArrowFunctionExpression(expr) => self.resolve_arrow_function(expr),
            Expression::ClassExpression(expr) => self.resolve_class_expression(expr),
            Expression::NewExpression(expr) => {
                self.resolve_expression(&expr.callee);
                for arg in &expr.args {
                    self.resolve_expression(arg);
                }
            }
            Expression::MetaProperty(_) => {}
            Expression::TemplateLiteral(template) => self.resolve_template_literal(template),
            Expression::TaggedTemplateExpression(expr) => {
                self.resolve_expression(&expr.tag);
                self.resolve_template_literal(&expr.template);
            }
            Expression::UpdateExpression(expr) => self.resolve_update_expression(expr),
            Expression::AssignmentExpression(expr) => {
                // Spec 4a Task 5 allowlist: the LHS is a WRITE TARGET (never a
                // value escape) — resolve via the safe-position path so a bare-key
                // target (`last = …`) is not mis-rejected. The RHS resolves
                // NORMALLY: a bare-key RHS is a safe ALIAS-COPY only in STATEMENT
                // position (value discarded) — the enclosing `ExpressionStatement`
                // / for-init / for-update sets `suppress_forin_key_value_reject`
                // for that case. A bare-key RHS whose assignment VALUE ESCAPES
                // (`return (x = c)`) is NOT suppressed → the key rejects.
                self.resolve_forin_key_safe_position(&expr.left);
                self.resolve_expression(&expr.right);

                self.reject_runtime_string_store(expr);
                self.reject_array_binding_scalar_reassignment(expr);
                self.reject_literal_array_unfoldable_mutation(expr);

                if self.resolve_late_env_assignment_mutation(expr) {
                    return;
                }

                if matches!(expr.operator, AssignmentOperator::Assign) {
                    if let Expression::MemberExpression(member) = &expr.left {
                        let dotted = Self::member_access_name(member)
                            .unwrap_or_else(|| member.property.clone());
                        if self.api_surface == "node"
                            && Self::is_process_env_mutation_path(&dotted)
                            && !Self::is_process_env_root_path(&dotted)
                        {
                            return;
                        }
                    }
                }

                if matches!(expr.operator, AssignmentOperator::Assign) {
                    if let Some(name) = self.resolve_update_binding_name(&expr.left) {
                        // Reassignment clears the previous static tracking, then
                        // re-establishes string-typedness from the new value so a
                        // later `+` on this binding is still recognized. When the
                        // right-hand side is provably non-string the flag stays
                        // cleared, keeping the check flow-aware (e.g.
                        // `let s = "x"; s = 5; s + 1` stays a valid numeric `6`).
                        let right_is_string = self.expression_is_string_typed(&expr.right);
                        // Structural runtime-array registry (C1): a
                        // reassignment whose RHS codegen backs as a runtime
                        // array (`new Array(n)` / `Array(n)` / a
                        // structural-identifier copy `a = b`) registers the
                        // target, mirroring codegen's `=` arm (literal.rs).
                        // Evaluated BEFORE `invalidate_static_binding` — the
                        // registry is grow-only and invalidation does not touch
                        // it, but the ordering keeps intent clear.
                        let right_is_array_shape = self.rhs_is_array_shape(&expr.right);
                        // Spec 4a Task 2: propagate `for..in` key provenance
                        // through a bare-identifier alias (`last = c`) —
                        // computed BEFORE `invalidate_static_binding` for the
                        // same reason as `right_is_array_shape` above (the
                        // registry itself is grow-only and untouched by
                        // invalidation; the ordering just keeps intent
                        // clear). Dormant: nothing reads this registry yet.
                        let right_for_in_key_shape = match &expr.right {
                            Expression::Identifier(rhs_name) => self.for_in_key_shape(rhs_name),
                            _ => None,
                        };
                        self.invalidate_static_binding(&name);
                        if right_is_string {
                            self.mark_binding_string_typed(&name);
                        }
                        if right_is_array_shape {
                            self.register_runtime_array_binding(&name);
                        }
                        if let Some(shape) = right_for_in_key_shape {
                            self.register_for_in_key(&name, shape);
                        }
                        // Spec 4a Task 5 fail-closed: propagate for-in-key VALUE
                        // provenance through an assignment alias whose RHS is a
                        // declarator-init value copy (`e = d` where `d` is
                        // `let d = c`) — the assignment-side sibling of the
                        // declarator-init taint, for the escape reject gate only.
                        self.propagate_for_in_key_value(&name, &expr.right);
                    }
                    return;
                }

                // Spec 4a Task 4: compound-assign to a computed for-in-key
                // object target `obj[c] += v` over a uniform-repr fixed shape is
                // ADMITTED — codegen decomposes it to `obj[c] = (obj[c] op v)`,
                // routing both the read of `obj[c]` and the write through Task
                // 3's dynamic slot lane. The accept condition is exactly the one
                // `reject_nonuniform_forin_key_object_access` uses for `obj[c] =
                // v` (base shape proven + shape-matched + uniform-repr + key
                // index). A non-for-in-key or non-uniform target is NOT admitted
                // here and still rejects fail-closed below.
                if !matches!(expr.operator, AssignmentOperator::NullishAssign) {
                    if let Expression::MemberExpression(member) = &expr.left {
                        if self.forin_key_member_target_is_uniform(member) {
                            return;
                        }
                    }
                }

                let Some(name) = self.resolve_update_binding_name(&expr.left) else {
                    let message = if matches!(expr.operator, AssignmentOperator::NullishAssign) {
                        "nullish assignment lowering is unavailable unless the target is a mutable local binding; use a mutable variable or the later compatibility path".to_string()
                    } else {
                        "compound assignment lowering is unavailable unless the target is a mutable local binding; use a mutable variable or the later compatibility path".to_string()
                    };
                    self.diagnostics
                        .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
                    return;
                };

                self.invalidate_static_binding(&name);

                if !self.binding_is_mutable(&name) {
                    let message = if matches!(expr.operator, AssignmentOperator::NullishAssign) {
                        format!(
                            "nullish assignment lowering is unavailable for binding '{}' unless it was declared with a mutable binding kind; use a mutable variable or the later compatibility path",
                            name
                        )
                    } else {
                        format!(
                            "compound assignment lowering is unavailable for binding '{}' unless it was declared with a mutable binding kind; use a mutable variable or the later compatibility path",
                            name
                        )
                    };
                    self.diagnostics
                        .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
                }
            }
            Expression::LogicalExpression(expr) => {
                self.resolve_expression(&expr.left);
                self.resolve_expression(&expr.right);
                let op_symbol = match expr.operator {
                    LogicalOperator::And => "&&",
                    LogicalOperator::Or => "||",
                    LogicalOperator::Coalesce => "??",
                };
                self.reject_forin_key_boolean_operand(&expr.left, op_symbol);
                self.reject_forin_key_boolean_operand(&expr.right, op_symbol);
            }
            Expression::ConditionalExpression(expr) => {
                self.resolve_expression(&expr.test);
                self.resolve_expression(&expr.consequent);
                self.resolve_expression(&expr.alternate);
                // A string-typed ternary TEST cannot be truthiness-tested here:
                // the conditional lowering is degenerate (it yields the test
                // value itself, ignoring the branches), so a string test would
                // print/return the raw string instead of selecting a branch.
                // Reject fail-closed. No base-correct string ternary exists (the
                // degenerate lowering was always wrong for a string test).
                self.reject_string_condition_expression(&expr.test);
                // H2: a for-in-key/alias test in a ternary lowers via default
                // `!= 0` truthiness (the `>= 0` sentinel path is `if`-only) —
                // reject fail-closed.
                self.reject_forin_key_test_operand(&expr.test, "ternary condition");
            }
            Expression::SequenceExpression(expr) => {
                for subexpr in &expr.expressions {
                    self.resolve_expression(subexpr);
                }
            }
            Expression::ParenthesizedExpression(expr) => self.resolve_expression(&expr.expression),
            Expression::YieldExpression(expr) => {
                if self.in_generator_function && expr.delegate {
                    self.has_generator_yield_delegation = true;
                }
                if !self.in_generator_function {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        generator_function_yield_lowering_unavailable_message(false, expr.delegate),
                    ));
                }
                if let Some(argument) = &expr.argument {
                    self.resolve_expression(argument);
                }
            }
            Expression::AwaitExpression(expr) => self.resolve_expression(&expr.argument),
            Expression::OptionalChainExpression(expr) => self.resolve_optional_chain(expr),
            Expression::ChainExpression(expr) => self.resolve_expression(&expr.expression),
            Expression::SpreadElement(expr) => self.resolve_expression(&expr.argument),
            Expression::RestElement(expr) => self.resolve_expression(&expr.argument),
            Expression::ImportExpression(expr) => self.resolve_import_expression(expr),
            Expression::DecoratedExpression(DecoratedExpression { expression }) => {
                self.resolve_expression(expression)
            }
            Expression::JsxElement(expr) => self.resolve_jsx_element(expr),
            Expression::JsxFragment(expr) => self.resolve_jsx_fragment(expr),
            Expression::JsxEmptyExpression => {}
            Expression::TypeAssertion(expr) => self.resolve_type_assertion(expr),
            Expression::SatisfiesExpression(expr) => self.resolve_satisfies_expression(expr),
            Expression::ThisExpression | Expression::SuperExpression => {}
            Expression::PrivateIdentifier(_) | Expression::BigIntLiteral(_) => {}
        }
    }

    pub(crate) fn resolve_update_expression(&mut self, expr: &UpdateExpression) {
        self.resolve_expression(&expr.argument);

        if !self.is_simple_update_target_expression(&expr.argument) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "update expression lowering is unavailable unless the target is a mutable local binding; use a local binding or the later compatibility path",
            ));
            return;
        }

        let Some(name) = self.resolve_update_binding_name(&expr.argument) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "update expression lowering is unavailable unless the target is a mutable local binding; use a local binding or the later compatibility path",
            ));
            return;
        };

        self.invalidate_static_binding(&name);

        if !self.binding_is_mutable(&name) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "update expression lowering is unavailable for binding '{}' unless it was declared with a mutable binding kind; use a mutable local binding or the later compatibility path",
                    name
                ),
            ));
        }
    }

    pub(crate) fn invalidate_static_binding(&mut self, name: &str) {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            if let Some(scope) = self.scopes.get_mut(&scope_id) {
                if scope.bindings.contains_key(name) {
                    scope.invalidate_static_binding(name);
                    return;
                }
                current = scope.parent;
            } else {
                return;
            }
        }

        if self.global_scope.bindings.contains_key(name) {
            self.global_scope.invalidate_static_binding(name);
        }
    }

    /// Records that the binding `name` currently holds a string value, in the
    /// scope where `name` is declared. Used after an assignment whose right-hand
    /// side is string-typed so that a later `+` on `name` is recognized as an
    /// unsupported string-typed-variable operand (see
    /// `reject_unsupported_string_variable_addition`).
    pub(crate) fn mark_binding_string_typed(&mut self, name: &str) {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            if let Some(scope) = self.scopes.get_mut(&scope_id) {
                if scope.bindings.contains_key(name) {
                    scope.static_string_typed.insert(name.to_string(), true);
                    return;
                }
                current = scope.parent;
            } else {
                return;
            }
        }

        if self.global_scope.bindings.contains_key(name) {
            self.global_scope
                .static_string_typed
                .insert(name.to_string(), true);
        }
    }

    pub(crate) fn binding_is_mutable(&self, name: &str) -> bool {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id).expect("scope exists");
            if scope.bindings.contains_key(name) {
                return scope.mutable_bindings.get(name).copied().unwrap_or(false);
            }
            current = scope.parent;
        }

        self.global_scope.bindings.contains_key(name)
            && self
                .global_scope
                .mutable_bindings
                .get(name)
                .copied()
                .unwrap_or(false)
    }

    pub(crate) fn resolve_import_expression(&mut self, expr: &ImportExpression) {
        self.resolve_static_string_fold_position(&expr.source);

        if let Some(source) = self.resolve_static_import_source(&expr.source) {
            match self.resolve_import_source(&source) {
                Ok(true) => {}
                Ok(false) => {
                    self.diagnostics.push(
                        Diagnostic::error(
                            e4::DYNAMIC_IMPORT_NOT_IN_LINKED_GRAPH as u32,
                            format!(
                                "dynamic import target '{}' could not be resolved in the linked graph",
                                source
                            ),
                        )
                        .with_suggestion(
                            "use a statically known import specifier or link the module in the build graph",
                        ),
                    );
                }
                Err(diagnostic) => self.diagnostics.push(diagnostic),
            }
        } else {
            self.diagnostics.push(
                Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "non-literal dynamic import() is unavailable in the current phase; use a statically known import specifier that can be resolved in the linked graph".to_string(),
                )
                .with_suggestion(
                    "rewrite the import() target so the compiler can determine a linked-graph module at compile time",
                ),
            );
        }
    }

    pub(crate) fn resolve_static_import_source(&self, expression: &Expression) -> Option<String> {
        self.resolve_static_string_expression(expression)
    }

    pub(crate) fn normalize_import_segment(value: &str) -> String {
        let trimmed = value.trim();
        if trimmed.len() >= 2 {
            let mut chars = trimmed.chars();
            let first = chars.next().unwrap();
            let last = chars.next_back().unwrap();
            if matches!((first, last), ('"', '"') | ('\'', '\'') | ('`', '`')) {
                return trimmed[1..trimmed.len() - 1].to_string();
            }
        }
        trimmed.to_string()
    }

    pub(crate) fn resolve_identifier(&mut self, name: &str) {
        if matches!(name, "unknown" | "undefined") {
            return;
        }

        // Spec 4a Task 5 STRUCTURAL DEFAULT-DENY (allowlist): a for-in-key VALUE
        // read (`is_for_in_key_value` — direct key / assignment alias / declarator
        // value-copy) whose repr was NOT lifted to `String` (so codegen emits the
        // raw ORDINAL, never a materialized string handle) is REJECTED in EVERY
        // value position — EXCEPT the four proven-safe positions where the ordinal
        // is the correct representation, each recognized WITHOUT reaching this
        // value-read path: (1) a COMPUTED INDEX `obj[key]` — the index is never
        // resolved as an expression, so it never arrives here; (2) an `if`
        // TRUTHINESS test and (3) an ALIAS-COPY RHS/target — resolved via
        // `resolve_forin_key_safe_position`, which sets `suppress_...`; (4) a
        // MATERIALIZED direct seeded key — excluded by `!identifier_repr_is_string`
        // (its repr IS `String`). By construction no un-enumerated value position
        // can leak the ordinal — the inverse of a sink denylist.
        if !self.suppress_forin_key_value_reject
            && self.for_in_key_value_binding_in_chain(name)
            && !self.identifier_repr_is_string(name)
        {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "a for..in key value ('{name}') is only usable as a computed index (`obj[{name}]`), an `if` condition, an alias copy to another key binding, or a MATERIALIZED direct key at a return / console.log / `+` / `==` position; every other value use is unavailable in the current direct-runtime path (it would leak the raw ordinal, not the field-name string)"
                ),
            ));
            return;
        }

        if matches!(name, "SharedArrayBuffer" | "Atomics") {
            if self.has_threaded_runtime_profile() {
                return;
            }
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "threaded runtime global '{}' is unavailable until the WASM-threaded profile is enabled",
                    name
                ),
            ));
            return;
        }

        if name == "Intl" {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "broader Intl support is unavailable until the later web/Intl compatibility path is enabled".to_string(),
            ));
            return;
        }

        if matches!(
            name,
            "Proxy" | "WeakMap" | "WeakSet" | "WeakRef" | "FinalizationRegistry"
        ) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "late object-model API '{}' is unavailable until the later object-model compatibility path is enabled",
                    name
                ),
            ));
            return;
        }

        if self.resolve_name(name).is_none() {
            self.diagnostics.push(
                Diagnostic::error(
                    e3::UNDEFINED_IDENTIFIER as u32,
                    format!("undefined identifier '{}'", name),
                )
                .with_suggestion("declare the name in the current module or import it"),
            );
        }
    }

    pub(crate) fn resolve_optional_chain(&mut self, expr: &OptionalChainExpression) {
        match expr.inner.as_ref() {
            OptionalChainInner::NonNull { object, .. } => self.resolve_expression(object),
        }
    }

    pub(crate) fn resolve_template_literal(&mut self, template: &TemplateLiteral) {
        for expr in &template.expressions {
            // A for-in-key value in a template interpolation `${c}` is a value
            // escape — rejected structurally by the default-deny in
            // `resolve_identifier` (interpolations are resolved as values). Real
            // templates usually desugar to `+` chains before this pass; a direct
            // SEEDED key there materializes (repr `String`), an un-seeded one
            // rejects.
            self.resolve_expression(expr);
        }
    }

    pub(crate) fn resolve_object_property(&mut self, property: &ObjectProperty) {
        self.resolve_property_name(&property.key);
        self.resolve_expression(&property.value);
        // Mirrors the array-literal element gate (`ArrayExpression` arm above):
        // an object-literal property VALUE stored at construction has no
        // string lane in codegen's aggregate-literal emission (Task 8) — only
        // `init` properties are actual stored values; `get`/`set` properties'
        // "value" is a function expression body, never a runtime string.
        if matches!(property.kind, ObjectPropertyKind::Init)
            && self.expression_is_runtime_string_value(&property.value)
        {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "a runtime string value is unavailable as an object-literal property value in the current direct-runtime path; use a statically-known string or the later compatibility path".to_string(),
            ));
        }
        // A non-materialized for-in-key value as an object-property value is a
        // value escape — rejected structurally by the default-deny in
        // `resolve_identifier` (the property value is resolved as a value above).
    }

    pub(crate) fn resolve_property_name(&mut self, name: &PropertyName) {
        match name {
            PropertyName::Identifier(_) | PropertyName::Number(_) | PropertyName::String(_) => {}
        }
    }

    pub(crate) fn resolve_type_assertion(&mut self, expr: &TypeAssertion) {
        self.resolve_type_annotation_text(&expr.type_name);
        self.resolve_expression(&expr.expression);
    }

    pub(crate) fn resolve_satisfies_expression(&mut self, expr: &kali_ast::SatisfiesExpression) {
        self.resolve_type_annotation_text(&expr.type_name);
        self.resolve_expression(&expr.expression);
    }

    pub(crate) fn resolve_relative_import_source(&self, base_dir: &Path, source: &str) -> bool {
        let candidate = base_dir.join(source);
        if candidate.is_file() {
            return true;
        }

        if candidate.is_dir() && self.resolve_directory_index_candidate(&candidate) {
            return true;
        }

        let extensions = [
            "ts", "tsx", "js", "jsx", "mts", "cts", "d.ts", "d.mts", "d.cts",
        ];
        extensions.iter().any(|extension| {
            let candidate = if source.ends_with(extension) {
                base_dir.join(source)
            } else {
                base_dir.join(format!("{}.{}", source, extension))
            };
            candidate.is_file()
                || (candidate.is_dir() && self.resolve_directory_index_candidate(&candidate))
        })
    }

    pub(crate) fn resolve_directory_index_candidate(&self, directory: &Path) -> bool {
        for index_name in [
            "index.ts",
            "index.tsx",
            "index.js",
            "index.jsx",
            "index.mts",
            "index.mjs",
            "index.cts",
            "index.cjs",
            "index.d.ts",
            "index.d.mts",
            "index.d.cts",
        ] {
            if directory.join(index_name).is_file() {
                return true;
            }
        }

        false
    }

    pub(crate) fn resolve_import_source(&self, source: &str) -> Result<bool, Diagnostic> {
        if self.api_surface == "node" && is_node_builtin_specifier(source) {
            return Ok(true);
        }

        if self.api_surface == "node" && source.starts_with("node:") {
            return Err(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "node builtin '{}' is not available on the explicit Node API surface",
                    source
                ),
            ));
        }

        let base_dir = self
            .base_path
            .as_ref()
            .and_then(|path| path.parent())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let project_root =
            kali_npm::discover_project_root(&base_dir).unwrap_or_else(|| base_dir.clone());

        if self.resolve_relative_import_source(&base_dir, source) {
            return Ok(true);
        }

        let Some(resolved) = kali_npm::resolve_materialized_import_with_browser_context(
            project_root,
            source,
            self.api_surface == "browser",
        ) else {
            return Ok(false);
        };

        if let Some(diagnostic) = reject_native_addon_package_source(&resolved) {
            return Err(diagnostic);
        }

        if self.api_surface != "node" {
            if let Ok(contents) = fs::read_to_string(&resolved) {
                if let Some(builtin) = kali_npm::source_mentions_node_only_host_api(&contents) {
                    return Err(Diagnostic::error(
                        e6::NODE_ONLY_HOST_APIS as u32,
                        format!(
                            "package uses Node-only host API '{}' in '{}' and falls outside the default standalone context; use the Phase-3 Node compatibility target",
                            builtin,
                            resolved.display()
                        ),
                    ));
                }
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
#[path = "expression_tests.rs"]
mod expression_tests;
