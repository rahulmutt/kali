"""Python ctypes binding helper for Kali's stable C ABI.

The stable C ABI is the public embedding layer that higher-level language
wrappers can build on. This module keeps the Python wrapper surface small and
fully deterministic: it discovers exported entrypoints from the generated C
header, validates the accompanying host ABI metadata, binds the exports onto a
lightweight wrapper object, and leaves transport control to the caller.
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
import ctypes
import json
import re
from typing import Sequence

HOST_ABI_VERSION = 2

__all__ = [
    "HOST_ABI_VERSION",
    "CabiMetadata",
    "Export",
    "KaliCAPI",
    "ensure_compatible_metadata",
    "load_library",
    "load_metadata",
    "parse_exports",
    "parse_metadata",
]

_EXPORT_RE = re.compile(
    r"^extern\s+int32_t\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\((?P<params>[^)]*)\);$",
    re.MULTILINE,
)


@dataclass(frozen=True)
class Export:
    """A single exported C ABI entrypoint discovered from the generated header."""

    name: str
    arity: int


@dataclass(frozen=True)
class CabiMetadata:
    """The deterministic host ABI metadata bundled with a generated C ABI artifact."""

    schema_version: int
    kind: str
    host_abi_version: int
    min_host_abi_version: int
    artifacts: dict[str, str]


def _is_int(value: object) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)


def _require_int(payload: Mapping[str, object], key: str) -> int:
    value = payload.get(key)
    if not _is_int(value):
        raise ValueError(f"cabi metadata field {key!r} must be an integer")
    return int(value)


def _require_str(payload: Mapping[str, object], key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str):
        raise ValueError(f"cabi metadata field {key!r} must be a string")
    return value


def _require_artifacts(payload: object) -> dict[str, str]:
    if not isinstance(payload, Mapping):
        raise ValueError("cabi metadata field 'artifacts' must be a JSON object")

    required_keys = ("wasmModule", "wit", "exportsHeader")
    artifacts: dict[str, str] = {}
    for key in required_keys:
        value = payload.get(key)
        if not isinstance(value, str):
            raise ValueError(f"cabi metadata field 'artifacts.{key}' must be a string")
        artifacts[key] = value

    # Keep the mapping order deterministic for callers that inspect the payload.
    return dict(sorted(artifacts.items()))


def parse_exports(header_text: str) -> list[Export]:
    """Parse a generated C header into a deterministic export list."""

    exports: list[Export] = []
    for match in _EXPORT_RE.finditer(header_text):
        params = match.group("params").strip()
        if not params or params == "void":
            arity = 0
        else:
            arity = len([param for param in params.split(",") if param.strip()])
        exports.append(Export(match.group("name"), arity))
    return exports


def parse_metadata(metadata_text: str) -> CabiMetadata:
    """Parse and validate the generated C ABI metadata payload."""

    payload = json.loads(metadata_text)
    if not isinstance(payload, Mapping):
        raise ValueError("cabi metadata must be a JSON object")

    schema_version = _require_int(payload, "schemaVersion")
    if schema_version != 1:
        raise ValueError(f"unsupported cabi metadata schemaVersion {schema_version}")

    kind = _require_str(payload, "kind")
    if kind != "cabi-metadata":
        raise ValueError(f"unsupported cabi metadata kind {kind!r}")

    host_abi_version = _require_int(payload, "hostAbiVersion")
    min_host_abi_version = payload.get("minHostAbiVersion", host_abi_version)
    if not _is_int(min_host_abi_version):
        raise ValueError("cabi metadata field 'minHostAbiVersion' must be an integer")

    artifacts = _require_artifacts(payload.get("artifacts"))
    return CabiMetadata(
        schema_version=schema_version,
        kind=kind,
        host_abi_version=host_abi_version,
        min_host_abi_version=int(min_host_abi_version),
        artifacts=artifacts,
    )


def load_metadata(path: str | Path) -> CabiMetadata:
    """Load generated C ABI metadata from disk."""

    return parse_metadata(Path(path).read_text())


def ensure_compatible_metadata(
    metadata: CabiMetadata,
    available_host_abi_version: int = HOST_ABI_VERSION,
) -> CabiMetadata:
    """Validate that a host ABI version can load the generated metadata."""

    if not _is_int(available_host_abi_version):
        raise ValueError("available_host_abi_version must be an integer")

    if not (
        metadata.min_host_abi_version
        <= int(available_host_abi_version)
        <= metadata.host_abi_version
    ):
        raise ValueError(
            "host ABI version "
            f"{available_host_abi_version} is incompatible with metadata range "
            f"{metadata.min_host_abi_version}..={metadata.host_abi_version}"
        )
    return metadata


def load_library(path: str | Path) -> ctypes.CDLL:
    """Load a compiled Kali C ABI library with ctypes."""

    return ctypes.CDLL(str(path))


class KaliCAPI:
    """Bind the exports declared in a Kali C ABI header onto a Python object."""

    def __init__(self, library: object, exports: Sequence[Export]):
        self._library = library
        self._exports = tuple(exports)
        self._bind_exports()

    @classmethod
    def from_header(cls, library: object, header_text: str) -> "KaliCAPI":
        return cls(library, parse_exports(header_text))

    @classmethod
    def from_header_and_metadata(
        cls,
        library: object,
        header_text: str,
        metadata_text: str,
        *,
        available_host_abi_version: int = HOST_ABI_VERSION,
    ) -> "KaliCAPI":
        ensure_compatible_metadata(
            parse_metadata(metadata_text),
            available_host_abi_version=available_host_abi_version,
        )
        return cls(library, parse_exports(header_text))

    @classmethod
    def from_library_path(cls, path: str | Path, header_text: str) -> "KaliCAPI":
        return cls(load_library(path), parse_exports(header_text))

    @property
    def exports(self) -> tuple[Export, ...]:
        return self._exports

    def _bind_exports(self) -> None:
        for export in self._exports:
            function = getattr(self._library, export.name)
            if hasattr(function, "argtypes"):
                function.argtypes = [ctypes.c_int32] * export.arity
            if hasattr(function, "restype"):
                function.restype = ctypes.c_int32
            setattr(self, export.name, function)
