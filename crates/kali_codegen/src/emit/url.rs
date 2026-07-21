//! URL + URLSearchParams emission (Stage P4). URL is a fixed 6-slot arena
//! struct of interned component handles + an embedded USP; USP is a growable
//! `[k0,v0,…]` pair-store. Handles are escape-restricted: every read flows
//! through the position-allowlist entry points here (or the declarator
//! intercept in `control_flow.rs`) or fails closed E5506.
//!
//! URL struct layout (48 bytes, arena `__alloc`, i64 slots at `slot*8`):
//! ```text
//! @+0  href        : string handle
//! @+8  origin      : string handle
//! @+16 pathname    : string handle
//! @+24 search      : string handle
//! @+32 hash        : string handle
//! @+40 searchParams: URLSearchParams handle (tagged growable pair-store)
//! ```
//! The handle for the whole URL is the i64 zero-extended struct pointer.
//! USP store layout is the standard growable `[len][cap][data_ptr]` header
//! (`emit_growable_alloc`) whose data block is `[k0,v0,k1,v1,…]` interned i64
//! string handles.

use crate::*;
use wasm_encoder::{BlockType, Function, Instruction, MemArg};

/// The receiver of a recognized URLSearchParams method call (Stage P4 Task 4):
/// a bare proven USP binding, or `<url-ident>.searchParams`. Each carries the
/// LIR node the store handle is emitted from (see [`FunctionEmitter::usp_receiver`]).
pub(crate) enum UspReceiver {
    /// Bare proven USP binding — the carried node is the USP identifier.
    Named(LirNodeId),
    /// `<url>.searchParams` — the carried node is the URL base; a slot-5 load
    /// after emitting it yields the embedded USP store handle.
    OfUrl(LirNodeId),
}

/// A recognized URL component member read. Five yield an interned string
/// handle; `SearchParams` yields the embedded USP handle.
pub(crate) enum UrlMember {
    Href,
    Origin,
    Pathname,
    Search,
    Hash,
    SearchParams,
}

impl UrlMember {
    /// Byte-offset slot index in the URL struct (see module layout).
    fn slot(&self) -> u64 {
        match self {
            UrlMember::Href => 0,
            UrlMember::Origin => 1,
            UrlMember::Pathname => 2,
            UrlMember::Search => 3,
            UrlMember::Hash => 4,
            UrlMember::SearchParams => 5,
        }
    }
}

impl<'a> FunctionEmitter<'a> {
    /// `<url-ident>.<component>` where the base is a proven URL local and the
    /// property is one of the six known members. Returns `(base_id, member)`.
    /// Any other field, or an unproven / non-URL base, returns `None` (falls
    /// through to the generic member paths, whose receiver emit hits the
    /// identifier choke point and denies E5506 — default-deny).
    pub(crate) fn url_member_read_parts(&self, node: LirNodeId) -> Option<(LirNodeId, UrlMember)> {
        let n = self.node(node);
        if n.kind != LirNodeKind::Value || n.children.len() != 1 {
            return None;
        }
        let base_id = n.children[0];
        let base = self.node(base_id);
        if !base.children.is_empty() {
            return None;
        }
        let base_name = base.text.as_deref()?;
        if !self.is_url(base_name) {
            return None;
        }
        Some((
            base_id,
            match n.text.as_deref()? {
                "href" => UrlMember::Href,
                "origin" => UrlMember::Origin,
                "pathname" => UrlMember::Pathname,
                "search" => UrlMember::Search,
                "hash" => UrlMember::Hash,
                "searchParams" => UrlMember::SearchParams,
                _ => return None,
            },
        ))
    }

    /// The single position-allowlist entry point for URL/USP receiver loads
    /// (mirrors `emit_abort_receiver_handle`): sets the position-allowlist flag,
    /// emits the receiver, restores the flag. Any URL/USP handle read NOT coming
    /// through here stays denied E5506 (default-deny).
    pub(crate) fn emit_url_receiver_handle(
        &mut self,
        function: &mut Function,
        receiver: LirNodeId,
    ) -> EmittedValue {
        let previous = self.admit_url_handle_read;
        self.admit_url_handle_read = true;
        let value = self.emit_node(function, receiver, true);
        self.admit_url_handle_read = previous;
        value
    }

    /// With a URL handle (i64 struct pointer) on the stack: load slot `slot`
    /// (byte offset `slot*8`), leaving the i64 slot value on the stack.
    pub(crate) fn emit_url_slot_load(&mut self, function: &mut Function, slot: u64) {
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: slot * 8,
            align: 3,
            memory_index: 0,
        }));
    }

    /// Store the compile-time i64 `value` into URL struct slot `slot` (byte
    /// offset `slot*8`); the struct base pointer (i64) is read from
    /// `struct_scratch`. The store-twin of `emit_url_slot_load`.
    pub(crate) fn emit_url_slot_store(
        &self,
        function: &mut Function,
        struct_scratch: u32,
        slot: u64,
        value: i64,
    ) {
        function.instruction(&Instruction::LocalGet(struct_scratch));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Const(value));
        function.instruction(&Instruction::I64Store(MemArg {
            offset: slot * 8,
            align: 3,
            memory_index: 0,
        }));
    }

    /// Full URL emission for a `const u = new URL(<string-literal>)` declarator:
    /// alloc the 48-byte struct, store the five interned string components at
    /// slots 0..5, build the embedded USP into slot 5, and leave the i64 struct
    /// handle on the stack for the declarator's bind. Uses the general-purpose
    /// trailing scratch (`self.locals.len()`) for the struct pointer; the USP
    /// builder uses the dedicated growable scratch (a different local), so the
    /// two never collide.
    pub(crate) fn emit_url_construction(
        &mut self,
        function: &mut Function,
        components: &crate::lower::UrlComponents,
    ) {
        let alloc = self.alloc_callee_index();
        let struct_scratch = self.locals.len() as u32;
        // hdr = __alloc(48), zero-extended into the struct scratch.
        function.instruction(&Instruction::I32Const(48));
        function.instruction(&Instruction::Call(alloc));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(struct_scratch));

        // Slots 0..5: interned string component handles.
        for (slot, text) in [
            &components.href,
            &components.origin,
            &components.pathname,
            &components.search,
            &components.hash,
        ]
        .into_iter()
        .enumerate()
        {
            let (offset, len) = self.strings.intern(text);
            self.emit_url_slot_store(
                function,
                struct_scratch,
                slot as u64,
                crate::encode_string_handle(offset, len),
            );
        }

        // Slot 5: the embedded USP (address pushed first, then the USP builder
        // leaves the tagged handle on top; `I64Store` pops handle then address).
        function.instruction(&Instruction::LocalGet(struct_scratch));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_usp_store_from_pairs(function, &components.query_pairs);
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 40,
            align: 3,
            memory_index: 0,
        }));

        // The i64 struct handle onto the stack for the declarator bind.
        function.instruction(&Instruction::LocalGet(struct_scratch));
    }

    /// Build a growable `[k0,v0,…]` store from compile-time pairs; leaves the
    /// tagged handle on the stack. Interns each key/value string. Models the
    /// growable seed path (`emit_growable_alloc` then `data[i] = seed_i` via the
    /// dedicated growable scratch's header pointer, which keeps the handle on
    /// the stack undisturbed — the `emit_growable_field_value` idiom).
    pub(crate) fn emit_usp_store_from_pairs(
        &mut self,
        function: &mut Function,
        pairs: &[(String, String)],
    ) {
        let mut flat: Vec<i64> = Vec::with_capacity(pairs.len() * 2);
        for (k, v) in pairs {
            let (ko, kl) = self.strings.intern(k);
            let (vo, vl) = self.strings.intern(v);
            flat.push(crate::encode_string_handle(ko, kl));
            flat.push(crate::encode_string_handle(vo, vl));
        }
        let seed_len = flat.len();
        let cap = seed_len.max(crate::emit::growable::GROWABLE_INITIAL_CAP);
        // Leaves the tagged handle on the stack and the header pointer in the
        // dedicated growable scratch.
        let allocated = self.emit_growable_alloc(function, seed_len, cap);
        if !allocated.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        // Seed each element via `data_ptr + i*8` (data_ptr read from the header
        // in the dedicated scratch, so the handle on the stack stays put).
        let scratch = self.growable_scratch_local();
        for (i, value) in flat.into_iter().enumerate() {
            self.emit_growable_scratch_hdr(function, scratch);
            function.instruction(&Instruction::I64Load(MemArg {
                offset: 16,
                align: 3,
                memory_index: 0,
            }));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I64Const(value));
            function.instruction(&Instruction::I64Store(MemArg {
                offset: (i * 8) as u64,
                align: 3,
                memory_index: 0,
            }));
        }
    }

    /// Slot index for a URL member (exposed for the component-read emit arm).
    pub(crate) fn url_member_slot(member: &UrlMember) -> u64 {
        member.slot()
    }

    /// The receiver of a method call recognized as a URLSearchParams: either a
    /// bare proven USP binding, or `<url-ident>.searchParams`. Both variants
    /// carry the LIR node whose emit (through the admit path) yields the store
    /// handle — `Named` the USP identifier node, `OfUrl` the URL base node
    /// (whose slot-5 load then yields the embedded USP). (The brief's
    /// `Named(String)` is realized as the ident NODE so `emit_usp_store_handle`
    /// can route both through the single `emit_url_receiver_handle` admit path.)
    pub(crate) fn usp_receiver(&self, callee_node: &LirNode) -> Option<UspReceiver> {
        let recv = callee_node.children.first().copied()?;
        let rn = self.node(recv);
        if rn.children.is_empty() {
            let name = rn.text.as_deref()?;
            return self
                .is_url_search_params(name)
                .then_some(UspReceiver::Named(recv));
        }
        if let Some((base_id, UrlMember::SearchParams)) = self.url_member_read_parts(recv) {
            return Some(UspReceiver::OfUrl(base_id));
        }
        None
    }

    /// Emit the USP store handle (the tagged growable pair-store, i64) onto the
    /// stack. Both variants flow through the single position-allowlist entry
    /// point `emit_url_receiver_handle` (which sets `admit_url_handle_read`),
    /// so the composition reaches slot 5 WITHOUT going through the (denying)
    /// bare component-read arm in `control_flow.rs`.
    pub(crate) fn emit_usp_store_handle(&mut self, function: &mut Function, recv: &UspReceiver) {
        match recv {
            UspReceiver::Named(ident) => {
                let handle = self.emit_url_receiver_handle(function, *ident);
                if !handle.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
            }
            UspReceiver::OfUrl(base_id) => {
                let handle = self.emit_url_receiver_handle(function, *base_id);
                if !handle.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
                self.emit_url_slot_load(function, 5);
            }
        }
    }

    /// `q.append(k, v)` inline growable push of ONE element onto the USP store
    /// (called twice — key then value). Self-contained mirror of
    /// `emit_growable_push` sourcing the handle from `emit_usp_store_handle`
    /// (the store lives in a binding local for `Named`, in URL slot 5 for
    /// `OfUrl` — neither a `GrowableHandle::Local`/`Field`, so the proven push
    /// idiom is reproduced here). The header indirection keeps the store handle
    /// stable across a realloc, so the two sequential appends compose (the
    /// second re-reads the same header and sees the updated `data_ptr`). Leaves
    /// NOTHING on the stack — append returns undefined.
    pub(crate) fn emit_usp_append(
        &mut self,
        function: &mut Function,
        recv: &UspReceiver,
        value: LirNodeId,
    ) {
        let scratch = self.growable_scratch_local();
        let generic_scratch = self.locals.len() as u32;
        let value_scratch = generic_scratch + 1;
        // Realloc allocator: mirror `emit_growable_push` — a push inside a
        // per-iteration loop arena must reallocate through the global heap so
        // the replacement data block survives the end-of-iteration reset.
        let alloc = if self
            .arena_frames
            .iter()
            .any(|frame| frame.loop_frame_index.is_some())
        {
            self.alloc_global_fn_index()
        } else {
            self.alloc_callee_index()
        };

        // v evaluated FIRST (JS arg order; also frees the generic scratch slots).
        let produced = self.emit_node(function, value, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::LocalSet(value_scratch));

        // hdr (masked, i64) into the dedicated growable scratch.
        self.emit_usp_store_handle(function, recv);
        function.instruction(&Instruction::I64Const(
            crate::emit::growable::GROWABLE_HANDLE_MASK,
        ));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(scratch));

        // if (len == cap) grow (cap*2, memory.copy of the live prefix).
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
            // new_data = alloc(cap * 2 * 8)
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

            // memory.copy(dst = new_data, src = data_ptr, n = len * 8)
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

            // hdr.data_ptr = new_data
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
    }

    /// Emit a USP method string argument through the normal expression path
    /// (a string literal or an ordinary String handle — NO admit flag). Pushes
    /// a `0` sentinel if the argument did not produce a value, keeping the value
    /// stack balanced for the synthetic call.
    pub(crate) fn emit_usp_method_argument(&mut self, function: &mut Function, arg: LirNodeId) {
        let produced = self.emit_node(function, arg, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
    }

    /// `true` iff `node_id` is a `<usp>.getAll(<key>)` call — the base whose
    /// growable result backs a `.length` read (`q.getAll('k').length`). Lets the
    /// `.length` lane route to `emit_growable_length` over the call node (whose
    /// emit leaves the tagged growable handle), since a call result is neither a
    /// named growable binding nor a growable field.
    pub(crate) fn is_usp_getall_call(&self, node_id: LirNodeId) -> bool {
        let node = self.node(self.unwrap_transparent(node_id));
        if node.kind != LirNodeKind::Call {
            return false;
        }
        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("getAll") {
            return false;
        }
        self.usp_receiver(callee_node).is_some()
    }

    /// Walk a member-access chain (dot AND computed, ANY depth) down to its
    /// ROOT identifier and report whether that root has URL/USP provenance in
    /// THIS emitter's view (local proof, module-scope twin, or captured twin).
    /// Task-6 review fix: the generic call fallback never emits a method call's
    /// receiver, so a receiver PATH the recognizers don't admit
    /// (`u['searchParams'].get(...)`, `u.searchParams.x.get(...)`) previously
    /// reached the placeholder terminals without ANY choke firing — silent `0`
    /// where node produces a value. Keying the deny on the root binding's
    /// provenance (not on the path shape) makes every receiver spelling over a
    /// URL/USP root deny by construction.
    ///
    /// Chain shapes (LIR): a childless `Value` with text = the root identifier;
    /// a 1-child `Value` with non-empty text = dot member; a 2-child `Value`
    /// whose text is not a binary operator = computed member `a[expr]`. Any
    /// other shape terminates the walk with no root (`false` — the deny stays
    /// scoped to provable URL/USP roots).
    pub(crate) fn receiver_root_is_url_provenance(&self, receiver: LirNodeId) -> bool {
        let mut current = self.unwrap_transparent(receiver);
        loop {
            let node = self.node(current);
            if node.kind != LirNodeKind::Value {
                return false;
            }
            match node.children.len() {
                0 => {
                    return node.text.as_deref().is_some_and(|name| {
                        self.is_url(name)
                            || self.is_url_search_params(name)
                            || self.is_module_scope_url_handle(name)
                            || self.is_captured_url_handle(name)
                    });
                }
                1 if node.text.as_deref().is_some_and(|text| !text.is_empty()) => {
                    current = self.unwrap_transparent(node.children[0]);
                }
                2 if !crate::lower::is_binary_operator_text(
                    node.text.as_deref().unwrap_or_default(),
                ) =>
                {
                    current = self.unwrap_transparent(node.children[0]);
                }
                _ => return false,
            }
        }
    }

    /// The five-namespace shadow guard for a URL/USP builtin constructor name
    /// (mirrors `is_event_target_new` / `instanceof_right_is_unshadowed`): a
    /// user binding of `URL`/`URLSearchParams` in ANY codegen namespace refutes
    /// the builtin interception and the `new` falls through to the normal call
    /// lane.
    pub(crate) fn url_ctor_unshadowed(&self, ctor: &str) -> bool {
        !(self.locals.contains_key(ctor)
            || self.bindings.contains_key(ctor)
            || self.module_binding_names.contains(ctor)
            || self.fn_valued_locals.contains_key(ctor)
            || self.functions.contains_key(ctor))
    }
}
