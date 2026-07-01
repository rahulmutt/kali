//! Guest-memory read/write helpers and spawn-arg decoding.
use crate::*;

pub(crate) fn decode_spawn_args(encoded: &str) -> Vec<String> {
    if encoded.is_empty() {
        return Vec::new();
    }

    let mut args: Vec<String> = encoded.split('|').map(str::to_owned).collect();
    if args.last().is_some_and(|arg| arg.is_empty()) {
        args.pop();
    }
    args
}

pub(crate) fn read_guest_string(
    caller: &mut Caller<'_, KaliHostState>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<String> {
    let bytes = read_guest_bytes(caller, ptr, len)?;
    String::from_utf8(bytes).map_err(|error| {
        wasmtime::Error::msg(format!("guest string is not valid UTF-8: {}", error))
    })
}

pub(crate) fn read_guest_string_handle(
    caller: &mut Caller<'_, KaliHostState>,
    value: i64,
) -> wasmtime::Result<String> {
    let raw = value as u64;
    if raw & STRING_HANDLE_TAG == 0 {
        return Err(wasmtime::Error::msg(
            "guest string handle is missing the string tag",
        ));
    }

    let offset = ((raw >> 32) & 0x7fff_ffff) as i32;
    let len = (raw & 0xffff_ffff) as i32;
    read_guest_string(caller, offset, len)
}

/// Decodes a tagged string handle to its guest bytes, or `None` when `value` is
/// not a tagged string handle. Shared by `format_console_value` and `string_concat`.
pub(crate) fn decode_string_handle_bytes(
    caller: &mut Caller<'_, KaliHostState>,
    value: i64,
) -> Option<Vec<u8>> {
    let raw = value as u64;
    if raw & STRING_HANDLE_TAG == 0 {
        return None;
    }
    let offset = ((raw >> 32) & 0x7fff_ffff) as i32;
    let len = (raw & 0xffff_ffff) as i32;
    read_guest_bytes(caller, offset, len).ok()
}

/// Resolves the exported `__heap` bump-pointer global (added in Task 5).
pub(crate) fn heap_global(
    caller: &mut Caller<'_, KaliHostState>,
) -> wasmtime::Result<wasmtime::Global> {
    match caller.get_export("__heap") {
        Some(Extern::Global(g)) => Ok(g),
        _ => Err(wasmtime::Error::msg(
            "guest module does not export __heap global",
        )),
    }
}

/// Allocates `bytes.len()` bytes at the current `__heap`, writes them, advances
/// `__heap`, and returns a tagged string handle
/// (`STRING_HANDLE_TAG | (offset << 32) | len`).
pub(crate) fn alloc_guest_string(
    caller: &mut Caller<'_, KaliHostState>,
    bytes: &[u8],
) -> wasmtime::Result<i64> {
    let global = heap_global(caller)?;
    let base = global
        .get(&mut *caller)
        .i32()
        .ok_or_else(|| wasmtime::Error::msg("__heap global is not an i32"))?;
    let memory = guest_memory(caller)?;
    let start = checked_offset(base)?;
    memory.write(&mut *caller, start, bytes).map_err(|error| {
        wasmtime::Error::msg(format!("failed to write guest memory: {}", error))
    })?;
    let next = base
        .checked_add(i32::try_from(bytes.len()).map_err(|_| {
            wasmtime::Error::msg("guest string allocation length exceeds i32 range")
        })?)
        .ok_or_else(|| wasmtime::Error::msg("guest heap pointer overflow"))?;
    global.set(&mut *caller, wasmtime::Val::I32(next))?;
    Ok((STRING_HANDLE_TAG | ((base as u64) << 32) | (bytes.len() as u64)) as i64)
}

pub(crate) fn read_guest_bytes(
    caller: &mut Caller<'_, KaliHostState>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<Vec<u8>> {
    let memory = guest_memory(caller)?;
    let start = checked_offset(ptr)?;
    let length = checked_offset(len)?;
    let end = start
        .checked_add(length)
        .ok_or_else(|| wasmtime::Error::msg("guest memory access overflow"))?;
    let data = memory.data(caller);
    let slice = data
        .get(start..end)
        .ok_or_else(|| wasmtime::Error::msg("guest memory access out of bounds"))?;
    Ok(slice.to_vec())
}

pub(crate) fn write_guest_bytes(
    caller: &mut Caller<'_, KaliHostState>,
    ptr: i32,
    cap: i32,
    bytes: &[u8],
) -> wasmtime::Result<i32> {
    let memory = guest_memory(caller)?;
    let start = checked_offset(ptr)?;
    let capacity = checked_offset(cap)?;
    if bytes.len() > capacity {
        return Err(wasmtime::Error::msg(format!(
            "guest output buffer too small: need {}, have {}",
            bytes.len(),
            capacity
        )));
    }
    memory.write(caller, start, bytes).map_err(|error| {
        wasmtime::Error::msg(format!("failed to write guest memory: {}", error))
    })?;
    Ok(bytes.len() as i32)
}

pub(crate) fn write_guest_string(
    caller: &mut Caller<'_, KaliHostState>,
    ptr: i32,
    cap: i32,
    value: impl AsRef<str>,
) -> wasmtime::Result<i32> {
    write_guest_bytes(caller, ptr, cap, value.as_ref().as_bytes())
}

pub(crate) fn guest_memory(caller: &mut Caller<'_, KaliHostState>) -> wasmtime::Result<Memory> {
    match caller.get_export("memory") {
        Some(Extern::Memory(memory)) => Ok(memory),
        _ => Err(wasmtime::Error::msg("guest module does not export memory")),
    }
}

pub(crate) fn checked_offset(value: i32) -> wasmtime::Result<usize> {
    usize::try_from(value).map_err(|_| wasmtime::Error::msg("negative guest memory offset"))
}

pub(crate) const STRING_HANDLE_TAG: u64 = 0x8000_0000_0000_0000;
