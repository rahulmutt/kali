# kali_capi Node binding helper

This directory packages a maintained Node helper for Kali's stable C ABI.
It provides deterministic helpers for generated C headers, `cabi-metadata`
sidecars, `binding-package` manifests, and a small `KaliCAPI` wrapper for
binding those exports onto an existing library object.

The package supports both ESM `import` and CommonJS `require` entrypoints so
callers can use the same stable ABI helper from either module system.

## Layout

- `kali_capi.mjs` — ESM helper module
- `kali_capi.cjs` — CommonJS entrypoint
- `kali_capi.core.cjs` — shared implementation used by both module systems
- `package.json` — deterministic packaging metadata
- `tests/` — Node smoke tests for the helper

## Usage

### ESM

```js
import {
  KaliCAPI,
  discoverBindingPackageManifestPath,
  loadBindingPackageManifest,
  loadBindingPackageManifestFromRoot,
  loadMetadata,
  parseExports,
} from './kali_capi.mjs';

const binding = KaliCAPI.fromBindingPackage(library, './dist/binding-package');
console.log(binding.exports);
```

### CommonJS

```js
const {
  KaliCAPI,
  discoverBindingPackageManifestPath,
  loadBindingPackageManifest,
  loadBindingPackageManifestFromRoot,
  loadMetadata,
  parseExports,
} = require('./kali_capi.cjs');

const binding = KaliCAPI.fromBindingPackage(library, './dist/binding-package');
console.log(binding.exports);
```

The helper keeps the binding workflow small and deterministic so higher-level
wrappers can discover the generated artifact layout without reimplementing the
manifest and metadata parsing rules. `loadBindingPackageManifestFromRoot()`
and `KaliCAPI.fromBindingPackage()` both resolve the generated manifest from a
bundle root, validate the companion metadata, and bind the exported entrypoints
onto an existing library object. `discoverBindingPackageManifestPath()` and
`loadBindingPackageManifestFromRoot()` now also have explicit `...WithName()`
variants for callers that need to target one stem-specific bundle directly;
`discoverMetadataPath()` / `loadMetadataFromRoot()` and their `...WithName()`
variants provide the same bundle-root discovery path for the generated
`cabi-metadata` sidecar, while `bindingPackageManifestSummary()` projects the
normalized manifest into a compact summary object for callers that want one
stable provenance snapshot. `loadBindingPackageManifestSummary()` /
`loadBindingPackageManifestSummaryFromRoot()` and `loadMetadataSummary()` /
`loadMetadataSummaryFromRoot()` perform the same load-and-project step in one
call. `cabiMetadataSummary()` keeps the companion metadata projection
normalized, preserving the optional runtime provenance fields when the compiler
emits those values. The resulting `KaliCAPI` instance also carries the
manifest's `maxSpecializations` provenance plus the normalized runtime
provenance tuple (`runtimeProfiles`, `hostContract`, and `runtimeBackend`) when
the bundle publishes them, so higher-level callers can inspect the same
specialization and runtime context that the CLI emitted. The
`fromHeaderAndMetadata()` convenience path preserves that same provenance tuple
when callers only have a header plus `cabi-metadata` sidecar, keeping the direct
and manifest-backed helper flows aligned. Both module systems expose the same
helper surface, so binding consumers can choose ESM or CommonJS without changing
the artifact contract.

Run the smoke tests with:

```bash
node --test tests/test_kali_capi.mjs
```

or, from the package root, `npm test`.
