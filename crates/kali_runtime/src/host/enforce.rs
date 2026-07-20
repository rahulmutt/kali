//! Sandbox enforcement, event-loop draining, and callback invocation.
use crate::*;

pub(crate) fn enforce_operation(
    state: &mut KaliHostState,
    op: HostOperation,
) -> wasmtime::Result<()> {
    if let Some(policy) = state.policy.as_ref() {
        policy.check_operation(op).map_err(|diagnostic| {
            state.pending_diagnostic = Some(diagnostic.clone());
            let marker = match diagnostic.code {
                Some(code) if code == e4::EFFECT_NOT_PERMITTED as u32 => "KALI_E4001",
                Some(code) if code == e4::RESOURCE_LIMIT_EXCEEDED as u32 => "KALI_E4003",
                _ => "KALI_E4000",
            };
            wasmtime::Error::msg(format!("{}: {}", marker, diagnostic.message))
        })
    } else {
        Ok(())
    }
}

/// Total scheduled-callback invocations (microtasks + timers) one drain may
/// perform before failing loudly. Terminating programs sit far below this;
/// only programs that would hang node (an uncleared `setInterval`, a
/// self-requeueing microtask) ever reach it — Stage D's bounded-drain
/// decision: trap distinctly instead of hanging the process and the gate.
pub(crate) const EVENT_LOOP_INVOCATION_BUDGET: u64 = 100_000;

pub(crate) fn drain_event_loop(
    instance: &Instance,
    store: &mut Store<KaliHostState>,
) -> Result<(), Diagnostic> {
    let mut invocations: u64 = 0;
    loop {
        if invocations >= EVENT_LOOP_INVOCATION_BUDGET {
            return Err(Diagnostic::error(
                e4::RESOURCE_LIMIT_EXCEEDED as u32,
                format!(
                    "event loop did not quiesce: {EVENT_LOOP_INVOCATION_BUDGET} scheduled-callback \
                     invocations were drained and callbacks are still pending (an uncleared \
                     setInterval or a self-requeueing microtask?)"
                ),
            ));
        }

        let microtask = {
            let state = store.data_mut();
            state.pending_microtasks.pop_front()
        };

        if let Some((callback_id, env_ptr)) = microtask {
            invocations += 1;
            invoke_callback(instance, store, callback_id, env_ptr)?;
            continue;
        }

        let next_timer = {
            let state = store.data();
            state
                .pending_timers
                .iter()
                .min_by_key(|(_, timer)| (timer.due_at_ms, timer.seq))
                .map(|(timer_id, timer)| (*timer_id, timer.clone()))
        };

        let Some((timer_id, timer)) = next_timer else {
            break;
        };

        {
            let state = store.data_mut();
            // Advance the virtual clock directly to the due time — no sleeping.
            state.virtual_clock_ms = state.virtual_clock_ms.max(timer.due_at_ms);
            state.pending_timers.remove(&timer_id);
        }

        invocations += 1;
        invoke_callback(instance, store, timer.callback_id, timer.env_ptr)?;

        if let Some(interval_ms) = timer.repeat_interval_ms {
            let cancelled = {
                let state = store.data_mut();
                state.cancelled_timers.remove(&timer_id)
            };

            if !cancelled {
                let state = store.data_mut();
                let seq = state.next_timer_seq;
                state.next_timer_seq += 1;
                let due_at_ms = state.virtual_clock_ms + interval_ms;
                state.pending_timers.insert(
                    timer_id,
                    ScheduledTimer {
                        callback_id: timer.callback_id,
                        env_ptr: timer.env_ptr,
                        due_at_ms,
                        seq,
                        repeat_interval_ms: Some(interval_ms),
                    },
                );
            }
        }
    }

    Ok(())
}

pub(crate) fn invoke_callback(
    instance: &Instance,
    store: &mut Store<KaliHostState>,
    callback_id: i32,
    env_ptr: i64,
) -> Result<(), Diagnostic> {
    // Stage C C3: a deferred callback must run with the `current_env` that was
    // active when it was scheduled (its capturing activation's env record),
    // not whatever the global happens to hold at drain time. Set the exported
    // `__current_env` global to the stored `env_ptr` before the nullary call and
    // RESTORE the previous value after, so queued/nested callbacks never leak
    // envs into each other. Modules that own no promotable env (the hand-written
    // runtime WAT fixtures, and any guest with no captures) export no
    // `__current_env` global; `get_global` returns `None` and this whole
    // save/set/restore dance is a no-op, exactly like baseline.
    let env_global = instance.get_global(&mut *store, "__current_env");
    let saved_env = env_global.map(|global| global.get(&mut *store));
    if let Some(global) = env_global {
        // `set` fails only on a type/immutability mismatch; the global is a
        // mutable i64 by construction, so a failure is a codegen bug, not a
        // guest-reachable condition. Surface it rather than silently miscompile.
        global
            .set(&mut *store, Val::I64(env_ptr))
            .map_err(|error| {
                runtime_error_diagnostic(format!(
                    "failed to set __current_env before callback: {error}"
                ))
            })?;
    }

    let result = invoke_callback_inner(instance, store, callback_id);

    if let (Some(global), Some(prev)) = (env_global, saved_env) {
        // Restore on BOTH the ok and error paths (before propagating), so a
        // trapping callback still leaves `current_env` as its caller expects.
        let _ = global.set(&mut *store, prev);
    }
    result
}

/// Synchronously invoke `__kali_callback_<id>` from INSIDE a host import,
/// while guest code (e.g. `_start`) is still on the wasm stack (Stage D
/// event dispatch). Mirrors `invoke_callback`'s export-lookup, env
/// save/restore, and result-arity handling — but `Caller`-based (there is no
/// `Instance`/`Store` split available inside a `func_wrap` closure) and
/// returning `wasmtime::Result` (a `Diagnostic` cannot be constructed here
/// the way `invoke_callback_inner` does, since there is no
/// `store.data_mut().pending_diagnostic` slot to consult before the
/// generic-trap fallback — the caller's `func_wrap` return type is
/// `wasmtime::Result`, matching every other host import in this file).
/// NEVER call this while holding a `caller.data_mut()`/`caller.data()`
/// borrow across the re-entrant call (E0499 otherwise) — snapshot any host
/// state needed by the caller BEFORE invoking.
pub(crate) fn invoke_callback_reentrant(
    caller: &mut Caller<'_, KaliHostState>,
    callback_id: i32,
    env_ptr: i64,
) -> wasmtime::Result<()> {
    let export_name = format!("__kali_callback_{}", callback_id);
    let callback = caller
        .get_export(&export_name)
        .and_then(|export| export.into_func())
        .ok_or_else(|| {
            wasmtime::Error::msg(format!("missing callback export '{}'", export_name))
        })?;

    // Stage C C3 parity with `invoke_callback`: run the listener with the
    // `current_env` that was active at registration time, restoring the
    // caller's (e.g. `_start`'s) env afterward on BOTH the ok and err paths.
    // Modules with no `__current_env` export (no promotable env) make this a
    // no-op, exactly like `invoke_callback`.
    let env_global = caller
        .get_export("__current_env")
        .and_then(|export| export.into_global());
    let saved_env = env_global.map(|global| global.get(&mut *caller));
    if let Some(global) = env_global {
        global
            .set(&mut *caller, Val::I64(env_ptr))
            .map_err(|error| {
                wasmtime::Error::msg(format!(
                    "failed to set __current_env before callback: {error}"
                ))
            })?;
    }

    // Callbacks are nullary but their result arity varies (see
    // `invoke_callback_inner`'s comment) — invoke through the untyped API and
    // discard any results so both hand-written `() -> ()` stubs and
    // codegen-emitted `() -> i64`/`() -> f64` functions dispatch.
    let mut results: Vec<Val> = callback
        .ty(&mut *caller)
        .results()
        .map(|ty| match ty {
            wasmtime::ValType::I32 => Val::I32(0),
            wasmtime::ValType::F32 => Val::F32(0),
            wasmtime::ValType::F64 => Val::F64(0),
            _ => Val::I64(0),
        })
        .collect();

    let call_result = callback
        .call(&mut *caller, &[], &mut results)
        .map_err(|error| {
            wasmtime::Error::msg(format!(
                "runtime trap in callback '{}': {}",
                export_name, error
            ))
        });

    if let (Some(global), Some(prev)) = (env_global, saved_env) {
        let _ = global.set(&mut *caller, prev);
    }

    call_result
}

fn invoke_callback_inner(
    instance: &Instance,
    store: &mut Store<KaliHostState>,
    callback_id: i32,
) -> Result<(), Diagnostic> {
    // The current guest ABI uses exported callback stubs named
    // `__kali_callback_<id>` for timer/microtask scheduling and `Kali.test`
    // callbacks. Callbacks are nullary but their result arity varies:
    // hand-authored stubs are `() -> ()`, while codegen-emitted functions
    // (e.g. arrow callbacks registered through `test_register`) are
    // `() -> i64`/`() -> f64` — every collected `FunctionPlan` carries
    // `result: true`. Invoke through the untyped API and discard any results
    // so both shapes dispatch.
    let export_name = format!("__kali_callback_{}", callback_id);
    let callback = instance
        .get_func(&mut *store, &export_name)
        .ok_or_else(|| {
            Diagnostic::error(
                e4::UNCAUGHT_ERROR as u32,
                format!("missing timer callback '{}'", export_name),
            )
        })?;
    let mut results: Vec<Val> = callback
        .ty(&*store)
        .results()
        .map(|ty| match ty {
            wasmtime::ValType::I32 => Val::I32(0),
            wasmtime::ValType::F32 => Val::F32(0),
            wasmtime::ValType::F64 => Val::F64(0),
            _ => Val::I64(0),
        })
        .collect();

    if let Err(error) = callback.call(&mut *store, &[], &mut results) {
        if let Some(diagnostic) = store.data_mut().pending_diagnostic.take() {
            return Err(diagnostic);
        }
        return Err(runtime_error_diagnostic(format!(
            "runtime trap in callback '{}': {}",
            export_name, error
        )));
    }

    Ok(())
}
