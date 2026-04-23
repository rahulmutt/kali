# Kali

Kali is an ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, with a CLI for checking, building, running, testing, formatting, and linting projects.

## Build from source

```bash
cargo build --workspace
cargo test --workspace
cargo run -p kali_cli --bin kali -- --help
```

If you are working on Lean proofs, use:

```bash
mise run lean-proofs
```

## Use the CLI

Baseline Phase-1 commands:

```bash
kali init
kali install
kali fmt
kali lint
kali check [files...]
kali build <file>
kali build --bundle <file>      # browser-targeted build lane
kali build --lib <file>         # base library artifact for exact-version consumers
kali run <file> [-- args...]
kali test [files...]
```

Additional public commands live in the current repository state:

```bash
kali build --capi <file>        # stable public C-ABI embedding flow
kali build --component <file>   # Component Model packaging flow
kali effects <file>
kali package-effects <package>
kali package-audit <package>
```

Helpful flags you will see often:

- `--api deno|node|browser` *(Phase 1 ships the Deno-oriented default plus the browser-targeted `check` / `build --bundle` context; `check` / `build` now also accept the documented Node analysis/build subset, while `run` / `test` support the documented Node execution subset and broader effect/registry-analysis breadth remains later)*
- `--sandbox <policy>`
- `--output json`
- `--verbose` / `--quiet`
- `--release` / `--release-advanced`

For the full CLI contract, arity, and flag behavior, see [`specs/12-cli.md`](./specs/12-cli.md).

## Command reference

| Command | Purpose |
| --- | --- |
| `init` | Create a new Kali project |
| `install` | Resolve and install dependencies |
| `fmt` | Format files or a discovered project |
| `lint` | Run lint diagnostics and optional safe fixes |
| `check` | Type-check and statically validate source |
| `build` | Produce executable artifacts, browser bundles, library artifacts, or embedding artifacts depending on flags |
| `run` | Compile and execute a source file |
| `test` | Compile and run tests |
| `effects` | Report conservative effects for a source graph |
| `package-effects` | Report conservative effects for one registry package |
| `package-audit` | Run context-free registry analysis / audit for one package |

## Project status

Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.

Phase 1 remains the baseline public contract, and the current repository has also opened selected later-phase surfaces including Node-capable `check` / `build` / `run` / `test`, `effects`, `package-effects`, `package-audit`, and the stable public embedding flows. See [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) for the exact command/context availability matrix and current-state notes.

## Documentation

- [`SPEC.md`](./SPEC.md) — normalized cross-spec rules and shared terminology
- [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) — shipped availability by phase
- [`specs/12-cli.md`](./specs/12-cli.md) — CLI shapes, flags, and output contracts
- [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) — current verification-claim boundary
