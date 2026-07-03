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
        let microtask_id = {
            let state = store.data_mut();
            state.pending_microtasks.pop_front()
        };

        if let Some(callback_id) = microtask_id {
            invoke_callback(instance, store, callback_id)?;
            continue;
        }

        let next_timer = {
            let state = store.data();
            state
                .pending_timers
                .iter()
                .min_by_key(|(_, timer)| timer.due_at)
                .map(|(timer_id, timer)| (*timer_id, timer.clone()))
        };

        let Some((timer_id, timer)) = next_timer else {
            break;
        };

        let now = Instant::now();
        if timer.due_at > now {
            thread::sleep(timer.due_at - now);
            continue;
        }

        {
            let state = store.data_mut();
            state.pending_timers.remove(&timer_id);
        }

        invoke_callback(instance, store, timer.callback_id)?;

        if let Some(interval) = timer.repeat_interval {
            let cancelled = {
                let state = store.data_mut();
                state.cancelled_timers.remove(&timer_id)
            };

            if !cancelled {
                let state = store.data_mut();
                state.pending_timers.insert(
                    timer_id,
                    ScheduledTimer {
                        callback_id: timer.callback_id,
                        due_at: Instant::now() + interval,
                        repeat_interval: Some(interval),
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
