import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createRequire } from 'node:module';
import { test } from 'node:test';

const require = createRequire(import.meta.url);

import {
  HOST_ABI_VERSION,
  KaliCAPI,
  bindingPackageManifestSummary,
  cabiMetadataSummary,
  discoverBindingPackageManifestPath,
  discoverBindingPackageManifestPathWithName,
  discoverMetadataPath,
  discoverMetadataPathWithName,
  ensureCompatibleBindingPackageManifest,
  ensureCompatibleMetadata,
  loadBindingPackageManifestFromRoot,
  loadBindingPackageManifestFromRootWithName,
  loadBindingPackageManifestSummary,
  loadBindingPackageManifestSummaryFromRoot,
  loadBindingPackageManifestSummaryFromRootWithName,
  loadMetadata,
  loadMetadataFromRoot,
  loadMetadataFromRootWithName,
  loadMetadataSummary,
  loadMetadataSummaryFromRoot,
  loadMetadataSummaryFromRootWithName,
  parseBindingPackageManifest,
  parseExports,
  parseMetadata,
} from '../kali_capi.mjs';

test('parses generated exports and cabi metadata deterministically', () => {
  const header = [
    '#ifndef KALI_CAPI_GENERATED_H',
    '#define KALI_CAPI_GENERATED_H',
    '#include <stdint.h>',
    'extern int32_t add(int32_t arg0, int32_t arg1);',
    'extern int32_t zero(void);',
    '#endif',
  ].join('\n');
  const metadata = JSON.stringify({
    schemaVersion: 1,
    kind: 'cabi-metadata',
    hostAbiVersion: HOST_ABI_VERSION,
    maxSpecializations: 8,
    runtimeProfiles: ['wasm-threads', 'fiber-threads', 'wasm-threads'],
    hostContract: 'kali-hosted',
    runtimeBackend: 'wasmtime',
    profileDataHash: 'sha256:sample-profile',
    artifacts: {
      exportsHeader: 'sample.h',
      wasmModule: 'sample.capi.wasm',
      wit: 'sample.wit',
    },
  });

  assert.deepEqual(parseExports(header), [
    { name: 'add', arity: 2 },
    { name: 'zero', arity: 0 },
  ]);

  assert.deepEqual(parseMetadata(metadata), {
    schemaVersion: 1,
    kind: 'cabi-metadata',
    hostAbiVersion: HOST_ABI_VERSION,
    minHostAbiVersion: HOST_ABI_VERSION,
    maxSpecializations: 8,
    runtimeProfiles: ['fiber-threads', 'wasm-threads'],
    hostContract: 'kali-hosted',
    runtimeBackend: 'wasmtime',
    profileDataHash: 'sha256:sample-profile',
    artifacts: {
      exportsHeader: 'sample.h',
      wasmModule: 'sample.capi.wasm',
      wit: 'sample.wit',
    },
  });
  assert.deepEqual(cabiMetadataSummary(parseMetadata(metadata)), {
    schemaVersion: 1,
    kind: 'cabi-metadata',
    hostAbiVersion: HOST_ABI_VERSION,
    minHostAbiVersion: HOST_ABI_VERSION,
    maxSpecializations: 8,
    runtimeProfiles: ['fiber-threads', 'wasm-threads'],
    hostContract: 'kali-hosted',
    runtimeBackend: 'wasmtime',
    profileDataHash: 'sha256:sample-profile',
    artifacts: {
      exportsHeader: 'sample.h',
      wasmModule: 'sample.capi.wasm',
      wit: 'sample.wit',
    },
  });
});

test('cabi metadata helpers sort sidecars and auto-discover single manifests', () => {
  const tempRoot = mkdtempSync(join(tmpdir(), 'kali-capi-node-meta-'));
  const metadataPath = join(tempRoot, 'sample.capi.meta.json');

  writeFileSync(
    metadataPath,
    JSON.stringify({
      schemaVersion: 1,
      kind: 'cabi-metadata',
      hostAbiVersion: HOST_ABI_VERSION,
      maxSpecializations: 8,
      runtimeProfiles: ['wasm-threads', 'fiber-threads', 'wasm-threads'],
      hostContract: 'kali-hosted',
      runtimeBackend: 'wasmtime',
      profileDataHash: 'sha256:sample-profile',
      artifacts: {
        exportsHeader: 'sample.h',
        metadata: 'sample.cabi.json',
        wasmModule: 'sample.capi.wasm',
        wit: 'sample.wit',
      },
    }),
  );
  writeFileSync(join(tempRoot, 'noise.txt'), 'ignore me');

  assert.equal(discoverMetadataPath(tempRoot), metadataPath);
  assert.equal(discoverMetadataPathWithName(tempRoot, 'sample.capi.meta.json'), metadataPath);

  const loaded = loadMetadataFromRoot(tempRoot);
  const summary = loadMetadataSummaryFromRoot(tempRoot);

  assert.deepEqual(loaded, {
    schemaVersion: 1,
    kind: 'cabi-metadata',
    hostAbiVersion: HOST_ABI_VERSION,
    minHostAbiVersion: HOST_ABI_VERSION,
    maxSpecializations: 8,
    runtimeProfiles: ['fiber-threads', 'wasm-threads'],
    hostContract: 'kali-hosted',
    runtimeBackend: 'wasmtime',
    profileDataHash: 'sha256:sample-profile',
    artifacts: {
      exportsHeader: 'sample.h',
      wasmModule: 'sample.capi.wasm',
      wit: 'sample.wit',
    },
  });
  assert.deepEqual(loadMetadataFromRootWithName(tempRoot, 'sample.capi.meta.json'), loaded);
  assert.deepEqual(summary, {
    schemaVersion: 1,
    kind: 'cabi-metadata',
    hostAbiVersion: HOST_ABI_VERSION,
    minHostAbiVersion: HOST_ABI_VERSION,
    maxSpecializations: 8,
    runtimeProfiles: ['fiber-threads', 'wasm-threads'],
    hostContract: 'kali-hosted',
    runtimeBackend: 'wasmtime',
    profileDataHash: 'sha256:sample-profile',
    artifacts: {
      exportsHeader: 'sample.h',
      wasmModule: 'sample.capi.wasm',
      wit: 'sample.wit',
    },
  });
  assert.deepEqual(loadMetadataSummaryFromRootWithName(tempRoot, 'sample.capi.meta.json'), summary);

  rmSync(tempRoot, { recursive: true, force: true });
});

test('binding package manifests sort glue paths and auto-discover single manifests', () => {
  const tempRoot = mkdtempSync(join(tmpdir(), 'kali-capi-node-'));
  const manifestPath = join(tempRoot, 'sample.binding-package.json');
  const metadataPath = join(tempRoot, 'sample.cabi.json');

  writeFileSync(
    manifestPath,
    JSON.stringify({
      schemaVersion: 1,
      kind: 'binding-package',
      moduleName: 'sample',
      hostAbiVersion: HOST_ABI_VERSION,
      maxSpecializations: 8,
      runtimeProfiles: ['wasm-threads', 'fiber-threads', 'wasm-threads'],
      artifacts: {
        glue: ['z.js', 'a.js'],
        library: 'sample.capi.wasm',
        metadata: 'sample.cabi.json',
        exportsHeader: 'sample.h',
      },
    }),
  );
  writeFileSync(
    metadataPath,
    JSON.stringify({
      schemaVersion: 1,
      kind: 'cabi-metadata',
      hostAbiVersion: HOST_ABI_VERSION,
      maxSpecializations: 8,
      runtimeProfiles: ['wasm-threads', 'fiber-threads', 'wasm-threads'],
      hostContract: 'kali-hosted',
      runtimeBackend: 'wasmtime',
      profileDataHash: 'sha256:sample-profile',
      artifacts: {
        exportsHeader: 'sample.h',
        wasmModule: 'sample.capi.wasm',
        wit: 'sample.wit',
      },
    }),
  );

  const resolvedManifestPath = discoverBindingPackageManifestPath(tempRoot);
  assert.equal(resolvedManifestPath, manifestPath);
  assert.equal(
    discoverBindingPackageManifestPathWithName(tempRoot, 'sample.binding-package.json'),
    manifestPath,
  );

  const manifest = loadBindingPackageManifestFromRoot(tempRoot);
  const metadata = loadMetadata(metadataPath);
  const metadataSummary = loadMetadataSummary(metadataPath);
  const summary = bindingPackageManifestSummary(manifest);

  assert.deepEqual(manifest, {
    schemaVersion: 1,
    kind: 'binding-package',
    moduleName: 'sample',
    hostAbiVersion: HOST_ABI_VERSION,
    minHostAbiVersion: HOST_ABI_VERSION,
    maxSpecializations: 8,
    runtimeProfiles: ['fiber-threads', 'wasm-threads'],
    hostContract: 'kali-hosted',
    runtimeBackend: 'wasmtime',
    artifacts: {
      exportsHeader: 'sample.h',
      glue: ['a.js', 'z.js'],
      library: 'sample.capi.wasm',
      metadata: 'sample.cabi.json',
    },
  });
  assert.deepEqual(metadataSummary, {
    schemaVersion: 1,
    kind: 'cabi-metadata',
    hostAbiVersion: HOST_ABI_VERSION,
    minHostAbiVersion: HOST_ABI_VERSION,
    maxSpecializations: 8,
    runtimeProfiles: ['fiber-threads', 'wasm-threads'],
    hostContract: 'kali-hosted',
    runtimeBackend: 'wasmtime',
    profileDataHash: 'sha256:sample-profile',
    artifacts: {
      exportsHeader: 'sample.h',
      wasmModule: 'sample.capi.wasm',
      wit: 'sample.wit',
    },
  });
  assert.deepEqual(summary, {
    moduleName: 'sample',
    hostAbiVersion: HOST_ABI_VERSION,
    minHostAbiVersion: HOST_ABI_VERSION,
    runtimeProfiles: ['fiber-threads', 'wasm-threads'],
    hostContract: 'kali-hosted',
    runtimeBackend: 'wasmtime',
    maxSpecializations: 8,
    artifacts: {
      exportsHeader: 'sample.h',
      glue: ['a.js', 'z.js'],
      library: 'sample.capi.wasm',
      metadata: 'sample.cabi.json',
    },
  });

  const normalizedSummary = bindingPackageManifestSummary({
    schemaVersion: 1,
    kind: 'binding-package',
    moduleName: 'sample',
    hostAbiVersion: HOST_ABI_VERSION,
    minHostAbiVersion: HOST_ABI_VERSION,
    maxSpecializations: 8,
    runtimeProfiles: ['wasm-threads', 'fiber-threads', 'wasm-threads'],
    hostContract: 'kali-hosted',
    runtimeBackend: 'wasmtime',
    artifacts: {
      exportsHeader: 'sample.h',
      glue: ['z.js', 'a.js', 'z.js'],
      library: 'sample.capi.wasm',
      metadata: 'sample.cabi.json',
    },
  });
  assert.deepEqual(normalizedSummary, {
    moduleName: 'sample',
    hostAbiVersion: HOST_ABI_VERSION,
    minHostAbiVersion: HOST_ABI_VERSION,
    runtimeProfiles: ['fiber-threads', 'wasm-threads'],
    hostContract: 'kali-hosted',
    runtimeBackend: 'wasmtime',
    maxSpecializations: 8,
    artifacts: {
      exportsHeader: 'sample.h',
      glue: ['a.js', 'z.js'],
      library: 'sample.capi.wasm',
      metadata: 'sample.cabi.json',
    },
  });
  assert.deepEqual(loadBindingPackageManifestSummary(manifestPath), summary);
  assert.deepEqual(loadBindingPackageManifestSummaryFromRoot(tempRoot), summary);
  assert.deepEqual(
    loadBindingPackageManifestFromRootWithName(tempRoot, 'sample.binding-package.json'),
    manifest,
  );
  assert.deepEqual(
    loadBindingPackageManifestSummaryFromRootWithName(tempRoot, 'sample.binding-package.json'),
    summary,
  );
  assert.deepEqual(ensureCompatibleBindingPackageManifest(manifest), manifest);
  assert.deepEqual(ensureCompatibleMetadata(metadata), metadata);

  rmSync(tempRoot, { recursive: true, force: true });
});

test('binding package manifests reject ambiguous auto-discovery and honor explicit manifest names', () => {
  const tempRoot = mkdtempSync(join(tmpdir(), 'kali-capi-node-'));
  const alphaManifestPath = join(tempRoot, 'alpha.binding-package.json');
  const betaManifestPath = join(tempRoot, 'beta.binding-package.json');

  writeFileSync(
    alphaManifestPath,
    JSON.stringify({
      schemaVersion: 1,
      kind: 'binding-package',
      moduleName: 'alpha',
      hostAbiVersion: HOST_ABI_VERSION,
      maxSpecializations: 8,
      artifacts: {
        glue: ['alpha.js'],
        library: 'alpha.capi.wasm',
        metadata: 'alpha.cabi.json',
        exportsHeader: 'alpha.h',
      },
    }),
  );
  writeFileSync(
    betaManifestPath,
    JSON.stringify({
      schemaVersion: 1,
      kind: 'binding-package',
      moduleName: 'beta',
      hostAbiVersion: HOST_ABI_VERSION,
      maxSpecializations: 8,
      artifacts: {
        glue: ['beta.js'],
        library: 'beta.capi.wasm',
        metadata: 'beta.cabi.json',
        exportsHeader: 'beta.h',
      },
    }),
  );

  assert.throws(() => discoverBindingPackageManifestPath(tempRoot), /ambiguous/);
  assert.equal(
    discoverBindingPackageManifestPath(tempRoot, 'beta.binding-package.json'),
    betaManifestPath,
  );

  const manifest = loadBindingPackageManifestFromRoot(tempRoot, 'alpha.binding-package.json');
  assert.deepEqual(manifest, {
    schemaVersion: 1,
    kind: 'binding-package',
    moduleName: 'alpha',
    hostAbiVersion: HOST_ABI_VERSION,
    minHostAbiVersion: HOST_ABI_VERSION,
    maxSpecializations: 8,
    runtimeProfiles: [],
    hostContract: 'kali-hosted',
    runtimeBackend: 'wasmtime',
    artifacts: {
      exportsHeader: 'alpha.h',
      glue: ['alpha.js'],
      library: 'alpha.capi.wasm',
      metadata: 'alpha.cabi.json',
    },
  });

  rmSync(tempRoot, { recursive: true, force: true });
});

test('binding package helpers reject incompatible host ABI metadata', () => {
  const metadata = parseMetadata(
    JSON.stringify({
      schemaVersion: 1,
      kind: 'cabi-metadata',
      hostAbiVersion: HOST_ABI_VERSION,
      minHostAbiVersion: HOST_ABI_VERSION + 1,
      artifacts: {
        exportsHeader: 'sample.h',
        wasmModule: 'sample.capi.wasm',
        wit: 'sample.wit',
      },
    }),
  );

  assert.throws(() => ensureCompatibleMetadata(metadata), /incompatible/);

  const manifest = parseBindingPackageManifest(
    JSON.stringify({
      schemaVersion: 1,
      kind: 'binding-package',
      moduleName: 'sample',
      hostAbiVersion: HOST_ABI_VERSION,
      minHostAbiVersion: HOST_ABI_VERSION + 1,
      maxSpecializations: 8,
      artifacts: {
        glue: ['support.js'],
        library: 'sample.capi.wasm',
        metadata: 'sample.cabi.json',
        exportsHeader: 'sample.h',
      },
    }),
  );

  assert.throws(() => ensureCompatibleBindingPackageManifest(manifest), /incompatible/);
});

test('node binding helper module binds exports from headers and manifests', () => {
  const tempRoot = mkdtempSync(join(tmpdir(), 'kali-capi-node-bind-'));
  const manifestPath = join(tempRoot, 'binding-package.json');
  const namedManifestPath = join(tempRoot, 'sample.binding-package.json');
  const headerPath = join(tempRoot, 'sample.h');
  const metadataPath = join(tempRoot, 'sample.cabi.json');

  const manifestPayload = JSON.stringify({
    schemaVersion: 1,
    kind: 'binding-package',
    moduleName: 'sample',
    hostAbiVersion: HOST_ABI_VERSION,
    maxSpecializations: 8,
    artifacts: {
      glue: ['support.js'],
      library: 'sample.capi.wasm',
      metadata: 'sample.cabi.json',
      exportsHeader: 'sample.h',
    },
  });

  writeFileSync(manifestPath, manifestPayload);
  writeFileSync(namedManifestPath, manifestPayload);
  writeFileSync(
    headerPath,
    [
      '#ifndef KALI_CAPI_GENERATED_H',
      '#define KALI_CAPI_GENERATED_H',
      '#include <stdint.h>',
      'extern int32_t add(int32_t arg0, int32_t arg1);',
      'extern int32_t zero(void);',
      '#endif',
    ].join('\n'),
  );
  writeFileSync(
    metadataPath,
    JSON.stringify({
      schemaVersion: 1,
      kind: 'cabi-metadata',
      hostAbiVersion: HOST_ABI_VERSION,
      artifacts: {
        exportsHeader: 'sample.h',
        wasmModule: 'sample.capi.wasm',
        wit: 'sample.wit',
      },
    }),
  );

  const library = {
    total: 0,
    add(left, right) {
      this.total += left + right;
      return this.total;
    },
    zero() {
      this.total += 1;
      return this.total;
    },
  };

  const binding = KaliCAPI.fromBindingPackage(library, tempRoot);
  const namedLibrary = {
    total: 0,
    add(left, right) {
      this.total += left + right;
      return this.total;
    },
    zero() {
      this.total += 1;
      return this.total;
    },
  };
  const namedBinding = KaliCAPI.fromBindingPackageWithName(
    namedLibrary,
    tempRoot,
    'sample.binding-package.json',
  );
  assert.deepEqual(binding.exports, [
    { name: 'add', arity: 2 },
    { name: 'zero', arity: 0 },
  ]);
  assert.equal(binding.maxSpecializations, 8);
  assert.deepEqual(binding.runtimeProfiles, []);
  assert.equal(binding.hostContract, 'kali-hosted');
  assert.equal(binding.runtimeBackend, 'wasmtime');
  assert.equal(binding.add(2, 3), 5);
  assert.equal(binding.zero(), 6);
  assert.equal(library.total, 6);
  assert.deepEqual(binding._exports, [
    { name: 'add', arity: 2 },
    { name: 'zero', arity: 0 },
  ]);
  assert.deepEqual(namedBinding.exports, binding.exports);
  assert.equal(namedBinding.maxSpecializations, binding.maxSpecializations);
  assert.deepEqual(namedBinding.runtimeProfiles, binding.runtimeProfiles);
  assert.equal(namedBinding.hostContract, binding.hostContract);
  assert.equal(namedBinding.runtimeBackend, binding.runtimeBackend);
  assert.equal(namedBinding.add(2, 3), 5);
  assert.equal(namedBinding.zero(), 6);

  rmSync(tempRoot, { recursive: true, force: true });
});

test('node binding helper module is importable from the package root', () => {
  const packageJson = JSON.parse(
    readFileSync(new URL('../package.json', import.meta.url), 'utf8'),
  );
  assert.equal(packageJson.type, 'module');
  assert.deepEqual(packageJson.exports, {
    '.': {
      import: './kali_capi.mjs',
      require: './kali_capi.cjs',
    },
    './package.json': './package.json',
  });
});

test('node binding helper module is requireable from the package root', () => {
  const nodeBinding = require('..');
  assert.equal(nodeBinding.HOST_ABI_VERSION, 2);
  assert.equal(typeof nodeBinding.KaliCAPI.fromBindingPackage, 'function');
  assert.equal(typeof nodeBinding.KaliCAPI.fromBindingPackageWithName, 'function');
  assert.equal(typeof nodeBinding.parseExports, 'function');
  assert.equal(typeof nodeBinding.bindingPackageManifestSummary, 'function');
  assert.equal(typeof nodeBinding.loadBindingPackageManifestFromRootWithName, 'function');
  assert.equal(typeof nodeBinding.loadBindingPackageManifestSummaryFromRootWithName, 'function');
});

test('node binding helper module is requireable from the explicit CommonJS entrypoint', () => {
  const nodeBinding = require('../kali_capi.cjs');
  assert.equal(nodeBinding.HOST_ABI_VERSION, 2);
  assert.equal(typeof nodeBinding.KaliCAPI.fromBindingPackage, 'function');
  assert.equal(typeof nodeBinding.KaliCAPI.fromBindingPackageWithName, 'function');
  assert.equal(typeof nodeBinding.loadBindingPackageManifestFromRoot, 'function');
  assert.equal(typeof nodeBinding.loadBindingPackageManifestFromRootWithName, 'function');
  assert.equal(typeof nodeBinding.bindingPackageManifestSummary, 'function');
});

test('node header-and-metadata bindings preserve metadata provenance', () => {
  const library = {
    total: 0,
    add(left, right) {
      this.total += left + right;
      return this.total;
    },
  };
  const binding = KaliCAPI.fromHeaderAndMetadata(
    library,
    [
      '#ifndef KALI_CAPI_GENERATED_H',
      '#define KALI_CAPI_GENERATED_H',
      '#include <stdint.h>',
      'extern int32_t add(int32_t arg0, int32_t arg1);',
      '#endif',
    ].join('\n'),
    JSON.stringify({
      schemaVersion: 1,
      kind: 'cabi-metadata',
      hostAbiVersion: HOST_ABI_VERSION,
      maxSpecializations: 12,
      runtimeProfiles: ['wasm-threads', 'fiber-threads', 'wasm-threads'],
      hostContract: 'browser-requested',
      runtimeBackend: 'browser-harness',
      artifacts: {
        exportsHeader: 'sample.h',
        wasmModule: 'sample.capi.wasm',
        wit: 'sample.wit',
      },
    }),
  );
  assert.equal(binding.maxSpecializations, 12);
  assert.deepEqual(binding.runtimeProfiles, ['fiber-threads', 'wasm-threads']);
  assert.equal(binding.hostContract, 'browser-requested');
  assert.equal(binding.runtimeBackend, 'browser-harness');
  assert.equal(binding.add(2, 3), 5);
  assert.equal(library.total, 5);
});
