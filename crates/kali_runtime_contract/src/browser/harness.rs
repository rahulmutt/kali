//! Browser harness-script generators: prelude, module script, page, and bundle variants.
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};

/// Build the shared browser-bundle smoke harness prelude.
///
/// The generated snippet installs a deterministic `fetch` shim that can resolve the
/// emitted `.wasm` file alongside the bundle glue, so higher-level browser-harness
/// callers only need to append the command-specific body that exercises the exports.
pub fn browser_bundle_harness_prelude(bundle_dir: &str, allow_subpaths: bool) -> String {
    if allow_subpaths {
        format!(
            r#"import fs from 'node:fs/promises';
import {{ fileURLToPath }} from 'node:url';

const bundleJs = new URL('./{bundle_dir}/{bundle_dir}.js', import.meta.url);
const bundleRoot = new URL('./{bundle_dir}/', import.meta.url);

globalThis.fetch = async (input) => {{
  const url = input instanceof URL ? input : new URL(String(input));
  if (url.href.startsWith(bundleRoot.href) && url.pathname.endsWith('.wasm')) {{
    const bytes = await fs.readFile(fileURLToPath(url));
    return new Response(bytes, {{ headers: {{ 'content-type': 'application/wasm' }} }});
  }}
  throw new Error(`unexpected fetch ${{String(input)}}`);
}};

"#,
            bundle_dir = bundle_dir,
        )
    } else {
        format!(
            r#"import fs from 'node:fs/promises';
import {{ fileURLToPath }} from 'node:url';

const bundleJs = new URL('./{bundle_dir}/{bundle_dir}.js', import.meta.url);
const wasmUrl = new URL('./{bundle_dir}/{bundle_dir}.wasm', import.meta.url);

globalThis.fetch = async (input) => {{
  const url = input instanceof URL ? input : new URL(String(input));
  if (url.href === wasmUrl.href) {{
    const bytes = await fs.readFile(fileURLToPath(url));
    return new Response(bytes, {{ headers: {{ 'content-type': 'application/wasm' }} }});
  }}
  throw new Error(`unexpected fetch ${{String(input)}}`);
}};

"#,
            bundle_dir = bundle_dir,
        )
    }
}

/// Build a complete browser-bundle harness script from the shared prelude and a body snippet.
pub fn browser_bundle_harness_script(bundle_dir: &str, allow_subpaths: bool, body: &str) -> String {
    format!(
        "{}{}",
        browser_bundle_harness_prelude(bundle_dir, allow_subpaths),
        body
    )
}

/// The completion binding a browser harness page invokes once its body has
/// finished (successfully or not). A DevTools/CDP driver installs it via
/// `Runtime.addBinding`; Chromium requires binding functions to be called
/// with exactly one string argument, so pages pass `''`.
pub const BROWSER_HARNESS_DONE_BINDING: &str = "__kaliHarnessDone";

/// Build a browser-native harness page for an HTTP-served bundle. Unlike the
/// node-only prelude above, this emits no `node:` imports and installs no
/// fetch shim: the bundle glue's own `fetch(wasmUrl)` works once the bundle
/// directory is served over HTTP next to this page. The module script defines
/// `bundleJs` for the body — the same body contract as
/// [`browser_bundle_harness_script`], though here the body runs inside a
/// `try` block, so its declarations are block-scoped — reports body failures via
/// `console.error`, and always invokes [`BROWSER_HARNESS_DONE_BINDING`] when
/// a driver has installed it.
pub fn browser_bundle_harness_page(bundle_dir: &str, body: &str) -> String {
    format!(
        r#"<!doctype html>
<meta charset="utf-8">
<title>Kali browser bundle harness</title>
<script type="module">
const bundleJs = new URL('./{bundle_dir}/{bundle_dir}.js', import.meta.url);
try {{
{body}}} catch (err) {{
  console.error('harness error: ' + (err && err.stack || err));
}}
if (globalThis.{binding}) {{ globalThis.{binding}(''); }}
</script>
"#,
        bundle_dir = bundle_dir,
        body = body,
        binding = BROWSER_HARNESS_DONE_BINDING
    )
}

/// Build a browser-bundle runtime harness module that loads the emitted bundle glue.
///
/// The generated module reuses the shared browser-bundle fetch shim, imports the emitted bundle,
/// and re-instantiates it with the canonical Kali runtime imports so future browser runtime
/// flows can observe console output and registered tests from the browser-targeted artifact set.
pub fn browser_bundle_runtime_harness_module_script(
    bundle_dir: &str,
    allow_subpaths: bool,
    args: &[String],
    run_registered_tests: bool,
) -> String {
    let args_json = serde_json::to_string(args).expect("serialize browser bundle runtime args");
    format!(
        r#"{}const runtimeArgs = {args_json};
const runRegisteredTests = {run_registered_tests};
let wasmMemory = null;
let wasmHeap = null;
let wasmAllocGlobal = null;
let wasmAllocCurrent = null;
const collectedTests = [];
let registeredTestFailures = 0;
const coverageHits = [];

function allocGuestString(bytes) {{
  if (wasmMemory === null) {{
    throw new Error('guest string allocation requires instantiated memory and __heap');
  }}
  let base;
  if (wasmAllocGlobal !== null) {{
    // Page-pool allocator (Task 5): call the exported __alloc_global, byte
    // length rounded up to a multiple of 8 to keep host-runtime strings
    // 8-aligned in the arena (mirrors kali_runtime::host::memory's Rust-side
    // rounding).
    const rounded = (bytes.length + 7) & ~7;
    base = Number(wasmAllocGlobal(rounded));
  }} else if (wasmHeap !== null) {{
    // Fallback for a stale cached module built pre-Task-5 (page-pool
    // allocator) with no __alloc_global export: bump __heap directly.
    base = Number(wasmHeap.value);
    wasmHeap.value = base + bytes.length;
  }} else {{
    throw new Error('guest string allocation requires instantiated memory and __heap');
  }}
  new Uint8Array(wasmMemory.buffer, base, bytes.length).set(bytes);
  return 0x8000000000000000n | (BigInt(base) << 32n) | BigInt(bytes.length);
}}

function allocGuestStringCurrent(bytes) {{
  if (wasmMemory === null) {{
    throw new Error('guest string allocation requires instantiated memory and __heap');
  }}
  let base;
  if (wasmAllocCurrent !== null) {{
    // Current-arena twin of allocGuestString (fasta Spec 7 Task 4d): call the
    // exported __alloc (resettable current arena) instead of __alloc_global,
    // 8-aligning the byte length exactly as the global path does.
    const rounded = (bytes.length + 7) & ~7;
    base = Number(wasmAllocCurrent(rounded));
  }} else if (wasmHeap !== null) {{
    // Stale pre-Task-5 module with no __alloc export: bump __heap directly.
    base = Number(wasmHeap.value);
    wasmHeap.value = base + bytes.length;
  }} else {{
    throw new Error('guest string allocation requires instantiated memory and __heap');
  }}
  new Uint8Array(wasmMemory.buffer, base, bytes.length).set(bytes);
  return 0x8000000000000000n | (BigInt(base) << 32n) | BigInt(bytes.length);
}}

function decodeStringHandleBytes(value) {{
  if ((value & 0x8000000000000000n) === 0n || wasmMemory === null) {{
    return new Uint8Array(0);
  }}
  const offset = Number((value >> 32n) & 0x7fffffffn);
  const length = Number(value & 0xffffffffn);
  if (offset < 0 || length < 0 || offset + length > wasmMemory.buffer.byteLength) {{
    return new Uint8Array(0);
  }}
  return new Uint8Array(wasmMemory.buffer.slice(offset, offset + length));
}}

const NULL_VALUE_TAG = -9223372036854775808n;
const UNDEFINED_VALUE_TAG = -9223372036854775807n;
function formatConsoleValue(val) {{
  if (typeof val === 'bigint') {{
    if (val === NULL_VALUE_TAG) {{
      return 'null';
    }}
    if (val === UNDEFINED_VALUE_TAG) {{
      return 'undefined';
    }}
    if ((val & 0x8000000000000000n) !== 0n && wasmMemory !== null) {{
      const offset = Number((val >> 32n) & 0x7fffffffn);
      const length = Number(val & 0xffffffffn);
      if (offset >= 0 && length >= 0 && offset + length <= wasmMemory.buffer.byteLength) {{
        const bytes = new Uint8Array(wasmMemory.buffer, offset, length);
        return new TextDecoder().decode(bytes);
      }}
    }}
    return val.toString();
  }}
  return String(val);
}}

const summaryFile = globalThis.process?.env?.["KALI_BROWSER_HARNESS_SUMMARY_FILE"]
  ?? globalThis.Deno?.env?.get?.("KALI_BROWSER_HARNESS_SUMMARY_FILE")
  ?? null;

async function emitBrowserRuntimeSummary(summary) {{
  const serialized = JSON.stringify(summary);
  if (summaryFile !== null) {{
    if (globalThis.Deno?.writeTextFile) {{
      await globalThis.Deno.writeTextFile(summaryFile, serialized);
      return;
    }}
    if (globalThis.process?.versions?.node !== undefined) {{
      const fs = await import('node:fs/promises');
      await fs.writeFile(summaryFile, serialized);
      return;
    }}
  }}
  console.log(serialized);
}}

// throw-fallout Stage 3 bucket #6 part 2: `crypto.subtle.digest` is ASYNC in
// browsers, but the guest await lane sequences the host call SYNCHRONOUSLY, so
// the host must return the digest bytes immediately. Preload node's SYNCHRONOUS
// `node:crypto` `createHash` (top-level await, node-only); in a real browser
// this stays null and `crypto_subtle_digest` returns -1 (browser digest is out
// of scope for this phase — no fixture exercises it).
const kaliNodeCreateHash =
  globalThis.process?.versions?.node !== undefined
    ? (await import('node:crypto')).createHash
    : null;

const kaliPendingMicrotasks = [];
const kaliPendingTimers = new Map();
const kaliCancelledTimers = new Set();
let kaliNextTimerId = 0;
let kaliNextTimerSeq = 1;
let kaliVirtualNowMs = 0;
const KALI_EVENT_LOOP_BUDGET = 100000;
function kaliScheduleTimer(callbackId, delayMs, repeat, envPtr) {{
  // Mirrors kali_runtime::state::schedule_timer: 1ms minimum clamp,
  // virtual-clock due time, fresh seq per (re)arm.
  const effective = Math.max(1, Number(delayMs) | 0);
  const id = kaliNextTimerId++;
  kaliPendingTimers.set(id, {{
    callbackId,
    envPtr,
    dueAtMs: kaliVirtualNowMs + effective,
    seq: kaliNextTimerSeq++,
    repeatMs: repeat ? effective : null,
  }});
  return id;
}}
function kaliCancelTimer(timerId) {{
  const id = Number(timerId);
  // Mirror kali_runtime::state::cancel_timer (I-1): only record a cancellation
  // for an id that was actually allocated (`0 <= id < kaliNextTimerId`).
  // Clearing a never-allocated / negative / already-dead id is a node-parity
  // no-op and must not poison a LATER timer that is eventually allocated with
  // that id (with the base at 0, the first interval IS id 0).
  if (!kaliPendingTimers.delete(id) && id >= 0 && id < kaliNextTimerId) {{
    kaliCancelledTimers.add(id);
  }}
}}
async function kaliInvokeCallback(instance, callbackId, envPtr) {{
  // Mirrors kali_runtime::host::enforce::invoke_callback: set the exported
  // __current_env to the registration-time env, restore after (both paths).
  const envGlobal = instance.exports.__current_env;
  const saved = envGlobal ? envGlobal.value : null;
  if (envGlobal) {{ envGlobal.value = envPtr; }}
  try {{
    await instance.exports[`__kali_callback_${{callbackId}}`]();
  }} finally {{
    if (envGlobal) {{ envGlobal.value = saved; }}
  }}
}}
async function kaliDrainEventLoop(instance) {{
  // Mirrors kali_runtime::host::enforce::drain_event_loop: all microtasks
  // FIFO before each timer; timers by (dueAtMs, seq); re-arm unless
  // cancelled while firing; bounded by the invocation budget.
  let invoked = 0;
  for (;;) {{
    if (invoked >= KALI_EVENT_LOOP_BUDGET) {{
      throw new Error('event loop did not quiesce: scheduled-callback budget exceeded');
    }}
    const microtask = kaliPendingMicrotasks.shift();
    if (microtask) {{
      invoked += 1;
      await kaliInvokeCallback(instance, microtask.callbackId, microtask.envPtr);
      continue;
    }}
    let next = null;
    for (const [id, timer] of kaliPendingTimers) {{
      if (
        next === null
        || timer.dueAtMs < next.timer.dueAtMs
        || (timer.dueAtMs === next.timer.dueAtMs && timer.seq < next.timer.seq)
      ) {{
        next = {{ id, timer }};
      }}
    }}
    if (next === null) {{ return; }}
    kaliPendingTimers.delete(next.id);
    kaliVirtualNowMs = Math.max(kaliVirtualNowMs, next.timer.dueAtMs);
    invoked += 1;
    await kaliInvokeCallback(instance, next.timer.callbackId, next.timer.envPtr);
    if (next.timer.repeatMs !== null && !kaliCancelledTimers.delete(next.id)) {{
      kaliPendingTimers.set(next.id, {{
        callbackId: next.timer.callbackId,
        envPtr: next.timer.envPtr,
        dueAtMs: kaliVirtualNowMs + next.timer.repeatMs,
        seq: kaliNextTimerSeq++,
        repeatMs: next.timer.repeatMs,
      }});
    }}
  }}
}}

const kaliEventListeners = new Map();
let kaliNextEventTargetId = 1;
// This site has no ptr/len guest-string reader (only the i64-handle-based
// `decodeStringHandleBytes`); add one mirroring `readGuestString`'s idiom
// from the sibling `browser_runtime_harness_module_script` site below.
function kaliReadGuestString(ptr, len) {{
  if (wasmMemory === null) {{
    throw new Error('guest memory is unavailable for event-surface imports');
  }}
  return new TextDecoder().decode(new Uint8Array(wasmMemory.buffer, ptr, len));
}}
function kaliEventKey(target, type) {{
  return `${{Number(target)}} ${{type}}`;
}}
function kaliInvokeCallbackSync(instance, callbackId, envPtr) {{
  // Mirrors kali_runtime::host::enforce::invoke_callback_reentrant: set the
  // exported __current_env to the registration-time env, restore after —
  // SYNCHRONOUSLY (dispatchEvent runs listeners before it returns).
  const envGlobal = instance.exports.__current_env;
  const saved = envGlobal ? envGlobal.value : null;
  if (envGlobal) {{ envGlobal.value = envPtr; }}
  try {{
    instance.exports[`__kali_callback_${{callbackId}}`]();
  }} finally {{
    if (envGlobal) {{ envGlobal.value = saved; }}
  }}
}}
function kaliEventDispatch(instance, target, type) {{
  // Snapshot first: listeners added during dispatch don't fire this round
  // (node parity; mirrors event_dispatch in imports_default.rs).
  const listeners = kaliEventListeners.get(kaliEventKey(target, type));
  const snapshot = listeners ? listeners.slice() : [];
  for (const entry of snapshot) {{
    kaliInvokeCallbackSync(instance, entry.callbackId, entry.envPtr);
  }}
  return 1;
}}

const importObject = {{
  "kali:rt": {{
    test_register(val, _envPtr) {{
      // `_envPtr` (Stage C C3): the guest's `current_env` at registration. The
      // browser harness runs registered tests synchronously and has no closure
      // env chain, so it is accepted for import-signature parity and ignored.
      collectedTests.push(formatConsoleValue(val));
    }},
    queueMicrotask(callbackId, envPtr) {{
      kaliPendingMicrotasks.push({{ callbackId, envPtr }});
    }},
    setTimeout(callbackId, delayMs, envPtr) {{
      return kaliScheduleTimer(callbackId, delayMs, false, envPtr);
    }},
    setInterval(callbackId, delayMs, envPtr) {{
      return kaliScheduleTimer(callbackId, delayMs, true, envPtr);
    }},
    clearTimeout(timerId) {{
      kaliCancelTimer(timerId);
    }},
    clearInterval(timerId) {{
      kaliCancelTimer(timerId);
    }},
    event_target_new() {{
      return BigInt(kaliNextEventTargetId++);
    }},
    event_listener_add(target, namePtr, nameLen, callbackId, envPtr) {{
      const type = kaliReadGuestString(namePtr, nameLen);
      const key = kaliEventKey(target, type);
      const existing = kaliEventListeners.get(key) || [];
      // Dedup by exact (callbackId, envPtr) pair — node's listener-identity rule.
      if (!existing.some((e) => e.callbackId === callbackId && e.envPtr === envPtr)) {{
        existing.push({{ callbackId, envPtr }});
      }}
      kaliEventListeners.set(key, existing);
    }},
    event_dispatch(target, namePtr, nameLen) {{
      return kaliEventDispatch(instance, target, kaliReadGuestString(namePtr, nameLen));
    }},
    coverage_hit(id) {{
      coverageHits.push(Number(id));
    }},
    int_to_string(value) {{
      return allocGuestString(new TextEncoder().encode(String(value)));
    }},
    string_concat(left, right) {{
      const leftBytes = decodeStringHandleBytes(left);
      const rightBytes = decodeStringHandleBytes(right);
      const combined = new Uint8Array(leftBytes.length + rightBytes.length);
      combined.set(leftBytes, 0);
      combined.set(rightBytes, leftBytes.length);
      return allocGuestString(combined);
    }},
    string_concat_arena(left, right) {{
      const leftBytes = decodeStringHandleBytes(left);
      const rightBytes = decodeStringHandleBytes(right);
      const combined = new Uint8Array(leftBytes.length + rightBytes.length);
      combined.set(leftBytes, 0);
      combined.set(rightBytes, leftBytes.length);
      return allocGuestStringCurrent(combined);
    }},
    float_to_fixed(value, digits) {{
      const clampedDigits = Math.min(Math.max(Number(digits), 0), 100);
      return allocGuestString(new TextEncoder().encode(Number(value).toFixed(clampedDigits)));
    }},
    float_to_string(value) {{
      return allocGuestString(new TextEncoder().encode(String(value)));
    }},
    args_len() {{
      return runtimeArgs.length;
    }},
    args_get(index, outPtr, outCap) {{
      const value = runtimeArgs[index];
      if (value === undefined || wasmMemory === null) {{ return -1; }}
      const bytes = new TextEncoder().encode(String(value));
      if (bytes.length > outCap) {{ return -1; }}
      new Uint8Array(wasmMemory.buffer, outPtr, bytes.length).set(bytes);
      return bytes.length;
    }},
    performance_now() {{
      return performance.now();
    }},
    crypto_get_random_values(outPtr, outLen) {{
      if (wasmMemory === null) {{ return 0; }}
      crypto.getRandomValues(new Uint8Array(wasmMemory.buffer, outPtr, outLen));
      return outLen;
    }},
    crypto_random_uuid(outPtr, outCap) {{
      if (wasmMemory === null) {{ return 0; }}
      const bytes = new TextEncoder().encode(crypto.randomUUID());
      if (bytes.length > outCap) {{ return -1; }}
      new Uint8Array(wasmMemory.buffer, outPtr, bytes.length).set(bytes);
      return bytes.length;
    }},
    crypto_subtle_digest(algoPtr, algoLen, inPtr, inLen, outPtr, outCap) {{
      if (wasmMemory === null || kaliNodeCreateHash === null) {{ return -1; }}
      const mem = new Uint8Array(wasmMemory.buffer);
      const algo = new TextDecoder()
        .decode(mem.slice(algoPtr, algoPtr + algoLen))
        .replace(/-/g, '')
        .toLowerCase();
      const digest = kaliNodeCreateHash(algo)
        .update(mem.slice(inPtr, inPtr + inLen))
        .digest();
      if (digest.length > outCap) {{ return -1; }}
      new Uint8Array(wasmMemory.buffer, outPtr, digest.length).set(digest);
      return digest.length;
    }},
    process_pid() {{
      return Number(globalThis.process?.pid ?? 0);
    }},
    cwd(_pathPtr, _pathLen, _outPtr, _outCap) {{
      return 0;
    }},
    math_max(left, right) {{
      return left > right ? left : right;
    }},
    math_min(left, right) {{
      return left < right ? left : right;
    }},
    math_abs(value) {{
      return value < 0n ? -value : value;
    }},
    math_sign(value) {{
      if (value === 0n) {{
        return 0n;
      }}
      return value < 0n ? -1n : 1n;
    }},
    math_round(value) {{
      return value;
    }},
    math_imul(left, right) {{
      return BigInt.asIntN(32, left * right);
    }},
    math_clz32(value) {{
      return BigInt(Math.clz32(Number(BigInt.asUintN(32, value))));
    }},
    math_pow(left, right) {{
      if (right < 0n) {{
        if (left === 1n) {{
          return 1n;
        }}
        if (left === -1n) {{
          return right % 2n === 0n ? 1n : -1n;
        }}
        throw new Error('Math.pow negative exponents are unavailable unless the base is a statically-known ±1 in the current phase; use a non-negative exponent or the later compatibility path');
      }}
      return BigInt.asIntN(64, left ** right);
    }},
    console_log(val) {{
      console.log(formatConsoleValue(val));
    }},
    console_error(val) {{
      console.error(formatConsoleValue(val));
    }},
    console_warn(val) {{
      console.warn(formatConsoleValue(val));
    }},
    console_info(val) {{
      if (typeof console !== 'undefined' && typeof console.info === 'function') {{
        console.info(formatConsoleValue(val));
      }} else if (typeof console !== 'undefined' && typeof console.log === 'function') {{
        console.log(formatConsoleValue(val));
      }}
    }},
    console_debug(val) {{
      if (typeof console !== 'undefined' && typeof console.debug === 'function') {{
        console.debug(formatConsoleValue(val));
      }} else if (typeof console !== 'undefined' && typeof console.log === 'function') {{
        console.log(formatConsoleValue(val));
      }}
    }},
  }},
}};

const bundle = await import(bundleJs.href);
if (typeof bundle.loadWithImports !== 'function') {{
  throw new Error('missing loadWithImports helper');
}}
const instance = await bundle.loadWithImports(importObject);
wasmMemory = instance.exports.memory ?? null;
wasmHeap = instance.exports.__heap ?? null;
wasmAllocGlobal = instance.exports.__alloc_global ?? null;
wasmAllocCurrent = instance.exports.__alloc ?? null;
if (typeof instance.exports._start === 'function') {{
  await instance.exports._start();
  await kaliDrainEventLoop(instance);
}}
if (runRegisteredTests) {{
  for (const callbackId of collectedTests) {{
    const callbackName = `__kali_callback_${{callbackId}}`;
    const callback = instance.exports[callbackName];
    if (typeof callback !== 'function') {{
      throw new Error(`missing browser runtime test callback: ${{callbackName}}`);
    }}
    try {{
      await callback();
    }} catch (error) {{
      registeredTestFailures += 1;
      console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    }}
  }}
}}
let summaryEmissionError = null;
try {{
  await emitBrowserRuntimeSummary({{ args: runtimeArgs, hostContract: "browser-requested", runtimeBackend: "browser-harness", tests: collectedTests, testsFailed: registeredTestFailures, coverageHits }});
}} catch (error) {{
  summaryEmissionError = error;
}}
if (registeredTestFailures > 0) {{
  throw new Error(`browser runtime test failures: ${{registeredTestFailures}}`);
}}
if (summaryEmissionError !== null) {{
  throw summaryEmissionError;
}}
"#,
        browser_bundle_harness_prelude(bundle_dir, allow_subpaths),
    )
}

/// Build a browser-host HTML wrapper for the browser-bundle runtime harness.
pub fn browser_bundle_runtime_harness_page(
    bundle_dir: &str,
    allow_subpaths: bool,
    args: &[String],
    run_registered_tests: bool,
) -> String {
    let module_script = browser_bundle_runtime_harness_module_script(
        bundle_dir,
        allow_subpaths,
        args,
        run_registered_tests,
    );
    format!(
        r#"<!doctype html>
<meta charset="utf-8">
<title>Kali browser bundle runtime harness</title>
<script type="module">
{module_script}
</script>
"#,
        module_script = module_script,
    )
}

/// Build a browser-bundle runtime harness script that loads the emitted bundle glue.
///
/// The generated module reuses the shared browser-bundle fetch shim, imports the emitted bundle,
/// and re-instantiates it with the canonical Kali runtime imports so future browser runtime
/// flows can observe console output and registered tests from the browser-targeted artifact set.
pub fn browser_bundle_runtime_harness_script(
    bundle_dir: &str,
    allow_subpaths: bool,
    args: &[String],
    run_registered_tests: bool,
) -> String {
    browser_bundle_runtime_harness_module_script(
        bundle_dir,
        allow_subpaths,
        args,
        run_registered_tests,
    )
}

pub(crate) fn browser_runtime_harness_module_script(
    wasm_bytes: &[u8],
    args: &[String],
    run_registered_tests: bool,
) -> String {
    let wasm_base64 = BASE64_STANDARD.encode(wasm_bytes);
    let args_json = serde_json::to_string(args).expect("serialize browser runtime args");
    format!(
        r#"const runtimeArgs = {args_json};
const runRegisteredTests = {run_registered_tests};
const runtimeWasm = decodeBase64("{wasm_base64}");
let wasmMemory = null;
let wasmHeap = null;
let wasmAllocGlobal = null;
let wasmAllocCurrent = null;
const collectedTests = [];
let registeredTestFailures = 0;
const coverageHits = [];

function allocGuestString(bytes) {{
  if (wasmMemory === null) {{
    throw new Error('guest string allocation requires instantiated memory and __heap');
  }}
  let base;
  if (wasmAllocGlobal !== null) {{
    // Page-pool allocator (Task 5): call the exported __alloc_global, byte
    // length rounded up to a multiple of 8 to keep host-runtime strings
    // 8-aligned in the arena (mirrors kali_runtime::host::memory's Rust-side
    // rounding).
    const rounded = (bytes.length + 7) & ~7;
    base = Number(wasmAllocGlobal(rounded));
  }} else if (wasmHeap !== null) {{
    // Fallback for a stale cached module built pre-Task-5 (page-pool
    // allocator) with no __alloc_global export: bump __heap directly.
    base = Number(wasmHeap.value);
    wasmHeap.value = base + bytes.length;
  }} else {{
    throw new Error('guest string allocation requires instantiated memory and __heap');
  }}
  new Uint8Array(wasmMemory.buffer, base, bytes.length).set(bytes);
  return 0x8000000000000000n | (BigInt(base) << 32n) | BigInt(bytes.length);
}}

function allocGuestStringCurrent(bytes) {{
  if (wasmMemory === null) {{
    throw new Error('guest string allocation requires instantiated memory and __heap');
  }}
  let base;
  if (wasmAllocCurrent !== null) {{
    // Current-arena twin of allocGuestString (fasta Spec 7 Task 4d): call the
    // exported __alloc (resettable current arena) instead of __alloc_global,
    // 8-aligning the byte length exactly as the global path does.
    const rounded = (bytes.length + 7) & ~7;
    base = Number(wasmAllocCurrent(rounded));
  }} else if (wasmHeap !== null) {{
    // Stale pre-Task-5 module with no __alloc export: bump __heap directly.
    base = Number(wasmHeap.value);
    wasmHeap.value = base + bytes.length;
  }} else {{
    throw new Error('guest string allocation requires instantiated memory and __heap');
  }}
  new Uint8Array(wasmMemory.buffer, base, bytes.length).set(bytes);
  return 0x8000000000000000n | (BigInt(base) << 32n) | BigInt(bytes.length);
}}

function decodeStringHandleBytes(value) {{
  if ((value & 0x8000000000000000n) === 0n || wasmMemory === null) {{
    return new Uint8Array(0);
  }}
  const offset = Number((value >> 32n) & 0x7fffffffn);
  const length = Number(value & 0xffffffffn);
  if (offset < 0 || length < 0 || offset + length > wasmMemory.buffer.byteLength) {{
    return new Uint8Array(0);
  }}
  return new Uint8Array(wasmMemory.buffer.slice(offset, offset + length));
}}

let threadTopology = {{
  totalInstances: 0,
  terminatedInstances: 0,
  liveInstances: [],
}};
let nextThreadInstanceId = 0;

function readGuestString(ptr, len) {{
  if (wasmMemory === null) {{
    throw new Error('guest memory is unavailable before thread spawn handling');
  }}
  const bytes = new Uint8Array(wasmMemory.buffer, ptr, len);
  return new TextDecoder().decode(bytes);
}}

function recordThreadInstance(scriptUrlValue) {{
  const trimmedScriptUrl = scriptUrlValue.trim();
  if (trimmedScriptUrl.length === 0 || trimmedScriptUrl !== scriptUrlValue) {{
    throw new Error('browser runtime thread_spawn scriptUrl must be a canonical absolute URL');
  }}
  let parsedScriptUrl;
  try {{
    parsedScriptUrl = new URL(trimmedScriptUrl);
  }} catch {{
    throw new Error('browser runtime thread_spawn scriptUrl must be a canonical absolute URL');
  }}
  if (parsedScriptUrl.href !== trimmedScriptUrl) {{
    throw new Error('browser runtime thread_spawn scriptUrl must be a canonical absolute URL');
  }}
  const instanceId = nextThreadInstanceId++;
  threadTopology.liveInstances.push({{
    instanceId,
    scriptUrl: parsedScriptUrl.href,
    postedMessages: [],
    postedSharedBuffers: [],
    wasTerminated: false,
  }});
  threadTopology.totalInstances =
    threadTopology.terminatedInstances + threadTopology.liveInstances.length;
  return instanceId;
}}

function decodeBase64(base64) {{
  const binary = typeof atob === 'function'
    ? atob(base64)
    : (typeof Buffer !== 'undefined'
        ? Buffer.from(base64, 'base64').toString('binary')
        : (() => {{ throw new Error('base64 decoding is unavailable in this host'); }})());
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {{
    bytes[index] = binary.charCodeAt(index);
  }}
  return bytes;
}}

const NULL_VALUE_TAG = -9223372036854775808n;
const UNDEFINED_VALUE_TAG = -9223372036854775807n;
function formatConsoleValue(val) {{
  if (typeof val === 'bigint') {{
    if (val === NULL_VALUE_TAG) {{
      return 'null';
    }}
    if (val === UNDEFINED_VALUE_TAG) {{
      return 'undefined';
    }}
    if ((val & 0x8000000000000000n) !== 0n && wasmMemory !== null) {{
      const offset = Number((val >> 32n) & 0x7fffffffn);
      const length = Number(val & 0xffffffffn);
      if (offset >= 0 && length >= 0 && offset + length <= wasmMemory.buffer.byteLength) {{
        const bytes = new Uint8Array(wasmMemory.buffer, offset, length);
        return new TextDecoder().decode(bytes);
      }}
    }}
    return val.toString();
  }}
  return String(val);
}}

const summaryFile = globalThis.process?.env?.["KALI_BROWSER_HARNESS_SUMMARY_FILE"]
  ?? globalThis.Deno?.env?.get?.("KALI_BROWSER_HARNESS_SUMMARY_FILE")
  ?? null;

async function emitBrowserRuntimeSummary(summary) {{
  const serialized = JSON.stringify(summary);
  if (summaryFile !== null) {{
    if (globalThis.Deno?.writeTextFile) {{
      await globalThis.Deno.writeTextFile(summaryFile, serialized);
      return;
    }}
    if (globalThis.process?.versions?.node !== undefined) {{
      const fs = await import('node:fs/promises');
      await fs.writeFile(summaryFile, serialized);
      return;
    }}
  }}
  console.log(serialized);
}}

// throw-fallout Stage 3 bucket #6 part 2: `crypto.subtle.digest` is ASYNC in
// browsers, but the guest await lane sequences the host call SYNCHRONOUSLY, so
// the host must return the digest bytes immediately. Preload node's SYNCHRONOUS
// `node:crypto` `createHash` (top-level await, node-only); in a real browser
// this stays null and `crypto_subtle_digest` returns -1 (browser digest is out
// of scope for this phase — no fixture exercises it).
const kaliNodeCreateHash =
  globalThis.process?.versions?.node !== undefined
    ? (await import('node:crypto')).createHash
    : null;

const kaliPendingMicrotasks = [];
const kaliPendingTimers = new Map();
const kaliCancelledTimers = new Set();
let kaliNextTimerId = 0;
let kaliNextTimerSeq = 1;
let kaliVirtualNowMs = 0;
const KALI_EVENT_LOOP_BUDGET = 100000;
function kaliScheduleTimer(callbackId, delayMs, repeat, envPtr) {{
  // Mirrors kali_runtime::state::schedule_timer: 1ms minimum clamp,
  // virtual-clock due time, fresh seq per (re)arm.
  const effective = Math.max(1, Number(delayMs) | 0);
  const id = kaliNextTimerId++;
  kaliPendingTimers.set(id, {{
    callbackId,
    envPtr,
    dueAtMs: kaliVirtualNowMs + effective,
    seq: kaliNextTimerSeq++,
    repeatMs: repeat ? effective : null,
  }});
  return id;
}}
function kaliCancelTimer(timerId) {{
  const id = Number(timerId);
  // Mirror kali_runtime::state::cancel_timer (I-1): only record a cancellation
  // for an id that was actually allocated (`0 <= id < kaliNextTimerId`).
  // Clearing a never-allocated / negative / already-dead id is a node-parity
  // no-op and must not poison a LATER timer that is eventually allocated with
  // that id (with the base at 0, the first interval IS id 0).
  if (!kaliPendingTimers.delete(id) && id >= 0 && id < kaliNextTimerId) {{
    kaliCancelledTimers.add(id);
  }}
}}
async function kaliInvokeCallback(instance, callbackId, envPtr) {{
  // Mirrors kali_runtime::host::enforce::invoke_callback: set the exported
  // __current_env to the registration-time env, restore after (both paths).
  const envGlobal = instance.exports.__current_env;
  const saved = envGlobal ? envGlobal.value : null;
  if (envGlobal) {{ envGlobal.value = envPtr; }}
  try {{
    await instance.exports[`__kali_callback_${{callbackId}}`]();
  }} finally {{
    if (envGlobal) {{ envGlobal.value = saved; }}
  }}
}}
async function kaliDrainEventLoop(instance) {{
  // Mirrors kali_runtime::host::enforce::drain_event_loop: all microtasks
  // FIFO before each timer; timers by (dueAtMs, seq); re-arm unless
  // cancelled while firing; bounded by the invocation budget.
  let invoked = 0;
  for (;;) {{
    if (invoked >= KALI_EVENT_LOOP_BUDGET) {{
      throw new Error('event loop did not quiesce: scheduled-callback budget exceeded');
    }}
    const microtask = kaliPendingMicrotasks.shift();
    if (microtask) {{
      invoked += 1;
      await kaliInvokeCallback(instance, microtask.callbackId, microtask.envPtr);
      continue;
    }}
    let next = null;
    for (const [id, timer] of kaliPendingTimers) {{
      if (
        next === null
        || timer.dueAtMs < next.timer.dueAtMs
        || (timer.dueAtMs === next.timer.dueAtMs && timer.seq < next.timer.seq)
      ) {{
        next = {{ id, timer }};
      }}
    }}
    if (next === null) {{ return; }}
    kaliPendingTimers.delete(next.id);
    kaliVirtualNowMs = Math.max(kaliVirtualNowMs, next.timer.dueAtMs);
    invoked += 1;
    await kaliInvokeCallback(instance, next.timer.callbackId, next.timer.envPtr);
    if (next.timer.repeatMs !== null && !kaliCancelledTimers.delete(next.id)) {{
      kaliPendingTimers.set(next.id, {{
        callbackId: next.timer.callbackId,
        envPtr: next.timer.envPtr,
        dueAtMs: kaliVirtualNowMs + next.timer.repeatMs,
        seq: kaliNextTimerSeq++,
        repeatMs: next.timer.repeatMs,
      }});
    }}
  }}
}}

const kaliEventListeners = new Map();
let kaliNextEventTargetId = 1;
function kaliEventKey(target, type) {{
  return `${{Number(target)}} ${{type}}`;
}}
function kaliInvokeCallbackSync(instance, callbackId, envPtr) {{
  // Mirrors kali_runtime::host::enforce::invoke_callback_reentrant: set the
  // exported __current_env to the registration-time env, restore after —
  // SYNCHRONOUSLY (dispatchEvent runs listeners before it returns).
  const envGlobal = instance.exports.__current_env;
  const saved = envGlobal ? envGlobal.value : null;
  if (envGlobal) {{ envGlobal.value = envPtr; }}
  try {{
    instance.exports[`__kali_callback_${{callbackId}}`]();
  }} finally {{
    if (envGlobal) {{ envGlobal.value = saved; }}
  }}
}}
function kaliEventDispatch(instance, target, type) {{
  // Snapshot first: listeners added during dispatch don't fire this round
  // (node parity; mirrors event_dispatch in imports_default.rs).
  const listeners = kaliEventListeners.get(kaliEventKey(target, type));
  const snapshot = listeners ? listeners.slice() : [];
  for (const entry of snapshot) {{
    kaliInvokeCallbackSync(instance, entry.callbackId, entry.envPtr);
  }}
  return 1;
}}

const importObject = {{
  "kali:rt": {{
    test_register(val, _envPtr) {{
      // `_envPtr` (Stage C C3): the guest's `current_env` at registration. The
      // browser harness runs registered tests synchronously and has no closure
      // env chain, so it is accepted for import-signature parity and ignored.
      collectedTests.push(formatConsoleValue(val));
    }},
    queueMicrotask(callbackId, envPtr) {{
      kaliPendingMicrotasks.push({{ callbackId, envPtr }});
    }},
    setTimeout(callbackId, delayMs, envPtr) {{
      return kaliScheduleTimer(callbackId, delayMs, false, envPtr);
    }},
    setInterval(callbackId, delayMs, envPtr) {{
      return kaliScheduleTimer(callbackId, delayMs, true, envPtr);
    }},
    clearTimeout(timerId) {{
      kaliCancelTimer(timerId);
    }},
    clearInterval(timerId) {{
      kaliCancelTimer(timerId);
    }},
    event_target_new() {{
      return BigInt(kaliNextEventTargetId++);
    }},
    event_listener_add(target, namePtr, nameLen, callbackId, envPtr) {{
      const type = readGuestString(namePtr, nameLen);
      const key = kaliEventKey(target, type);
      const existing = kaliEventListeners.get(key) || [];
      // Dedup by exact (callbackId, envPtr) pair — node's listener-identity rule.
      if (!existing.some((e) => e.callbackId === callbackId && e.envPtr === envPtr)) {{
        existing.push({{ callbackId, envPtr }});
      }}
      kaliEventListeners.set(key, existing);
    }},
    event_dispatch(target, namePtr, nameLen) {{
      return kaliEventDispatch(instance, target, readGuestString(namePtr, nameLen));
    }},
    coverage_hit(id) {{
      coverageHits.push(Number(id));
    }},
    int_to_string(value) {{
      return allocGuestString(new TextEncoder().encode(String(value)));
    }},
    string_concat(left, right) {{
      const leftBytes = decodeStringHandleBytes(left);
      const rightBytes = decodeStringHandleBytes(right);
      const combined = new Uint8Array(leftBytes.length + rightBytes.length);
      combined.set(leftBytes, 0);
      combined.set(rightBytes, leftBytes.length);
      return allocGuestString(combined);
    }},
    string_concat_arena(left, right) {{
      const leftBytes = decodeStringHandleBytes(left);
      const rightBytes = decodeStringHandleBytes(right);
      const combined = new Uint8Array(leftBytes.length + rightBytes.length);
      combined.set(leftBytes, 0);
      combined.set(rightBytes, leftBytes.length);
      return allocGuestStringCurrent(combined);
    }},
    float_to_fixed(value, digits) {{
      const clampedDigits = Math.min(Math.max(Number(digits), 0), 100);
      return allocGuestString(new TextEncoder().encode(Number(value).toFixed(clampedDigits)));
    }},
    float_to_string(value) {{
      return allocGuestString(new TextEncoder().encode(String(value)));
    }},
    thread_spawn(scriptUrlPtr, scriptUrlLen) {{
      const scriptUrl = readGuestString(scriptUrlPtr, scriptUrlLen);
      return recordThreadInstance(scriptUrl);
    }},
    args_len() {{
      return runtimeArgs.length;
    }},
    args_get(index, outPtr, outCap) {{
      const value = runtimeArgs[index];
      if (value === undefined || wasmMemory === null) {{ return -1; }}
      const bytes = new TextEncoder().encode(String(value));
      if (bytes.length > outCap) {{ return -1; }}
      new Uint8Array(wasmMemory.buffer, outPtr, bytes.length).set(bytes);
      return bytes.length;
    }},
    performance_now() {{
      return performance.now();
    }},
    crypto_get_random_values(outPtr, outLen) {{
      if (wasmMemory === null) {{ return 0; }}
      crypto.getRandomValues(new Uint8Array(wasmMemory.buffer, outPtr, outLen));
      return outLen;
    }},
    crypto_random_uuid(outPtr, outCap) {{
      if (wasmMemory === null) {{ return 0; }}
      const bytes = new TextEncoder().encode(crypto.randomUUID());
      if (bytes.length > outCap) {{ return -1; }}
      new Uint8Array(wasmMemory.buffer, outPtr, bytes.length).set(bytes);
      return bytes.length;
    }},
    crypto_subtle_digest(algoPtr, algoLen, inPtr, inLen, outPtr, outCap) {{
      if (wasmMemory === null || kaliNodeCreateHash === null) {{ return -1; }}
      const mem = new Uint8Array(wasmMemory.buffer);
      const algo = new TextDecoder()
        .decode(mem.slice(algoPtr, algoPtr + algoLen))
        .replace(/-/g, '')
        .toLowerCase();
      const digest = kaliNodeCreateHash(algo)
        .update(mem.slice(inPtr, inPtr + inLen))
        .digest();
      if (digest.length > outCap) {{ return -1; }}
      new Uint8Array(wasmMemory.buffer, outPtr, digest.length).set(digest);
      return digest.length;
    }},
    process_pid() {{
      return Number(globalThis.process?.pid ?? 0);
    }},
    cwd(_pathPtr, _pathLen, _outPtr, _outCap) {{
      return 0;
    }},
    math_max(left, right) {{
      return left > right ? left : right;
    }},
    math_min(left, right) {{
      return left < right ? left : right;
    }},
    math_abs(value) {{
      return value < 0n ? -value : value;
    }},
    math_sign(value) {{
      if (value === 0n) {{
        return 0n;
      }}
      return value < 0n ? -1n : 1n;
    }},
    math_round(value) {{
      return value;
    }},
    math_imul(left, right) {{
      return BigInt.asIntN(32, left * right);
    }},
    math_clz32(value) {{
      return BigInt(Math.clz32(Number(BigInt.asUintN(32, value))));
    }},
    math_pow(left, right) {{
      if (right < 0n) {{
        if (left === 1n) {{
          return 1n;
        }}
        if (left === -1n) {{
          return right % 2n === 0n ? 1n : -1n;
        }}
        throw new Error('Math.pow negative exponents are unavailable unless the base is a statically-known ±1 in the current phase; use a non-negative exponent or the later compatibility path');
      }}
      return BigInt.asIntN(64, left ** right);
    }},
    console_log(val) {{
      console.log(formatConsoleValue(val));
    }},
    console_error(val) {{
      console.error(formatConsoleValue(val));
    }},
    console_warn(val) {{
      console.warn(formatConsoleValue(val));
    }},
    console_info(val) {{
      if (typeof console !== 'undefined' && typeof console.info === 'function') {{
        console.info(formatConsoleValue(val));
      }} else if (typeof console !== 'undefined' && typeof console.log === 'function') {{
        console.log(formatConsoleValue(val));
      }}
    }},
    console_debug(val) {{
      if (typeof console !== 'undefined' && typeof console.debug === 'function') {{
        console.debug(formatConsoleValue(val));
      }} else if (typeof console !== 'undefined' && typeof console.log === 'function') {{
        console.log(formatConsoleValue(val));
      }}
    }},
  }},
}};

const {{ instance }} = await WebAssembly.instantiate(runtimeWasm, importObject);
wasmMemory = instance.exports.memory ?? null;
wasmHeap = instance.exports.__heap ?? null;
wasmAllocGlobal = instance.exports.__alloc_global ?? null;
wasmAllocCurrent = instance.exports.__alloc ?? null;
if (typeof instance.exports._start === 'function') {{
  await instance.exports._start();
  await kaliDrainEventLoop(instance);
}}
if (runRegisteredTests) {{
  for (const callbackId of collectedTests) {{
    const callbackName = `__kali_callback_${{callbackId}}`;
    const callback = instance.exports[callbackName];
    if (typeof callback !== 'function') {{
      throw new Error(`missing browser runtime test callback: ${{callbackName}}`);
    }}
    try {{
      await callback();
    }} catch (error) {{
      registeredTestFailures += 1;
      console.error(error instanceof Error ? error.stack ?? error.message : String(error));
    }}
  }}
}}
let summaryEmissionError = null;
try {{
  await emitBrowserRuntimeSummary({{ args: runtimeArgs, hostContract: "browser-requested", runtimeBackend: "browser-harness", tests: collectedTests, testsFailed: registeredTestFailures, threadTopology, coverageHits }});
}} catch (error) {{
  summaryEmissionError = error;
}}
if (registeredTestFailures > 0) {{
  throw new Error(`browser runtime test failures: ${{registeredTestFailures}}`);
}}
if (summaryEmissionError !== null) {{
  throw summaryEmissionError;
}}
"#,
        args_json = args_json,
        run_registered_tests = run_registered_tests,
        wasm_base64 = wasm_base64,
    )
}

/// Build a self-contained browser-runtime harness script from embedded WASM bytes.
///
/// The generated module is intentionally generic: it instantiates the supplied WASM bytes, wires
/// the canonical Kali runtime imports for console/argument handling, and optionally emits a simple
/// test summary payload for future browser-runtime test plumbing.
pub fn browser_runtime_harness_script(
    wasm_bytes: &[u8],
    args: &[String],
    run_registered_tests: bool,
) -> String {
    browser_runtime_harness_module_script(wasm_bytes, args, run_registered_tests)
}

/// Build a browser-host HTML wrapper for the self-contained browser-runtime harness.
///
/// This wrapper is intended for real browser hosts that can open an HTML entrypoint while still
/// executing the same browser-friendly module body used by the in-process harness.
pub fn browser_runtime_harness_page(
    wasm_bytes: &[u8],
    args: &[String],
    run_registered_tests: bool,
) -> String {
    let module_script =
        browser_runtime_harness_module_script(wasm_bytes, args, run_registered_tests);
    format!(
        r#"<!doctype html>
<meta charset="utf-8">
<title>Kali browser runtime harness</title>
<script type="module">
{module_script}
</script>
"#,
        module_script = module_script,
    )
}

#[cfg(test)]
#[path = "harness_tests.rs"]
mod harness_tests;
