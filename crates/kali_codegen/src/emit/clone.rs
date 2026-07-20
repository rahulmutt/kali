//! Stage P2 Lane 2: per-shape deep-clone synthetic `__clone_shape_<ShapeId>`.
//! Allocates a fresh object of the same shape, copies scalar slots verbatim,
//! and deep-copies growable-i64 array fields into fresh handles so the clone
//! shares no mutable storage with the source. Emitted only for shapes whose
//! every field is in the P2 allowlist (scalar or GrowableArrayI64); the call
//! site (Task 8) gates that before requesting emission.
//!
//! The body is HAND-EMITTED (not lowered from LIR), so it lives here as a free
//! function writing raw instructions with fixed local indices — the same idiom
//! as `emit_join_growable_body` (lower.rs). Its local layout (all i64; local 0
//! is the `src` param):
//!   1 = dst      new object base pointer (zext i32 → i64)
//!   2 = srch     source growable handle (tagged), reused per growable field
//!   3 = new_hdr  fresh growable header pointer (zext)
//!   4 = new_data fresh growable data-block pointer (zext)
//!   5 = len      source growable length
//!   6 = cap      new capacity (`max(len, GROWABLE_INITIAL_CAP)`)
//! These must stay in sync with the `local_decls` reservation in
//! `lower.rs` (`__clone_shape_` branch: `(6, ValType::I64)`).
use crate::*;

/// The number of hand-emitted i64 locals a `__clone_shape_N` body declares
/// (indices 1..=6; local 0 is the `src` param). Shared with the `local_decls`
/// reservation in `lower.rs` so the two never drift.
pub(crate) const CLONE_SHAPE_LOCAL_COUNT: u32 = 6;

/// Name of the deep-clone synthetic for `shape`: `__clone_shape_<ShapeId>`.
/// Task 8's `structuredClone` dispatch resolves the callee index through
/// `function_name_to_index[&clone_shape_synthetic_name(shape)]`.
pub(crate) fn clone_shape_synthetic_name(shape: kali_common::ShapeId) -> String {
    format!("__clone_shape_{}", shape.0)
}

/// Strict ALLOWLIST gate for the P2 deep-clone envelope: every field of the
/// shape must be a `I64`/`F64` scalar or a `GrowableArrayI64` array. `Object(_)`
/// (a nested object) and `String` fields are NOT clonable by
/// [`emit_clone_shape_body`] (its `else` arm verbatim-copies the 8-byte slot,
/// which would SHALLOW-SHARE a nested object's pointer and be a soundness bug) —
/// so the call site (Task 8) must reject such a shape fail-closed and never
/// request a synthetic for it. Shared by the emit-time dispatch gate
/// (`FunctionEmitter::shape_is_clone_envelope`) and the plan-time collection
/// scan (`collect_requested_clone_shapes`) so the two never disagree on which
/// shapes get a `__clone_shape_N` slot.
pub(crate) fn fields_are_clone_envelope(fields: &[(String, kali_common::Repr)]) -> bool {
    fields.iter().all(|(_, repr)| {
        matches!(
            repr,
            kali_common::Repr::I64 | kali_common::Repr::F64 | kali_common::Repr::GrowableArrayI64
        )
    })
}

/// Parse the `ShapeId` back out of a `__clone_shape_<n>` synthetic name.
/// Returns `None` for any name lacking the prefix or a valid numeric suffix.
pub(crate) fn clone_shape_id_from_name(name: &str) -> Option<kali_common::ShapeId> {
    name.strip_prefix("__clone_shape_")
        .and_then(|n| n.parse::<u32>().ok())
        .map(kali_common::ShapeId)
}

/// Emit the hand-written body of `__clone_shape_N` for the fixed layout
/// `fields` (in shape order, field `i` at byte offset `i * 8`). `alloc_index`
/// is the allocator the object + its fresh array blocks are bump-allocated
/// through. Task 8 wires the escape-safe GLOBAL allocator (`__alloc_global`)
/// here — a `structuredClone` result escapes its arena, so it must not dangle
/// across an arena reset (mirrors the `__join`-global / `__join_arena` split).
///
/// The body leaves the new object's base pointer (i64, untagged) on the stack;
/// the dispatch loop in `lower.rs` appends the trailing `End` (NO `End` here —
/// same contract as every hand-emitted synthetic).
///
/// Field handling (P2 allowlist — the call site gates emission to these):
///   * `GrowableArrayI64`: load the source tagged handle, allocate a FRESH
///     header + data block through the SAME 24-byte `[len][cap][data_ptr]`
///     layout and `ARRAY_HANDLE_TAG` handle encoding the growable lane uses
///     (`emit_growable_alloc` offsets 0/8/16, mirrored here), bulk-copy the
///     live `len * 8` bytes with `memory.copy` (the same idiom the growable
///     realloc uses), and store the new tagged handle. The clone shares NO
///     mutable array storage with the source.
///   * everything else (I64 / F64 / String scalar slot): a verbatim 8-byte
///     slot copy — correct for numbers and immutable string handles alike.
///     (`Repr::Object` nested-object fields are NOT in the P2 allowlist and
///     Task 8 must not request a clone for a shape containing one; a verbatim
///     copy of such a slot would be a shallow share.)
pub(crate) fn emit_clone_shape_body(
    func: &mut Function,
    fields: &[(String, kali_common::Repr)],
    alloc_index: u32,
) {
    // Mask clearing ARRAY_HANDLE_TAG to recover a header pointer, identical to
    // growable.rs `GROWABLE_HANDLE_MASK` / `emit_join_growable_body`.
    let mask = !(crate::ARRAY_HANDLE_TAG) as i64;
    const DST: u32 = 1;
    const SRCH: u32 = 2;
    const NEW_HDR: u32 = 3;
    const NEW_DATA: u32 = 4;
    const LEN: u32 = 5;
    const CAP: u32 = 6;

    // dst = zext(__alloc(nfields * 8)) — the object base, same layout as
    // `emit_object_allocation` (no header word; field `i` at `i * 8`).
    func.instruction(&Instruction::I32Const((fields.len() * 8) as i32));
    func.instruction(&Instruction::Call(alloc_index));
    func.instruction(&Instruction::I64ExtendI32U);
    func.instruction(&Instruction::LocalSet(DST));

    for (index, (_name, repr)) in fields.iter().enumerate() {
        let off = (index * 8) as u64;
        if matches!(repr, kali_common::Repr::GrowableArrayI64) {
            // srch = *(src + off)  (tagged source handle)
            func.instruction(&Instruction::LocalGet(0));
            func.instruction(&Instruction::I32WrapI64);
            func.instruction(&Instruction::I64Load(mem(off)));
            func.instruction(&Instruction::LocalSet(SRCH));

            // len = *((srch & mask) + 0)
            func.instruction(&Instruction::LocalGet(SRCH));
            func.instruction(&Instruction::I64Const(mask));
            func.instruction(&Instruction::I64And);
            func.instruction(&Instruction::I32WrapI64);
            func.instruction(&Instruction::I64Load(mem(0)));
            func.instruction(&Instruction::LocalSet(LEN));

            // cap = max(len, GROWABLE_INITIAL_CAP) — mirrors the seeded-literal
            // growable allocation (`cap = seed_len.max(4)`), so a zero-length
            // array still allocates the minimum data block (never __alloc(0)).
            func.instruction(&Instruction::LocalGet(LEN));
            func.instruction(&Instruction::I64Const(
                super::growable::GROWABLE_INITIAL_CAP as i64,
            ));
            func.instruction(&Instruction::I64LtS);
            func.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            func.instruction(&Instruction::I64Const(
                super::growable::GROWABLE_INITIAL_CAP as i64,
            ));
            func.instruction(&Instruction::Else);
            func.instruction(&Instruction::LocalGet(LEN));
            func.instruction(&Instruction::End);
            func.instruction(&Instruction::LocalSet(CAP));

            // new_hdr = zext(__alloc(24))
            func.instruction(&Instruction::I32Const(24));
            func.instruction(&Instruction::Call(alloc_index));
            func.instruction(&Instruction::I64ExtendI32U);
            func.instruction(&Instruction::LocalSet(NEW_HDR));

            // new_data = zext(__alloc(cap * 8))
            func.instruction(&Instruction::LocalGet(CAP));
            func.instruction(&Instruction::I64Const(8));
            func.instruction(&Instruction::I64Mul);
            func.instruction(&Instruction::I32WrapI64);
            func.instruction(&Instruction::Call(alloc_index));
            func.instruction(&Instruction::I64ExtendI32U);
            func.instruction(&Instruction::LocalSet(NEW_DATA));

            // *(new_hdr + 0) = len
            func.instruction(&Instruction::LocalGet(NEW_HDR));
            func.instruction(&Instruction::I32WrapI64);
            func.instruction(&Instruction::LocalGet(LEN));
            func.instruction(&Instruction::I64Store(mem(0)));
            // *(new_hdr + 8) = cap
            func.instruction(&Instruction::LocalGet(NEW_HDR));
            func.instruction(&Instruction::I32WrapI64);
            func.instruction(&Instruction::LocalGet(CAP));
            func.instruction(&Instruction::I64Store(mem(8)));
            // *(new_hdr + 16) = new_data
            func.instruction(&Instruction::LocalGet(NEW_HDR));
            func.instruction(&Instruction::I32WrapI64);
            func.instruction(&Instruction::LocalGet(NEW_DATA));
            func.instruction(&Instruction::I64Store(mem(16)));

            // memory.copy(dst = new_data, src = *((srch & mask) + 16), n = len * 8)
            func.instruction(&Instruction::LocalGet(NEW_DATA));
            func.instruction(&Instruction::I32WrapI64);
            func.instruction(&Instruction::LocalGet(SRCH));
            func.instruction(&Instruction::I64Const(mask));
            func.instruction(&Instruction::I64And);
            func.instruction(&Instruction::I32WrapI64);
            func.instruction(&Instruction::I64Load(mem(16)));
            func.instruction(&Instruction::I32WrapI64);
            func.instruction(&Instruction::LocalGet(LEN));
            func.instruction(&Instruction::I64Const(8));
            func.instruction(&Instruction::I64Mul);
            func.instruction(&Instruction::I32WrapI64);
            func.instruction(&Instruction::MemoryCopy {
                src_mem: 0,
                dst_mem: 0,
            });

            // *(dst + off) = zext(new_hdr) | ARRAY_HANDLE_TAG
            func.instruction(&Instruction::LocalGet(DST));
            func.instruction(&Instruction::I32WrapI64);
            func.instruction(&Instruction::LocalGet(NEW_HDR));
            func.instruction(&Instruction::I64Const(crate::ARRAY_HANDLE_TAG as i64));
            func.instruction(&Instruction::I64Or);
            func.instruction(&Instruction::I64Store(mem(off)));
        } else {
            // Depth-2 defense (reviewer-suggested): the call site
            // (`fields_are_clone_envelope`) is the primary gate, but assert here
            // that no `Object`/`String` field ever reaches the verbatim-copy arm
            // — a verbatim 8-byte copy of a nested-object pointer would
            // SHALLOW-SHARE it (soundness bug), and a string handle is not in the
            // P2 envelope. If this fires, a caller requested a clone for an
            // out-of-envelope shape.
            debug_assert!(
                matches!(repr, kali_common::Repr::I64 | kali_common::Repr::F64),
                "clone-shape verbatim slot copy reached a non-scalar field repr {repr:?}; \
                 the call site must gate on `fields_are_clone_envelope`"
            );
            // Verbatim 8-byte slot copy: *(dst + off) = *(src + off).
            func.instruction(&Instruction::LocalGet(DST));
            func.instruction(&Instruction::I32WrapI64);
            func.instruction(&Instruction::LocalGet(0));
            func.instruction(&Instruction::I32WrapI64);
            func.instruction(&Instruction::I64Load(mem(off)));
            func.instruction(&Instruction::I64Store(mem(off)));
        }
    }

    // Result: the new object base pointer (untagged i64). No trailing End —
    // the dispatch loop appends it.
    func.instruction(&Instruction::LocalGet(DST));
}

/// 8-byte-aligned `MemArg` at `offset` in the single default memory.
fn mem(offset: u64) -> MemArg {
    MemArg {
        offset,
        align: 3,
        memory_index: 0,
    }
}
