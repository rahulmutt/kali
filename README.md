# Kali

Kali is an ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, with a CLI for checking, building, running, testing, formatting, and linting projects.

## Build from source

```bash
cargo build --workspace
cargo test --workspace
cargo run -p kali_cli --bin kali -- --help
```

Tagged releases are produced by `.github/workflows/release.yml`, which currently builds Linux x86_64, Linux arm64, and macOS arm64 `kali` CLI binaries and publishes matching SLSA3 provenance for the release assets. macOS releases are arm64-only.

If you are working on Lean proofs, use:

```bash
mise run lean-proofs
```

## Use the CLI

Baseline Phase-1 commands:

```bash
kali doctor                         # inspect local tool/environment selection
kali doctor --output json           # emit the schema-v1 doctor result envelope
kali init
kali init --lib                 # create the minimal library scaffold
kali init --output json         # emit the schema-v1 init result envelope
kali init --output json --lib    # emit the library scaffold result envelope
kali install
kali fmt
kali lint
kali check [files...]
kali check --api browser main.ts # browser-targeted analysis lane
kali build <file>
kali build --validate-ir <file>  # run internal HIR/MIR/LIR validators
kali build --bundle --api browser <file> # browser-targeted build lane
kali build --bundle --format cjs <file> # browser-targeted CommonJS browser bundle wrapper
kali build --lib <file>         # base library artifact for exact-version consumers
kali run <file> [-- args...]
kali test [files...]
kali test --coverage [files...]
```

Additional public commands live in the current repository state:

```bash
kali build --capi <file>        # stable public C-ABI embedding flow
kali build --component <file>   # Component Model packaging flow
kali build --lib --api browser lib.ts # invalid usage (E5508): browser library artifact modes are not part of the browser-targeted command set
kali build --capi --api browser lib.ts # invalid usage (E5508): browser embedding artifact modes are not part of the browser-targeted command set
kali build --component --api browser lib.ts # invalid usage (E5508): browser component artifact modes are not part of the browser-targeted command set
kali build --lib --api browser --sandbox kali.policy.json lib.ts # invalid usage (E5508): sandboxing does not create a browser library artifact mode
kali build --capi --api browser --sandbox kali.policy.json lib.ts # invalid usage (E5508): sandboxing does not create a browser embedding artifact mode
kali build --component --api browser --sandbox kali.policy.json lib.ts # invalid usage (E5508): sandboxing does not create a browser component artifact mode
kali build --profile pgo-profile.json main.ts # load deterministic PGO profile data
kali check --sandbox kali.policy.json main.ts
kali build --bundle --sandbox kali.policy.json main.ts
kali build --lib --sandbox kali.policy.json lib.ts
kali build --capi --sandbox kali.policy.json lib.ts
kali build --component --sandbox kali.policy.json lib.ts
kali effects <file>
kali effects --output json main.ts
kali package-effects <package>
kali package-effects --output json lodash
kali package-audit <package>
kali package-audit --output json lodash
```

Helpful flags you will see often:

- `--api deno|node|browser` *(Phase 1 ships the Deno-oriented default plus the browser-targeted `check` / `build --bundle` context; `check` / `build` now also accept the documented Node analysis/build subset, while `run` / `test` support the documented Node execution subset and broader effect/registry-analysis breadth remains later)*
- `--validate-ir` *(debug aid for `build`; validates the lowered HIR/MIR/LIR tree and can be forced on when you want an explicit validation pass)*
- `--sandbox <policy>`
- `--output json`
- `--verbose` / `--quiet` *(verbose adds extra context and error docs links in human output)*
- `--release` / `--release-advanced`

For the full CLI contract, arity, and flag behavior, see [`specs/12-cli.md`](./specs/12-cli.md).

## Command reference

| Command | Purpose |
| --- | --- |
| `doctor` | Inspect local Kali tool/environment selection, including the browser harness command and browser runtime contract snapshot |
| `init` | Create a new Kali project or library scaffold |
| `install` | Resolve and install dependencies |
| `fmt` | Format files or a discovered project |
| `lint` | Run lint diagnostics and optional safe fixes |
| `check` | Type-check and statically validate source |
| `build` | Produce executable artifacts, browser bundles, library artifacts, embedding artifacts, or an explicit IR validation pass depending on flags |
| `run` | Compile and execute a source file |
| `test` | Compile and run tests; `--coverage` adds deterministic function-coverage data through the stable result payload |
| `effects` | Report conservative effects for a source graph |
| `package-effects` | Report conservative effects for one registry package |
| `package-audit` | Run context-free registry analysis / audit for one package |

## Project status

Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.

Phase 1 remains the baseline public contract, and the current repository has also opened selected later-phase surfaces including Node-capable `check` / `build` / `run` / `test`, Node-capable `effects` / `package-effects`, `package-audit`, the stable public embedding flows, and the tag-based multi-platform GitHub release workflow with SLSA3 provenance for the CLI binaries. See [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) for the exact command/context availability matrix and current-state notes.

## Documentation

- [`SPEC.md`](./SPEC.md) — normalized cross-spec rules and shared terminology
- [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) — shipped availability by phase
- [`specs/12-cli.md`](./specs/12-cli.md) — CLI shapes, flags, and output contracts
- [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) — current verification-claim boundary
