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

/// Decodes an array handle (an `i64` pointer into `__heap`) into its element
/// low-byte stream: reads the `i64` length at offset 0, then `len` `i64`
/// elements at offsets 8, 16, …, masking each to its low 8 bits. Used by
/// `stdout_write_bytes` to emit a byte buffer built in the array lane.
///
/// The whole `[len][elem…]` range is bounds-checked against the actual guest
/// memory slice BEFORE any allocation (mirroring `read_guest_bytes`), so a
/// forged/garbage handle with an enormous length prefix yields an empty result
/// rather than triggering a giant allocation that would abort the host process.
pub(crate) fn decode_array_low_bytes(
    caller: &mut Caller<'_, KaliHostState>,
    handle: i64,
) -> wasmtime::Result<Vec<u8>> {
    let memory = guest_memory(caller)?;
    Ok(decode_low_bytes_from_slice(memory.data(caller), handle))
}

/// Pure, bounds-checked core of `decode_array_low_bytes`, factored out so the
/// oversized/forged-length safety can be unit-tested without a live wasmtime
/// store. Returns an empty vec for any handle whose declared length would read
/// past the end of `data` (or whose offset arithmetic overflows).
pub(crate) fn decode_low_bytes_from_slice(data: &[u8], handle: i64) -> Vec<u8> {
    if handle <= 0 {
        return Vec::new();
    }
    let base = handle as usize;
    // Length prefix (i64 @ base+0) must fit.
    let Some(len_bytes) = base.checked_add(8).and_then(|end| data.get(base..end)) else {
        return Vec::new();
    };
    let len =
        i64::from_le_bytes(len_bytes.try_into().expect("8-byte length slice")).max(0) as usize;
    // The full element range [base+8 .. base+8+len*8) must fit within the slice
    // BEFORE we allocate — checked arithmetic so the offset itself can't wrap.
    let Some(end) = len
        .checked_mul(8)
        .and_then(|elem_span| base.checked_add(8).and_then(|s| s.checked_add(elem_span)))
    else {
        return Vec::new();
    };
    let elem_start = base + 8;
    let Some(elems) = data.get(elem_start..end) else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(len);
    for chunk in elems.chunks_exact(8) {
        let value = i64::from_le_bytes(chunk.try_into().expect("8-byte element slice"));
        out.push((value & 0xFF) as u8);
    }
    out
}

/// Resolves the exported `__heap` global. Since Task 5 (the page-pool
/// allocator) this is the page *frontier*, not a flat bump pointer, and is
/// consulted here only as the fallback path in `alloc_guest_string` below —
/// the primary path calls the exported `__alloc_global` function instead.
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

/// Allocates `bytes.len()` bytes for a host-runtime string and returns a
/// tagged string handle (`STRING_HANDLE_TAG | (offset << 32) | len`).
///
/// Since Task 5 (the page-pool allocator), the primary path calls the guest's
/// exported `__alloc_global(i32) -> i32` — the same "global" arena trio the
/// guest's own `Kali`-runtime-string-producing calls use, which is never
/// touched by `__arena_reset` — with the byte length rounded up to a
/// multiple of 8 (matching the browser JS glue's `allocGuestString`, so host
/// strings stay 8-aligned in the arena regardless of which runtime produced
/// them). A stale cached module built pre-Task-5 has no `__alloc_global`
/// export; for that case only, this falls back to the old direct-`__heap`
/// bump so previously-compiled `.wasm` artifacts keep working.
pub(crate) fn alloc_guest_string(
    caller: &mut Caller<'_, KaliHostState>,
    bytes: &[u8],
) -> wasmtime::Result<i64> {
    let len = i32::try_from(bytes.len())
        .map_err(|_| wasmtime::Error::msg("guest string allocation length exceeds i32 range"))?;
    let base = match caller.get_export("__alloc_global") {
        Some(Extern::Func(alloc_global)) => {
            let typed = alloc_global
                .typed::<i32, i32>(&mut *caller)
                .map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "__alloc_global has an unexpected signature: {error}"
                    ))
                })?;
            let rounded = (len + 7) & !7;
            typed.call(&mut *caller, rounded).map_err(|error| {
                wasmtime::Error::msg(format!("__alloc_global call failed: {error}"))
            })?
        }
        _ => {
            // Fallback for a stale cached module built pre-Task-5 (page-pool
            // allocator) with no `__alloc_global` export: bump `__heap`
            // directly, exactly as before this task.
            let global = heap_global(caller)?;
            let heap_base = global
                .get(&mut *caller)
                .i32()
                .ok_or_else(|| wasmtime::Error::msg("__heap global is not an i32"))?;
            let next = heap_base
                .checked_add(len)
                .ok_or_else(|| wasmtime::Error::msg("guest heap pointer overflow"))?;
            global.set(&mut *caller, wasmtime::Val::I32(next))?;
            heap_base
        }
    };
    let memory = guest_memory(caller)?;
    let start = checked_offset(base)?;
    memory.write(&mut *caller, start, bytes).map_err(|error| {
        wasmtime::Error::msg(format!("failed to write guest memory: {}", error))
    })?;
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

#[cfg(test)]
mod memory_tests {
    use super::decode_low_bytes_from_slice;

    /// Builds a well-formed array-lane buffer: `[len i64][elem i64…]` at `base`.
    fn array_buffer(base: usize, elems: &[i64]) -> Vec<u8> {
        let mut data = vec![0u8; base];
        data.extend_from_slice(&(elems.len() as i64).to_le_bytes());
        for &e in elems {
            data.extend_from_slice(&e.to_le_bytes());
        }
        data
    }

    #[test]
    fn decode_low_bytes_reads_well_formed_array() {
        let base = 16usize;
        let data = array_buffer(base, &[80, 52, 10, 52, 32]);
        assert_eq!(
            decode_low_bytes_from_slice(&data, base as i64),
            vec![80u8, 52, 10, 52, 32]
        );
    }

    #[test]
    fn decode_low_bytes_masks_low_byte_only() {
        let base = 8usize;
        // 0x1FF -> 0xFF, negative -1 -> 0xFF, 256 -> 0x00: never multi-byte/sign-extended.
        let data = array_buffer(base, &[0x1FF, -1, 256]);
        assert_eq!(
            decode_low_bytes_from_slice(&data, base as i64),
            vec![0xFFu8, 0xFF, 0x00]
        );
    }

    #[test]
    fn decode_low_bytes_oversized_length_yields_empty_not_abort() {
        // Forged handle: length prefix claims a huge element count that would
        // make `Vec::with_capacity` abort the host. Must be rejected pre-alloc.
        let base = 0usize;
        let mut data = vec![0u8; 8]; // only the length prefix, no elements
        data[..8].copy_from_slice(&(i64::MAX).to_le_bytes());
        assert_eq!(
            decode_low_bytes_from_slice(&data, base as i64),
            Vec::<u8>::new()
        );
    }

    #[test]
    fn decode_low_bytes_truncated_element_range_yields_empty() {
        // Length says 4 elements but only 2 elements' worth of bytes follow.
        let base = 0usize;
        let mut data = Vec::new();
        data.extend_from_slice(&4i64.to_le_bytes());
        data.extend_from_slice(&1i64.to_le_bytes());
        data.extend_from_slice(&2i64.to_le_bytes());
        assert_eq!(
            decode_low_bytes_from_slice(&data, base as i64),
            Vec::<u8>::new()
        );
    }

    #[test]
    fn decode_low_bytes_nonpositive_and_out_of_range_handle_yields_empty() {
        let data = array_buffer(0, &[1, 2, 3]);
        assert_eq!(decode_low_bytes_from_slice(&data, 0), Vec::<u8>::new());
        assert_eq!(decode_low_bytes_from_slice(&data, -5), Vec::<u8>::new());
        // Handle points past the end of the slice.
        assert_eq!(decode_low_bytes_from_slice(&data, 10_000), Vec::<u8>::new());
    }

    #[test]
    fn decode_low_bytes_zero_length_array_yields_empty() {
        let base = 8usize;
        let data = array_buffer(base, &[]);
        assert_eq!(
            decode_low_bytes_from_slice(&data, base as i64),
            Vec::<u8>::new()
        );
    }
}
