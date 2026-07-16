//! Stage C (closures): environment-pointer allocation surface.
//!
//! Task 2 reserves the WASM global that will hold the active environment
//! record pointer and adds the entry point that will allocate a new
//! environment record. This module is deliberately behavior-neutral this
//! task: `emit_env_alloc` is not called from anywhere yet (Task 3 wires call
//! sites once `kali_mir::derive_env_plans` output is consumed by codegen),
//! and the header store that would write `parent_ptr` into the freshly
//! allocated record is left for Task 3, where a running fixture can prove
//! the addressing against the heap-object header pattern used elsewhere in
//! `emit/` (e.g. `emit/object.rs`'s field stores).

use crate::*;

/// Reserved WASM global holding the active environment record pointer (i64;
/// 0 = no env). Allocated immediately after the arena trio; see the global
/// map at `lower.rs` (around the `GlobalSection` build) and
/// `RESERVED_GLOBAL_COUNT`, which rises from 8 to 9 to reserve this slot as
/// g8. Reserved-but-unused this task — nothing reads or writes it yet.
#[allow(dead_code)]
pub(crate) const CURRENT_ENV_GLOBAL: u32 = 8;

/// Emit: allocate `header + cells*8` bytes in the GLOBAL (never-reset)
/// region, leaving the new env record's base pointer (i64) on the stack.
/// `parent_ptr` is read from `CURRENT_ENV_GLOBAL` by the caller before
/// calling this; storing it into the new record's header is deferred to
/// Task 3 (see the module doc comment above) — this task only reserves the
/// global and this allocation entry point, and is behavior-neutral.
#[allow(dead_code)]
pub(crate) fn emit_env_alloc(function: &mut Function, alloc_global_index: u32, cell_count: u32) {
    let bytes = 8 + cell_count * 8; // parent-pointer header + cells
    function.instruction(&Instruction::I32Const(bytes as i32));
    function.instruction(&Instruction::Call(alloc_global_index)); // -> i32 base ptr
    function.instruction(&Instruction::I64ExtendI32U); // -> i64 base ptr on stack
                                                       // header <- parent (CURRENT_ENV_GLOBAL): the store is intentionally NOT
                                                       // emitted here (Task 3 finalizes the addressing against a running
                                                       // fixture); this entry point is unwired and unused this task.
}
