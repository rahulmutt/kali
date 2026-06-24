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
    // `__kali_callback_<id>` for timer and microtask scheduling.
    let export_name = format!("__kali_callback_{}", callback_id);
    let callback = instance
        .get_typed_func::<(), ()>(&mut *store, &export_name)
        .map_err(|error| {
            Diagnostic::error(
                e4::UNCAUGHT_ERROR as u32,
                format!("missing timer callback '{}': {}", export_name, error),
            )
        })?;

    if let Err(error) = callback.call(&mut *store, ()) {
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
