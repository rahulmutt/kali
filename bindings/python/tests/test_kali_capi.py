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
    binding_package_manifest_summary,
    cabi_metadata_summary,
    discover_binding_package_manifest_path,
    discover_metadata_path,
    discover_metadata_path_with_name,
    ensure_compatible_binding_package_manifest,
    ensure_compatible_metadata,
    load_binding_package_manifest,
    load_binding_package_manifest_from_root,
    load_binding_package_manifest_summary,
    load_binding_package_manifest_summary_from_root,
    load_metadata,
    load_metadata_from_root,
    load_metadata_from_root_with_name,
    load_metadata_summary,
    load_metadata_summary_from_root,
    load_metadata_summary_from_root_with_name,
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
                        "maxSpecializations": 8,
                        "runtimeProfiles": ["wasm-threads", "fiber-threads", "wasm-threads"],
                        "hostContract": "kali-hosted",
                        "runtimeBackend": "wasmtime",
                        "profileDataHash": "sha256:sample-profile",
                        "artifacts": {
                            "exportsHeader": "sample.h",
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
                        "runtimeProfiles": ["wasm-threads", "fiber-threads", "wasm-threads"],
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
            self.assertEqual(metadata.max_specializations, 8)
            self.assertEqual(metadata.runtime_profiles, ("fiber-threads", "wasm-threads"))
            self.assertEqual(metadata.host_contract, "kali-hosted")
            self.assertEqual(metadata.runtime_backend, "wasmtime")
            self.assertEqual(
                metadata.artifacts,
                {
                    "exportsHeader": "sample.h",
                    "wasmModule": "sample.capi.wasm",
                    "wit": "sample.wit",
                },
            )
            self.assertEqual(ensure_compatible_metadata(metadata), metadata)
            self.assertEqual(
                cabi_metadata_summary(metadata),
                {
                    "schemaVersion": 1,
                    "kind": "cabi-metadata",
                    "hostAbiVersion": 2,
                    "minHostAbiVersion": 2,
                    "runtimeProfiles": ["fiber-threads", "wasm-threads"],
                    "hostContract": "kali-hosted",
                    "runtimeBackend": "wasmtime",
                    "profileDataHash": "sha256:sample-profile",
                    "maxSpecializations": 8,
                    "artifacts": {
                        "exportsHeader": "sample.h",
                        "wasmModule": "sample.capi.wasm",
                        "wit": "sample.wit",
                    },
                },
            )
            self.assertEqual(load_metadata_summary(metadata_path), cabi_metadata_summary(metadata))

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
                    runtime_profiles=("fiber-threads", "wasm-threads"),
                    host_contract="kali-hosted",
                    runtime_backend="wasmtime",
                    artifacts={
                        "exportsHeader": "sample.h",
                        "glue": ("shim.py", "support.py"),
                        "library": "sample.capi.wasm",
                        "metadata": "sample.cabi.json",
                    },
                ),
            )
            self.assertEqual(manifest.runtime_profiles, ("fiber-threads", "wasm-threads"))
            self.assertEqual(manifest.host_contract, "kali-hosted")
            self.assertEqual(manifest.runtime_backend, "wasmtime")
            self.assertEqual(ensure_compatible_binding_package_manifest(manifest), manifest)
            self.assertEqual(
                binding_package_manifest_summary(manifest),
                {
                    "moduleName": "sample",
                    "hostAbiVersion": 2,
                    "minHostAbiVersion": 2,
                    "runtimeProfiles": ["fiber-threads", "wasm-threads"],
                    "hostContract": "kali-hosted",
                    "runtimeBackend": "wasmtime",
                    "maxSpecializations": 8,
                    "artifacts": {
                        "exportsHeader": "sample.h",
                        "glue": ["shim.py", "support.py"],
                        "library": "sample.capi.wasm",
                        "metadata": "sample.cabi.json",
                    },
                },
            )
            normalized_summary = binding_package_manifest_summary(
                BindingPackageManifest(
                    schema_version=1,
                    kind="binding-package",
                    module_name="sample",
                    host_abi_version=2,
                    min_host_abi_version=2,
                    max_specializations=8,
                    runtime_profiles=("wasm-threads", "fiber-threads", "wasm-threads"),
                    host_contract="kali-hosted",
                    runtime_backend="wasmtime",
                    artifacts={
                        "exportsHeader": "sample.h",
                        "glue": ("z.py", "a.py", "z.py"),
                        "library": "sample.capi.wasm",
                        "metadata": "sample.cabi.json",
                    },
                )
            )
            self.assertEqual(
                normalized_summary,
                {
                    "moduleName": "sample",
                    "hostAbiVersion": 2,
                    "minHostAbiVersion": 2,
                    "runtimeProfiles": ["fiber-threads", "wasm-threads"],
                    "hostContract": "kali-hosted",
                    "runtimeBackend": "wasmtime",
                    "maxSpecializations": 8,
                    "artifacts": {
                        "exportsHeader": "sample.h",
                        "glue": ["a.py", "z.py"],
                        "library": "sample.capi.wasm",
                        "metadata": "sample.cabi.json",
                    },
                },
            )
            self.assertEqual(
                load_binding_package_manifest_summary(manifest_path),
                binding_package_manifest_summary(manifest),
            )
            self.assertEqual(
                load_binding_package_manifest_summary_from_root(root),
                binding_package_manifest_summary(manifest),
            )

            binding = KaliCAPI.from_binding_package(DummyLibrary(), root)
            self.assertEqual(binding.exports, tuple(exports))
            self.assertEqual(binding.max_specializations, 8)
            self.assertEqual(binding.add(2, 3), 5)
            self.assertEqual(binding.zero(), 7)
            self.assertEqual(binding._library.calls, [("add", 2, 3), ("zero",)])

    def test_cabi_metadata_with_stem_specific_sidecar_is_auto_discovered(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            metadata_path = root / "sample.capi.meta.json"
            metadata_path.write_text(
                json.dumps(
                    {
                        "schemaVersion": 1,
                        "kind": "cabi-metadata",
                        "hostAbiVersion": 2,
                        "minHostAbiVersion": 2,
                        "maxSpecializations": 8,
                        "runtimeProfiles": ["wasm-threads", "fiber-threads", "wasm-threads"],
                        "hostContract": "kali-hosted",
                        "runtimeBackend": "wasmtime",
                        "profileDataHash": "sha256:sample-profile",
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
            (root / "noise.txt").write_text("ignore me")

            self.assertEqual(discover_metadata_path(root), metadata_path)
            self.assertEqual(
                discover_metadata_path_with_name(root, "sample.capi.meta.json"),
                metadata_path,
            )

            loaded = load_metadata_from_root(root)
            self.assertEqual(loaded.host_abi_version, 2)
            self.assertEqual(loaded.min_host_abi_version, 2)
            self.assertEqual(loaded.max_specializations, 8)
            self.assertEqual(loaded.runtime_profiles, ("fiber-threads", "wasm-threads"))
            self.assertEqual(loaded.host_contract, "kali-hosted")
            self.assertEqual(loaded.runtime_backend, "wasmtime")
            self.assertEqual(loaded.profile_data_hash, "sha256:sample-profile")
            self.assertEqual(
                loaded.artifacts,
                {
                    "exportsHeader": "sample.h",
                    "wasmModule": "sample.capi.wasm",
                    "wit": "sample.wit",
                },
            )
            self.assertEqual(load_metadata_from_root_with_name(root, "sample.capi.meta.json"), loaded)
            self.assertEqual(load_metadata_summary_from_root(root), cabi_metadata_summary(loaded))
            self.assertEqual(
                load_metadata_summary_from_root_with_name(root, "sample.capi.meta.json"),
                cabi_metadata_summary(loaded),
            )

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
            self.assertEqual(manifest.runtime_profiles, ())
            self.assertEqual(manifest.host_contract, "kali-hosted")
            self.assertEqual(manifest.runtime_backend, "wasmtime")
            self.assertEqual(manifest.artifacts["glue"], ("shim.py",))

            binding = KaliCAPI.from_binding_package(DummyLibrary(), root)
            self.assertEqual(binding.max_specializations, 8)
            self.assertEqual(binding.runtime_profiles, ())
            self.assertEqual(binding.host_contract, "kali-hosted")
            self.assertEqual(binding.runtime_backend, "wasmtime")
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
                    runtime_profiles=(),
                    host_contract="kali-hosted",
                    runtime_backend="wasmtime",
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

            header_text = "#include <stdint.h>\nextern int32_t add(int32_t arg0, int32_t arg1);\n"
            header_path.write_text(header_text)
            metadata_payload = {
                "schemaVersion": 1,
                "kind": "cabi-metadata",
                "hostAbiVersion": 2,
                "maxSpecializations": 8,
                "runtimeProfiles": ["wasm-threads", "fiber-threads", "wasm-threads"],
                "hostContract": "browser-requested",
                "runtimeBackend": "browser-harness",
                "minHostAbiVersion": 2,
                "artifacts": {
                    "exportsHeader": "sample.h",
                    "metadata": "sample.cabi.json",
                    "wasmModule": "sample.capi.wasm",
                    "wit": "sample.wit",
                },
            }
            metadata_path.write_text(
                json.dumps(
                    metadata_payload,
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
            self.assertEqual(manifest.runtime_profiles, ())
            self.assertEqual(manifest.host_contract, "kali-hosted")
            self.assertEqual(manifest.runtime_backend, "wasmtime")
            with self.assertRaises(ValueError):
                ensure_compatible_binding_package_manifest(manifest, available_host_abi_version=3)

            binding = KaliCAPI.from_header_and_metadata(
                DummyLibrary(),
                header_text,
                json.dumps(metadata_payload),
            )
            self.assertEqual(binding.max_specializations, 8)
            self.assertEqual(binding.runtime_profiles, ("fiber-threads", "wasm-threads"))
            self.assertEqual(binding.host_contract, "browser-requested")
            self.assertEqual(binding.runtime_backend, "browser-harness")
            self.assertEqual(binding.add(4, 5), 9)

            with self.assertRaises(ValueError):
                KaliCAPI.from_binding_package(
                    DummyLibrary(),
                    root,
                    available_host_abi_version=3,
                )


if __name__ == "__main__":
    unittest.main()
