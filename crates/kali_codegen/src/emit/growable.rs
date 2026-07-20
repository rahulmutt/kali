//! Growable runtime-array emission (throw-fallout Stage 4).
//!
//! Lowers the bindings the types-side promotion
//! (`kali_types`' growable safe-position allowlist + i64 repr gate, carried
//! on `ReprTable::is_growable_array_binding`) marked growable. Layout (the
//! authoritative Stage 4 memory layout, Step-5 encoding as ruled):
//!
//! ```text
//! handle : i64 = zero_extend(hdr_ptr) | ARRAY_HANDLE_TAG          ; bit 62
//! hdr    @ hdr_ptr  : [ len:i64 @+0 ][ cap:i64 @+8 ][ data_ptr:i64 @+16 ]
//! data   @ data_ptr : [ v0:i64 @+0 ][ v1:i64 @+8 ] … [ v(cap-1) ]
//! ```
//!
//! Element slots are i64 values (Task 2: numbers; Task 3 adds tagged string
//! handles). `push` grows geometrically (`cap * 2`) through a fresh
//! `__alloc`/`__alloc_global` (`alloc_callee_index` — the existing arena
//! lane; GC-less: a dropped data block is reclaimed by arena reset/release,
//! never traced). Realloc rewrites `data_ptr`/`cap` INSIDE the header, so
//! the tagged handle — and the binding local holding it — is stable across
//! growth (no binding-local update on realloc, by construction).
//!
//! This is a SEPARATE lane from the plain inline `[len][elem…]` arrays
//! (`emit_array_allocation_with_len`): the two layouts must never conflate,
//! which the disjoint `growable_array_bindings` / `array_bindings` oracles
//! guarantee.

use crate::*;

/// Capacity a growable array starts with (`const x = []` → `cap = 4`;
/// seeded literals use `max(seed_len, 4)`).
pub(crate) const GROWABLE_INITIAL_CAP: usize = 4;

/// i64 mask clearing `ARRAY_HANDLE_TAG`: `handle & GROWABLE_HANDLE_MASK`
/// yields the zero-extended header pointer (decode = mask + `I32WrapI64`,
/// the string-handle idiom). `pub(crate)` so the USP `.append` inline-push
/// lane (Stage P4 Task 4, `emit/url.rs`) shares the exact same decode.
pub(crate) const GROWABLE_HANDLE_MASK: i64 = !(crate::ARRAY_HANDLE_TAG) as i64;

/// Source of a growable array's tagged i64 handle for `emit_growable_push`
/// (Stage P2 Lane 1 Task 5). Both variants leave the SAME single i64 handle on
/// the stack before the shared mask+store step — the symmetry the two receiver
/// branches must preserve.
pub(crate) enum GrowableHandle {
    /// Named-binding receiver `a.push(v)`: the tagged handle lives in a stable
    /// per-binding local (unchanged across realloc — the header indirection).
    Local(u32),
    /// Field-read receiver `o.values.push(v)`: materialize the tagged handle by
    /// emitting the field read; the object slot already holds the tagged handle
    /// (Task 3 interned the growable-i64 field with it).
    Field(LirNodeId),
}

/// Recognized `.push` receiver (Stage P2 Lane 1 Task 5): a named growable
/// binding or a `GrowableArrayI64` object field. Field receivers carry the
/// member-read node so the emit path can materialize the handle from the slot.
pub(crate) enum GrowablePushReceiver {
    Named(String),
    Field(LirNodeId),
}

impl<'a> FunctionEmitter<'a> {
    /// Index of the dedicated i64 growable scratch local reserved by
    /// `collect_function_locals` for any function with a growable binding.
    /// Panics if missing — reservation and emission share the single
    /// `growable_scratch_local_name` helper, so a miss is a provisioning bug.
    pub(crate) fn growable_scratch_local(&self) -> u32 {
        self.locals
            .get(crate::lower::growable_scratch_local_name().as_str())
            .copied()
            .expect("growable scratch local reserved for any function with a growable binding")
    }

    /// Push `hdr_ptr` (i32) of the handle held in the dedicated scratch.
    pub(crate) fn emit_growable_scratch_hdr(&self, function: &mut Function, scratch: u32) {
        function.instruction(&Instruction::LocalGet(scratch));
        function.instruction(&Instruction::I32WrapI64);
    }

    /// Allocate a growable array: 24-byte header + `cap * 8`-byte data
    /// block, `len = seed_len`, through `alloc_callee_index()` (arena lane).
    /// Leaves the TAGGED i64 handle on the stack; the header pointer is also
    /// left in the dedicated growable scratch local so the declarator can
    /// store seed elements. Seed VALUES are the caller's job (they need the
    /// declarator's element nodes).
    pub(crate) fn emit_growable_alloc(
        &mut self,
        function: &mut Function,
        seed_len: usize,
        cap: usize,
    ) -> EmittedValue {
        let scratch = self.growable_scratch_local();
        let alloc = self.alloc_callee_index();

        // hdr = __alloc(24), zero-extended into the dedicated scratch. The
        // scratch (not a generic trailing slot) survives the caller's later
        // seed-element emission.
        function.instruction(&Instruction::I32Const(24));
        function.instruction(&Instruction::Call(alloc));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(scratch));

        // hdr.data_ptr = __alloc(cap * 8)  (stored zero-extended)
        self.emit_growable_scratch_hdr(function, scratch);
        function.instruction(&Instruction::I32Const((cap * 8) as i32));
        function.instruction(&Instruction::Call(alloc));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 16,
            align: 3,
            memory_index: 0,
        }));

        // hdr.len = seed_len
        self.emit_growable_scratch_hdr(function, scratch);
        function.instruction(&Instruction::I64Const(seed_len as i64));
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));

        // hdr.cap = cap
        self.emit_growable_scratch_hdr(function, scratch);
        function.instruction(&Instruction::I64Const(cap as i64));
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 8,
            align: 3,
            memory_index: 0,
        }));

        // Result: zero_extend(hdr) | ARRAY_HANDLE_TAG
        function.instruction(&Instruction::LocalGet(scratch));
        function.instruction(&Instruction::I64Const(crate::ARRAY_HANDLE_TAG as i64));
        function.instruction(&Instruction::I64Or);
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    /// Allocate + seed a growable array for an OBJECT FIELD value (Stage P2
    /// Lane 1 Task 5), leaving the TAGGED i64 handle on the stack for the
    /// caller's field store. The object-field twin of the growable BINDING
    /// declarator lane (control_flow.rs): both `emit_growable_alloc` then seed
    /// `*(data_ptr + i*8) = seed_i`. The difference is the handle lives on the
    /// stack (not a binding local) throughout seeding — the seed addresses are
    /// derived from the dedicated growable scratch's header pointer, so the
    /// handle on the stack is never disturbed. Promotion admits only an
    /// array-literal initializer of scalar seeds; anything else fails closed.
    pub(crate) fn emit_growable_field_value(
        &mut self,
        function: &mut Function,
        value_id: LirNodeId,
    ) {
        let aggregate = self
            .resolve_literal_aggregate(value_id)
            .map(|id| self.node(id).clone())
            .filter(|node| self.is_array_literal(node));
        let Some(aggregate) = aggregate else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "a growable-array object field must be initialized with an array literal"
                    .to_string(),
            ));
            function.instruction(&Instruction::I64Const(0));
            return;
        };
        let seed_len = aggregate.children.len();
        let cap = seed_len.max(GROWABLE_INITIAL_CAP);
        // Leaves the tagged handle on the stack and the header pointer in the
        // dedicated growable scratch.
        let allocated = self.emit_growable_alloc(function, seed_len, cap);
        if !allocated.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        // Seed each element via `data_ptr + i*8` (data_ptr read from the header
        // in the dedicated scratch, so the handle on the stack stays put).
        let scratch = self.growable_scratch_local();
        for (i, child) in aggregate.children.iter().copied().enumerate() {
            self.emit_growable_scratch_hdr(function, scratch);
            function.instruction(&Instruction::I64Load(MemArg {
                offset: 16,
                align: 3,
                memory_index: 0,
            }));
            function.instruction(&Instruction::I32WrapI64);
            let produced = self.emit_node(function, child, true);
            if !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            }
            function.instruction(&Instruction::I64Store(MemArg {
                offset: (i * 8) as u64,
                align: 3,
                memory_index: 0,
            }));
        }
    }

    /// Append `value` to the growable array whose tagged handle lives in
    /// `handle_local`: grow (`cap * 2`, `memory.copy` of the live prefix)
    /// when full, store at `data_ptr + len*8`, bump `len`. Leaves the NEW
    /// LENGTH on the stack (JS `push` returns it).
    pub(crate) fn emit_growable_push(
        &mut self,
        function: &mut Function,
        handle: GrowableHandle,
        value: LirNodeId,
    ) -> EmittedValue {
        // Fail-closed: a float value has no i64 element encoding. The types
        // promotion gate already excludes float pushes; this is the codegen
        // mirror so a gate regression can never store a raw f64 bit pattern.
        if self.is_float_valued(value) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "pushing a floating-point value onto a growable array is unavailable in the current phase".to_string(),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        let scratch = self.growable_scratch_local();
        // The two generic trailing scratch slots are free to use here ONLY
        // after `value` has been fully emitted (its emission may use them
        // internally); the code below never re-enters `emit_node`.
        let generic_scratch = self.locals.len() as u32;
        let value_scratch = generic_scratch + 1;
        // Realloc allocator: a push lexically inside a PER-ITERATION loop
        // arena must NOT allocate the replacement data block from the current
        // arena — the loop's end-of-iteration reset would recycle it while
        // the (outer-lived) binding still points at it: use-after-reset.
        // Today the MIR arena gate never grants a loop arena to a loop whose
        // body contains a `.push` (unknown-call conservatism — verified
        // empirically for both the object/array and string-site channels),
        // so this branch routes to `__alloc_global` only if that
        // conservatism is ever relaxed — closed BY CONSTRUCTION, not by
        // analysis coupling. Outside any loop-arena frame the existing
        // function-level arena lane applies (`alloc_callee_index`).
        let alloc = if self
            .arena_frames
            .iter()
            .any(|frame| frame.loop_frame_index.is_some())
        {
            self.alloc_global_fn_index()
        } else {
            self.alloc_callee_index()
        };

        // v — evaluated FIRST (JS argument order; also frees the generic
        // scratch slots for the sequence below).
        let produced = self.emit_node(function, value, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::LocalSet(value_scratch));

        // hdr (i64, zero-extended) into the dedicated scratch. A named receiver
        // reads its stable handle local; a field receiver materializes the SAME
        // tagged handle by emitting the field read (the slot holds it — Task 3).
        // Both leave one i64 handle on the stack before the shared mask+store,
        // so everything downstream is byte-identical.
        match handle {
            GrowableHandle::Local(handle_local) => {
                function.instruction(&Instruction::LocalGet(handle_local));
            }
            GrowableHandle::Field(receiver_id) => {
                let produced = self.emit_growable_receiver_handle(function, receiver_id);
                if !produced.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
            }
        }
        function.instruction(&Instruction::I64Const(GROWABLE_HANDLE_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(scratch));

        // if (len == cap) grow
        self.emit_growable_scratch_hdr(function, scratch);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
        self.emit_growable_scratch_hdr(function, scratch);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 8,
            align: 3,
            memory_index: 0,
        }));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        {
            // new_data = __alloc(cap * 2 * 8)
            self.emit_growable_scratch_hdr(function, scratch);
            function.instruction(&Instruction::I64Load(MemArg {
                offset: 8,
                align: 3,
                memory_index: 0,
            }));
            function.instruction(&Instruction::I64Const(16));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::Call(alloc));
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(generic_scratch));

            // memory.copy(dst = new_data, src = data_ptr, n = len * 8) —
            // the live prefix moves; the old block is dead to this array
            // (arena reclamation frees it; GC-less by design).
            function.instruction(&Instruction::LocalGet(generic_scratch));
            function.instruction(&Instruction::I32WrapI64);
            self.emit_growable_scratch_hdr(function, scratch);
            function.instruction(&Instruction::I64Load(MemArg {
                offset: 16,
                align: 3,
                memory_index: 0,
            }));
            function.instruction(&Instruction::I32WrapI64);
            self.emit_growable_scratch_hdr(function, scratch);
            function.instruction(&Instruction::I64Load(MemArg {
                offset: 0,
                align: 3,
                memory_index: 0,
            }));
            function.instruction(&Instruction::I64Const(8));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::MemoryCopy {
                src_mem: 0,
                dst_mem: 0,
            });

            // hdr.data_ptr = new_data (the HANDLE is untouched — stability
            // across realloc is the whole point of the header indirection).
            self.emit_growable_scratch_hdr(function, scratch);
            function.instruction(&Instruction::LocalGet(generic_scratch));
            function.instruction(&Instruction::I64Store(MemArg {
                offset: 16,
                align: 3,
                memory_index: 0,
            }));

            // hdr.cap = cap * 2
            self.emit_growable_scratch_hdr(function, scratch);
            self.emit_growable_scratch_hdr(function, scratch);
            function.instruction(&Instruction::I64Load(MemArg {
                offset: 8,
                align: 3,
                memory_index: 0,
            }));
            function.instruction(&Instruction::I64Const(2));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Store(MemArg {
                offset: 8,
                align: 3,
                memory_index: 0,
            }));
        }
        function.instruction(&Instruction::End);

        // *(data_ptr + len * 8) = v
        self.emit_growable_scratch_hdr(function, scratch);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 16,
            align: 3,
            memory_index: 0,
        }));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_growable_scratch_hdr(function, scratch);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(8));
        function.instruction(&Instruction::I32Mul);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::LocalGet(value_scratch));
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));

        // hdr.len = len + 1
        self.emit_growable_scratch_hdr(function, scratch);
        self.emit_growable_scratch_hdr(function, scratch);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));

        // Result: the new length (JS `push` semantics).
        self.emit_growable_scratch_hdr(function, scratch);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    /// `x.length` over a growable handle expression: decode + `hdr.len`.
    /// Stack-only (no locals).
    /// Emit a growable-array RECEIVER handle (`receiver` is a named binding or
    /// an `o.field` member) for a recognized growable operation, with the C-2
    /// growable-field-read gate lifted so an `object_field_is_growable_array`
    /// field read is admitted HERE (an allowlisted SAFE position). Restores the
    /// prior gate state afterward. Harmless for a named receiver — a bare
    /// binding never trips the field gate in `emit_unary`. This is the single
    /// entry every growable receiver load flows through, so any growable field
    /// read that does NOT come through here stays denied E5506 (default-deny).
    pub(crate) fn emit_growable_receiver_handle(
        &mut self,
        function: &mut Function,
        receiver: LirNodeId,
    ) -> EmittedValue {
        let previous = self.admit_growable_field_read;
        self.admit_growable_field_read = true;
        let value = self.emit_node(function, receiver, true);
        self.admit_growable_field_read = previous;
        value
    }

    pub(crate) fn emit_growable_length(
        &mut self,
        function: &mut Function,
        handle: LirNodeId,
    ) -> EmittedValue {
        let base = self.emit_growable_receiver_handle(function, handle);
        if !base.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::I64Const(GROWABLE_HANDLE_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    /// `x[i]` read over a growable handle expression:
    /// `*( *(hdr+16) + i*8 )`. Stack-only — the index expression is emitted
    /// with the data pointer already on the stack (wasm is a stack machine;
    /// any internal scratch use by the index emission is balanced before the
    /// final add/load). In-range reads only this stage: an out-of-bounds
    /// `i >= len` read yields whatever the slot holds instead of JS
    /// `undefined` (recorded Stage 4 follow-up; no target fixture indexes
    /// out of bounds).
    pub(crate) fn emit_growable_index_read(
        &mut self,
        function: &mut Function,
        handle: LirNodeId,
        index: LirNodeId,
    ) -> EmittedValue {
        // Fail-closed: a float-valued index would put an f64 under the
        // `I32WrapI64` below (type-invalid wasm). The plain-array lane shares
        // this shape gap; reject with a diagnostic here rather than emitting
        // a module that fails validation.
        if self.is_float_valued(index) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "indexing a growable array with a floating-point value is unavailable in the current phase".to_string(),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }
        let base = self.emit_growable_receiver_handle(function, handle);
        if !base.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::I64Const(GROWABLE_HANDLE_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 16,
            align: 3,
            memory_index: 0,
        }));
        function.instruction(&Instruction::I32WrapI64);
        let index_value = self.emit_node(function, index, true);
        if !index_value.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(8));
        function.instruction(&Instruction::I32Mul);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    /// Runtime `for..of` element load `data[index]` where `index` is a wasm
    /// i64 LOCAL (not a LIR node) — the counted-loop lane (throw-fallout Stage 4
    /// Task 4). Decodes `handle` (the bare-identifier growable iterable, which
    /// resolves to the binding's handle local) to `hdr_ptr`, loads `data_ptr`
    /// (`hdr+16`), and loads the i64 element at `data_ptr + index*8`. Leaves the
    /// element on the stack. Sibling of `emit_growable_index_read`, which takes
    /// the index as a LIR node; the loop index has no LIR node, so this variant
    /// reads it straight from a local.
    pub(crate) fn emit_growable_index_read_at_local(
        &mut self,
        function: &mut Function,
        handle: LirNodeId,
        index_local: u32,
    ) -> EmittedValue {
        let base = self.emit_growable_receiver_handle(function, handle);
        if !base.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::I64Const(GROWABLE_HANDLE_MASK));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 16,
            align: 3,
            memory_index: 0,
        }));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(8));
        function.instruction(&Instruction::I32Mul);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    /// `(base_name, args)` iff `node` is a `<growable>.push(…)` member call
    /// over a bare-identifier growable receiver — the codegen half of the
    /// push recognizer (mirrors `runtime_join_call_parts`' shape). Arity is
    /// NOT checked here: the emit arm rejects a non-1 arity fail-closed
    /// (E5506) rather than silently falling through to the generic no-op.
    pub(crate) fn growable_push_call_parts(
        &self,
        node: &LirNode,
    ) -> Option<(GrowablePushReceiver, Vec<LirNodeId>)> {
        if node.kind != LirNodeKind::Call || node.children.is_empty() {
            return None;
        }
        let callee = self.resolve_transparent_callable_node(node.children[0])?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("push") {
            return None;
        }
        let receiver = callee_node.children.first().copied()?;
        let receiver_node = self.node(self.unwrap_transparent(receiver));
        let base = receiver_node.text.as_deref()?;
        // Named-binding growable receiver (bare identifier: no children).
        if receiver_node.children.is_empty() {
            return self.is_growable_array(base).then(|| {
                (
                    GrowablePushReceiver::Named(base.to_string()),
                    node.children[1..].to_vec(),
                )
            });
        }
        // Field-read growable receiver `o.values.push(v)` (Task 5): admitted
        // ONLY through the positive `object_field_is_growable_array` proof — an
        // allowlist, so a non-growable member (`o.count.push`, a string
        // `.length`, a nested `o.x.y` chain) keeps the current rejection.
        if self.object_field_is_growable_array(receiver) {
            return Some((
                GrowablePushReceiver::Field(receiver),
                node.children[1..].to_vec(),
            ));
        }
        None
    }

    /// Emit a recognized growable `.push` call. One argument appends;
    /// anything else (0 or 2+ — shapes the types promotion never admits)
    /// rejects fail-closed.
    pub(crate) fn emit_growable_push_call(
        &mut self,
        function: &mut Function,
        receiver: GrowablePushReceiver,
        args: &[LirNodeId],
    ) -> EmittedValue {
        // Arity is receiver-independent: one argument appends; anything else
        // (0 or 2+ — shapes the types promotion never admits) rejects closed.
        if args.len() != 1 {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Array.prototype.push on a growable array requires exactly one argument in the current phase".to_string(),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }
        match receiver {
            GrowablePushReceiver::Named(base_name) => {
                // Self-push guard (Stage 4 Task 4 review fix): a push onto the
                // array a runtime `for..of` is CURRENTLY iterating grows the
                // array under node but not the counted loop's once-snapshotted
                // length — a silent node-divergent miscompile. The resolve-phase
                // for..of gate already rejects this shape (its syntactic body
                // walk); this is the by-construction codegen mirror — every
                // growable push emission flows through here, so nothing the walk
                // might miss can slip past. Pushes onto a DIFFERENT binding (the
                // target fixture's `out.push(v)` inside `for (const v of o)`)
                // are unaffected.
                if self.growable_for_of_active.as_deref() == Some(base_name.as_str()) {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "pushing to growable array `{base_name}` inside a for-of loop iterating it is unavailable in the current phase (the iteration count is fixed at loop entry, diverging from JS growth semantics); use an index loop over `.length` or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                let Some(handle_local) = self.locals.get(&base_name).copied() else {
                    // No local slot: provisioning bug — fail closed, never a
                    // silent no-op.
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "growable array `{base_name}` has no local slot; push lowering is unavailable"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };
                self.emit_growable_push(function, GrowableHandle::Local(handle_local), args[0])
            }
            GrowablePushReceiver::Field(receiver_id) => {
                // Field-receiver self-push mirror (Task 5): a push onto the SAME
                // `o.values` a growable `for..of` is iterating is the same
                // once-snapshotted-length miscompile as the named case. The
                // for-of field lane records its iterable as a `base.field` key
                // in `growable_for_of_active` (a bare binding name never
                // contains `.`, so the two key spaces are disjoint); reject when
                // this push's field key matches. Resolve rejects it first; this
                // is the by-construction codegen mirror.
                if let Some(field_key) = self.growable_field_receiver_key(receiver_id) {
                    if self.growable_for_of_active.as_deref() == Some(field_key.as_str()) {
                        self.diagnostics.push(Diagnostic::error(
                            e5::FEATURE_UNAVAILABLE as u32,
                            format!(
                                "pushing to growable array field `{field_key}` inside a for-of loop iterating it is unavailable in the current phase (the iteration count is fixed at loop entry, diverging from JS growth semantics); use an index loop over `.length` or the later compatibility path"
                            ),
                        ));
                        function.instruction(&Instruction::Unreachable);
                        return EmittedValue {
                            produced: false,
                            shape: ValueShape::Unknown,
                        };
                    }
                }
                self.emit_growable_push(function, GrowableHandle::Field(receiver_id), args[0])
            }
        }
    }

    /// Growable mirror of `dynamic_array_read_base`: `Some(base)` when
    /// `node` is a member READ (`x[i]` literal/identifier 1-child form, or
    /// computed 2-child form) whose base is a growable binding. Same shape
    /// guards as the plain-lane recognizer (`.length` excluded — the length
    /// lane wins first; binary operators and static index folds excluded).
    pub(crate) fn growable_array_read_base(&self, node: &LirNode) -> Option<String> {
        match node.children.len() {
            1 => {
                let index_text = node.text.as_deref()?;
                if index_text.is_empty() || index_text == "length" {
                    return None;
                }
                let base_name = self.assignment_target_name(node, node.children[0])?;
                self.is_growable_array(&base_name).then_some(base_name)
            }
            2 => {
                if is_binary_operator_text(node.text.as_deref().unwrap_or_default()) {
                    return None;
                }
                let base_name = self.assignment_target_name(node, node.children[0])?;
                self.is_growable_array(&base_name).then_some(base_name)
            }
            _ => None,
        }
    }

    /// Field-receiver twin of `growable_array_read_base` (Task 5): `true` when
    /// `node` is a member READ (`o.values[i]`) whose base `node.children[0]` is
    /// a `GrowableArrayI64` object field. Same shape guards as the named-lane
    /// recognizer (`.length` excluded — the length lane wins first; binary
    /// operators excluded). Admits ONLY via the positive
    /// `object_field_is_growable_array` proof (allowlist), so a non-growable
    /// member read keeps its existing route. i64 elements only (Task 3 conflicts
    /// a string array field to E5506), so no string-element classification in
    /// `operators.rs` needs to consume this — those arms correctly see it as a
    /// non-string i64 read.
    pub(crate) fn growable_field_read_base(&self, node: &LirNode) -> bool {
        match node.children.len() {
            1 => {
                let index_text = node.text.as_deref().unwrap_or_default();
                if index_text.is_empty() || index_text == "length" {
                    return false;
                }
                self.object_field_is_growable_array(node.children[0])
            }
            2 => {
                if is_binary_operator_text(node.text.as_deref().unwrap_or_default()) {
                    return false;
                }
                self.object_field_is_growable_array(node.children[0])
            }
            _ => false,
        }
    }

    /// Canonical `base.field` key for a `GrowableArrayI64` field-read receiver
    /// (Task 5), used as the growable `for..of` self-push identity. Returns
    /// `None` for a receiver whose base is not a bare binding (no key ⇒ the
    /// self-push guard cannot fire, but resolve rejects that shape first). A
    /// bare binding name never contains `.`, so a field key
    /// (`"o.values"`) never collides with a named-binding key.
    pub(crate) fn growable_field_receiver_key(&self, receiver_id: LirNodeId) -> Option<String> {
        let node = self.node(self.unwrap_transparent(receiver_id));
        if node.children.len() != 1 {
            return None;
        }
        let field = node.text.as_deref().filter(|t| !t.is_empty())?;
        let base = self.assignment_target_name(node, node.children[0])?;
        Some(format!("{base}.{field}"))
    }

    /// True when any node in `id`'s subtree names a growable binding of this
    /// function (Task 6 re-review fix): the multi-argument console lowering
    /// fail-closes when an argument reads a growable array, because the
    /// dynamic console lane prints only the first argument and silently drops
    /// the rest (pre-existing lane limitation; the growable lane is new this
    /// stage, so it must not ship into it). Identifier texts in LIR are bare
    /// names; string literals keep their quotes, so a same-spelled string
    /// literal never false-positives.
    pub(crate) fn subtree_mentions_growable(&self, id: LirNodeId) -> bool {
        let node = self.node(id);
        if node
            .text
            .as_deref()
            .is_some_and(|text| self.is_growable_array(text))
        {
            return true;
        }
        // Growable-array FIELD read (`o.values`, Stage P2 Lane 1): a member
        // access whose field is a `GrowableArrayI64` slot also reads a growable
        // array, so a multi-argument `console.log` containing one must fail
        // closed too — the field twin of the named-binding detection above.
        // Without this a `console.log(o.count, o.values.length)` would silently
        // drop the growable read (the exact hole the named guard closes).
        if node.children.len() == 1 && self.object_field_is_growable_array(node.children[0]) {
            return true;
        }
        node.children
            .iter()
            .any(|child| self.subtree_mentions_growable(*child))
    }
}
