//! Node host-import registration (`kali:node` namespace) for the wasmtime linker.
use crate::*;

pub(crate) fn register_node_host_imports(
    linker: &mut Linker<KaliHostState>,
    node_projection: NodeRuntimeProjection,
) -> Result<(), Diagnostic> {
    let fs_promises = node_projection.fs_promises().clone();
    let fs_promises_for_read_file = fs_promises.clone();
    let fs_promises_for_write_text = fs_promises.clone();
    let fs_promises_for_write_file = fs_promises.clone();
    let process = std::sync::Arc::new(std::sync::Mutex::new(node_projection.process().clone()));
    let process_for_len = std::sync::Arc::clone(&process);
    let process_for_argv_get = std::sync::Arc::clone(&process);
    let process_for_env_get = std::sync::Arc::clone(&process);
    let stream = node_projection.stream();
    let http = node_projection.http();
    let child_process: NodeChildProcess = node_projection.child_process();
    let os = node_projection.os();

    linker
        .func_wrap(
            "kali:node",
            "path_normalize",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let normalized = NodePath::normalize(Path::new(&path));
                write_guest_string(&mut caller, out_ptr, out_cap, normalized.to_string_lossy())
            },
        )
        .map_err(|error| host_import_error("path_normalize", error))?;

    linker
        .func_wrap(
            "kali:node",
            "path_join",
            |mut caller: Caller<'_, KaliHostState>,
             base_ptr: i32,
             base_len: i32,
             segment_ptr: i32,
             segment_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let base = read_guest_string(&mut caller, base_ptr, base_len)?;
                let segment = read_guest_string(&mut caller, segment_ptr, segment_len)?;
                let joined = NodePath::join(Path::new(&base), Path::new(&segment));
                write_guest_string(&mut caller, out_ptr, out_cap, joined.to_string_lossy())
            },
        )
        .map_err(|error| host_import_error("path_join", error))?;

    linker
        .func_wrap(
            "kali:node",
            "path_resolve",
            |mut caller: Caller<'_, KaliHostState>,
             base_ptr: i32,
             base_len: i32,
             input_ptr: i32,
             input_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let base = read_guest_string(&mut caller, base_ptr, base_len)?;
                let input = read_guest_string(&mut caller, input_ptr, input_len)?;
                let resolved = NodePath::resolve(Path::new(&base), Path::new(&input));
                write_guest_string(&mut caller, out_ptr, out_cap, resolved.to_string_lossy())
            },
        )
        .map_err(|error| host_import_error("path_resolve", error))?;

    linker
        .func_wrap(
            "kali:node",
            "path_dirname",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let dirname = NodePath::dirname(Path::new(&path));
                write_guest_string(&mut caller, out_ptr, out_cap, dirname.to_string_lossy())
            },
        )
        .map_err(|error| host_import_error("path_dirname", error))?;

    linker
        .func_wrap(
            "kali:node",
            "path_basename",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let basename = NodePath::basename(Path::new(&path));
                write_guest_string(&mut caller, out_ptr, out_cap, basename)
            },
        )
        .map_err(|error| host_import_error("path_basename", error))?;

    linker
        .func_wrap(
            "kali:node",
            "path_extname",
            |mut caller: Caller<'_, KaliHostState>,
             path_ptr: i32,
             path_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let path = read_guest_string(&mut caller, path_ptr, path_len)?;
                let extname = NodePath::extname(Path::new(&path));
                write_guest_string(&mut caller, out_ptr, out_cap, extname)
            },
        )
        .map_err(|error| host_import_error("path_extname", error))?;

    linker
        .func_wrap(
            "kali:node",
            "path_relative",
            |mut caller: Caller<'_, KaliHostState>,
             from_ptr: i32,
             from_len: i32,
             to_ptr: i32,
             to_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let from = read_guest_string(&mut caller, from_ptr, from_len)?;
                let to = read_guest_string(&mut caller, to_ptr, to_len)?;
                let relative = NodePath::relative(Path::new(&from), Path::new(&to));
                write_guest_string(&mut caller, out_ptr, out_cap, relative.to_string_lossy())
            },
        )
        .map_err(|error| host_import_error("path_relative", error))?;

    linker
        .func_wrap(
            "kali:node",
            "url_parse",
            |mut caller: Caller<'_, KaliHostState>,
             input_ptr: i32,
             input_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let input = read_guest_string(&mut caller, input_ptr, input_len)?;
                let parsed = NodeUrl::parse(&input)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_string(&mut caller, out_ptr, out_cap, parsed.as_str())
            },
        )
        .map_err(|error| host_import_error("url_parse", error))?;

    linker
        .func_wrap(
            "kali:node",
            "url_resolve",
            |mut caller: Caller<'_, KaliHostState>,
             base_ptr: i32,
             base_len: i32,
             input_ptr: i32,
             input_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let base = read_guest_string(&mut caller, base_ptr, base_len)?;
                let input = read_guest_string(&mut caller, input_ptr, input_len)?;
                let resolved = NodeUrl::resolve(&base, &input)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_string(&mut caller, out_ptr, out_cap, resolved.as_str())
            },
        )
        .map_err(|error| host_import_error("url_resolve", error))?;

    linker
        .func_wrap(
            "kali:node",
            "crypto_create_hash",
            |mut caller: Caller<'_, KaliHostState>,
             algorithm_ptr: i32,
             algorithm_len: i32,
             data_ptr: i32,
             data_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let algorithm = read_guest_string(&mut caller, algorithm_ptr, algorithm_len)?;
                let data = read_guest_bytes(&mut caller, data_ptr, data_len)?;
                let digest = NodeCrypto::create_hash(&algorithm, &data)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_string(&mut caller, out_ptr, out_cap, digest)
            },
        )
        .map_err(|error| host_import_error("crypto_create_hash", error))?;

    linker
        .func_wrap(
            "kali:node",
            "crypto_create_hmac",
            |mut caller: Caller<'_, KaliHostState>,
             algorithm_ptr: i32,
             algorithm_len: i32,
             key_ptr: i32,
             key_len: i32,
             data_ptr: i32,
             data_len: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let algorithm = read_guest_string(&mut caller, algorithm_ptr, algorithm_len)?;
                let key = read_guest_bytes(&mut caller, key_ptr, key_len)?;
                let data = read_guest_bytes(&mut caller, data_ptr, data_len)?;
                let digest = NodeCrypto::create_hmac(&algorithm, &key, &data)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_string(&mut caller, out_ptr, out_cap, digest)
            },
        )
        .map_err(|error| host_import_error("crypto_create_hmac", error))?;

    linker
        .func_wrap(
            "kali:node",
            "crypto_random_uuid",
            |mut caller: Caller<'_, KaliHostState>,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let uuid = NodeCrypto::random_uuid_v4()
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_string(&mut caller, out_ptr, out_cap, uuid)
            },
        )
        .map_err(|error| host_import_error("crypto_random_uuid", error))?;

    linker
        .func_wrap(
            "kali:node",
            "crypto_random_bytes",
            |mut caller: Caller<'_, KaliHostState>,
             length: i32,
             out_ptr: i32,
             out_cap: i32|
             -> wasmtime::Result<i32> {
                let length = checked_offset(length)?;
                let bytes = NodeCrypto::random_bytes(length)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, &bytes)
            },
        )
        .map_err(|error| host_import_error("crypto_random_bytes", error))?;

    linker
        .func_wrap(
            "kali:node",
            "os_platform",
            move |mut caller: Caller<'_, KaliHostState>,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                write_guest_string(&mut caller, out_ptr, out_cap, os.platform())
            },
        )
        .map_err(|error| host_import_error("os_platform", error))?;

    linker
        .func_wrap(
            "kali:node",
            "os_arch",
            move |mut caller: Caller<'_, KaliHostState>,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                write_guest_string(&mut caller, out_ptr, out_cap, os.arch())
            },
        )
        .map_err(|error| host_import_error("os_arch", error))?;

    linker
        .func_wrap(
            "kali:node",
            "os_eol",
            move |mut caller: Caller<'_, KaliHostState>,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                write_guest_string(&mut caller, out_ptr, out_cap, os.eol())
            },
        )
        .map_err(|error| host_import_error("os_eol", error))?;

    linker
        .func_wrap(
            "kali:node",
            "os_tmpdir",
            move |mut caller: Caller<'_, KaliHostState>,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                write_guest_string(&mut caller, out_ptr, out_cap, os.tmpdir().to_string_lossy())
            },
        )
        .map_err(|error| host_import_error("os_tmpdir", error))?;

    linker
        .func_wrap("kali:node", "os_cpus", move || -> i32 { os.cpus() as i32 })
        .map_err(|error| host_import_error("os_cpus", error))?;

    linker
        .func_wrap(
            "kali:node",
            "fs_promises_read_text_file",
            move |mut caller: Caller<'_, KaliHostState>,
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
                let text = fs_promises.read_text_file(&host_path).map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "failed to read '{}': {}",
                        host_path.display(),
                        error
                    ))
                })?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, text.as_bytes())
            },
        )
        .map_err(|error| host_import_error("fs_promises_read_text_file", error))?;

    linker
        .func_wrap(
            "kali:node",
            "fs_promises_read_file",
            move |mut caller: Caller<'_, KaliHostState>,
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
                let bytes = fs_promises_for_read_file
                    .read_file(&host_path)
                    .map_err(|error| {
                        wasmtime::Error::msg(format!(
                            "failed to read '{}': {}",
                            host_path.display(),
                            error
                        ))
                    })?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, &bytes)
            },
        )
        .map_err(|error| host_import_error("fs_promises_read_file", error))?;

    linker
        .func_wrap(
            "kali:node",
            "fs_promises_write_text_file",
            move |mut caller: Caller<'_, KaliHostState>,
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
                let text = String::from_utf8(data).map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "node fs/promises write_text_file expects UTF-8: {}",
                        error
                    ))
                })?;
                fs_promises_for_write_text
                    .write_text_file(&host_path, text)
                    .map_err(|error| {
                        wasmtime::Error::msg(format!(
                            "failed to write '{}': {}",
                            host_path.display(),
                            error
                        ))
                    })?;
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("fs_promises_write_text_file", error))?;

    linker
        .func_wrap(
            "kali:node",
            "fs_promises_write_file",
            move |mut caller: Caller<'_, KaliHostState>,
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
                fs_promises_for_write_file
                    .write_file(&host_path, &data)
                    .map_err(|error| {
                        wasmtime::Error::msg(format!(
                            "failed to write '{}': {}",
                            host_path.display(),
                            error
                        ))
                    })?;
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("fs_promises_write_file", error))?;

    linker
        .func_wrap(
            "kali:node",
            "stream_concat",
            move |mut caller: Caller<'_, KaliHostState>,
                  left_ptr: i32,
                  left_len: i32,
                  right_ptr: i32,
                  right_len: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let left = read_guest_bytes(&mut caller, left_ptr, left_len)?;
                let right = read_guest_bytes(&mut caller, right_ptr, right_len)?;
                let concatenated = stream.concat_bytes(&left, &right);
                write_guest_bytes(&mut caller, out_ptr, out_cap, &concatenated)
            },
        )
        .map_err(|error| host_import_error("stream_concat", error))?;

    linker
        .func_wrap(
            "kali:node",
            "http_get",
            move |mut caller: Caller<'_, KaliHostState>,
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
                let response = http
                    .request_get(&url)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, response.body())
            },
        )
        .map_err(|error| host_import_error("http_get", error))?;

    linker
        .func_wrap(
            "kali:node",
            "buffer_to_hex",
            move |mut caller: Caller<'_, KaliHostState>,
                  data_ptr: i32,
                  data_len: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let data = read_guest_bytes(&mut caller, data_ptr, data_len)?;
                let hex = NodeBuffer::from_bytes(data).to_hex();
                write_guest_string(&mut caller, out_ptr, out_cap, hex)
            },
        )
        .map_err(|error| host_import_error("buffer_to_hex", error))?;

    linker
        .func_wrap(
            "kali:node",
            "buffer_from_hex",
            move |mut caller: Caller<'_, KaliHostState>,
                  input_ptr: i32,
                  input_len: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let input = read_guest_string(&mut caller, input_ptr, input_len)?;
                let buffer = NodeBuffer::from_hex(&input)
                    .map_err(|error| wasmtime::Error::msg(error.to_string()))?;
                write_guest_bytes(&mut caller, out_ptr, out_cap, buffer.as_slice())
            },
        )
        .map_err(|error| host_import_error("buffer_from_hex", error))?;

    linker
        .func_wrap(
            "kali:node",
            "event_on",
            move |mut caller: Caller<'_, KaliHostState>,
                  event_ptr: i32,
                  event_len: i32,
                  callback_id: i32|
                  -> wasmtime::Result<i32> {
                let event_type = read_guest_string(&mut caller, event_ptr, event_len)?;
                caller
                    .data_mut()
                    .register_event_listener(event_type, callback_id);
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("event_on", error))?;

    linker
        .func_wrap(
            "kali:node",
            "event_listener_count",
            move |mut caller: Caller<'_, KaliHostState>,
                  event_ptr: i32,
                  event_len: i32|
                  -> wasmtime::Result<i32> {
                let event_type = read_guest_string(&mut caller, event_ptr, event_len)?;
                Ok(caller.data().event_listener_count(&event_type) as i32)
            },
        )
        .map_err(|error| host_import_error("event_listener_count", error))?;

    linker
        .func_wrap(
            "kali:node",
            "event_emit",
            move |mut caller: Caller<'_, KaliHostState>,
                  event_ptr: i32,
                  event_len: i32|
                  -> wasmtime::Result<i32> {
                let event_type = read_guest_string(&mut caller, event_ptr, event_len)?;
                let callback_ids = caller.data().event_listener_callbacks(&event_type);
                for callback_id in &callback_ids {
                    caller.data_mut().queue_microtask(*callback_id);
                }
                Ok(callback_ids.len() as i32)
            },
        )
        .map_err(|error| host_import_error("event_emit", error))?;

    linker
        .func_wrap(
            "kali:node",
            "util_format",
            move |mut caller: Caller<'_, KaliHostState>,
                  left_ptr: i32,
                  left_len: i32,
                  right_ptr: i32,
                  right_len: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let left = read_guest_string(&mut caller, left_ptr, left_len)?;
                let right = read_guest_string(&mut caller, right_ptr, right_len)?;
                let formatted = NodeUtil::format(&[left.as_str(), right.as_str()]);
                write_guest_string(&mut caller, out_ptr, out_cap, formatted)
            },
        )
        .map_err(|error| host_import_error("util_format", error))?;

    linker
        .func_wrap(
            "kali:node",
            "assert_equal",
            move |mut caller: Caller<'_, KaliHostState>,
                  actual_ptr: i32,
                  actual_len: i32,
                  expected_ptr: i32,
                  expected_len: i32|
                  -> wasmtime::Result<i32> {
                let actual = read_guest_string(&mut caller, actual_ptr, actual_len)?;
                let expected = read_guest_string(&mut caller, expected_ptr, expected_len)?;
                NodeAssert::equal(&actual, &expected, "assert_equal")
                    .map_err(wasmtime::Error::msg)?;
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("assert_equal", error))?;

    linker
        .func_wrap("kali:node", "process_args_len", move || -> i32 {
            process_for_len
                .lock()
                .expect("node process mutex poisoned")
                .argv_len() as i32
        })
        .map_err(|error| host_import_error("process_args_len", error))?;

    let process_for_exit = std::sync::Arc::clone(&process);
    linker
        .func_wrap(
            "kali:node",
            "process_exit",
            move |mut caller: Caller<'_, KaliHostState>, code: i64| -> wasmtime::Result<()> {
                let exit_code = i32::try_from(code).unwrap_or_else(|_| {
                    if code.is_negative() {
                        i32::MIN
                    } else {
                        i32::MAX
                    }
                });
                process_for_exit
                    .lock()
                    .expect("node process mutex poisoned")
                    .set_exit_code(exit_code);
                caller.data_mut().pending_exit_code = Some(exit_code);
                Err(wasmtime::Error::msg(format!(
                    "process.exit({exit_code}) requested guest termination"
                )))
            },
        )
        .map_err(|error| host_import_error("process_exit", error))?;

    linker
        .func_wrap(
            "kali:node",
            "process_args_get",
            move |mut caller: Caller<'_, KaliHostState>,
                  index: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let Some(value) = process_for_argv_get
                    .lock()
                    .expect("node process mutex poisoned")
                    .argv_at(index as usize)
                    .map(str::to_owned)
                else {
                    return Ok(-1);
                };
                write_guest_bytes(&mut caller, out_ptr, out_cap, value.as_bytes())
            },
        )
        .map_err(|error| host_import_error("process_args_get", error))?;

    linker
        .func_wrap(
            "kali:node",
            "process_env_get",
            move |mut caller: Caller<'_, KaliHostState>,
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
                let Some(value) = process_for_env_get
                    .lock()
                    .expect("node process mutex poisoned")
                    .env_get(&key)
                    .map(str::to_owned)
                else {
                    return Ok(-1);
                };
                write_guest_bytes(&mut caller, out_ptr, out_cap, value.as_bytes())
            },
        )
        .map_err(|error| host_import_error("process_env_get", error))?;

    linker
        .func_wrap(
            "kali:node",
            "process_stdout_write",
            move |mut caller: Caller<'_, KaliHostState>,
                  text_ptr: i32,
                  text_len: i32|
                  -> wasmtime::Result<i32> {
                let text = read_guest_string(&mut caller, text_ptr, text_len)?;
                append_stdout_raw(caller.data_mut(), text);
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("process_stdout_write", error))?;

    linker
        .func_wrap(
            "kali:node",
            "process_stderr_write",
            move |mut caller: Caller<'_, KaliHostState>,
                  text_ptr: i32,
                  text_len: i32|
                  -> wasmtime::Result<i32> {
                let text = read_guest_string(&mut caller, text_ptr, text_len)?;
                append_stderr_raw(caller.data_mut(), text);
                Ok(0)
            },
        )
        .map_err(|error| host_import_error("process_stderr_write", error))?;

    linker
        .func_wrap(
            "kali:node",
            "process_spawn",
            move |mut caller: Caller<'_, KaliHostState>,
                  command_ptr: i32,
                  command_len: i32,
                  args_ptr: i32,
                  args_len: i32,
                  out_ptr: i32,
                  out_cap: i32|
                  -> wasmtime::Result<i32> {
                let command = read_guest_string(&mut caller, command_ptr, command_len)?;
                let encoded_args = read_guest_string(&mut caller, args_ptr, args_len)?;
                enforce_operation(
                    caller.data_mut(),
                    HostOperation::ProcessSpawn {
                        executable: command.clone(),
                    },
                )?;
                let args = decode_spawn_args(&encoded_args);
                {
                    let state = caller.data_mut();
                    state.begin_spawn()?;
                }
                let output = match child_process.spawn(&command, &args) {
                    Ok(output) => output,
                    Err(error) => {
                        caller.data_mut().finish_spawn();
                        return Err(wasmtime::Error::msg(error.to_string()));
                    }
                };
                {
                    let state = caller.data_mut();
                    state.finish_spawn();
                }
                let stdout = output.stdout();
                write_guest_bytes(&mut caller, out_ptr, out_cap, stdout)?;
                Ok(output.status())
            },
        )
        .map_err(|error| host_import_error("process_spawn", error))?;

    Ok(())
}
