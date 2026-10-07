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
    /// Growable-runtime-arrays spec §3.5, A-6: the element repr of a growable
    /// value — a growable binding of this function, a direct call to a
    /// growable-returning function, or a `.slice(…)` of either. `None`
    /// otherwise. The codegen twin of the resolver's `growable_receiver_elem`:
    /// a call's callee is keyed through `ReprTable::array_return_callee_key`
    /// (inference's own `const`-alias and shadow facts), never by codegen's
    /// own binding-following alias walk, so the two sides cannot disagree.
    pub(crate) fn growable_value_elem(&self, id: LirNodeId) -> Option<kali_common::Repr> {
        let id = self.unwrap_transparent(id);
        if let Some(name) = self.bare_identifier_name(id) {
            return self
                .is_growable_array(&name)
                .then(|| self.array_elem_repr(&name));
        }
        let node = self.node(id);
        if node.kind != LirNodeKind::Call {
            return None;
        }
        let callee = *node.children.first()?;
        if let Some(name) = self.bare_identifier_name(callee) {
            return self
                .repr_table
                .array_return_callee_key(&self.function_name, &name)
                .and_then(|key| self.repr_table.growable_return(key));
        }
        let member = self.node(self.resolve_transparent_callable_node(callee)?);
        if member.text.as_deref() == Some("slice") && member.children.len() == 1 {
            return self.growable_value_elem(member.children[0]);
        }
        None
    }

    /// Growable-runtime-arrays spec §3.4: turn the value just emitted for an
    /// element slot (`produced`; `value` its LIR node) into the slot's i64 bit
    /// pattern for an array of element repr `elem`. An f64 element is stored
    /// bit-reinterpreted; an integer value bound for an f64 array is converted
    /// first (a missing value was padded with `i64.const 0`, an integer). i64
    /// and string elements are stored as-is. The one store-side encoder every
    /// growable slot write (seed, push, index write) shares.
    pub(crate) fn emit_growable_slot_encode(
        &self,
        function: &mut Function,
        elem: kali_common::Repr,
        value: LirNodeId,
        produced: bool,
    ) {
        if elem == kali_common::Repr::F64 {
            if !produced || !self.is_float_valued(value) {
                function.instruction(&Instruction::F64ConvertI64S);
            }
            function.instruction(&Instruction::I64ReinterpretF64);
        }
    }

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
    /// block, `len = seed_len`, through `alloc_callee_index()` (arena lane)
    /// or, when `global` (growable-runtime-arrays spec §3.3: the array leaves
    /// its creating function), through `__alloc_global` (never reclaimed).
    /// Leaves the TAGGED i64 handle on the stack; the header pointer is also
    /// left in the dedicated growable scratch local so the declarator can
    /// store seed elements. Seed VALUES are the caller's job (they need the
    /// declarator's element nodes).
    pub(crate) fn emit_growable_alloc(
        &mut self,
        function: &mut Function,
        seed_len: usize,
        cap: usize,
        global: bool,
    ) -> EmittedValue {
        let scratch = self.growable_scratch_local();
        let alloc = if global {
            self.alloc_global_fn_index()
        } else {
            self.alloc_callee_index()
        };

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
        let allocated = self.emit_growable_alloc(function, seed_len, cap, false);
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
    /// LENGTH on the stack (JS `push` returns it). `elem` is the array's
    /// element repr (an f64 value is stored bit-reinterpreted, spec §3.4);
    /// `global` (spec §3.3: the array leaves its creating function) makes the
    /// grown data block come from `__alloc_global`.
    pub(crate) fn emit_growable_push(
        &mut self,
        function: &mut Function,
        handle: GrowableHandle,
        value: LirNodeId,
        elem: kali_common::Repr,
        global: bool,
    ) -> EmittedValue {
        let value_is_float = self.is_float_valued(value);
        // Belt: inference makes any array holding a float an f64 array, so a
        // float value never reaches an integer or string array.
        if value_is_float && elem != kali_common::Repr::F64 {
            return self.deny_e5506(
                function,
                "pushing a floating-point value onto a growable array of integers or strings is unavailable in the current phase",
            );
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
        // analysis coupling. An escaping array (`global`, spec §3.3) always
        // grows into global memory. Otherwise the existing function-level
        // arena lane applies (`alloc_callee_index`).
        let alloc = if global
            || self
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
        self.emit_growable_slot_encode(function, elem, value, produced.produced);
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

    /// Push the growable value `handle`'s tagged handle, then `index` as an
    /// i64 — the operand prefix every growable element read and write shares,
    /// evaluated in JS order. A float-valued index is refused here (E5506) for
    /// both lanes: it would put an f64 under the i64 index operand
    /// (type-invalid wasm). Returns `false` after a refusal (the stack is
    /// then polymorphic: `deny_e5506` emits `unreachable`).
    fn emit_growable_handle_and_index(
        &mut self,
        function: &mut Function,
        handle: LirNodeId,
        index: LirNodeId,
    ) -> bool {
        if self.is_float_valued(index) {
            let _ = self.deny_e5506(
                function,
                "indexing a growable array with a floating-point value is unavailable in the current phase",
            );
            return false;
        }
        let base = self.emit_growable_receiver_handle(function, handle);
        if !base.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        // Stage P5 T-new-E: a `String()`-result index on a growable array is a
        // numeric-consumption sink; route through the numeric-materialization
        // choke so it fails closed instead of reading a garbage element.
        let index_value = self.emit_numeric_operand(function, index);
        if !index_value.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        true
    }

    /// Push the kali bounds-trap message handle (the `msg` operand of the
    /// growable bounds guards).
    fn emit_growable_bounds_message(&mut self, function: &mut Function) {
        let (offset, len) = self
            .strings
            .intern(kali_common::runtime_array_index_out_of_bounds_message());
        function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
    }

    /// Growable-runtime-arrays spec §3.5: push the i32 address of element
    /// `index` of the growable value `handle`, bounds-checked by
    /// `__growable_elem_addr` (`index < 0` or `index >= length` traps with the
    /// kali bounds message). Load at `offset` 0. READS only: a write must not
    /// hold a slot address across its right-hand side (which may grow the
    /// array and move the data block) — see `emit_growable_index_write`.
    /// Returns `false` after a float-index refusal.
    pub(crate) fn emit_growable_element_address(
        &mut self,
        function: &mut Function,
        handle: LirNodeId,
        index: LirNodeId,
    ) -> bool {
        if !self.emit_growable_handle_and_index(function, handle, index) {
            return false;
        }
        self.emit_growable_bounds_message(function);
        function.instruction(&Instruction::Call(self.growable_elem_addr_fn_index()));
        function.instruction(&Instruction::I32WrapI64);
        true
    }

    /// `base[index] = value` on a growable binding (spec §3.5). The handle,
    /// the index and the value are evaluated first, in JS order; the bounds
    /// check, the data pointer and the store then happen together in
    /// `__growable_store`, so a right-hand side that grows the array stores
    /// into the live data block and is checked against the live length.
    /// `index == length` traps too (node would append). Leaves the stored
    /// value (the assignment expression's result).
    pub(crate) fn emit_growable_index_write(
        &mut self,
        function: &mut Function,
        base: LirNodeId,
        index: LirNodeId,
        value: LirNodeId,
    ) {
        let elem = self
            .growable_value_elem(base)
            .unwrap_or(kali_common::Repr::I64);
        if !self.emit_growable_handle_and_index(function, base, index) {
            return;
        }
        let rhs = self.emit_node(function, value, true);
        if !rhs.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        self.emit_growable_slot_encode(function, elem, value, rhs.produced);
        self.emit_growable_bounds_message(function);
        function.instruction(&Instruction::Call(self.growable_store_fn_index()));
        if elem == kali_common::Repr::F64 {
            function.instruction(&Instruction::F64ReinterpretI64);
        }
    }

    /// `x[i]` read over a growable handle expression (spec §3.5): the element
    /// address comes from the bounds guard `__growable_elem_addr`, so an
    /// index outside `0 <= i < length` traps with the kali bounds message
    /// (node yields `undefined`), and a float index is refused (E5506). An
    /// f64 element is reinterpreted back from its slot bits.
    pub(crate) fn emit_growable_index_read(
        &mut self,
        function: &mut Function,
        handle: LirNodeId,
        index: LirNodeId,
    ) -> EmittedValue {
        // Field receivers are i64 (the object-field lane); named and call
        // receivers carry their element repr.
        let elem = self
            .growable_value_elem(handle)
            .unwrap_or(kali_common::Repr::I64);
        if !self.emit_growable_element_address(function, handle, index) {
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
        if elem == kali_common::Repr::F64 {
            function.instruction(&Instruction::F64ReinterpretI64);
        }
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

    /// Emit a recognized growable `.push` call (growable-runtime-arrays spec
    /// §3.5): each argument is appended in order and the call's value is the
    /// last new length; `push()` with no arguments stores nothing and yields
    /// the length.
    pub(crate) fn emit_growable_push_call(
        &mut self,
        function: &mut Function,
        receiver: GrowablePushReceiver,
        args: &[LirNodeId],
    ) -> EmittedValue {
        if args.is_empty() {
            let handle = match &receiver {
                GrowablePushReceiver::Named(name) => {
                    self.alloc_scratch_node(LirNodeKind::Value, Some(name.clone()), vec![])
                }
                GrowablePushReceiver::Field(id) => *id,
            };
            return self.emit_growable_length(function, handle);
        }
        // Stage P5 T-new-A (review finding I-3): `g.push(fb)` is the growable
        // twin of the aggregate store — the handle lands in a slot whose later
        // read has no binding name (`g[0].length` printed the growable's
        // length, `2`, where node reads `4`). Same lane, same close.
        if args
            .iter()
            .any(|arg| self.is_crypto_random_result_value(*arg))
        {
            return self.deny_e5506(function, Self::CRYPTO_RANDOM_RESULT_STORE_DENY);
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
                let elem = self.array_elem_repr(&base_name);
                let global = !self
                    .repr_table
                    .is_growable_local_only(&self.function_name, &base_name);
                let mut last = EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
                for (i, arg) in args.iter().copied().enumerate() {
                    if i > 0 {
                        function.instruction(&Instruction::Drop);
                    }
                    last = self.emit_growable_push(
                        function,
                        GrowableHandle::Local(handle_local),
                        arg,
                        elem,
                        global,
                    );
                }
                last
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
                // The object-field lane stays i64 and arena-allocated.
                let mut last = EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
                for (i, arg) in args.iter().copied().enumerate() {
                    if i > 0 {
                        function.instruction(&Instruction::Drop);
                    }
                    last = self.emit_growable_push(
                        function,
                        GrowableHandle::Field(receiver_id),
                        arg,
                        kali_common::Repr::I64,
                        false,
                    );
                }
                last
            }
        }
    }

    /// Growable mirror of `dynamic_array_read_base`: `Some(base)` when
    /// `node` is a member READ (`x[i]` literal/identifier 1-child form, or
    /// computed 2-child form) whose base is a growable binding. Same shape
    /// guards as the plain-lane recognizer (`.length` excluded — the length
    /// lane wins first; binary operators and static index folds excluded).
    /// Growable-runtime-arrays spec §3.5: the element repr of a growable
    /// index write `a[i] = v` used as a value (`f[1] = (f[0] = 0.25)`,
    /// `const r = (f[0] = …)`). The write leaves the stored value in the
    /// element's own repr (`emit_growable_index_write`), so the value oracles
    /// (`is_float_valued`, `is_string_valued`) classify the assignment by it.
    /// Same target recognizer as the read lane (`growable_array_read_base`)
    /// over the same store-target view `emit_assignment` takes.
    pub(crate) fn growable_index_write_elem(&self, node: &LirNode) -> Option<kali_common::Repr> {
        if node.kind != LirNodeKind::Value
            || node.children.len() != 2
            || node.text.as_deref() != Some("=")
        {
            return None;
        }
        let target = self.store_target_node(node.children[0]);
        if target.kind != LirNodeKind::Value {
            return None;
        }
        let base = self.growable_array_read_base(&target)?;
        Some(self.array_elem_repr(&base))
    }

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

    /// True when any node in `id`'s subtree reads a growable object FIELD
    /// (`o.values`, Stage P2 Lane 1: a member access whose field is a
    /// `GrowableArrayI64` slot). The multi-argument console lowering keeps its
    /// pre-existing refusal for such reads (`soundness/structured_clone.toml`
    /// pins it); named growable arrays print through the multi-argument lane
    /// (growable-runtime-arrays spec A-9).
    pub(crate) fn subtree_mentions_growable_field(&self, id: LirNodeId) -> bool {
        let node = self.node(id);
        if node.children.len() == 1 && self.object_field_is_growable_array(node.children[0]) {
            return true;
        }
        node.children
            .iter()
            .any(|child| self.subtree_mentions_growable_field(*child))
    }
}

/// A growable method this lane lowers (growable-runtime-arrays spec §3.5);
/// `push` and `join` have their own recognizers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GrowableMethod {
    Pop,
    IndexOf,
    Includes,
    Slice,
}

impl<'a> FunctionEmitter<'a> {
    /// `(method, receiver, args)` iff `node` is `<growable value>.<method>(…)`.
    pub(crate) fn growable_method_call_parts(
        &self,
        node: &LirNode,
    ) -> Option<(GrowableMethod, LirNodeId, Vec<LirNodeId>)> {
        if node.kind != LirNodeKind::Call || node.children.is_empty() {
            return None;
        }
        let callee = self.resolve_transparent_callable_node(node.children[0])?;
        let callee_node = self.node(callee);
        if callee_node.children.len() != 1 {
            return None;
        }
        let method = match callee_node.text.as_deref()? {
            "pop" => GrowableMethod::Pop,
            "indexOf" => GrowableMethod::IndexOf,
            "includes" => GrowableMethod::Includes,
            "slice" => GrowableMethod::Slice,
            _ => return None,
        };
        let receiver = callee_node.children[0];
        self.growable_value_elem(receiver)?;
        Some((method, receiver, node.children[1..].to_vec()))
    }

    /// `a.pop()`: the last element, removed; an empty array traps with the
    /// kali pop message.
    pub(crate) fn emit_growable_pop(
        &mut self,
        function: &mut Function,
        receiver: LirNodeId,
    ) -> EmittedValue {
        let elem = self
            .growable_value_elem(receiver)
            .unwrap_or(kali_common::Repr::I64);
        // Belt for inference's snapshot refusal (spec A-8).
        if let Some(name) = self.bare_identifier_name(receiver) {
            if self.growable_for_of_active.as_deref() == Some(name.as_str()) {
                let message = kali_common::growable_for_of_mutation_message(
                    &kali_common::growable_binding_subject(&name, &self.function_name),
                );
                return self.deny_e5506(function, &message);
            }
        }
        let base = self.emit_growable_receiver_handle(function, receiver);
        if !base.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        let (offset, len) = self
            .strings
            .intern(kali_common::growable_pop_empty_message());
        function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
        function.instruction(&Instruction::Call(self.growable_pop_fn_index()));
        if elem == kali_common::Repr::F64 {
            function.instruction(&Instruction::F64ReinterpretI64);
        }
        EmittedValue {
            produced: true,
            shape: if elem == kali_common::Repr::String {
                ValueShape::String
            } else {
                ValueShape::Scalar
            },
        }
    }

    /// `a.indexOf(x)` (strict) / `a.includes(x)` (SameValueZero).
    pub(crate) fn emit_growable_search(
        &mut self,
        function: &mut Function,
        receiver: LirNodeId,
        needle: Option<LirNodeId>,
        includes: bool,
    ) -> EmittedValue {
        let elem = self
            .growable_value_elem(receiver)
            .unwrap_or(kali_common::Repr::I64);
        let base = self.emit_growable_receiver_handle(function, receiver);
        if !base.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        let Some(needle) = needle else {
            // `indexOf()` searches for `undefined`, which a number or string
            // array never holds.
            function.instruction(&Instruction::Drop);
            function.instruction(&Instruction::I64Const(if includes { 0 } else { -1 }));
            return EmittedValue {
                produced: true,
                shape: if includes {
                    ValueShape::Boolean
                } else {
                    ValueShape::Scalar
                },
            };
        };
        let value = self.emit_node(function, needle, true);
        if !value.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        // The needle is encoded exactly as a stored slot (spec A-14: the
        // search value is an element store), so `__growable_find` compares
        // slot bits.
        self.emit_growable_slot_encode(function, elem, needle, value.produced);
        let mode = match elem {
            kali_common::Repr::F64 if includes => 2,
            kali_common::Repr::F64 => 1,
            kali_common::Repr::String => 3,
            _ => 0,
        };
        function.instruction(&Instruction::I64Const(mode));
        function.instruction(&Instruction::Call(self.growable_find_fn_index()));
        if includes {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GeS);
            function.instruction(&Instruction::I64ExtendI32U);
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    /// `a.slice(s?, e?)`: a new growable array (spec §3.5). A float bound is
    /// truncated with `i64.trunc_sat_f64_s` (ToIntegerOrInfinity, A-13).
    pub(crate) fn emit_growable_slice(
        &mut self,
        function: &mut Function,
        receiver: LirNodeId,
        args: &[LirNodeId],
    ) -> EmittedValue {
        let base = self.emit_growable_receiver_handle(function, receiver);
        if !base.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        for (position, default) in [(0usize, 0i64), (1, i64::MAX)] {
            match args.get(position).copied() {
                None => {
                    function.instruction(&Instruction::I64Const(default));
                }
                Some(bound) => {
                    let value = self.emit_node(function, bound, true);
                    if !value.produced {
                        function.instruction(&Instruction::I64Const(default));
                    } else if self.is_float_valued(bound) {
                        function.instruction(&Instruction::I64TruncSatF64S);
                    }
                }
            }
        }
        function.instruction(&Instruction::Call(self.growable_slice_fn_index()));
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }
}

#[cfg(test)]
#[path = "growable_tests.rs"]
mod growable_tests;
