from __future__ import annotations

from pathlib import Path
import json
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from kali_capi import (  # noqa: E402
    BindingPackageManifest,
    Export,
    KaliCAPI,
    discover_binding_package_manifest_path,
    ensure_compatible_binding_package_manifest,
    ensure_compatible_metadata,
    load_binding_package_manifest,
    load_binding_package_manifest_from_root,
    load_metadata,
    parse_exports,
)


class DummyLibrary:
    def __init__(self) -> None:
        self.calls: list[tuple[object, ...]] = []

    def add(self, left: int, right: int) -> int:
        self.calls.append(("add", left, right))
        return left + right

    def zero(self) -> int:
        self.calls.append(("zero",))
        return 7


class KaliCapiSmokeTests(unittest.TestCase):
    def test_binding_package_round_trip(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            header_path = root / "sample.h"
            metadata_path = root / "sample.cabi.json"
            manifest_path = root / "binding-package.json"

            header_path.write_text(
                "\n".join(
                    [
                        "#include <stdint.h>",
                        "extern int32_t add(int32_t arg0, int32_t arg1);",
                        "extern int32_t zero(void);",
                    ]
                )
                + "\n"
            )
            metadata_path.write_text(
                json.dumps(
                    {
                        "schemaVersion": 1,
                        "kind": "cabi-metadata",
                        "hostAbiVersion": 2,
                        "minHostAbiVersion": 2,
                        "artifacts": {
                            "exportsHeader": "sample.h",
                            "metadata": "sample.cabi.json",
                            "wasmModule": "sample.capi.wasm",
                            "wit": "sample.wit",
                        },
                    },
                    sort_keys=True,
                )
            )
            manifest_path.write_text(
                json.dumps(
                    {
                        "schemaVersion": 1,
                        "kind": "binding-package",
                        "moduleName": "sample",
                        "hostAbiVersion": 2,
                        "minHostAbiVersion": 2,
                        "maxSpecializations": 8,
                        "artifacts": {
                            "exportsHeader": "sample.h",
                            "glue": ["shim.py", "support.py"],
                            "library": "sample.capi.wasm",
                            "metadata": "sample.cabi.json",
                        },
                    },
                    sort_keys=True,
                )
            )
            (root / "sample.capi.wasm").write_bytes(b"")

            exports = parse_exports(header_path.read_text())
            self.assertEqual(exports, [Export("add", 2), Export("zero", 0)])

            metadata = load_metadata(metadata_path)
            self.assertEqual(metadata.host_abi_version, 2)
            self.assertEqual(metadata.min_host_abi_version, 2)
            self.assertEqual(
                metadata.artifacts,
                {
                    "exportsHeader": "sample.h",
                    "wasmModule": "sample.capi.wasm",
                    "wit": "sample.wit",
                },
            )
            self.assertEqual(ensure_compatible_metadata(metadata), metadata)

            discovered = discover_binding_package_manifest_path(root)
            self.assertEqual(discovered, manifest_path)

            manifest = load_binding_package_manifest_from_root(root)
            self.assertEqual(
                manifest,
                BindingPackageManifest(
                    schema_version=1,
                    kind="binding-package",
                    module_name="sample",
                    host_abi_version=2,
                    min_host_abi_version=2,
                    max_specializations=8,
                    artifacts={
                        "exportsHeader": "sample.h",
                        "glue": ("shim.py", "support.py"),
                        "library": "sample.capi.wasm",
                        "metadata": "sample.cabi.json",
                    },
                ),
            )
            self.assertEqual(ensure_compatible_binding_package_manifest(manifest), manifest)

            binding = KaliCAPI.from_binding_package(DummyLibrary(), root)
            self.assertEqual(binding.exports, tuple(exports))
            self.assertEqual(binding.max_specializations, 8)
            self.assertEqual(binding.add(2, 3), 5)
            self.assertEqual(binding.zero(), 7)
            self.assertEqual(binding._library.calls, [("add", 2, 3), ("zero",)])

    def test_binding_package_with_stem_specific_manifest_is_auto_discovered(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            header_path = root / "sample.h"
            metadata_path = root / "sample.cabi.json"
            manifest_path = root / "sample.binding-package.json"

            header_path.write_text(
                "\n".join(
                    [
                        "#include <stdint.h>",
                        "extern int32_t add(int32_t arg0, int32_t arg1);",
                        "extern int32_t zero(void);",
                    ]
                )
                + "\n"
            )
            metadata_path.write_text(
                json.dumps(
                    {
                        "schemaVersion": 1,
                        "kind": "cabi-metadata",
                        "hostAbiVersion": 2,
                        "minHostAbiVersion": 2,
                        "artifacts": {
                            "exportsHeader": "sample.h",
                            "metadata": "sample.cabi.json",
                            "wasmModule": "sample.capi.wasm",
                            "wit": "sample.wit",
                        },
                    },
                    sort_keys=True,
                )
            )
            manifest_path.write_text(
                json.dumps(
                    {
                        "schemaVersion": 1,
                        "kind": "binding-package",
                        "moduleName": "sample",
                        "hostAbiVersion": 2,
                        "minHostAbiVersion": 2,
                        "maxSpecializations": 8,
                        "artifacts": {
                            "exportsHeader": "sample.h",
                            "glue": ["shim.py"],
                            "library": "sample.capi.wasm",
                            "metadata": "sample.cabi.json",
                        },
                    },
                    sort_keys=True,
                )
            )
            (root / "sample.capi.wasm").write_bytes(b"")

            manifest = load_binding_package_manifest_from_root(root)
            self.assertEqual(manifest.max_specializations, 8)
            self.assertEqual(manifest.artifacts["glue"], ("shim.py",))

            binding = KaliCAPI.from_binding_package(DummyLibrary(), root)
            self.assertEqual(binding.max_specializations, 8)
            self.assertEqual(binding.add(2, 3), 5)
            self.assertEqual(binding.zero(), 7)
            self.assertEqual(binding._library.calls, [("add", 2, 3), ("zero",)])

    def test_binding_package_manifest_discovery_rejects_ambiguity_and_accepts_explicit_names(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            alpha_manifest_path = root / "alpha.binding-package.json"
            beta_manifest_path = root / "beta.binding-package.json"

            alpha_manifest_path.write_text(
                json.dumps(
                    {
                        "schemaVersion": 1,
                        "kind": "binding-package",
                        "moduleName": "alpha",
                        "hostAbiVersion": 2,
                        "minHostAbiVersion": 2,
                        "maxSpecializations": 8,
                        "artifacts": {
                            "exportsHeader": "alpha.h",
                            "glue": ["alpha.py"],
                            "library": "alpha.capi.wasm",
                            "metadata": "alpha.cabi.json",
                        },
                    },
                    sort_keys=True,
                )
            )
            beta_manifest_path.write_text(
                json.dumps(
                    {
                        "schemaVersion": 1,
                        "kind": "binding-package",
                        "moduleName": "beta",
                        "hostAbiVersion": 2,
                        "minHostAbiVersion": 2,
                        "maxSpecializations": 8,
                        "artifacts": {
                            "exportsHeader": "beta.h",
                            "glue": ["beta.py"],
                            "library": "beta.capi.wasm",
                            "metadata": "beta.cabi.json",
                        },
                    },
                    sort_keys=True,
                )
            )

            with self.assertRaises(ValueError):
                discover_binding_package_manifest_path(root)

            self.assertEqual(
                discover_binding_package_manifest_path(root, "beta.binding-package.json"),
                beta_manifest_path,
            )

            manifest = load_binding_package_manifest_from_root(root, "alpha.binding-package.json")
            self.assertEqual(
                manifest,
                BindingPackageManifest(
                    schema_version=1,
                    kind="binding-package",
                    module_name="alpha",
                    host_abi_version=2,
                    min_host_abi_version=2,
                    max_specializations=8,
                    artifacts={
                        "exportsHeader": "alpha.h",
                        "glue": ("alpha.py",),
                        "library": "alpha.capi.wasm",
                        "metadata": "alpha.cabi.json",
                    },
                ),
            )

    def test_incompatible_binding_package_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            header_path = root / "sample.h"
            metadata_path = root / "sample.cabi.json"
            manifest_path = root / "binding-package.json"

            header_path.write_text(
                "#include <stdint.h>\nextern int32_t add(int32_t arg0, int32_t arg1);\n"
            )
            metadata_path.write_text(
                json.dumps(
                    {
                        "schemaVersion": 1,
                        "kind": "cabi-metadata",
                        "hostAbiVersion": 2,
                        "minHostAbiVersion": 2,
                        "artifacts": {
                            "exportsHeader": "sample.h",
                            "metadata": "sample.cabi.json",
                            "wasmModule": "sample.capi.wasm",
                            "wit": "sample.wit",
                        },
                    },
                    sort_keys=True,
                )
            )
            manifest_path.write_text(
                json.dumps(
                    {
                        "schemaVersion": 1,
                        "kind": "binding-package",
                        "moduleName": "sample",
                        "hostAbiVersion": 2,
                        "minHostAbiVersion": 2,
                        "maxSpecializations": 8,
                        "artifacts": {
                            "exportsHeader": "sample.h",
                            "glue": ["shim.py"],
                            "library": "sample.capi.wasm",
                            "metadata": "sample.cabi.json",
                        },
                    },
                    sort_keys=True,
                )
            )
            (root / "sample.capi.wasm").write_bytes(b"")

            manifest = load_binding_package_manifest(manifest_path)
            self.assertEqual(manifest.max_specializations, 8)
            with self.assertRaises(ValueError):
                ensure_compatible_binding_package_manifest(manifest, available_host_abi_version=3)

            with self.assertRaises(ValueError):
                KaliCAPI.from_binding_package(
                    DummyLibrary(),
                    root,
                    available_host_abi_version=3,
                )


if __name__ == "__main__":
    unittest.main()
