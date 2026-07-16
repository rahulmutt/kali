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
//! local/global/module/placeholder resolution, so every out-of-scope shape
//! (heap/closure captures — the C2 surface — non-`i64` scalars, multi-level
//! chains, captured params) stays byte-identical to the pre-Stage-C compiler.
//! This deliberately does NOT emit a fresh E5506 for those shapes: doing so
//! turned a main-green benchmark red (`nested-wrapper-pruning`, which compiles
//! closure-as-value captures), and their existing behavior (a captured
//! compound-assign still hits the local-miss E5506; a captured read still hits
//! the identifier placeholder) is preserved unchanged.

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
    /// `scalar_only` selects the promotion predicate: the C1 scalar-i64 gate
    /// (write paths) or the unified C1/C2 gate (read/declaration paths). Both
    /// consult the OWNER's repr namespace (Finding 1): a captured cell was
    /// promoted — and thus allocated — by its owner, so the capturer must gate
    /// on the owner's verdict, not its own namespace (where an outer name
    /// defaults to `I64`).
    fn resolve_capture_access_inner(&self, name: &str, scalar_only: bool) -> Option<(u32, u32)> {
        let promotable = |owner: &str, is_scalar: bool| -> bool {
            if scalar_only {
                self.promotable_scalar_cell_in(owner, name, is_scalar)
            } else {
                crate::closure::cell_is_promotable(self.repr_table, owner, name, is_scalar)
            }
        };
        if let Some(cell) = self.env_plan.cell_for(name) {
            // An own cell resolves in THIS function's namespace (it is the owner)
            // at env-walk depth 0.
            return promotable(&self.function_name, cell.is_scalar).then_some((0, cell.offset));
        }
        if let Some(reference) = self.env_plan.captured_for(name) {
            // A capture through the parent chain: gate on the OWNER's promotion
            // verdict AND a provable env-walk depth (`env_walk_depth_for`, which
            // fails closed on the unprovable `mir_depth >= 2` shapes). This
            // covers both the single-level capture from a non-owning function
            // (depth 0) and the genuine one-hop walk from an env-owning capturer
            // (depth 1); deeper chains fall through to baseline unchanged.
            if promotable(&reference.owner, reference.is_scalar) {
                if let Some(walk) = self.env_walk_depth_for(reference.depth) {
                    return Some((walk, reference.offset));
                }
            }
            return None;
        }
        None
    }

    /// Read site: load a captured scalar. `None` when `name` is not a
    /// C1-promoted capture (caller falls through to its own resolution).
    pub(crate) fn try_emit_captured_read(
        &mut self,
        function: &mut Function,
        name: &str,
    ) -> Option<EmittedValue> {
        let (depth, offset) = self.resolve_capture_access(name)?;
        crate::closure::emit_cell_load(function, self.current_env_global(), depth, offset);
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
        op: &str,
        name: &str,
        right: LirNodeId,
    ) -> Option<bool> {
        if !matches!(op, "=" | "+=" | "-=" | "*=" | "/=" | "%=") {
            return None;
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
            // Unreachable: the outer `matches!` guard already returned for any
            // other operator. `op` is a `&str`, not an AST/plan enum.
            _ => unreachable!("assign op set fixed by the guard above"),
        }
        // Assignment expression value: re-load the freshly stored cell.
        crate::closure::emit_cell_load(function, env_global, depth, offset);
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
