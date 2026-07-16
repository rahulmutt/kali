//! Stage C (closures): environment-record allocation + cell addressing.
//!
//! An environment record is a heap block in the GLOBAL (never-reset) region:
//!
//! ```text
//! env_ptr → [ parent_env_ptr : i64 @ +0 ][ cell_0 : i64 @ +8 ][ cell_1 @ +16 ] …
//! ```
//!
//! `parent_env_ptr` links to the enclosing activation's env (the **env chain**;
//! `0` = no parent). Cell offsets are header-relative: cell `k`'s value lives at
//! byte `8 + offset` from the record base (the 8-byte parent header precedes the
//! cells). Records are allocated with `__alloc_global` so they never fall to an
//! arena reset — the chain stays valid after a parent activation returns.
//!
//! `CURRENT_ENV_GLOBAL` (g8) holds the active record pointer as an i64 (a
//! zero-extended i32 linear-memory address; `0` = no env). Only a function that
//! OWNS cells mutates it (its prologue saves the incoming value, allocates,
//! sets g8; its epilogue/return restores). A function that merely captures
//! outer cells and owns none leaves g8 untouched and reads through the inherited
//! record directly — so a **synchronous** nested call needs no call-site
//! threading (C1).

use crate::*;

/// Reserved WASM global holding the active environment record pointer (i64;
/// 0 = no env). Allocated immediately after the arena trio; see the global
/// map at `lower.rs` (around the `GlobalSection` build) and
/// `RESERVED_GLOBAL_COUNT`, which reserves this slot as g8.
pub(crate) const CURRENT_ENV_GLOBAL: u32 = 8;

/// Name of the dedicated i64 local reserved (per env-OWNING function) to hold
/// the parent `current_env` saved on entry — restored to `CURRENT_ENV_GLOBAL`
/// on every exit path. Reserved by `collect`-time provisioning in `lower.rs`
/// and resolved back by name through `FunctionEmitter::locals`, so the two
/// cannot disagree on naming. The `#env` suffix is unrepresentable as a source
/// identifier, so it never collides with a real binding.
pub(crate) fn env_save_local_name() -> String {
    "__env_save#env".to_string()
}

/// A `MemArg` for an 8-byte-aligned i64 access at `offset`.
fn env_memarg(offset: u32) -> MemArg {
    MemArg {
        offset: offset as u64,
        align: 3,
        memory_index: 0,
    }
}

/// Emit: allocate `header + cells*8` bytes in the GLOBAL (never-reset) region,
/// set `env_global` (g8) to the new record's base pointer, and write the parent
/// pointer (read from `save_local`, where the prologue stashed the incoming g8)
/// into the record's `parent_env_ptr` header. Leaves nothing on the stack.
///
/// `alloc_global_index` is the `__alloc_global` function index — the env record
/// must outlive any per-loop/per-function arena reset (the env chain stays
/// valid after a parent activation returns). The `__alloc*` ABI is i32-in /
/// i32-out, so the returned pointer is zero-extended to the i64 g8 convention.
pub(crate) fn emit_env_alloc(
    function: &mut Function,
    alloc_global_index: u32,
    cell_count: u32,
    env_global: u32,
    save_local: u32,
) {
    let bytes = 8 + cell_count * 8; // parent-pointer header + cells
    function.instruction(&Instruction::I32Const(bytes as i32));
    function.instruction(&Instruction::Call(alloc_global_index)); // -> i32 base ptr
    function.instruction(&Instruction::I64ExtendI32U); // -> i64 base ptr
    function.instruction(&Instruction::GlobalSet(env_global)); // g8 = new record
                                                               // header[parent] <- saved incoming env (g8 before this activation).
    function.instruction(&Instruction::GlobalGet(env_global)); // i64 base
    function.instruction(&Instruction::I32WrapI64); // i32 address
    function.instruction(&Instruction::LocalGet(save_local)); // i64 parent
    function.instruction(&Instruction::I64Store(env_memarg(0)));
}

/// Emit the i32 linear-memory BASE address of the env record `depth` parent
/// links up from `current_env` (`depth` 0 = `current_env` itself). Leaves an
/// i32 on the stack; the cell value then lives at `+ (8 + offset)`.
pub(crate) fn emit_env_base_addr(function: &mut Function, current_env_global: u32, depth: u32) {
    function.instruction(&Instruction::GlobalGet(current_env_global)); // i64 env
    for _ in 0..depth {
        // env = *(env + 0)  — follow one parent link.
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(env_memarg(0)));
    }
    function.instruction(&Instruction::I32WrapI64); // i32 base of target record
}

/// Load the i64 cell at `offset` from the env `depth` links up the parent
/// chain. Leaves the cell value on the stack.
pub(crate) fn emit_cell_load(
    function: &mut Function,
    current_env_global: u32,
    depth: u32,
    offset: u32,
) {
    emit_env_base_addr(function, current_env_global, depth);
    function.instruction(&Instruction::I64Load(env_memarg(8 + offset)));
}

/// Store the i64 value already on the stack into the cell at `offset` from the
/// env `depth` links up the parent chain (consumes the value). `scratch_local`
/// is a spare i64 local used to hold the value while the store address is
/// computed beneath it (wasm has no stack-swap).
pub(crate) fn emit_cell_store(
    function: &mut Function,
    current_env_global: u32,
    depth: u32,
    offset: u32,
    scratch_local: u32,
) {
    function.instruction(&Instruction::LocalSet(scratch_local)); // stash value
    emit_env_base_addr(function, current_env_global, depth); // i32 base beneath
    function.instruction(&Instruction::LocalGet(scratch_local)); // value on top
    function.instruction(&Instruction::I64Store(env_memarg(8 + offset)));
}
