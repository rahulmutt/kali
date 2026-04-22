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

Common commands:

```bash
kali init
kali install
kali fmt
kali lint
kali check [files...]
kali build <file>
kali build --bundle <file>
kali run <file> [-- args...]
kali test [files...]
```

Helpful flags you will see often:

- `--api deno|node|browser`
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
| `build` | Produce executable, library, or bundle artifacts |
| `run` | Compile and execute a source file |
| `test` | Compile and run tests |
| `effects` | Analyze effect usage and sandbox interactions |
| `package-effects` | Analyze a single package’s effects |
| `package-audit` | Audit a package in the later registry-analysis flow |

## Project status

Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.

Phase 1 is the main public surface; see [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) for the exact shipped availability.

## Documentation

- [`SPEC.md`](./SPEC.md) — normalized cross-spec rules and shared terminology
- [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) — shipped availability by phase
- [`specs/12-cli.md`](./specs/12-cli.md) — CLI shapes, flags, and output contracts
- [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) — current verification-claim boundary
