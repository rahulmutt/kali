//! Stage C (closures) C1 — captured-scalar access sites (read / compound-assign
//! / update-expression / promoted declaration).
//!
//! A name that resolves to neither an own WASM local/param, a module global,
//! nor a module binding may be a scalar captured into an env cell (this
//! function's own cell, or an outer scope's read through the parent chain). All
//! four access shapes route through [`FunctionEmitter::resolve_capture_access`],
//! which returns `Some(offset)` for the ONE shape C1 lowers — a synchronous,
//! single-level, scalar `i64` binding — and `None` for everything else.
//!
//! `None` means "not a C1 capture": the caller keeps its existing
//! local/global/module resolution. Captured-bindings phase 1 (spec §3.1)
//! closes the placeholder end of that fallthrough: a read or plain `=` of a
//! name this function captures that `resolve_capture_access` did not lower is
//! refused with E5506 by [`FunctionEmitter::unlowered_capture_refusal`]
//! instead of reading (or dropping a store to) the zero placeholder. Captured
//! compound-assign and update expressions keep their existing local-miss E5506.

use crate::*;

impl<'a> FunctionEmitter<'a> {
    /// `(env_walk_depth, header-relative offset)` the promoted cell `name`
    /// resolves to, or `None` when it is not a lowerable capture. An own cell
    /// resolves at env-walk depth 0 (the prologue set `current_env` to this
    /// activation's own record). An outer capture resolves at the env-walk depth
    /// computed by [`Self::env_walk_depth_for`] — 0 for a single-level capture
    /// from a non-env-owning function, 1 when THIS function owns its own record
    /// and the owner is its parent link. Purely a lookup — emits nothing.
    pub(crate) fn resolve_capture_access(&self, name: &str) -> Option<(u32, u32)> {
        // Unified predicate (C1 scalar-i64 OR C2 fixed-shape object): the READ
        // and promoted-DECLARATION paths load/store the cell as a raw i64
        // (a scalar value, or an object's base pointer), so both shapes resolve
        // here.
        self.resolve_capture_access_inner(name, false)
    }

    /// SCALAR-only capture resolution for the arithmetic write paths
    /// (compound-assign / update). A fixed-shape object cell resolves via
    /// [`Self::resolve_capture_access`] for reads, but a `+=`/`++` on an object
    /// pointer is not a meaningful i64 op — those helpers gate on this so a
    /// captured-object write falls through to the pre-Stage-C baseline path
    /// (reassigning `obj` / `obj.n = v` from a nested fn is NOT part of C2's
    /// read surface; see the task's heap-write scope note).
    pub(crate) fn resolve_scalar_capture_access(&self, name: &str) -> Option<(u32, u32)> {
        self.resolve_capture_access_inner(name, true)
    }

    /// R-11 T4: the plan-key (`ReprTable`/`numeric_bindings` namespace) that
    /// LEXICALLY DECLARES `name` — the OWNER of the scalar env cell
    /// `resolve_scalar_capture_access` already validated for `name`, not
    /// `self.function_name` (the function CURRENTLY EMITTING the write, which
    /// for a capture read/written from a nested closure is the CAPTURER, a
    /// different function). `repr_infer` files the `numeric_bindings` proof
    /// under the binding's own declaring scope
    /// (`record_numeric_binding_write`'s `binding_scope`), so a caller that
    /// wants the OWNER's proof — not the capturer's, and not the module's —
    /// must resolve this key explicitly rather than reuse
    /// `FunctionEmitter::binding_is_proven_numeric`'s `self.function_name`/
    /// `_start` heuristic, which is tuned for the local/module shapes and
    /// does not fit a captured write.
    ///
    /// Mirrors `resolve_capture_access_inner`'s own two branches without
    /// re-deriving them: an OWN cell's owner is this function itself; an
    /// outer capture's owner is `CapturedRef::owner` (the ancestor whose env
    /// record actually holds the cell — see that struct's own doc on why the
    /// OWNER's namespace, not the capturer's, is authoritative). Callers must
    /// only invoke this after `resolve_scalar_capture_access(name)` already
    /// returned `Some` for the same `name` — it does not re-verify
    /// promotability itself, only reads the same underlying plan data.
    fn scalar_capture_owner(&self, name: &str) -> Option<String> {
        if let Some(active) = self
            .active_iterations
            .iter()
            .rev()
            .find(|active| active.plan.cell_for(name).is_some())
        {
            return Some(
                crate::iteration::owner_repr_namespace(self.env_plans, &active.label).to_string(),
            );
        }
        if self.env_plan.cell_for(name).is_some() {
            return Some(self.function_name.clone());
        }
        self.env_plan.captured_for(name).map(|reference| {
            crate::iteration::owner_repr_namespace(self.env_plans, &reference.owner).to_string()
        })
    }

    /// R-11 T4 review round 4: the shadow guard, now backed by the SHARED
    /// classifier instead of a second, hand-mirrored copy of
    /// `emit_identifier`'s resolution order.
    ///
    /// Rounds 1-3 each independently re-derived that order: round 1/2
    /// widened a denylist of specific module-binding tables; round 3
    /// replaced that with a hand-mirrored `!(A || B || … )` of every
    /// predicate `emit_identifier` calls. The round-3 review PROVED that
    /// mirror could still drift: it added ONE arm to `emit_identifier`
    /// (`"Reflect"` alongside `"Set"`/`"Map"`), left the round-3 guard
    /// untouched, rebuilt, and reproduced the identical silent-wrong-value
    /// failure mode as rounds 1 and 2 — the sixth such incident
    /// project-wide, and it took only one added arm to show it, disproving
    /// the round-3 doc's claim that an unmirrored arm could only over-deny.
    ///
    /// This round deletes the second copy entirely.
    /// `control_flow.rs::resolve_identifier_kind` is now the ONLY place
    /// `emit_identifier`'s resolution order is written, returning an
    /// [`IdentifierResolution`]; `emit_identifier` dispatches on it with an
    /// EXHAUSTIVE `match` (no `_` arm), and this guard asks the identical
    /// question the classifier already answers: "does anything ahead of the
    /// captured-cell lane claim this name?" A future arm added to
    /// `emit_identifier` MUST add a new `IdentifierResolution` variant (the
    /// exhaustive `match` there is a compile error otherwise) — and this
    /// guard, unaware of that variant by name, does not match it either, so
    /// it correctly denies rather than silently admitting. Divergence is now
    /// impossible by construction, not by discipline: there is exactly one
    /// resolution order, not two.
    ///
    /// `"Infinity"` / `"NaN"` remain a SEPARATE, explicit, DOCUMENTED
    /// carve-out — NOT part of `IdentifierResolution` at all, because their
    /// real interception mechanism is NOT `emit_identifier`'s resolution
    /// order. They are resolved by `resolve_static_object_identity_value`
    /// (`intrinsics/object.rs`), a wholly separate static-fold consumer used
    /// by `console.log` and others, which ALREADY treats ANY binding named
    /// `Infinity`/`NaN` (not just a captured one — a measured, PRE-EXISTING,
    /// out-of-scope bug: a bare `let Infinity = 12; console.log(Infinity);`,
    /// zero closures involved, already fails `E4201` on the pre-R-11 parent)
    /// as the JS global unconditionally. Denying the BITWISE WRITE for these
    /// two names does not fix that general bug (a plain local named
    /// `Infinity` is still broken either way) — it only prevents THIS task's
    /// new captured-cell admission from turning that pre-existing breakage
    /// into a silent WRONG VALUE at exit 0 instead of its pre-existing
    /// `E4201`. Folding this into the classifier would be DISHONEST — it
    /// would claim `resolve_identifier_kind` is a complete mirror of
    /// `emit_identifier` when it is not (that function has no
    /// `Infinity`/`NaN` arm at all) — so it stays a separate, visible `if`
    /// here instead.
    ///
    /// Returns `true` when `name` is proven interception-free — i.e. it is
    /// safe to route the bitwise write through the captured-cell lane. The
    /// caller denies whenever this returns `false`.
    pub(crate) fn identifier_read_resolves_only_through_captured_cell(&self, name: &str) -> bool {
        if matches!(name, "Infinity" | "NaN") {
            return false;
        }
        matches!(
            self.resolve_identifier_kind(name),
            IdentifierResolution::CapturedCellOrPlaceholder
        )
    }

    /// Env-walk depth (number of `parent_env` links to follow from
    /// `current_env`) for a capture whose MIR `depth` is `mir_depth` env-owning
    /// hops to the owner. `None` when this task cannot PROVE the record chain is
    /// intact for that shape (fail closed to baseline).
    ///
    /// The runtime env chain links only records that were actually allocated —
    /// records of functions with a PROMOTABLE cell (`cell_is_promotable`, a
    /// repr-dependent verdict). MIR's `depth` counts env-owning ancestors by
    /// STRUCTURAL cell ownership (repr-independent — kali_mir cannot see repr).
    /// The two agree exactly when no intermediate ancestor owns a cell that is
    /// structurally an env owner but NOT promotable (e.g. a captured `F64`
    /// scalar, or a `Closure`/`Array` heap cell) — such a frame allocates no
    /// record, so a MIR depth that counted it would over-walk the chain and
    /// address the wrong record (a silent miscompile).
    ///
    /// The provable subset is `mir_depth == 1`: the owner is the single
    /// env-owning ancestor on the path, so every ancestor STRICTLY between the
    /// capturer and the owner owns no cell at all (transparent) — no repr
    /// ambiguity is possible. Then:
    /// - this function owns no record → `current_env` is already the owner's
    ///   record (transparent intermediates were skipped): env-walk depth 0;
    /// - this function owns its own record → `current_env` is THIS record, whose
    ///   parent link is the owner's record (nearest env-owning ancestor):
    ///   env-walk depth 1 — a genuine one-hop `parent_env` walk.
    ///
    /// `mir_depth >= 2` is NOT proven here (an intermediate env-owning frame may
    /// be non-promotable, absent from the runtime chain) and falls through to
    /// baseline — the pre-existing, unchanged behavior for that shape. See the
    /// Task 5 report for the boundary and the general-solution follow-up.
    fn env_walk_depth_for(&self, mir_depth: u32) -> Option<u32> {
        if mir_depth != 1 {
            return None;
        }
        Some(if self.owns_promotable_env() { 1 } else { 0 })
    }

    /// Shared body of the two resolvers, returning `(env_walk_depth, offset)`.
    /// A cell of an active owner loop (block-scoping §3.3) is searched first;
    /// the other two branches add one link per active loop record.
    /// `scalar_only` selects the promotion predicate: the C1 scalar-i64 gate
    /// (write paths) or the unified C1/C2 gate (read/declaration paths). Both
    /// consult the OWNER's repr namespace (Finding 1): a captured cell was
    /// promoted — and thus allocated — by its owner, so the capturer must gate
    /// on the owner's verdict, not its own namespace (where an outer name
    /// defaults to `I64`).
    fn resolve_capture_access_inner(&self, name: &str, scalar_only: bool) -> Option<(u32, u32)> {
        use crate::closure::Widening;
        let promotable = |owner: &str, is_scalar: bool, widening: Widening| -> bool {
            if scalar_only {
                self.promotable_scalar_cell_in(owner, name, is_scalar, widening)
            } else {
                crate::closure::cell_is_promotable(
                    self.repr_table,
                    owner,
                    name,
                    is_scalar,
                    widening,
                )
            }
        };
        // Block-scoping §3.3: a cell of an active owner loop lives in that
        // loop's current record. `g8` is the innermost active loop's record
        // and each record's parent is the `g8` its loop was entered with, so
        // the `k`-th active loop from the innermost is `k` links up.
        for (k, active) in self.active_iterations.iter().rev().enumerate() {
            if let Some(cell) = active.plan.cell_for(name) {
                let namespace =
                    crate::iteration::owner_repr_namespace(self.env_plans, &active.label);
                // iteration cells are not widened (captured-bindings §1.1)
                return promotable(namespace, cell.is_scalar, Widening::Baseline)
                    .then_some((k as u32, cell.offset));
            }
        }
        // Every active loop record sits between `g8` and this function's own
        // record (or the record it was entered with).
        let extra = self.active_iterations.len() as u32;
        if let Some(cell) = self.env_plan.cell_for(name) {
            // An own cell resolves in THIS function's namespace (it is the owner)
            // at env-walk depth 0, plus the active loop records.
            // The same arguments `lower.rs` promoted this function-plan cell with.
            let widening = Widening::CapturedBindings {
                is_tagged: cell.is_tagged,
            };
            return promotable(&self.function_name, cell.is_scalar, widening)
                .then_some((extra, cell.offset));
        }
        if let Some(reference) = self.env_plan.captured_for(name) {
            // A capture through the parent chain: gate on the OWNER's promotion
            // verdict AND a provable env-walk depth (`env_walk_depth_for`, which
            // fails closed on the unprovable `mir_depth >= 2` shapes). This
            // covers both the single-level capture from a non-owning function
            // (depth 0) and the genuine one-hop walk from an env-owning capturer
            // (depth 1); deeper chains fall through to baseline unchanged.
            let owner = crate::iteration::owner_repr_namespace(self.env_plans, &reference.owner);
            // The owner plan decides the widening (`Widening::for_captured_ref`):
            // a function plan's cell under captured-bindings, an iteration
            // plan's at baseline — what the owner's promotion site passed.
            let widening = Widening::for_captured_ref(self.env_plans, reference);
            if promotable(owner, reference.is_scalar, widening) {
                if let Some(walk) = self.env_walk_depth_for(reference.depth) {
                    return Some((walk + extra, reference.offset));
                }
            }
            return None;
        }
        None
    }

    /// The root identifier of a static member chain (`o.a`, `o.a.b`), or
    /// `None` when `member` is not one (a bare identifier, an operator, a
    /// call, a computed access).
    fn static_member_chain_root<'n>(&'n self, member: &'n LirNode) -> Option<&'n str> {
        let mut node = member;
        let mut depth = 0;
        loop {
            let text = node.text.as_deref()?;
            if node.children.is_empty() {
                return (depth > 0 && !text.is_empty()).then_some(text);
            }
            if node.kind != LirNodeKind::Value
                || node.children.len() != 1
                || crate::lower::is_unary_operator_text(text)
                || matches!(text, "await" | "?")
            {
                return None;
            }
            node = self.node(node.children[0]);
            depth += 1;
        }
    }

    /// Captured-bindings ruling R9: the refusal for a static member access
    /// whose root is a capture this function did not lower. The member lanes
    /// resolve the receiver by name and never emit the identifier, so they
    /// bypass the read choke and used to read the zero placeholder.
    pub(crate) fn unlowered_capture_member_refusal(&self, member: &LirNode) -> Option<String> {
        self.unlowered_capture_refusal(self.static_member_chain_root(member)?)
    }

    /// Captured-bindings ruling R13: the refusal at the member read / store /
    /// update FALLBACK (no lane resolved the shape) for a static member chain
    /// rooted at ANY capture of this function, lowered or not — a promoted C2
    /// cell whose member no lane resolves used to read the zero placeholder
    /// (`function mk(){return {a:1};} function f(){ let o=mk(); const g=()=>o.a;
    /// return g(); }` printed `0`). An unlowered capture keeps its §3.1 reason;
    /// a lowered one takes the value-type reason. `None` for this function's
    /// own local or cell, and for a non-capture.
    pub(crate) fn capture_member_fallback_refusal(&self, member: &LirNode) -> Option<String> {
        let name = self.static_member_chain_root(member)?;
        if self.locals.contains_key(name) || self.env_plan.cell_for(name).is_some() {
            return None;
        }
        self.env_plan.captured_for(name)?;
        Some(self.unlowered_capture_refusal(name).unwrap_or_else(|| {
            kali_common::captured_binding_unavailable_message(
                &self.function_name,
                name,
                kali_common::CaptureRefusal::ValueType,
            )
        }))
    }

    /// Captured-bindings spec §3.1 (A-2.3, A-2.5): the refusal for a name
    /// this function captures but `resolve_capture_access` did not lower.
    /// `None` when `name` is this function's own local, is not one of its
    /// captures, or is a capture block-scoping's iteration refusals own.
    pub(crate) fn unlowered_capture_refusal(&self, name: &str) -> Option<String> {
        if self.locals.contains_key(name) || self.resolve_capture_access(name).is_some() {
            return None;
        }
        let reference = self.env_plan.captured_for(name)?;
        let owned_by_iteration = self
            .env_plans
            .get(&reference.owner)
            .is_some_and(|plan| plan.iteration_of.is_some());
        if reference.through_iteration || owned_by_iteration {
            return None;
        }
        let reason = if reference.depth >= 2 {
            kali_common::CaptureRefusal::Depth
        } else if reference.is_parameter {
            kali_common::CaptureRefusal::Parameter {
                owner: &reference.owner,
            }
        } else {
            kali_common::CaptureRefusal::ValueType
        };
        Some(kali_common::captured_binding_unavailable_message(
            &self.function_name,
            name,
            reason,
        ))
    }

    /// Captured-bindings A-2.1: the owner namespace of a lowered capture of a
    /// boolean `const`, or `None`. Phase 1 refuses the read; phase 2 (Task 10)
    /// gives it `ValueShape::Boolean`.
    pub(crate) fn captured_boolean_const_owner(&self, name: &str) -> Option<String> {
        if self.locals.contains_key(name) || self.env_plan.cell_for(name).is_some() {
            return None;
        }
        self.resolve_capture_access(name)?;
        let owner = self.scalar_capture_owner(name)?;
        self.repr_table
            .binding_is_boolean_const(&owner, name)
            .then_some(owner)
    }

    /// Captured-bindings Task 9: the owner repr of a name the capture lane
    /// lowers, so a read can carry its float shape.
    pub(crate) fn captured_cell_repr(&self, name: &str) -> Option<kali_common::Repr> {
        self.resolve_capture_access(name)?;
        let owner = self.scalar_capture_owner(name)?;
        Some(self.repr_table.scalar(&owner, name))
    }

    /// Whether the lowered capture `name` is an F64 cell (A-4): its slot holds
    /// the double's bits, read through `f64.reinterpret_i64` and written
    /// through `i64.reinterpret_f64`.
    fn captured_cell_is_f64(&self, name: &str) -> bool {
        self.captured_cell_repr(name) == Some(kali_common::Repr::F64)
    }

    /// Ruling R14 (A-4): `name` is an F64 cell this function reaches as a
    /// capturer, not as its owner. `repr_infer` types a free identifier in a
    /// nested function by that function's own node (default `I64`, never
    /// joined to the owner's binding), so an f64 read here would flow into
    /// integer-typed wasm places (E4201). Only `=` (whose right-hand side
    /// does not read the cell) and `+= -= *= /=` lower; every other read is
    /// refused with the value-type reason.
    fn captured_f64_in_capturer(&self, name: &str) -> bool {
        self.env_plan.cell_for(name).is_none() && self.captured_cell_is_f64(name)
    }

    fn captured_f64_read_refusal(&mut self, function: &mut Function, name: &str) -> EmittedValue {
        let message = kali_common::captured_binding_unavailable_message(
            &self.function_name,
            name,
            kali_common::CaptureRefusal::ValueType,
        );
        self.deny_e5506(function, &message)
    }

    /// Captured-bindings A-4: turn the just-emitted `value` into an f64 on
    /// the stack for an F64 cell, choosing from what was emitted (fix
    /// round 1):
    /// - an emitted `ValueShape::Float` is already an f64;
    /// - otherwise `is_float_valued` decides, as on the local F64 lane, and
    ///   an integer is converted with `f64.convert_i64_s`.
    ///
    /// `false` (the caller refuses) when nothing was produced, or when the
    /// emitter produced a non-float shape that `is_float_valued` calls float
    /// for a literal: `1e20` is interned as a string handle (an i64), which
    /// would be invalid wasm (E4201) or, converted, a wrong value.
    fn emit_f64_operand(
        &mut self,
        function: &mut Function,
        value: LirNodeId,
        emitted: EmittedValue,
    ) -> bool {
        if !emitted.produced {
            return false;
        }
        if emitted.shape == ValueShape::Float {
            return true;
        }
        let literal = self.node(self.unwrap_transparent(value)).kind == LirNodeKind::Literal;
        if self.is_float_valued(value) {
            return !literal;
        }
        function.instruction(&Instruction::F64ConvertI64S);
        true
    }

    /// Captured-bindings fix round 1: the f64 a right-hand side denotes when
    /// it is a numeric literal (`1e20`, `1e300`, which the generic literal
    /// lane interns as a string handle), a global `NaN` / `Infinity`
    /// resolved by no other lane (it reads a placeholder there), or a unary
    /// `-` / `+` over one of those.
    fn f64_constant_rhs(&self, value: LirNodeId) -> Option<f64> {
        let node = self.node(self.unwrap_transparent(value));
        match node.kind {
            LirNodeKind::Literal => {
                parse_numeric_literal_value(node.text.as_deref()?).filter(|v| !v.is_nan())
            }
            LirNodeKind::Value if node.children.is_empty() => {
                let name = node.text.as_deref()?;
                let global = match name {
                    "NaN" => f64::NAN,
                    "Infinity" => f64::INFINITY,
                    _ => return None,
                };
                let unclaimed = matches!(
                    self.resolve_identifier_kind(name),
                    IdentifierResolution::CapturedCellOrPlaceholder
                ) && self.env_plan.cell_for(name).is_none()
                    && self.env_plan.captured_for(name).is_none();
                unclaimed.then_some(global)
            }
            LirNodeKind::Value if node.children.len() == 1 => {
                let operand = self.f64_constant_rhs(node.children[0])?;
                match node.text.as_deref() {
                    Some("-") => Some(-operand),
                    Some("+") => Some(operand),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Emit `value` as an f64 for an F64 cell (`f64_constant_rhs`, else
    /// `emit_node` + `emit_f64_operand`). `false` when the caller must refuse.
    fn emit_f64_rhs(&mut self, function: &mut Function, value: LirNodeId) -> bool {
        if let Some(constant) = self.f64_constant_rhs(value) {
            function.instruction(&Instruction::F64Const(constant.into()));
            return true;
        }
        let emitted = self.emit_node(function, value, true);
        self.emit_f64_operand(function, value, emitted)
    }

    /// Whether `id` is the node whose value `emit_aggregate_literal`'s
    /// sequence loop drops (an expression statement), looking through
    /// transparent wrappers.
    fn value_is_discarded(&self, id: LirNodeId) -> bool {
        self.discarded_value_node
            .is_some_and(|node| self.unwrap_transparent(node) == self.unwrap_transparent(id))
    }

    fn captured_f64_value_refusal(&mut self, function: &mut Function, name: &str) {
        self.captured_f64_read_refusal(function, name);
        function.instruction(&Instruction::I64Const(0));
    }

    /// Read site: load a captured scalar. `None` when `name` is not a
    /// C1-promoted capture (caller falls through to its own resolution).
    pub(crate) fn try_emit_captured_read(
        &mut self,
        function: &mut Function,
        name: &str,
    ) -> Option<EmittedValue> {
        if self.captured_boolean_const_owner(name).is_some() {
            return Some(self.deny_e5506(
                function,
                &kali_common::captured_binding_unavailable_message(
                    &self.function_name,
                    name,
                    kali_common::CaptureRefusal::ValueType,
                ),
            ));
        }
        if self.captured_f64_in_capturer(name) {
            return Some(self.captured_f64_read_refusal(function, name));
        }
        let (depth, offset) = self.resolve_capture_access(name)?;
        crate::closure::emit_cell_load(function, self.current_env_global(), depth, offset);
        if self.captured_cell_is_f64(name) {
            function.instruction(&Instruction::F64ReinterpretI64);
            return Some(EmittedValue {
                produced: true,
                shape: ValueShape::Float,
            });
        }
        Some(EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        })
    }

    /// Promoted DECLARATION: store a `let`/`var`/`const` initializer into the
    /// owner's env cell (the binding has no WASM local slot). `Some(())` when
    /// `name` is a C1-promoted own cell (handled here); `None` otherwise, so the
    /// caller keeps its normal declarator lowering (heap/closure captures, etc.).
    pub(crate) fn try_emit_captured_decl(
        &mut self,
        function: &mut Function,
        name: &str,
        init: LirNodeId,
    ) -> Option<()> {
        let (depth, offset) = self.resolve_capture_access(name)?;
        let env_global = self.current_env_global();
        let scratch = self.locals.len() as u32;
        if self.captured_cell_is_f64(name) {
            if !self.emit_f64_rhs(function, init) {
                self.captured_f64_value_refusal(function, name);
                return Some(());
            }
            function.instruction(&Instruction::I64ReinterpretF64);
            crate::closure::emit_cell_store(function, env_global, depth, offset, scratch);
            return Some(());
        }
        let produced = self.emit_node(function, init, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        crate::closure::emit_cell_store(function, env_global, depth, offset, scratch);
        Some(())
    }

    /// Write site: assign (`=`) or compound-assign (`+= -= *= /= %=`) to a
    /// captured scalar. `None` when `name` is not a C1-promoted capture. Leaves
    /// the assignment expression's value (the stored value) on the stack,
    /// matching the local/global assignment lanes. A promoted target with an
    /// unsupported operator (`**= ??= &&= ||=`) falls through (`None`) to the
    /// caller's existing handling rather than silently doing the wrong thing.
    pub(crate) fn try_emit_captured_assign(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        op: &str,
        name: &str,
        right: LirNodeId,
    ) -> Option<bool> {
        if !matches!(
            op,
            "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=" | ">>>="
        ) {
            return None;
        }
        if self.captured_cell_is_f64(name) {
            return self.try_emit_captured_f64_assign(function, id, op, name, right);
        }
        // Scalar-only: a captured OBJECT cell (C2) keeps its baseline write path
        // — `=`/compound-assign through the capture is out of C2's read scope.
        let (depth, offset) = self.resolve_scalar_capture_access(name)?;
        let env_global = self.current_env_global();
        let scratch = self.locals.len() as u32;
        match op {
            "=" => {
                let rhs = self.emit_node(function, right, true);
                if !rhs.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
                crate::closure::emit_cell_store(function, env_global, depth, offset, scratch);
            }
            "+=" | "-=" | "*=" | "/=" | "%=" => {
                crate::closure::emit_cell_load(function, env_global, depth, offset);
                let rhs = self.emit_node(function, right, true);
                if !rhs.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
                match op {
                    "+=" => function.instruction(&Instruction::I64Add),
                    "-=" => function.instruction(&Instruction::I64Sub),
                    "*=" => function.instruction(&Instruction::I64Mul),
                    "/=" => function.instruction(&Instruction::I64DivS),
                    "%=" => function.instruction(&Instruction::I64RemS),
                    // Unreachable: the arm guard fixes `op` to this set. `op` is
                    // a `&str`, not an AST/plan enum — mirrors
                    // `emit_module_global_assignment` (`literal.rs`).
                    _ => unreachable!("compound op set fixed by the arm guard"),
                };
                crate::closure::emit_cell_store(function, env_global, depth, offset, scratch);
            }
            "&=" | "|=" | "^=" | "<<=" | ">>=" | ">>>=" => {
                // R-11 T4: bitwise compound on a captured scalar env cell —
                // the sibling shape to the local (`literal.rs`'s
                // `emit_local_compound_assignment`) and module-global
                // (`emit_module_global_assignment`) bitwise arms, over a
                // THIRD storage location (the env cell this function either
                // owns or reaches through the parent chain).
                //
                // TARGET axis — the identical "default is not a proof" trap
                // those two arms already closed applies here with an extra
                // twist: `FunctionEmitter::binding_is_proven_numeric` (the
                // helper those two arms call directly) has its OWN
                // `self.function_name`/`_start` heuristic baked in, tuned for
                // "this name is either a local of the CURRENTLY EMITTING
                // function or a module global" — neither holds for a captured
                // write, where `self.function_name` is the CAPTURING
                // function (e.g. `set`), not the LEXICALLY DECLARING one
                // (e.g. `outer`) `repr_infer` files the proof under
                // (`record_numeric_binding_write`'s `binding_scope`). Calling
                // that helper as-is would consult `_start`'s namespace (or
                // this function's own), find no entry, and — because
                // `numeric_bindings` membership is a HashSet lookup with no
                // default — correctly return `false` for every genuinely
                // admissible case too, denying 100% of this shape rather than
                // leaking. That is a safe failure mode, but not the right
                // one: `scalar_capture_owner` resolves the OWNER's plan key
                // (the same key `resolve_scalar_capture_access` already
                // proved this cell is promoted under) and the OWNER's
                // `ReprTable::numeric_bindings` entry is consulted directly,
                // bypassing the mismatched heuristic entirely.
                //
                // RHS axis — reused verbatim, unchanged from the local/module
                // arms: `bitwise_compound_rhs_is_provably_i64` already
                // refuses a float, a string, a BigInt literal, and every
                // identifier (positive evidence only).
                //
                // BigInt target axis — `numeric_bindings` admits a BigInt
                // literal write exactly like a plain number
                // (`write_value_is_numeric`'s `BigIntLiteral` arm), so
                // `binding_is_proven_numeric` alone cannot tell `let flags =
                // 6n;` from `let flags = 6;` — the identical gap Task 3 found
                // for module globals and closed with a separate, additive
                // whole-program BigInt-taint scan
                // (`module_global_bigint_targets`). `captured_cell_bigint_targets`
                // is that same scan (`collect_bigint_tainted_captured_cells`,
                // `lower.rs`) reapplied to promoted scalar cell names —
                // required here because, unlike the local lane (a known,
                // deferred, pre-existing gap this task does not touch), this
                // whole shape is NEW: before this task every bitwise op on a
                // captured cell refused uniformly (resolve denied it
                // entirely), so silently truncating a BigInt now would be a
                // fresh regression, not an inherited one.
                //
                // FLOAT target axis — T4 review Important 1.
                // `self.repr_table.scalar(&owner, name) != Repr::I64` (never
                // `self.scalar_repr`, which is `self.function_name`-scoped —
                // the CAPTURING function, not the owner, the identical
                // capturer/owner mismatch the target-axis paragraph above
                // already closed for `binding_is_proven_numeric`) mirrors the
                // local/module arms' own additional check
                // (`literal.rs`'s local arm, `:1196`) and is harmless to keep,
                // but by ITSELF it is not sufficient here: it queries the
                // exact same `(owner, name)` key
                // `crate::closure::cell_is_promotable` already required to be
                // `I64` before this cell was ever promoted in the first
                // place, so it is redundant for every case promotion already
                // saw — and blind to exactly the same case promotion was
                // blind to. That blind spot is real: `repr_infer`'s
                // scalar-repr union-find resolves an off-scope write's node
                // key via `binding_scope`, which cannot name the true OWNER
                // when the write is reached from a THIRD function (neither
                // the owner nor top-level module scope) — e.g. a SIBLING
                // closure of the one performing the bitwise op, both nested
                // inside the true owner. Such a write is filed under a
                // DIFFERENT, disconnected union-find node
                // `scalar(&owner, name)` never sees (measured: `function
                // o(){ let n=6; function w(){ n=6.5; } function s(){ n&=3; }
                // w(); s(); ... }` — `w`'s float write reaches neither this
                // check nor `cell_is_promotable`, so the cell promotes and
                // this check passes, and the raw `I32WrapI64` combiner then
                // emits WASM the validator rejects outright, `E4201`, which
                // the plan's Global Constraints forbid). Closed instead by
                // `captured_cell_bigint_targets`'s sibling,
                // `captured_cell_float_targets`
                // (`collect_float_tainted_captured_cells`, `lower.rs`): an
                // ADDITIVE, whole-program, NAME-keyed scan that does not
                // depend on `binding_scope` naming the right owner at all —
                // it walks every declarator/reassignment directly.
                let owner = self.scalar_capture_owner(name)?;
                if self.repr_table.scalar(&owner, name) != kali_common::Repr::I64
                    || !self.repr_table.binding_is_proven_numeric(&owner, name)
                    || self.captured_cell_bigint_targets.contains(name)
                    || self.captured_cell_float_targets.contains(name)
                    || !self.bitwise_compound_rhs_is_provably_i64(right)
                {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "bitwise compound assignment '{op}' on a captured binding '{name}' is unavailable in the current phase"
                        ),
                    ));
                    function.instruction(&Instruction::I64Const(0));
                    return Some(true);
                }
                crate::closure::emit_cell_load(function, env_global, depth, offset);
                function.instruction(&Instruction::I32WrapI64);
                self.emit_float_operand(function, right, false);
                function.instruction(&Instruction::I32WrapI64);
                self.emit_bitwise_i32_op_extend(function, op);
                crate::closure::emit_cell_store(function, env_global, depth, offset, scratch);
            }
            // Unreachable: the outer `matches!` guard already returned for any
            // other operator. `op` is a `&str`, not an AST/plan enum.
            _ => unreachable!("assign op set fixed by the guard above"),
        }
        // Assignment expression value: re-load the freshly stored cell.
        crate::closure::emit_cell_load(function, env_global, depth, offset);
        Some(true)
    }

    /// Captured-bindings A-4: `=`, `+=`, `-=`, `*=`, `/=` on an F64 cell, with
    /// f64 arithmetic over the reinterpreted bits. Leaves the stored double on
    /// the stack, as the local F64 assignment lane does. `%=` (wasm has no f64
    /// remainder) and the bitwise operators return `None`, so they keep the
    /// caller's existing E5506.
    ///
    /// Ruling R14: in a capturer the stored double left on the stack is an
    /// f64 the capturer's `repr_infer` types do not expect, so the assignment
    /// lowers only as an expression statement, whose value is dropped
    /// (`id` is `discarded_value_node`); used as a value it is a read of the
    /// cell and is refused. The `check` pass mirrors the rule.
    fn try_emit_captured_f64_assign(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        op: &str,
        name: &str,
        right: LirNodeId,
    ) -> Option<bool> {
        let arithmetic = match op {
            "=" => None,
            "+=" => Some(Instruction::F64Add),
            "-=" => Some(Instruction::F64Sub),
            "*=" => Some(Instruction::F64Mul),
            "/=" => Some(Instruction::F64Div),
            _ => return None,
        };
        if self.captured_f64_in_capturer(name) && !self.value_is_discarded(id) {
            self.captured_f64_value_refusal(function, name);
            return Some(true);
        }
        let (depth, offset) = self.resolve_capture_access(name)?;
        let env_global = self.current_env_global();
        let scratch = self.locals.len() as u32;
        if let Some(instruction) = &arithmetic {
            crate::closure::emit_cell_load(function, env_global, depth, offset);
            function.instruction(&Instruction::F64ReinterpretI64);
            if !self.emit_f64_rhs(function, right) {
                self.captured_f64_value_refusal(function, name);
                return Some(true);
            }
            function.instruction(instruction);
        } else if !self.emit_f64_rhs(function, right) {
            self.captured_f64_value_refusal(function, name);
            return Some(true);
        }
        function.instruction(&Instruction::I64ReinterpretF64);
        crate::closure::emit_cell_store(function, env_global, depth, offset, scratch);
        // Assignment expression value: the stored double.
        crate::closure::emit_cell_load(function, env_global, depth, offset);
        function.instruction(&Instruction::F64ReinterpretI64);
        Some(true)
    }

    /// Update site: `c++ / c-- / ++c / --c` on a captured scalar. `None` when
    /// `name` is not a C1-promoted capture. Leaves the expression's value
    /// (post-value for prefix, pre-value for postfix) on the stack.
    pub(crate) fn try_emit_captured_update(
        &mut self,
        function: &mut Function,
        name: &str,
        op: &str,
    ) -> Option<EmittedValue> {
        // Scalar-only: `++`/`--` on a captured OBJECT pointer is not a
        // meaningful i64 op — keep the baseline path for object cells.
        let (depth, offset) = self.resolve_scalar_capture_access(name)?;
        let env_global = self.current_env_global();
        let value_scratch = self.locals.len() as u32; // consumed by emit_cell_store
        let old_scratch = value_scratch + 1; // holds the pre-value for postfix
        let is_increment = matches!(op, "prefix++" | "postfix++");
        let is_prefix = matches!(op, "prefix++" | "prefix--");

        crate::closure::emit_cell_load(function, env_global, depth, offset); // [old]
        function.instruction(&Instruction::LocalTee(old_scratch)); // save old, keep on stack
        function.instruction(&Instruction::I64Const(1));
        if is_increment {
            function.instruction(&Instruction::I64Add);
        } else {
            function.instruction(&Instruction::I64Sub);
        }
        crate::closure::emit_cell_store(function, env_global, depth, offset, value_scratch);
        if is_prefix {
            crate::closure::emit_cell_load(function, env_global, depth, offset);
        } else {
            function.instruction(&Instruction::LocalGet(old_scratch));
        }
        Some(EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        })
    }
}
