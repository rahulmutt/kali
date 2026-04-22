# SPEC mailbox

Status: cleared on 2026-04-22.

The actionable spec decisions that had accumulated here have now been folded into the normative spec set. `SPEC-MAILBOX.md` is no longer carrying open follow-up items.

## Decisions incorporated

- `specs/12-cli.md` and `specs/19-feature-maturity.md` now expose the build-only `kali build --profile <file>` PGO input as an explicit opt-in surface instead of a hidden implementation detail.
- `SPEC.md`, `specs/13-embedding.md`, and `specs/18-schemas.md` now align on the shared `binding-package` sidecar artifact emitted by later embedding flows, including the deterministic stem-specific bundle index behavior for `build --capi` and `build --component`.
- `specs/12-cli.md` and `specs/19-feature-maturity.md` now use the synchronized execution shape `kali run <file> [-- args...]`.
- `specs/09-sandboxing.md`, `specs/12-cli.md`, and `specs/18-schemas.md` now reflect the subprocess-budget contract where `resources.maxSpawnedProcesses` / `--max-spawned-processes` accept positive values once subprocess support exists, while browser-targeted constraints remain separate.
- `specs/18-schemas.md`, `specs/12-cli.md`, `specs/16-testing.md`, and `specs/19-feature-maturity.md` now treat `kali test --coverage` as the stable coverage selector with a deterministic machine-readable payload, including normalized relative paths and deterministic file ordering.
- `specs/16-testing.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, `README.md`, and `proofs/BOUNDARY.md` now share the canonical proof-backed summary wording and synchronized proof-summary inventory updates.

## Mailbox rule going forward

Only keep genuinely unresolved spec follow-ups here. Once a decision is reflected in the owning normative files, remove the prose from this mailbox instead of preserving a second shadow copy.

- [2026-04-22] Stage 5.2 browser/runtime host expansion would benefit from documenting optional `hostContract` and `runtimeBackend` fields on artifact metadata sidecars in `specs/18-schemas.md`. The intended meaning is provenance-only: record the build/execution host contract and runtime backend selection (currently the Kali-hosted Wasmtime baseline) so browser-targeted artifacts can remain explicit about which host produced them without implying standalone browser runtime support.
