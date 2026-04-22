"""Python ctypes binding helper for Kali's stable C ABI.

The stable C ABI is the public embedding layer that higher-level language
wrappers can build on. This module keeps the Python wrapper surface small and
fully deterministic: it discovers exported entrypoints from the generated C
header, binds them onto a lightweight wrapper object, and leaves transport
control to the caller.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import ctypes
import re
from typing import Iterable, Sequence

HOST_ABI_VERSION = 2

__all__ = ["HOST_ABI_VERSION", "Export", "KaliCAPI", "load_library", "parse_exports"]

_EXPORT_RE = re.compile(
    r"^extern\s+int32_t\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\((?P<params>[^)]*)\);$",
    re.MULTILINE,
)


@dataclass(frozen=True)
class Export:
    """A single exported C ABI entrypoint discovered from the generated header."""

    name: str
    arity: int


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
