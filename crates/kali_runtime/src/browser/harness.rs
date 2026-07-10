//! Browser harness-script generators: prelude, module script, page, and bundle variants.
use crate::*;

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

const importObject = {{
  "kali:rt": {{
    test_register(val) {{
      collectedTests.push(formatConsoleValue(val));
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
  await emitBrowserRuntimeSummary({{ args: runtimeArgs, hostContract: "browser-requested", runtimeBackend: "browser-harness", tests: collectedTests, testsFailed: registeredTestFailures }});
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

const importObject = {{
  "kali:rt": {{
    test_register(val) {{
      collectedTests.push(formatConsoleValue(val));
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
  await emitBrowserRuntimeSummary({{ args: runtimeArgs, hostContract: "browser-requested", runtimeBackend: "browser-harness", tests: collectedTests, testsFailed: registeredTestFailures, threadTopology }});
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
