//! Default host-import registration (`kali:rt` namespace) for the wasmtime linker.
use crate::*;
use kali_common::js_number::format_js_number;

pub(crate) fn register_default_host_imports(
    linker: &mut Linker<KaliHostState>,
) -> Result<(), Diagnostic> {
    linker
        .func_wrap(
            "kali:rt",
            "console_log",
            |mut caller: Caller<'_, KaliHostState>, val: i64| -> wasmtime::Result<()> {
                enforce_operation(caller.data_mut(), HostOperation::Console)?;
                let rendered = format_console_value(&mut caller, val);
                append_stdout(caller.data_mut(), rendered);
                Ok(())
            },
        )
        .map_err(|error| host_import_error("console_log", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "stdout_write_bytes",
            |mut caller: Caller<'_, KaliHostState>, handle: i64| -> wasmtime::Result<()> {
                enforce_operation(caller.data_mut(), HostOperation::Console)?;
                let bytes = decode_array_low_bytes(&mut caller, handle)?;
                caller.data_mut().stdout_bytes.extend_from_slice(&bytes);
                Ok(())
            },
        )
        .map_err(|error| host_import_error("stdout_write_bytes", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "console_error",
            |mut caller: Caller<'_, KaliHostState>, val: i64| -> wasmtime::Result<()> {
                enforce_operation(caller.data_mut(), HostOperation::Console)?;
                let rendered = format_console_value(&mut caller, val);
                append_stderr(caller.data_mut(), rendered);
                Ok(())
            },
        )
        .map_err(|error| host_import_error("console_error", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "console_warn",
            |mut caller: Caller<'_, KaliHostState>, val: i64| -> wasmtime::Result<()> {
                enforce_operation(caller.data_mut(), HostOperation::Console)?;
                let rendered = format_console_value(&mut caller, val);
                // No prefix: node's `console.warn` writes the message alone, and
                // NONE of the four JS import mirrors prefixed it either -- so
                // this line made kali's own runtimes disagree with each other
                // before it made kali disagree with node. R-33.
                append_stderr(caller.data_mut(), rendered);
                Ok(())
            },
        )
        .map_err(|error| host_import_error("console_warn", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "console_info",
            |mut caller: Caller<'_, KaliHostState>, val: i64| -> wasmtime::Result<()> {
                enforce_operation(caller.data_mut(), HostOperation::Console)?;
                let rendered = format_console_value(&mut caller, val);
                append_stdout(caller.data_mut(), rendered);
                Ok(())
            },
        )
        .map_err(|error| host_import_error("console_info", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "console_debug",
            |mut caller: Caller<'_, KaliHostState>, val: i64| -> wasmtime::Result<()> {
                enforce_operation(caller.data_mut(), HostOperation::Console)?;
                let rendered = format_console_value(&mut caller, val);
                append_stdout(caller.data_mut(), rendered);
                Ok(())
            },
        )
        .map_err(|error| host_import_error("console_debug", error))?;

    linker
        .func_wrap("kali:rt", "performance_now", || -> f64 {
            performance_now()
        })
        .map_err(|error| host_import_error("performance_now", error))?;

    linker
        .func_wrap("kali:rt", "performanceNow", || -> f64 { performance_now() })
        .map_err(|error| host_import_error("performanceNow", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "crypto_get_random_values",
            |mut caller: Caller<'_, KaliHostState>,
             out_ptr: i32,
             out_len: i32|
             -> wasmtime::Result<i32> {
                enforce_operation(caller.data_mut(), HostOperation::Random)?;
                let memory = guest_memory(&mut caller)?;
                let start = checked_offset(out_ptr)?;
                let len = checked_offset(out_len)?;
                start
                    .checked_add(len)
                    .ok_or_else(|| wasmtime::Error::msg("guest memory access overflow"))?;
                let mut bytes = vec![0u8; len];
                fill_random_values(&mut bytes).map_err(|error| {
                    wasmtime::Error::msg(format!("failed to generate random bytes: {}", error))
                })?;
                memory.write(&mut caller, start, &bytes).map_err(|error| {
                    wasmtime::Error::msg(format!("failed to write guest memory: {}", error))
                })?;
                Ok(out_len)
            },
        )
        .map_err(|error| host_import_error("crypto_get_random_values", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "cryptoGetRandomValues",
            |mut caller: Caller<'_, KaliHostState>,
             out_ptr: i32,
             out_len: i32|
             -> wasmtime::Result<i32> {
                let memory = guest_memory(&mut caller)?;
                let start = checked_offset(out_ptr)?;
                let len = checked_offset(out_len)?;
                start
                    .checked_add(len)
                    .ok_or_else(|| wasmtime::Error::msg("guest memory access overflow"))?;
                let mut bytes = vec![0u8; len];
                fill_random_values(&mut bytes).map_err(|error| {
                    wasmtime::Error::msg(format!("failed to generate random bytes: {}", error))
                })?;
                memory.write(&mut caller, start, &bytes).map_err(|error| {
                    wasmtime::Error::msg(format!("failed to write guest memory: {}", error))
                })?;
                Ok(out_len)
            },
        )
        .map_err(|error| host_import_error("cryptoGetRandomValues", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "crypto_random_uuid",
            |mut caller: Caller<'_, KaliHostState>,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                enforce_operation(caller.data_mut(), HostOperation::Random)?;
                let uuid = random_uuid().map_err(|error| {
                    wasmtime::Error::msg(format!("failed to generate random UUID: {}", error))
                })?;
                write_guest_string(&mut caller, out_ptr, out_cap, uuid)
            },
        )
        .map_err(|error| host_import_error("crypto_random_uuid", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "cryptoRandomUUID",
            |mut caller: Caller<'_, KaliHostState>,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                enforce_operation(caller.data_mut(), HostOperation::Random)?;
                let uuid = random_uuid().map_err(|error| {
                    wasmtime::Error::msg(format!("failed to generate random UUID: {}", error))
                })?;
                write_guest_string(&mut caller, out_ptr, out_cap, uuid)
            },
        )
        .map_err(|error| host_import_error("cryptoRandomUUID", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "crypto_subtle_digest",
            |mut caller: Caller<'_, KaliHostState>,
             algo_ptr: i32,
             algo_len: i32,
             in_ptr: i32,
             in_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                // `SubtleCrypto::digest` is deterministic and reads no ambient
                // entropy, but it is a Web Crypto operation like getRandomValues /
                // randomUUID, so it is gated under the same `Random` host operation
                // for policy consistency across the crypto imports.
                enforce_operation(caller.data_mut(), HostOperation::Random)?;
                let algorithm = read_guest_string(&mut caller, algo_ptr, algo_len)?;
                let input = read_guest_bytes(&mut caller, in_ptr, in_len)?;
                let digest = SubtleCrypto
                    .digest(&algorithm, &input)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, &digest)
            },
        )
        .map_err(|error| host_import_error("crypto_subtle_digest", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "cryptoSubtleDigest",
            |mut caller: Caller<'_, KaliHostState>,
             algo_ptr: i32,
             algo_len: i32,
             in_ptr: i32,
             in_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                enforce_operation(caller.data_mut(), HostOperation::Random)?;
                let algorithm = read_guest_string(&mut caller, algo_ptr, algo_len)?;
                let input = read_guest_bytes(&mut caller, in_ptr, in_len)?;
                let digest = SubtleCrypto
                    .digest(&algorithm, &input)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, &digest)
            },
        )
        .map_err(|error| host_import_error("cryptoSubtleDigest", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "thread_spawn",
            |mut caller: Caller<'_, KaliHostState>,
             script_url_ptr: i32,
             script_url_len: i32|
             -> wasmtime::Result<i32> {
                let script_url = read_guest_string(&mut caller, script_url_ptr, script_url_len)?;
                let instance_id = caller.data_mut().spawn_thread_instance(script_url)?;
                Ok(instance_id as i32)
            },
        )
        .map_err(|error| host_import_error("thread_spawn", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "test_register",
            // Trailing `env_ptr` (Stage C C3): the `current_env` active at
            // registration, so a capturing `Kali.test(...)` callback resolves
            // its enclosing bindings when the host invokes it later.
            |mut caller: Caller<'_, KaliHostState>,
             callback_id: i32,
             env_ptr: i64|
             -> wasmtime::Result<()> {
                caller
                    .data_mut()
                    .registered_tests
                    .push((callback_id, env_ptr));
                Ok(())
            },
        )
        .map_err(|error| host_import_error("test_register", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "coverage_hit",
            |mut caller: Caller<'_, KaliHostState>, coverage_id: i32| -> wasmtime::Result<()> {
                if coverage_id >= 0 {
                    caller.data_mut().coverage_hits.insert(coverage_id as u32);
                }
                Ok(())
            },
        )
        .map_err(|error| host_import_error("coverage_hit", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "fs_read_text_file",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let host_path = resolve_host_path(caller.data(), Path::new(&path));
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::FileRead {
                        path: host_path.clone(),
                    },
                )?;
                let bytes = fs::read(&host_path).map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "failed to read '{}': {}",
                        host_path.display(),
                        error
                    ))
                })?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, &bytes)
            },
        )
        .map_err(|error| host_import_error("fs_read_text_file", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "fs_read_file",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let host_path = resolve_host_path(caller.data(), Path::new(&path));
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::FileRead {
                        path: host_path.clone(),
                    },
                )?;
                let bytes = fs::read(&host_path).map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "failed to read '{}': {}",
                        host_path.display(),
                        error
                    ))
                })?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, &bytes)
            },
        )
        .map_err(|error| host_import_error("fs_read_file", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "fs_write_text_file",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             data_ptr: i32,
             data_len: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let data = read_guest_bytes(&mut caller, data_ptr, data_len)?;
                let host_path = resolve_host_path(caller.data(), Path::new(&path));
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::FileWrite {
                        path: host_path.clone(),
                    },
                )?;
                if let Some(parent) = host_path.parent() {
                    fs::create_dir_all(parent).map_err(|error| {
                        wasmtime::Error::msg(format!(
                            "failed to create '{}': {}",
                            parent.display(),
                            error
                        ))
                    })?;
                }
                fs::write(&host_path, data).map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "failed to write '{}': {}",
                        host_path.display(),
                        error
                    ))
                })?;
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("fs_write_text_file", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "fs_mkdir",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let host_path = resolve_host_path(caller.data(), Path::new(&path));
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::FileWrite {
                        path: host_path.clone(),
                    },
                )?;
                fs::create_dir_all(&host_path).map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "failed to create '{}': {}",
                        host_path.display(),
                        error
                    ))
                })?;
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("fs_mkdir", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "fs_remove",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             recursive: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let host_path = resolve_host_path(caller.data(), Path::new(&path));
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::FileWrite {
                        path: host_path.clone(),
                    },
                )?;
                let metadata = fs::metadata(&host_path).map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "failed to inspect '{}': {}",
                        host_path.display(),
                        error
                    ))
                })?;
                if metadata.is_dir() {
                    if recursive != 0 {
                        fs::remove_dir_all(&host_path).map_err(|error| {
                            wasmtime::Error::msg(format!(
                                "failed to remove '{}': {}",
                                host_path.display(),
                                error
                            ))
                        })?;
                    } else {
                        fs::remove_dir(&host_path).map_err(|error| {
                            wasmtime::Error::msg(format!(
                                "failed to remove '{}': {}",
                                host_path.display(),
                                error
                            ))
                        })?;
                    }
                } else {
                    fs::remove_file(&host_path).map_err(|error| {
                        wasmtime::Error::msg(format!(
                            "failed to remove '{}': {}",
                            host_path.display(),
                            error
                        ))
                    })?;
                }
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("fs_remove", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "env_get",
            |mut caller: Caller<'_, KaliHostState>,
             key_ptr: i32,
             key_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let key = read_guest_string(&mut caller, key_ptr, key_len)?;
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::EnvironmentRead { key: key.clone() },
                )?;
                let Some(value) = caller.data().env.get(&key).cloned() else {
                    return Ok(0);
                };
                let written = write_guest_bytes(&mut caller, out_ptr, out_cap, value.as_bytes())?;
                Ok(written.saturating_add(1))
            },
        )
        .map_err(|error| host_import_error("env_get", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "env_has",
            |mut caller: Caller<'_, KaliHostState>,
             key_ptr: i32,
             key_len: i32|
             -> wasmtime::Result<i32> {
                let key = read_guest_string(&mut caller, key_ptr, key_len)?;
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::EnvironmentRead { key: key.clone() },
                )?;
                Ok(i32::from(caller.data().env.contains_key(&key)))
            },
        )
        .map_err(|error| host_import_error("env_has", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "env_set",
            |mut caller: Caller<'_, KaliHostState>,
             key_ptr: i32,
             key_len: i32,
             value_ptr: i32,
             value_len: i32|
             -> wasmtime::Result<i32> {
                let key = read_guest_string(&mut caller, key_ptr, key_len)?;
                let value = read_guest_string(&mut caller, value_ptr, value_len)?;
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::EnvironmentWrite { key: key.clone() },
                )?;
                caller.data_mut().env.insert(key, value);
                Ok(1)
            },
        )
        .map_err(|error| host_import_error("env_set", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "env_delete",
            |mut caller: Caller<'_, KaliHostState>,
             key_ptr: i32,
             key_len: i32,
             _out_ptr: i32,
             _out_cap: i32|
             -> wasmtime::Result<i32> {
                let key = read_guest_string(&mut caller, key_ptr, key_len)?;
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::EnvironmentWrite { key: key.clone() },
                )?;
                Ok(if caller.data_mut().env.remove(&key).is_some() {
                    1
                } else {
                    0
                })
            },
        )
        .map_err(|error| host_import_error("env_delete", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "args_len",
            |caller: Caller<'_, KaliHostState>| -> i32 { caller.data().args.len() as i32 },
        )
        .map_err(|error| host_import_error("args_len", error))?;

    linker
        .func_wrap("kali:rt", "process_pid", || -> i32 {
            i32::try_from(std::process::id()).unwrap_or(i32::MAX)
        })
        .map_err(|error| host_import_error("process_pid", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "process_exit",
            |mut caller: Caller<'_, KaliHostState>, code: i64| -> wasmtime::Result<()> {
                let exit_code = i32::try_from(code).unwrap_or_else(|_| {
                    if code.is_negative() {
                        i32::MIN
                    } else {
                        i32::MAX
                    }
                });
                caller.data_mut().pending_exit_code = Some(exit_code);
                Err(wasmtime::Error::msg(format!(
                    "process.exit({exit_code}) requested guest termination"
                )))
            },
        )
        .map_err(|error| host_import_error("process_exit", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "cwd",
            |mut caller: Caller<'_, KaliHostState>,
             _path_ptr: i32,
             _path_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let cwd = caller.data().cwd.clone();
                write_guest_string(&mut caller, out_ptr, out_cap, cwd.to_string_lossy())
            },
        )
        .map_err(|error| host_import_error("cwd", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "cwd_set",
            |mut caller: Caller<'_, KaliHostState>, path_value: i64| -> wasmtime::Result<i32> {
                let path = read_guest_string_handle(&mut caller, path_value)?;
                let host_path = normalize_path(resolve_host_path(caller.data(), Path::new(&path)));
                let metadata = fs::metadata(&host_path).map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "failed to change directory to '{}': {}",
                        host_path.display(),
                        error
                    ))
                })?;
                if !metadata.is_dir() {
                    return Err(wasmtime::Error::msg(format!(
                        "failed to change directory to '{}': not a directory",
                        host_path.display()
                    )));
                }
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::ProcessChdir {
                        path: host_path.clone(),
                    },
                )?;
                caller.data_mut().cwd = host_path;
                Ok(1)
            },
        )
        .map_err(|error| host_import_error("cwd_set", error))?;

    linker
        .func_wrap("kali:rt", "math_max", |left: i64, right: i64| -> i64 {
            left.max(right)
        })
        .map_err(|error| host_import_error("math_max", error))?;

    linker
        .func_wrap("kali:rt", "math_min", |left: i64, right: i64| -> i64 {
            left.min(right)
        })
        .map_err(|error| host_import_error("math_min", error))?;

    linker
        .func_wrap("kali:rt", "math_abs", |value: i64| -> i64 {
            value.wrapping_abs()
        })
        .map_err(|error| host_import_error("math_abs", error))?;

    linker
        .func_wrap("kali:rt", "math_sign", |value: i64| -> i64 {
            value.signum()
        })
        .map_err(|error| host_import_error("math_sign", error))?;

    linker
        .func_wrap("kali:rt", "math_round", |value: i64| -> i64 { value })
        .map_err(|error| host_import_error("math_round", error))?;

    linker
        .func_wrap("kali:rt", "math_imul", |left: i64, right: i64| -> i64 {
            (left as i32).wrapping_mul(right as i32) as i64
        })
        .map_err(|error| host_import_error("math_imul", error))?;

    linker
        .func_wrap("kali:rt", "math_clz32", |value: i64| -> i64 {
            i64::from((value as u32).leading_zeros())
        })
        .map_err(|error| host_import_error("math_clz32", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "math_pow",
            |left: i64, right: i64| -> wasmtime::Result<i64> {
                if right < 0 {
                    if left == 1 {
                        return Ok(1);
                    }
                    if left == -1 {
                        return Ok(if right % 2 == 0 { 1 } else { -1 });
                    }
                    return Err(wasmtime::Error::msg(
                        "Math.pow negative exponents are unavailable unless the base is a statically-known ±1 in the current phase; use a non-negative exponent or the later compatibility path",
                    ));
                }
                Ok(left.wrapping_pow(right as u32))
            },
        )
        .map_err(|error| host_import_error("math_pow", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "int_to_string",
            |mut caller: Caller<'_, KaliHostState>, value: i64| -> i64 {
                let text = value.to_string();
                alloc_guest_string(&mut caller, text.as_bytes()).unwrap_or(0)
            },
        )
        .map_err(|error| host_import_error("int_to_string", error))?;

    // The terminal arm of `emit_as_string` for the CONSOLE sink. Identical in
    // behavior to what the console imports already do to a raw value -- decode a
    // tagged string handle, else render the integer -- but it returns a guest
    // string handle instead of writing to a stream, so codegen can put it in a
    // ladder. That is what lets the single- and multi-argument console lanes
    // share one renderer with the host rather than each having their own.
    //
    // Deliberately NOT the terminal arm for `+`/template literals:
    // STRING_HANDLE_TAG is the sign bit, so every negative integer attempts a
    // decode here and survives only because the bounds check fails. Widening
    // that to the concat population is a hazard this project declines to take
    // (spec §3.1).
    linker
        .func_wrap(
            "kali:rt",
            "value_to_string",
            |mut caller: Caller<'_, KaliHostState>, value: i64| -> i64 {
                let text = format_console_value(&mut caller, value);
                alloc_guest_string(&mut caller, text.as_bytes()).unwrap_or(0)
            },
        )
        .map_err(|error| host_import_error("value_to_string", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "string_concat",
            |mut caller: Caller<'_, KaliHostState>, a: i64, b: i64| -> i64 {
                let mut bytes = decode_string_handle_bytes(&mut caller, a).unwrap_or_default();
                bytes.extend(decode_string_handle_bytes(&mut caller, b).unwrap_or_default());
                alloc_guest_string(&mut caller, &bytes).unwrap_or(0)
            },
        )
        .map_err(|error| host_import_error("string_concat", error))?;

    // Current-arena twin of `string_concat` (fasta Spec 7 Task 4d): identical
    // except it allocates the result into the resettable current arena via
    // `alloc_guest_string_current` (`__alloc`). Codegen selects it per concat
    // site for a `+` whose result the escape gate proved iteration-local.
    linker
        .func_wrap(
            "kali:rt",
            "string_concat_arena",
            |mut caller: Caller<'_, KaliHostState>, a: i64, b: i64| -> i64 {
                let mut bytes = decode_string_handle_bytes(&mut caller, a).unwrap_or_default();
                bytes.extend(decode_string_handle_bytes(&mut caller, b).unwrap_or_default());
                alloc_guest_string_current(&mut caller, &bytes).unwrap_or(0)
            },
        )
        .map_err(|error| host_import_error("string_concat_arena", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "float_to_fixed",
            |mut caller: Caller<'_, KaliHostState>, value: f64, digits: i32| -> i64 {
                let d = digits.clamp(0, 100) as usize;
                let s = format!("{:.*}", d, value);
                alloc_guest_string(&mut caller, s.as_bytes()).unwrap_or(0)
            },
        )
        .map_err(|error| host_import_error("float_to_fixed", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "float_to_string",
            |mut caller: Caller<'_, KaliHostState>, value: f64| -> i64 {
                let text = format_js_number(value);
                alloc_guest_string(&mut caller, text.as_bytes()).unwrap_or(0)
            },
        )
        .map_err(|error| host_import_error("float_to_string", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "args_get",
            |mut caller: Caller<'_, KaliHostState>,
             index: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let Some(value) = caller.data().args.get(index as usize).cloned() else {
                    return Ok(-1);
                };
                write_guest_bytes(&mut caller, out_ptr, out_cap, value.as_bytes())
            },
        )
        .map_err(|error| host_import_error("args_get", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "fetch",
            |mut caller: Caller<'_, KaliHostState>,
             url_ptr: i32,
             url_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let url = read_guest_string(&mut caller, url_ptr, url_len)?;
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::NetworkFetch { url: url.clone() },
                )?;
                let response = blocking::get(&url)
                    .and_then(|resp| resp.error_for_status())
                    .map_err(|error| {
                        wasmtime::Error::msg(format!("failed to fetch '{}': {}", url, error))
                    })?;
                let bytes = response.bytes().map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "failed to read response body from '{}': {}",
                        url, error
                    ))
                })?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, bytes.as_ref())
            },
        )
        .map_err(|error| host_import_error("fetch", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "timer_set",
            |mut caller: Caller<'_, KaliHostState>,
             callback_id: i32,
             delay_ms: i32,
             repeat: i32,
             env_ptr: i64|
             -> wasmtime::Result<i32> {
                caller
                    .data_mut()
                    .schedule_timer(callback_id, delay_ms, repeat != 0, env_ptr)
            },
        )
        .map_err(|error| host_import_error("timer_set", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "setTimeout",
            |mut caller: Caller<'_, KaliHostState>,
             callback_id: i32,
             delay_ms: i32,
             env_ptr: i64|
             -> wasmtime::Result<i32> {
                caller
                    .data_mut()
                    .schedule_timer(callback_id, delay_ms, false, env_ptr)
            },
        )
        .map_err(|error| host_import_error("setTimeout", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "setInterval",
            |mut caller: Caller<'_, KaliHostState>,
             callback_id: i32,
             delay_ms: i32,
             env_ptr: i64|
             -> wasmtime::Result<i32> {
                caller
                    .data_mut()
                    .schedule_timer(callback_id, delay_ms, true, env_ptr)
            },
        )
        .map_err(|error| host_import_error("setInterval", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "timer_clear",
            |mut caller: Caller<'_, KaliHostState>, timer_id: i32| -> wasmtime::Result<()> {
                caller.data_mut().cancel_timer(timer_id)?;
                Ok(())
            },
        )
        .map_err(|error| host_import_error("timer_clear", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "clearTimeout",
            |mut caller: Caller<'_, KaliHostState>, timer_id: i32| -> wasmtime::Result<()> {
                caller.data_mut().cancel_timer(timer_id)?;
                Ok(())
            },
        )
        .map_err(|error| host_import_error("clearTimeout", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "clearInterval",
            |mut caller: Caller<'_, KaliHostState>, timer_id: i32| -> wasmtime::Result<()> {
                caller.data_mut().cancel_timer(timer_id)?;
                Ok(())
            },
        )
        .map_err(|error| host_import_error("clearInterval", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "event_target_new",
            |mut caller: Caller<'_, KaliHostState>| -> i64 {
                i64::from(caller.data_mut().register_event_target())
            },
        )
        .map_err(|error| host_import_error("event_target_new", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "event_listener_add",
            |mut caller: Caller<'_, KaliHostState>,
             target: i64,
             name_ptr: i32,
             name_len: i32,
             callback_id: i32,
             env_ptr: i64|
             -> wasmtime::Result<()> {
                let event_type = read_guest_string(&mut caller, name_ptr, name_len)?;
                caller.data_mut().add_event_listener(
                    target as u32,
                    event_type,
                    callback_id,
                    env_ptr,
                );
                Ok(())
            },
        )
        .map_err(|error| host_import_error("event_listener_add", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "event_dispatch",
            |mut caller: Caller<'_, KaliHostState>,
             target: i64,
             name_ptr: i32,
             name_len: i32|
             -> wasmtime::Result<i32> {
                let event_type = read_guest_string(&mut caller, name_ptr, name_len)?;
                // Snapshot BEFORE re-entering the guest: node parity
                // (listeners added during dispatch don't fire this round)
                // AND the borrow constraint (no `data()`/`data_mut()` borrow
                // may live across a re-entrant guest call).
                let snapshot = caller
                    .data()
                    .event_listener_snapshot(target as u32, &event_type);
                for (callback_id, env_ptr) in snapshot {
                    invoke_callback_reentrant(&mut caller, callback_id, env_ptr)?;
                }
                Ok(1)
            },
        )
        .map_err(|error| host_import_error("event_dispatch", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "queue_microtask",
            |mut caller: Caller<'_, KaliHostState>,
             callback_id: i32,
             env_ptr: i64|
             -> wasmtime::Result<()> {
                caller.data_mut().queue_microtask(callback_id, env_ptr);
                Ok(())
            },
        )
        .map_err(|error| host_import_error("queue_microtask", error))?;

    linker
        .func_wrap(
            "kali:rt",
            "queueMicrotask",
            |mut caller: Caller<'_, KaliHostState>,
             callback_id: i32,
             env_ptr: i64|
             -> wasmtime::Result<()> {
                caller.data_mut().queue_microtask(callback_id, env_ptr);
                Ok(())
            },
        )
        .map_err(|error| host_import_error("queueMicrotask", error))?;

    Ok(())
}
