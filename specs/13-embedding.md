# 13 — Embedding & C API

## Rust Library API (`kali_embed`)

Kali is designed to be used as a Rust library, similar to Deno's embedding API.

### Core API
```rust
use kali_embed::{Runtime, Config, SandboxPolicy, Value};

// Create a runtime
let config = Config::builder()
    .api(ApiSurface::Deno)
    .max_memory_mb(256)
    .max_cpu_time_ms(10_000)
    .sandbox(SandboxPolicy::from_file("kali.policy.json")?)
    .build();

let mut runtime = Runtime::new(config)?;

// Compile and run a source string
let result = runtime.run_string("inline.ts", "const x: number = 1 + 2; x")?;
assert_eq!(result.as_number(), Some(3.0));

// Compile a module graph into one linked artifact
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

### Custom Host Functions
```rust
runtime.register_host_function("myApi", "getData", |args: &[Value]| -> Result<Value> {
    let key = args[0].as_string()?;
    Ok(Value::String(my_database.get(key)?))
})?;
```

### Event Handling
```rust
// Run with async event loop
let result = runtime.run_async("main.ts").await?;

// Run until idle (for servers)
runtime.run_event_loop().await?;

// Step-by-step execution (for debugging)
let mut runner = runtime.step_runner("main.ts")?;
while let Some(step) = runner.next_step()? {
    println!("Executing: {:?}", step);
}
```

## C API (`kali_capi`)

Exposes Kali functionality via a stable C ABI for embedding from any language.

### Header (`kali.h`)
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
void kali_config_set_max_memory(KaliConfig* config, uint64_t bytes);
void kali_config_set_max_cpu_time(KaliConfig* config, uint64_t ms);
void kali_config_set_sandbox(KaliConfig* config, const char* policy_path);
void kali_config_free(KaliConfig* config);

// Runtime
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
const char* kali_error_message(const KaliError* error);
int kali_error_code(const KaliError* error);
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
- Exposing a C ABI does **not** imply linking any C/C++ implementation into Kali itself; the runtime and compiler remain Rust-only internally

### Error Handling Convention
- Functions that can fail return `NULL` on error
- Call `kali_last_error()` to get error details
- Error includes code, message, and JSON representation

### Building
`kali build --capi` is an **artifact-generation mode for embedded programs**, not a request to turn user TypeScript directly into a native shared library.

```bash
kali build --capi lib.ts                   # Produces lib.wasm + generated kali.h/metadata for use with kali_capi
```

The host-side C ABI itself is provided by the `kali_capi` crate:
```bash
cargo build --release -p kali_capi         # Build the C API shared/static library
```

Typical embedding flow:
1. Build or ship `kali_capi` as the native C ABI layer.
2. Compile Kali/TypeScript code to `foo.wasm` with `kali build --capi foo.ts`.
3. Load that artifact through the `kali_*` API from C or another FFI consumer.

The shared library exports only `kali_*` symbols. All Rust internals are hidden.

## Language Bindings (Future)

The C API enables bindings for:
- Python (`ctypes` or `cffi`)
- Go (`cgo`)
- Ruby (`ffi`)
- Java (`JNI`)
- C# (`P/Invoke`)
- Zig (direct C interop)

Initial release focuses on C API stability; language-specific bindings are community-driven.
