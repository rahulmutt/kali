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

pub(crate) fn drain_event_loop(
    instance: &Instance,
    store: &mut Store<KaliHostState>,
) -> Result<(), Diagnostic> {
    loop {
        let microtask = {
            let state = store.data_mut();
            state.pending_microtasks.pop_front()
        };

        if let Some((callback_id, env_ptr)) = microtask {
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
            .map_err(|error| runtime_error_diagnostic(format!(
                "failed to set __current_env before callback: {error}"
            )))?;
    }

    let result = invoke_callback_inner(instance, store, callback_id);

    if let (Some(global), Some(prev)) = (env_global, saved_env) {
        // Restore on BOTH the ok and error paths (before propagating), so a
        // trapping callback still leaves `current_env` as its caller expects.
        let _ = global.set(&mut *store, prev);
    }
    result
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
