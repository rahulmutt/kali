# 13 — Embedding, WIT & C ABI

Embedding is intentionally phased and follows the shared **embedding-stability split** from [SPEC.md](../SPEC.md):
- **Phase 1 MVP**: Kali should have a reusable library-first internal decomposition, and `kali build --lib` is the early **base library artifact** shape **when Kali can determine a statically known export surface**. That artifact is intentionally useful for exported-module workflows immediately, but only for **exact-version consumers** as defined in [SPEC.md](../SPEC.md); it does **not** yet count as the stable **public embedding surface**. In particular, Phase 1 does not yet promise a stable Rust API, a stable public **WIT-first** library contract, a stable C ABI, a cross-version host-loading guarantee, or default WIT sidecars for plain `--lib`.
- **Phase 2 target**: the **public embedding surface** arrives — the Rust embedding API, the stable public **WIT-first** library contract for `kali build --lib`, the C ABI, and `kali build --capi` / `kali build --component` artifact flows.

Repository-state note: names such as `kali_embed` and `kali_capi`, along with API sketches and cargo commands in this chapter, describe the target public contract and intended implementation structure. The repository may already contain some of those crates/artifact flows, but actual public availability still comes from [19 — Feature Maturity](19-feature-maturity.md) rather than from the presence of a crate name or an API sketch in this chapter.

Reading rule:
- this chapter may define the intended stable shape of later embedding artifact flows (`--capi`, `--component`, stable public **WIT-first** `--lib` + default WIT) before they are phase-enabled
- the same chapter may also describe the earliest support contract for flows that are already live in the current repository; do not infer present-tense availability from a section title such as `Phase 2 target` without checking [19 — Feature Maturity](19-feature-maturity.md)
- treat those sections as artifact-contract definitions first, not as implied Phase-1 availability promises
- [19 — Feature Maturity](19-feature-maturity.md) remains the availability owner for when those flows are actually supported

Practical simplification:
- there is one exported-library contract, not three unrelated embedding semantics
- Phase 1 plain `--lib` establishes that contract as the **base library artifact**
- in Phase 1, that base artifact is intentionally useful for **exact-version consumers**, but it is **not** yet a stable cross-version public ABI contract
- Phase 2 promotes the same `--lib` path into the canonical stable **WIT-first** public library contract and adds WIT by default
- `--capi` and `--component` then project/package that same **statically known export surface** for specific host interop workflows rather than redefining what the library exports mean
- Component Model packaging remains an explicit `--component` choice rather than an implicit default for every public library build; the default stable public library path is plain `--lib` + WIT

Consumer lanes at a glance:

| Consumer type | Phase-1 `--lib` base artifact | Phase-2 public embedding surface |
|---|---|---|
| Repository-local tooling / pinned host integration | Yes, as an **exact-version consumer** | Yes |
| Independently versioned public host loading | No | Yes, once the public contract is frozen |
| Stable Rust embedding API consumers | No stable public API yet | Yes |
| Stable C ABI / Component Model consumers | No | Yes via `--capi` / `--component` |

Reading shortcut:
- if the consumer pins the exact producing Kali toolchain/runtime and upgrades with it, the Phase-1 **base library artifact** is in scope
- if the consumer expects a stable cross-version contract or a language-neutral public ABI, wait for the Phase-2 **public embedding surface**

Artifact-progression shorthand:
- `--lib` is the earliest export-oriented build path: the **minimum Phase-1 contract** is the core `wasm-module` only, and the later stable public contract keeps the same selector while adding the default WIT sidecar.
- in the **current repository state**, that later stable public `--lib` + WIT contract is already live; keep using the maturity matrix for exact availability wording instead of inferring it from the historical Phase-1 minimum alone.
- `--capi` is not a second export model; it is the Phase-2 C-facing projection of that same **statically known export surface** (`wasm-module` + `wit` + generated exports header + C-ABI metadata).
- `--component` is likewise packaging over that same **statically known export surface** rather than a different library contract (`wasm-module` + `wit` + outer component wrapper).
- all three library-oriented selectors share one export-surface gate: if frontend lowering cannot determine one fixed host-callable export set, `--lib` / `--capi` / `--component` all fail on the same canonical `E5511` path instead of diverging into selector-specific fallback rules.
- Because plain public `--lib` is the canonical stable default once Phase 2 lands, callers should choose `--component` only when they explicitly want Component Model packaging semantics.

## Current repository-state snapshot

This chapter is mostly phase-labeled contract prose, so keep one current-state shortcut here to avoid forcing readers to bounce between the historical Phase-1 minimum and the current repo state:

| Surface | Earliest contract | Current repository state | Canonical availability owner |
|---|---|---|---|
| `kali build --lib` | Phase 1 **base library artifact**, promoted in Phase 2 to the stable public **WIT-first** library contract | live with the stable WIT sidecar/output contract described by the maturity matrix | [19 — Feature Maturity](19-feature-maturity.md) |
| `kali build --capi` | Phase 2 target | live in the current repo | [19 — Feature Maturity](19-feature-maturity.md) |
| `kali build --component` | Phase 2 target | live in the current repo | [19 — Feature Maturity](19-feature-maturity.md) |
| Stable public Rust embedding API / host metadata flows | Phase 2 target | live for the documented public embedding surface; later ABI/platform widening still follows its own rows | [19 — Feature Maturity](19-feature-maturity.md), [18 — Schemas](18-schemas.md) |

Reading rule:
- use the phase-labeled sections below to understand the **minimum contract and artifact shape** each selector means
- use the maturity matrix to answer whether that surface is currently supported for the exact command/context you care about
- do not misread Phase-1 wording such as "no default WIT sidecar" as a claim about the current repository state after the Phase-2 rows have opened

Bootstrap-alignment shortcut:
- the bootstrap brief asks for embeddability, a C API, and WIT / Component Model integration
- this chapter preserves that ask by phasing it into one exported-library contract: Phase 1 establishes the base `--lib` artifact, and the current repo has already opened the later stable public `--lib` + WIT, `--capi`, and `--component` flows documented by the maturity matrix

## Phase 1 — Base library artifact

Phase 1 needs one export-oriented build path early so Kali is genuinely embeddable and library-first internally, but the spec should keep that early promise narrow.

What plain `kali build --lib` means in Phase 1, once Kali can determine the required **statically known export surface**:

| Phase-1 property | Guaranteed? | Meaning |
|---|---|---|
| Export-oriented WASM output | Yes, when Kali can determine that export surface | Emit one linked `wasm-module` (`role: primary-library`) whose host-facing surface comes from the required **statically known export surface** |
| Useful for **exact-version consumers** | Yes, on that same export-surface-known path | Hosts/integrations that pin the exact producing Kali toolchain may consume it immediately |
| Stable cross-version host-loading contract | No | Phase 1 support stops at **exact-version consumers**; cross-version/public loading belongs to the later **public embedding surface** |
| Stable public Rust API | No | That is part of the later **public embedding surface** |
| Stable public WIT contract / default WIT sidecar | No | Plain `--lib` adds default `wit` output only once the Phase-2 public library contract is frozen |
| Stable C ABI / `--capi` flow | No | That is Phase 2 work |
| Stable Component Model packaging / `--component` flow | No | That is Phase 2 work |

Canonical library-artifact normalization table:

| Selector | Earliest phase | Compile intent | Artifact contract summary |
|---|---|---|---|
| `--lib` | Phase 1 MVP | library | Phase 1: `wasm-module` (`role: primary-library`) as the **base library artifact** when Kali can determine the required **statically known export surface**. Until Phase 2 freezes the public interface contract, this output is export-oriented but not yet a stable public ABI/WIT promise. From the Phase 2 target onward, the same selector becomes the stable public **WIT-first** library contract and adds `wit` (`role: interface-wit`) by default. |
| `--capi` | Phase 2 target | library | The same **statically known export surface** as a `wasm-module` (`role: primary-library`), plus `wit`, a generated **program-specific exports header** (`kind: c-header`, distinct from the stable host ABI header `kali.h`), and `cabi-metadata`. |
| `--component` | Phase 2 target | library | The same **statically known export surface** as a `wasm-module` (`role: primary-library`), plus `wit` and a `wasm-component` wrapper. |

This table is a summary only. The normative artifact kinds/roles still live in [SPEC.md](../SPEC.md), [18 — Schemas](./18-schemas.md), and [19 — Feature Maturity](./19-feature-maturity.md).

Header-split simplification:
- `kali build --capi` emits the generated **program-specific exports header** for one compiled library
- it does **not** emit the stable host ABI header `kali.h`; that header ships with the host-side `kali_capi` library

Phase-1 practical-use rule for `--lib`:
- the Phase-1 **base library artifact** exists only on the export-surface-known path and may then be consumed by **exact-version consumers** or explicitly unstable experiments
- it must **not** be described as part of the stable public embedding surface until the Phase-2 public library/WIT contract is frozen
- docs and tooling should therefore avoid implying that plain Phase-1 `--lib` output alone guarantees long-term host-call compatibility across Kali releases

Preferred short support wording:
- **`kali build --lib <file>` is buildable for exact-version consumers in the shared Deno-oriented build context (schema v1), provided Kali can determine a statically known export surface.**
- avoid shorter summaries such as **"embedding ships in Phase 1"** or **"plain --lib is a stable ABI"**, because they blur the Phase-1 base-artifact path into the later public embedding surface

Practical non-promises for plain Phase-1 `--lib`:
- no stable public Rust embedding API
- no default WIT sidecar
- no generated C-ABI embedding header/metadata flow
- no Component Model packaging flow
- no promise that independently versioned hosts can rely on cross-release loading or ABI compatibility without pinning the exact Kali toolchain

Compact Phase-1 support-claim table for library-oriented builds:

| Question | Phase-1 answer |
|---|---|
| What is actually shipped in Phase 1? | One export-oriented path is **buildable for exact-version consumers**: `kali build --lib <file>` emits the **base library artifact** when Kali can determine a **statically known export surface**. |
| Can that same build take `--sandbox`? | Yes: `kali build --lib --sandbox <policy> <file>` stays the same library-oriented build plus static policy-schema/config validation. |
| Does that make plain `--lib` part of the stable public embedding surface? | No: Phase 1 support stops at **exact-version consumers**; the stable public embedding surface is still later. |
| Does plain `--lib` emit WIT by default? | No: default WIT emission belongs to the Phase-2 stable public **WIT-first** contract. |
| Do browser or early Node library builds become supported through embedding wording? | No: browser/library combinations stay command-shape contradictions, and Node/library combinations stay on the ordinary Node maturity gate. |

API-surface gating simplification for library-oriented embedding builds:
- `--lib`, `--capi`, and `--component` all reuse the same exported-library contract rather than defining separate host families
- they also reuse the same **statically known export surface** gate described above, so selector changes do not change the underlying `E5511` boundary
- the Phase-1 **base library artifact** is only **buildable for exact-version consumers** in the shared **Deno-oriented build context (schema v1)** from [SPEC.md](../SPEC.md); browser/library combinations stay command-shape contradictions and Node/library combinations stay on the ordinary Node maturity gate
- in that sentence, `apiSurface = deno` is a build/analysis-context choice (ambient typing, package resolution, and command gating), not a claim that the emitted exported-library interface is itself a Deno-specific public ABI
- effective `apiSurface = browser` therefore remains a command-shape contradiction for those library-oriented modes in early phases rather than a second browser embedding profile
- effective `apiSurface = node` follows the ordinary Node build gate; embedding-oriented selectors do not create an earlier Node availability path
- attaching `--sandbox` is orthogonal to those artifact modes: once the underlying library-oriented build shape is otherwise valid, `--sandbox` adds the same static sandbox-policy step without changing API-surface gating or compile intent

This keeps the embedding chapter aligned with the build matrix in [SPEC.md](../SPEC.md), the CLI rules in [12 — CLI](./12-cli.md), and the availability rows in [19 — Feature Maturity](./19-feature-maturity.md) without repeating a second per-artifact gate table.

## Phase 2 target — Rust Library API (`kali_embed`)

Kali is designed to be used as a Rust library, similar to Deno's embedding API, once the public embedding surface reaches Phase 2.

Availability rule:
- the Rust embedding API described in this section is the intended **Phase 2 public surface**
- Phase 1 may already have internal reusable crates, exact-version-consumer flows, and unstable embedding helpers, but those do **not** yet count as the stable public Rust API promised by the maturity matrix

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
    .max_memory_bytes(256 * 1024 * 1024)
    .max_cpu_time_ms(10_000)
    .max_open_files(32)
    .max_spawned_processes(0)
    .max_threads(0)
    .sandbox(SandboxPolicy::from_file("kali.policy.json")?)
    .build();

let mut runtime = Runtime::new(config)?;

// Compile and run a source string through the executable-intent convenience path
let result = runtime.run_executable_string("inline.ts", "const x: number = 1 + 2; x")?;
assert_eq!(result.as_number(), Some(3.0));

// Compile an executable-intent module graph into one linked WASM payload artifact
let module = runtime.compile_executable_file("main.ts")?;
let result = runtime.run_executable_module(&module)?;

// Compile a library-intent module graph and call through the statically known export surface
let module = runtime.compile_library_file("lib.ts")?;
let instance = runtime.instantiate(&module)?;
let result = instance.call("add", &[Value::Number(1.0), Value::Number(2.0)])?;

// The same compiled module may be instantiated more than once.
let second_instance = runtime.instantiate(&module)?;
let second = second_instance.call("add", &[Value::Number(3.0), Value::Number(4.0)])?;
assert_eq!(second.as_number(), Some(7.0));

// Get effect analysis (Phase 2 target)
// Before then, this API may be absent or return the canonical feature-maturity error.
let effects = runtime.analyze_effects_file("program.ts")?;
println!("{}", serde_json::to_string(&effects)?);
```

Canonical embedding-alignment rule:
- the public embedding config should expose the same semantic knobs as CLI/config where they matter to compilation and execution: **API surface**, **build mode**, **runtime profiles**, **compat features**, and the cross-cutting execution-budget/resource-limit knobs
- prefer set-like builder methods such as `runtime_profiles([...])` and `compat_features([...])` over boolean enable/disable pairs so the embedding vocabulary stays aligned with `kali.json`
- execution-budget setters should mirror the shared runtime limit model from CLI/schema v1 instead of inventing an embedding-only vocabulary: memory, CPU time, open files, spawned-process cap, and thread cap
- the canonical runtime-profile name remains `wasm-threads`; if the Rust API exposes `RuntimeProfile::WasmThreads`, that is just the typed embedding spelling of the same underlying profile selected by CLI `--wasm-threads`
- WIT is the canonical host-facing interface description for public library/component outputs; plain public `--lib` is therefore the default stable library artifact shape once Phase 2 lands, and Rust-typed helpers, generated program-specific exports headers, and later component wrappers are projections of that same exported interface contract rather than unrelated parallel ABI descriptions
- embedding APIs may use idiomatic Rust enums/builders instead of the JSON field names, but they should not invent a second incompatible vocabulary for the same concepts
- build-oriented embedding calls obey the same API-surface gates as the CLI/spec matrix: for example, `ApiSurface::Node` remains Phase 3-gated for compile/build flows, while browser-targeted build output is still the `--bundle`-style path rather than a generic library/export mode
- embedding compilation must keep the shared **compile intent** from [SPEC.md](../SPEC.md) explicit, either through separate helpers or an explicit compile option, so hosts do not have to guess exported-library semantics from a later `run_executable_module(...)` vs `instantiate(...)` call
- compiled modules are reusable immutable artifacts: `instantiate(&module)` borrows the compiled module instead of consuming it, and hosts may create multiple instances from one compiled module when that matches the host lifecycle
- executable-style helpers should keep executable intent explicit in their names, for example `run_executable_string(...)` and `run_executable_module(...)`, so hosts do not have to guess whether a generic `run(...)`/`run_module(...)` helper expects executable or library compile intent
- export-oriented/library flows use `instantiate(...).call(...)` and must not rely on a synthetic executable entry being invented for them
- export-oriented/library flows remain part of **Kali-hosted execution** from [SPEC.md](../SPEC.md): module instantiation still performs normal top-level initialization, calls through the **statically known export surface** run under the same sandbox/resource envelope as other embedding execution paths, and there is no special unsandboxed "library mode"
- if a host calls an executable helper on a library-intent module, or tries to treat an executable-intent module as a library without the required **statically known export surface**, that mismatch should fail explicitly rather than being repaired by fallback heuristics
- export-oriented embedding calls require the same **statically known export surface** as the CLI's library-oriented artifact modes; if Kali cannot determine one fixed host-callable export set after frontend lowering, embedding-facing compile/instantiate flows must fail with the same canonical `E5511` path rather than exposing reflection-based export discovery
- if a CLI/config/runtime feature is phase-gated (for example `ApiSurface::Node`, `RuntimeProfile::WasmThreads`, or `CompatFeature::Eval`), the embedding API should surface the same canonical `E5506`-style availability failure rather than silently ignoring the request

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

## Host-Registered Sandbox Predicates (Later Compatibility)

This section is the embedding-side counterpart to the later programmable-policy path described in [09 — Sandboxing & Effects](./09-sandboxing.md) and staged in [19 — Feature Maturity](./19-feature-maturity.md).

Canonical contract:
- project policy files stay declarative JSON data
- any programmable policy logic is registered by a **trusted embedding host**, not loaded from project source code
- predicates are a **narrowing layer only**: they may deny operations that the declarative policy would otherwise allow, but they must not widen a declarative deny, bypass a feature-maturity gate, or conjure an unavailable API surface/capability into existence
- predicate inputs should use the same capability vocabulary as the schema-v1 sandbox/effect model so embedding logic does not invent a second policy language

Illustrative Rust shape:
```rust
use kali_embed::{OperationContext, PredicateDecision};

runtime.register_sandbox_predicate(
    "effects.network.fetch",
    |ctx: &OperationContext| -> PredicateDecision {
        if ctx.resource == "https://api.internal.example" {
            PredicateDecision::Allow
        } else {
            PredicateDecision::Deny("host policy rejected fetch target".into())
        }
    },
)?;
```

Design rules:
- predicates should be synchronous, deterministic, and side-effect free
- they should receive normalized metadata such as the capability kind, target path/URL, and requested operation shape rather than raw host handles
- denial reporting should preserve the canonical Kali diagnostic/error contract, with host-specific predicate detail attached as additional context rather than as an alternate error format
- if this feature is unavailable in the current phase, embedding APIs should fail with the same canonical availability path (`E5506`) used elsewhere rather than silently registering dead callbacks

## Phase 2 target — C ABI (`kali_capi`)

This section describes the intended stable C ABI for embedding from any language once the public embedding surface reaches Phase 2.

Availability rule:
- Phase 1 keeps the compiler/runtime Rust-only internally and may still produce the **base library artifact** through `kali build --lib`
- the stable host-side C ABI, `kali_capi` library, and `kali build --capi` artifact flow are all part of the later **public embedding surface**, not Phase 1 promises

### Host ABI Header (`kali.h`)
The C declarations below describe the intended stable ABI surface starting in the Phase 2 target window.
They are the canonical **host ABI header** from [SPEC.md](../SPEC.md) and come from the host-side `kali_capi` library itself.

Header-split rule:
- `kali.h` is the stable host-side ABI header shipped with `kali_capi`
- `kali build --capi` additionally emits a generated **program-specific exports header** (for example `lib.exports.h`) for the compiled library's **statically known export surface**
- docs should not use `kali.h` as a loose name for both files, because the stable host ABI and the per-build exported-function declarations version and evolve at different layers

Shape simplification rules:
- keep the same compiled-module vs instantiated-instance split as the Rust API in this chapter
- compilation produces a `KaliModule`
- the compile/run/instantiate surface must preserve explicit **compile intent**; hosts must not have to infer exported-library semantics only from whichever post-compile call they try first
- library/export calls go through an instantiated `KaliInstance`, not directly through the compiled module handle
- executable-style convenience entrypoints may still compile-and-run in one step, but that must not blur the library-oriented instantiation contract
- public FFI naming should preserve the canonical **API surface** term from [SPEC.md](../SPEC.md): prefer `api_surface`-style spellings over a generic `api` setter name

```c
#ifndef KALI_H
#define KALI_H

#include <stdint.h>
#include <stdbool.h>

typedef struct KaliRuntime KaliRuntime;
typedef struct KaliConfig KaliConfig;
typedef struct KaliModule KaliModule;
typedef struct KaliInstance KaliInstance;
typedef struct KaliValue KaliValue;
typedef struct KaliError KaliError;

typedef enum KaliApiSurface {
    KALI_API_SURFACE_DENO = 0,
    KALI_API_SURFACE_NODE = 1,
    KALI_API_SURFACE_BROWSER = 2,
} KaliApiSurface;

typedef enum KaliBuildMode {
    KALI_BUILD_MODE_FAST = 0,
    KALI_BUILD_MODE_RELEASE = 1,
    KALI_BUILD_MODE_RELEASE_ADVANCED = 2,
} KaliBuildMode;

typedef enum KaliRuntimeProfile {
    KALI_RUNTIME_PROFILE_WASM_THREADS = 1,
} KaliRuntimeProfile;

typedef enum KaliCompatFeature {
    KALI_COMPAT_FEATURE_EVAL = 1,
} KaliCompatFeature;

typedef enum KaliValueType {
    KALI_VALUE_NUMBER = 1,
    KALI_VALUE_STRING = 2,
    KALI_VALUE_BOOL = 3,
    KALI_VALUE_NULL = 4,
    KALI_VALUE_UNDEFINED = 5,
} KaliValueType;

// Configuration
KaliConfig* kali_config_new(void);
bool kali_config_set_api_surface(KaliConfig* config, KaliApiSurface api_surface);
bool kali_config_set_build_mode(KaliConfig* config, KaliBuildMode build_mode);
void kali_config_clear_runtime_profiles(KaliConfig* config);
bool kali_config_add_runtime_profile(KaliConfig* config, KaliRuntimeProfile profile);
void kali_config_clear_compat_features(KaliConfig* config);
bool kali_config_add_compat_feature(KaliConfig* config, KaliCompatFeature feature);
bool kali_config_set_max_memory(KaliConfig* config, uint64_t bytes);
bool kali_config_set_max_cpu_time(KaliConfig* config, uint64_t ms);
bool kali_config_set_max_open_files(KaliConfig* config, uint32_t count);
bool kali_config_set_max_spawned_processes(KaliConfig* config, uint32_t count);
bool kali_config_set_max_threads(KaliConfig* config, uint32_t count);
bool kali_config_set_sandbox(KaliConfig* config, const char* policy_path);
void kali_config_free(KaliConfig* config);

// Runtime
uint32_t kali_host_abi_version(void);
KaliRuntime* kali_runtime_new(const KaliConfig* config);
void kali_runtime_free(KaliRuntime* runtime);

// Compilation
KaliModule* kali_compile_executable_string(KaliRuntime* runtime, const char* filename, const char* source);
KaliModule* kali_compile_executable_file(KaliRuntime* runtime, const char* path);
KaliModule* kali_compile_library_string(KaliRuntime* runtime, const char* filename, const char* source);
KaliModule* kali_compile_library_file(KaliRuntime* runtime, const char* path);
void kali_module_free(KaliModule* module);

// Instantiation / execution
KaliInstance* kali_instantiate(KaliRuntime* runtime, const KaliModule* module);
void kali_instance_free(KaliInstance* instance);
KaliValue* kali_run_executable_module(KaliRuntime* runtime, const KaliModule* module);
KaliValue* kali_run_executable_string(KaliRuntime* runtime, const char* filename, const char* source);
KaliValue* kali_call(KaliInstance* instance, const char* fn_name,
                     KaliValue** args, uint32_t argc);

// Values
KaliValueType kali_value_type(const KaliValue* value);
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
const char* kali_error_code(const KaliError* error); // stable string code such as "E5506"
const char* kali_error_json(const KaliError* error);

// Effects analysis (Phase 2 target; part of the same stabilized effect-report pipeline
// as `kali effects` once the stable public embedding surface exists)
const char* kali_analyze_effects_file(KaliRuntime* runtime, const char* path);
const char* kali_analyze_effects_string(KaliRuntime* runtime, const char* filename, const char* source);
void kali_free_string(const char* s);

// Host function registration
typedef KaliValue* (*KaliHostFn)(KaliValue** args, uint32_t argc, void* userdata);
bool kali_register_host_function(KaliRuntime* runtime, const char* module,
                                 const char* name, KaliHostFn fn, void* userdata);

#endif // KALI_H
```

### Memory Management
- All `kali_*_new` / `kali_*_free` pairs — caller manages lifetime
- `KaliConfig*` is a caller-owned builder object. `kali_runtime_new(const KaliConfig* config)` snapshots the effective config and does **not** consume ownership, so the caller may free the config immediately after runtime creation succeeds or fails.
- `KaliModule*` and `KaliInstance*` are distinct owned handles: `kali_instantiate(..., const KaliModule* module)` and `kali_run_executable_module(..., const KaliModule* module)` borrow the compiled module rather than consuming it, so one compiled module may back multiple instances or repeated runs when that matches its explicit compile intent. Freeing a compiled module does not implicitly free any instantiated instance unless a later ABI revision documents that ownership transfer explicitly.
- Borrowed string views such as `kali_value_as_string(...)`, `kali_error_message(...)`, and `kali_error_code(...)` remain owned by Kali and are valid only while the referenced value/error object remains alive
- Owned string results such as `kali_error_json(...)`, `kali_analyze_effects_file(...)`, and `kali_analyze_effects_string(...)` must be freed with `kali_free_string`
- because the public C ABI itself is a **Phase 2 target**, pre-Phase-2 internal prototypes are free to omit unstable helpers such as the effect-analysis entrypoints instead of pretending they already exist as a stable callable contract
- Thread safety: one `KaliRuntime` per thread in the initial implementation
- The C config surface follows the same set-like semantics as `kali.json` and the Rust builder API: runtime profiles and compat features are unordered unique sets, not boolean toggle pairs
- enum spellings such as `KaliApiSurface`, `KaliBuildMode`, `KaliRuntimeProfile`, and `KaliCompatFeature` are the typed C-ABI counterparts of the canonical config/CLI vocabularies `apiSurface`, `buildMode`, `runtimeProfiles`, and `compat.features`
- the resource-limit setters `kali_config_set_max_memory`, `kali_config_set_max_cpu_time`, `kali_config_set_max_open_files`, `kali_config_set_max_spawned_processes`, and `kali_config_set_max_threads` mirror the shared execution-budget model from CLI/schema v1 instead of inventing C-only names
- unit spelling stays implementation-friendly but semantically aligned: `kali_config_set_max_memory(..., bytes)` uses raw bytes for FFI friendliness, `kali_config_set_max_cpu_time(..., ms)` uses milliseconds, and the Rust builder should prefer the same normalized byte/ms/count model (for example `max_memory_bytes(...)`) even though the policy schema still stores the memory cap as `maxMemoryMB`
- for those setters, `max_memory`, `max_cpu_time`, and `max_open_files` keep the same positive-only semantics as CLI/schema v1, while `max_spawned_processes` and `max_threads` may use `0` as an explicit deny/tightening value
- mutating config helpers return `bool` so validation/allocation/phase-gating failures all use one C-friendly convention instead of mixing `void` setters with out-of-band failure cases
- C config/runtime setters for API surface, build mode, runtime profiles, compat features, and resource limits follow the same phase-gating rules as the CLI/config surface; unsupported requests fail with the canonical availability error instead of degrading silently
- Exposing a C ABI does **not** imply linking any C/C++ implementation into Kali itself; the runtime and compiler remain Rust-only internally

### Error Handling Convention
- Pointer-returning functions that can fail return `NULL` on error
- Boolean-returning registration/configuration helpers return `false` on error
- Call `kali_last_error()` for runtime-bound failures, or `kali_global_last_error()` for failures that happen before a `KaliRuntime` exists (for example `kali_runtime_new` returning `NULL`)
- failed config mutations also report through `kali_global_last_error()` because they may occur before a runtime exists
- `kali_run_executable_module(...)` and `kali_run_executable_string(...)` are executable-style convenience paths and should fail for a compiled module or source input that does not have an executable entry contract; export-oriented/library flows should use `kali_compile_library_*` + `kali_instantiate(...)` + `kali_call(...)` instead of expecting the runtime to invent a synthetic entrypoint
- Error includes the stable string diagnostic code, message, and JSON representation so embedders see the same canonical machine contract as the CLI

### Building
Artifact selection follows the canonical build matrix and shared **embedding-stability split** in [SPEC.md](../SPEC.md): plain `--lib` is the Phase-1 **base library artifact**, and `kali build --capi` / `kali build --component` are Phase-2 **public embedding artifact flows** over that same exported-library contract rather than unrelated semantics.

`kali build --capi` and `kali build --component` are **Phase 2 targets** and are artifact-generation modes for embedded programs, not requests to turn user TypeScript directly into a native shared library.

```bash
kali build --lib lib.ts                    # Minimum historical Phase-1 contract: lib.wasm only (base library artifact). Current repo: the same selector is live as the stable public WIT-first contract and emits lib.wasm + lib.wit.
kali build --capi lib.ts                   # Earliest contract: Phase 2 target. Current repo: live as lib.wasm + lib.wit + generated lib.exports.h + lib.cabi.json for use with kali_capi.
kali build --component lib.ts              # Earliest contract: Phase 2 target. Current repo: live as lib.wasm + lib.wit + lib.component.wasm for Component Model consumers.
```

Example-filename rule:
- build examples in this chapter use basename-derived filenames (`lib.ts` → `lib.wasm`, `lib.wit`, `lib.exports.h`, `lib.component.wasm`) as illustrative examples only
- the normative machine contract is the emitted artifact list's `kind` + `role` metadata plus the availability/gating rules in [specs/12-cli.md](12-cli.md), [specs/18-schemas.md](18-schemas.md), and [specs/19-feature-maturity.md](19-feature-maturity.md)

Artifact-role clarification:
- `kali build --lib` is the base exported-library path in Phase 1 and the canonical stable public **WIT-first** library path from the Phase 2 target onward; once stabilized, that plain public `--lib` output emits `wit` (`role: interface-wit`) by default alongside the core `wasm-module` (`role: primary-library`)
- `kali build --capi` uses that same core exported-library artifact (`role: primary-library`) plus `wit` (`role: interface-wit`), the generated **program-specific exports header** such as `lib.exports.h` (`role: embedding-header`), the generated `cabi-metadata` file such as `lib.cabi.json` (`kind: cabi-metadata`, `role: embedding-metadata`), and the deterministic stem-specific `binding-package` sidecar manifest for higher-level language bindings
- `kali build --component` keeps the same linked core library payload (`role: primary-library`) and WIT sidecar (`role: interface-wit`), then adds the outer Component Model wrapper as `kind: wasm-component`, `role: primary-component`, plus the same deterministic stem-specific `binding-package` sidecar manifest
- library-oriented embedding outputs require the same **statically known export surface** defined in [SPEC.md](../SPEC.md); WIT, generated program-specific exports headers, and component packaging are projections of that same explicit export surface rather than separate reflection-based APIs
- if that export surface cannot be determined statically, the build must fail with `E5511` instead of synthesizing reflection-based exports for embedding
- that outer component wrapper is packaging over the already-linked core payload, not a second independently linked guest-program graph; this keeps embedding/component outputs aligned with the single-linked-core-payload rule from [SPEC.md](../SPEC.md)

Important distinction:
- `kali_capi` ships the stable **host ABI header**: `kali.h`
- `kali build --capi foo.ts` emits the **program-specific exports header** such as `foo.exports.h` plus the generated `cabi-metadata` file such as `foo.cabi.json`
- the **minimum historical Phase-1 contract** for plain `kali build --lib foo.ts` is the **base library artifact** (`wasm-module`) only; that wording explains the earliest promise, not the full current repository state after the Phase-2 embedding rows opened
- in the **current repository state**, plain public `kali build --lib foo.ts` emits a WIT sidecar by default, and `--capi` / `--component` reuse that same canonical exported interface description instead of defining a second export vocabulary or becoming implicit replacements for plain `--lib`
- library-oriented outputs follow the shared **library-oriented instantiation rule** from [SPEC.md](../SPEC.md): no synthetic executable entry invocation is added, normal module-instantiation behavior still runs at host instantiation time, and exported functions are the host-callable surface layered on top of that
- In CLI JSON/artifact manifests, these outputs use the canonical artifact kinds `wasm-module`, `wit`, `wasm-component`, `c-header`, `cabi-metadata`, and `binding-package`

This keeps the shared **host ABI header vs program-specific exports header** split from [SPEC.md](../SPEC.md) intact and avoids overloading the name `kali.h` for two different purposes.

The host-side C ABI itself is intended to be provided by the `kali_capi` crate once that host-side crate lands:
```bash
cargo build --release -p kali_capi         # Illustrative target command once the host-side C ABI crate exists
```

## ABI Versioning and Compatibility

To keep embedding stable and machine-checkable, the C ABI needs one explicit compatibility rule instead of relying on prose:

- `kali_capi` publishes a monotonically increasing **host ABI version** integer
- the stable host header exposes it via constants/macros such as:
  - `KALI_CAPI_ABI_VERSION`
  - `KALI_CAPI_ABI_MIN_COMPAT_VERSION` *(optional if compatible windowing is needed)*
- the host ABI library exports `uint32_t kali_host_abi_version(void);`
- `kali build --capi foo.ts` embeds the expected host ABI version in emitted metadata so loaders can reject incompatible host/program combinations before instantiation; the canonical metadata-file shape belongs to [specs/18-schemas.md](18-schemas.md)'s **C ABI Metadata Schema (schema v1)** section rather than to ad hoc per-chapter prose
- the host library reports its own version through `uint32_t kali_host_abi_version(void);`; the function name intentionally says **host ABI** to match the metadata field names (`hostAbiVersion`, `minHostAbiVersion`) and avoid ambiguity with guest/runtime ABI concepts
- incompatible ABI versions are a hard load-time error; they must not silently proceed on a best-effort basis

Compatibility policy:
- additive C-ABI changes that preserve layout/call compatibility may keep the same major host ABI version
- signature changes, ownership-convention changes, struct layout changes, or semantic changes that break existing embedders require a new host ABI version
- the generated **program-specific exports header** (`foo.exports.h`) may evolve independently from the stable **host ABI header** (`kali.h`), but its emitted metadata must still declare which host ABI version it expects

Typical embedding flow:
1. Build or ship `kali_capi` as the native C ABI layer (including the stable `kali.h` host header).
2. Compile Kali/TypeScript library code with `kali build --capi lib.ts` to obtain the C-embedding artifact set — illustratively named files such as `lib.wasm`, `lib.wit`, `lib.exports.h`, and generated metadata.
3. Verify ABI compatibility between the emitted metadata and the available `kali_capi` host library.
4. Load that artifact through the `kali_*` API from C or another FFI consumer.

Typical component flow:
1. Compile Kali/TypeScript library code with `kali build --component lib.ts`.
2. Use the emitted WIT sidecar — illustratively `lib.wit` — as the canonical interface description for tooling/review.
3. Load the emitted component wrapper — illustratively `lib.component.wasm` — in a Component Model host that matches the documented runtime/profile constraints.

The shared library exports only `kali_*` symbols. All Rust internals are hidden.

## Language Bindings (Future)

The C ABI enables bindings for:
- Python (`ctypes` or `cffi`)
- Go (`cgo`)
- Ruby (`ffi`)
- Java (`JNI`)
- C# (`P/Invoke`)
- Zig (direct C interop)

Once the public embedding surface lands, the stable embedding contract includes both the public Rust API and the C ABI/WIT-based host contract. For non-Rust languages, the first cross-language stable focus is the C ABI, while language-specific bindings remain community-driven or higher-level wrappers over that ABI.
