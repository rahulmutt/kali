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

The package is intentionally small and deterministic so the binding workflow can be
reproduced from the generated C ABI header, metadata, and manifest files.
