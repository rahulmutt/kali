# 13 — Embedding & C API

Public embedding is intentionally phased:
- **Phase 1**: reusable internal crates exist so the CLI is built library-first, but the public embedding surface may still change freely.
- **Phase 2 target**: the Rust embedding API, C ABI, WIT interface emission, and `kali build --capi` / `kali build --component` artifact flows become the first stable public embedding contract.

## Rust Library API (`kali_embed`)

Kali is designed to be used as a Rust library, similar to Deno's embedding API, once the public embedding surface reaches Phase 2.

### Core API
The API below describes the intended stable shape for the Phase 2 public surface; earlier internal versions may differ.

```rust
use kali_embed::{Runtime, Config, SandboxPolicy, Value};

// Create a runtime
let config = Config::builder()
    .api_surface(ApiSurface::Deno)
    .build_mode(BuildMode::Fast)
    .runtime_profiles([])
    .compat_features([])
    .max_memory_mb(256)
    .max_cpu_time_ms(10_000)
    .sandbox(SandboxPolicy::from_file("kali.policy.json")?)
    .build();

let mut runtime = Runtime::new(config)?;

// Compile and run a source string
let result = runtime.run_string("inline.ts", "const x: number = 1 + 2; x")?;
assert_eq!(result.as_number(), Some(3.0));

// Compile a module graph into one linked WASM payload artifact
let module = runtime.compile_file("main.ts")?;
let result = runtime.run_module(&module)?;

// Call exported functions
let module = runtime.compile_file("lib.ts")?;
let instance = runtime.instantiate(&module)?;
let result = instance.call("add", &[Value::Number(1.0), Value::Number(2.0)])?;

// Get effect analysis (Phase 2 target)
// Before then, this API may be absent or return the canonical feature-maturity error.
let effects = runtime.analyze_effects_file("program.ts")?;
println!("{}", serde_json::to_string(&effects)?);
```

Canonical embedding-alignment rule:
- the public embedding config should expose the same semantic knobs as CLI/config where they matter to compilation and execution: **API surface**, **build mode**, **runtime profiles**, and **compat features**
- prefer set-like builder methods such as `runtime_profiles([...])` and `compat_features([...])` over boolean enable/disable pairs so the embedding vocabulary stays aligned with `kali.json`
- the canonical runtime-profile name remains `wasm-threads`; if the Rust API exposes `RuntimeProfile::WasmThreads`, that is just the typed embedding spelling of the same underlying profile selected by CLI `--wasm-threads`
- WIT is the canonical host-facing interface description for public library/component outputs; Rust-typed helpers, generated C headers, and later component wrappers are projections of that same exported interface contract rather than unrelated parallel ABI descriptions
- embedding APIs may use idiomatic Rust enums/builders instead of the JSON field names, but they should not invent a second incompatible vocabulary for the same concepts
- build-oriented embedding calls obey the same API-surface gates as the CLI/spec matrix: for example, `ApiSurface::Node` remains Phase 3-gated for compile/build flows, while browser-targeted build output is still the `--bundle`-style path rather than a generic library/export mode
- if a CLI/config/runtime feature is phase-gated (for example `ApiSurface::Node`, `RuntimeProfile::WasmThreads`, or `CompatFeature::Eval`), the embedding API should surface the same canonical `E5006`-style availability failure rather than silently ignoring the request

### Custom Host Functions
```rust
runtime.register_host_function("myApi", "getData", |args: &[Value]| -> Result<Value> {
    let key = args[0].as_string()?;
    Ok(Value::String(my_database.get(key)?))
})?;
```

### Optional Runtime-Control Conveniences
These APIs are intentionally **optional/later convenience layers** over the minimal Phase 2 embedding contract.

The stable Phase 2 promise is the compile / instantiate / run / call surface above plus aligned config and error contracts. More opinionated host-loop helpers such as async driving, server-style idle loops, or step-by-step execution may arrive later or remain host-specific wrappers rather than part of the minimum embedding guarantee.

Illustrative examples:
```rust
// Optional async convenience once the host-loop contract is stabilized
let result = runtime.run_async("main.ts").await?;

// Optional host-driven event-loop helper for long-lived embeddings
runtime.run_event_loop().await?;

// Optional step-by-step execution/debug wrapper
let mut runner = runtime.step_runner("main.ts")?;
while let Some(step) = runner.next_step()? {
    println!("Executing: {:?}", step);
}
```

## C API (`kali_capi`)

Exposes Kali functionality via a stable C ABI for embedding from any language.

### Host ABI Header (`kali.h`)
The C declarations below describe the intended stable ABI surface for Phase 2+.
They come from the host-side `kali_capi` library itself.

```c
#ifndef KALI_H
#define KALI_H

#include <stdint.h>
#include <stdbool.h>

typedef struct KaliRuntime KaliRuntime;
typedef struct KaliConfig KaliConfig;
typedef struct KaliModule KaliModule;
typedef struct KaliValue KaliValue;
typedef struct KaliError KaliError;

// Configuration
KaliConfig* kali_config_new(void);
void kali_config_set_api(KaliConfig* config, int api_surface);
void kali_config_set_build_mode(KaliConfig* config, int build_mode);
void kali_config_clear_runtime_profiles(KaliConfig* config);
void kali_config_add_runtime_profile(KaliConfig* config, int profile);
void kali_config_clear_compat_features(KaliConfig* config);
void kali_config_add_compat_feature(KaliConfig* config, int feature);
void kali_config_set_max_memory(KaliConfig* config, uint64_t bytes);
void kali_config_set_max_cpu_time(KaliConfig* config, uint64_t ms);
void kali_config_set_sandbox(KaliConfig* config, const char* policy_path);
void kali_config_free(KaliConfig* config);

// Runtime
uint32_t kali_runtime_abi_version(void);
KaliRuntime* kali_runtime_new(KaliConfig* config);
void kali_runtime_free(KaliRuntime* runtime);

// Compilation
KaliModule* kali_compile_string(KaliRuntime* runtime, const char* filename, const char* source);
KaliModule* kali_compile_file(KaliRuntime* runtime, const char* path);
void kali_module_free(KaliModule* module);

// Execution
KaliValue* kali_run(KaliRuntime* runtime, KaliModule* module);
KaliValue* kali_run_string(KaliRuntime* runtime, const char* filename, const char* source);
KaliValue* kali_call(KaliRuntime* runtime, KaliModule* module,
                     const char* fn_name, KaliValue** args, uint32_t argc);

// Values
int kali_value_type(const KaliValue* value);
double kali_value_as_number(const KaliValue* value);
const char* kali_value_as_string(const KaliValue* value);
bool kali_value_as_bool(const KaliValue* value);
KaliValue* kali_value_new_number(double n);
KaliValue* kali_value_new_string(const char* s);
KaliValue* kali_value_new_bool(bool b);
KaliValue* kali_value_new_null(void);
void kali_value_free(KaliValue* value);

// Error handling
const KaliError* kali_last_error(const KaliRuntime* runtime);
const KaliError* kali_global_last_error(void); // for failures before a runtime exists
const char* kali_error_message(const KaliError* error);
const char* kali_error_code(const KaliError* error); // stable string code such as "E5006"
const char* kali_error_json(const KaliError* error);

// Effects analysis (Phase 2 target; before then these return NULL and expose the canonical
// feature-maturity error via kali_last_error())
const char* kali_analyze_effects_file(KaliRuntime* runtime, const char* path);
const char* kali_analyze_effects_string(KaliRuntime* runtime, const char* filename, const char* source);
void kali_free_string(const char* s);

// Host function registration
typedef KaliValue* (*KaliHostFn)(KaliValue** args, uint32_t argc, void* userdata);
void kali_register_host_function(KaliRuntime* runtime, const char* module,
                                  const char* name, KaliHostFn fn, void* userdata);

#endif // KALI_H
```

### Memory Management
- All `kali_*_new` / `kali_*_free` pairs — caller manages lifetime
- Strings returned by Kali must be freed with `kali_free_string`
- Thread safety: one `KaliRuntime` per thread in the initial implementation
- The C config surface follows the same set-like semantics as `kali.json` and the Rust builder API: runtime profiles and compat features are unordered unique sets, not boolean toggle pairs
- C config/runtime setters for build mode, runtime profiles, and compat features follow the same phase-gating rules as the CLI/config surface; unsupported requests fail with the canonical availability error instead of degrading silently
- Exposing a C ABI does **not** imply linking any C/C++ implementation into Kali itself; the runtime and compiler remain Rust-only internally

### Error Handling Convention
- Functions that can fail return `NULL` on error
- Call `kali_last_error()` for runtime-bound failures, or `kali_global_last_error()` for failures that happen before a `KaliRuntime` exists (for example `kali_runtime_new` returning `NULL`)
- Error includes the stable string diagnostic code, message, and JSON representation so embedders see the same canonical machine contract as the CLI

### Building
Artifact selection follows the canonical build matrix in [SPEC.md](../SPEC.md): plain `--lib` is the Phase-1 **base exported-library** mode, and `kali build --capi` / `kali build --component` are **Phase 2** packaging layers over that same exported-library contract rather than unrelated semantics.

`kali build --capi` and `kali build --component` are **Phase 2 targets** and are artifact-generation modes for embedded programs, not requests to turn user TypeScript directly into a native shared library.

```bash
kali build --capi lib.ts                   # Produces lib.wasm + lib.wit + generated lib.exports.h + metadata for use with kali_capi
kali build --component lib.ts              # Produces lib.wasm + lib.wit + lib.component.wasm for Component Model consumers
```

Artifact-role clarification:
- `kali build --capi` uses the core `wasm-module` as the exported-library artifact (`role: primary-library`) plus `wit` (`role: interface-wit`), generated header (`role: embedding-header`), and metadata (`role: embedding-metadata`)
- `kali build --component` keeps the same linked core library payload (`role: primary-library`) and WIT sidecar (`role: interface-wit`), then adds the outer Component Model wrapper as `kind: wasm-component`, `role: primary-component`
- the exported host-facing surface for all three library-oriented modes is derived from the module's explicit exports; WIT, generated C headers, and component packaging are projections of that same explicit export surface rather than separate reflection-based APIs
- that outer component wrapper is packaging over the already-linked core payload, not a second independently linked guest-program graph; this keeps embedding/component outputs aligned with the single-linked-core-payload rule from [SPEC.md](../SPEC.md)

Important distinction:
- `kali_capi` ships the stable host ABI header: `kali.h`
- `kali build --capi foo.ts` emits a **program-specific** exports header such as `foo.exports.h` plus metadata
- Phase 1 plain `kali build --lib foo.ts` emits the base library `wasm-module` only; this is intentionally useful before the public embedding contract is frozen, but it should be treated as the pre-stable exported-library artifact rather than as the full public embedding surface
- once the public interface contract stabilizes in Phase 2+, library/component-oriented outputs emit a WIT sidecar by default so C bindings and Component Model wrappers derive from the same canonical exported interface description
- library builds omit any synthetic executable entry invocation, but ordinary top-level module initialization still occurs when the host instantiates the artifact; exported functions are the host-callable surface layered on top of that normal module-instantiation behavior
- In CLI JSON/artifact manifests, these outputs use the canonical artifact kinds `wasm-module`, `wit`, `wasm-component`, `c-header`, and `cabi-metadata`

This avoids overloading the name `kali.h` for two different purposes and keeps C ABI generation aligned with the Component Model path.

The host-side C ABI itself is provided by the `kali_capi` crate:
```bash
cargo build --release -p kali_capi         # Build the C API shared/static library
```

## ABI Versioning and Compatibility

To keep embedding stable and machine-checkable, the C ABI needs one explicit compatibility rule instead of relying on prose:

- `kali_capi` publishes a monotonically increasing **host ABI version** integer
- the stable host header exposes it via constants/macros such as:
  - `KALI_CAPI_ABI_VERSION`
  - `KALI_CAPI_ABI_MIN_COMPAT_VERSION` *(optional if compatible windowing is needed)*
- the runtime exports `uint32_t kali_runtime_abi_version(void);`
- `kali build --capi foo.ts` embeds the expected host ABI version in emitted metadata so loaders can reject incompatible host/program combinations before instantiation
- incompatible ABI versions are a hard load-time error; they must not silently proceed on a best-effort basis

Compatibility policy:
- additive C-ABI changes that preserve layout/call compatibility may keep the same major host ABI version
- signature changes, ownership-convention changes, struct layout changes, or semantic changes that break existing embedders require a new host ABI version
- the generated program-specific header (`foo.exports.h`) may evolve independently from the stable host header `kali.h`, but its emitted metadata must still declare which host ABI version it expects

Typical embedding flow:
1. Build or ship `kali_capi` as the native C ABI layer (including the stable `kali.h` host header).
2. Compile Kali/TypeScript code to `foo.wasm` with `kali build --capi foo.ts` to obtain `foo.wasm` plus `foo.wit`, `foo.exports.h`, and metadata.
3. Verify ABI compatibility between the emitted metadata and the available `kali_capi` host library.
4. Load that artifact through the `kali_*` API from C or another FFI consumer.

Typical component flow:
1. Compile Kali/TypeScript code with `kali build --component foo.ts`.
2. Use the emitted `foo.wit` as the canonical interface description for tooling/review.
3. Load `foo.component.wasm` in a Component Model host that matches the documented runtime/profile constraints.

The shared library exports only `kali_*` symbols. All Rust internals are hidden.

## Language Bindings (Future)

The C API enables bindings for:
- Python (`ctypes` or `cffi`)
- Go (`cgo`)
- Ruby (`ffi`)
- Java (`JNI`)
- C# (`P/Invoke`)
- Zig (direct C interop)

Once the public embedding surface lands, the stable contract focuses first on the C ABI; language-specific bindings remain community-driven or higher-level wrappers over that ABI.
