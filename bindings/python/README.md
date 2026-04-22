# kali_capi Python binding helper

This directory packages the maintained Python ctypes helper for Kali's stable C ABI.
It provides deterministic Python ctypes bindings for Kali's stable C ABI.

## Layout

- `kali_capi/` — importable Python module
- `pyproject.toml` — deterministic packaging metadata

## Usage

```python
from pathlib import Path
from kali_capi import KaliCAPI

binding = KaliCAPI.from_binding_package(
    library,
    Path("dist/binding-package"),
)
```

`from_binding_package()` looks for an explicit `binding-package.json` first and then
falls back to a single stem-specific `*.binding-package.json` manifest in the bundle
root, which matches the generated Kali layout.

The package is intentionally small and deterministic so the binding workflow can be
reproduced from the generated C ABI header, metadata, and manifest files.
