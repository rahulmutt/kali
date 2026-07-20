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
use wasm_encoder::{Function, Instruction, MemArg};

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
