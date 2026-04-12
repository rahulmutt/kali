# Stage 1.12 Status Update

**Date:** 2026-04-12  
**Status:** ✅ Developer workflow stage complete

## Summary

Stage 1.12 is now complete. The developer workflow surface is wired end-to-end: `kali init`, `kali fmt`, and `kali lint` are available with the canonical Phase-1 behaviors, and project discovery now follows the shared source-walk rules used by the rest of the CLI.

## Evidence

- `cargo test -p kali_cli --lib` ✅
- `cargo test -p kali_cli --test runtime_smoke` ✅
- `cargo test --workspace` ✅

## Notable Deliverables

- `kali init` and `kali init --lib` scaffold executable and library projects
- `kali fmt` supports in-place formatting and `--check`
- `kali lint` supports the initial Phase-1 built-in `W2xxx` rule set and `--fix`
- Source-file discovery now excludes hidden directories, nested project roots, and test files while still honoring declaration-file handling where required

## Next Step

Move on to Stage 1.13 — Diagnostics & Schemas.
