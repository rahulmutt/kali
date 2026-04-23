# kali_capi Node binding helper

This directory packages a maintained Node ESM helper for Kali's stable C ABI.
It provides deterministic helpers for generated C headers, `cabi-metadata`
sidecars, and `binding-package` manifests.

## Layout

- `kali_capi.mjs` — importable Node helper module
- `package.json` — deterministic packaging metadata
- `tests/` — Node smoke tests for the helper

## Usage

```js
import {
  discoverBindingPackageManifestPath,
  loadBindingPackageManifest,
  loadMetadata,
  parseExports,
} from './kali_capi.mjs';
```

The helper keeps the binding workflow small and deterministic so higher-level
wrappers can discover the generated artifact layout without reimplementing the
manifest and metadata parsing rules.

Run the smoke tests with:

```bash
node --test tests/test_kali_capi.mjs
```

or, from the package root, `npm test`.
