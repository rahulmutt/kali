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

Common Phase-1 commands:

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

Helpful flags you will see often:

- `--api deno|node|browser` *(Phase 1 ships the Deno-oriented default plus the browser-targeted `check` / `build --bundle` context; broad `node` support is later)*
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
| `build` | Produce executable artifacts, browser bundles, or the Phase-1 base library artifact |
| `run` | Compile and execute a source file |
| `test` | Compile and run tests |
| `effects` | Later Phase-2 effect-report command family |
| `package-effects` | Later Phase-2 single-package effect-report command family |
| `package-audit` | Later Phase-4 registry-analysis/audit command family |

## Project status

Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.

Phase 1 is the main public surface; later documented commands such as `effects`, `package-effects`, and `package-audit` keep their shape documented without being implied as already shipped. See [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) for exact availability.

## Documentation

- [`SPEC.md`](./SPEC.md) — normalized cross-spec rules and shared terminology
- [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) — shipped availability by phase
- [`specs/12-cli.md`](./specs/12-cli.md) — CLI shapes, flags, and output contracts
- [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) — current verification-claim boundary
