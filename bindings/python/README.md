# kali_capi Python binding helper

This directory packages the maintained Python ctypes helper for Kali's stable C ABI.
It provides deterministic Python ctypes bindings for Kali's stable C ABI.

## Layout

- `kali_capi/` — importable Python module
- `pyproject.toml` — deterministic packaging metadata

## Usage

```python
from pathlib import Path
from kali_capi import (
    KaliCAPI,
    load_binding_package_manifest_from_root,
)

manifest = load_binding_package_manifest_from_root(Path("dist/binding-package"))
binding = KaliCAPI.from_binding_package(
    library,
    Path("dist/binding-package"),
)
```

`load_binding_package_manifest_from_root()` and `from_binding_package()` both
look for an explicit `binding-package.json` first and then fall back to a single
stem-specific `*.binding-package.json` manifest in the bundle root, which matches
the generated Kali layout. `binding_package_manifest_summary()` projects that
normalized manifest into a compact summary object for callers that want one
stable provenance snapshot, and `load_binding_package_manifest_summary()` / `load_binding_package_manifest_summary_from_root()`
perform the same load-and-project step in one call. `cabi_metadata_summary()`
and `load_metadata_summary()` provide the same deterministic projection for the
companion `cabi-metadata` sidecar, preserving the optional runtime provenance
fields when the compiler emits them. The resulting `KaliCAPI` wrapper also
exposes the bundle's `max_specializations` provenance plus the normalized
runtime provenance tuple (`runtime_profiles`, `host_contract`, and
`runtime_backend`) when the manifest publishes it, so callers can inspect the
same deterministic specialization and runtime context that the CLI emitted.

The package is intentionally small and deterministic so the binding workflow can be
reproduced from the generated C ABI header, metadata, and manifest files.
