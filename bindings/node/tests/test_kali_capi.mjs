import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

import {
  HOST_ABI_VERSION,
  discoverBindingPackageManifestPath,
  ensureCompatibleBindingPackageManifest,
  ensureCompatibleMetadata,
  loadBindingPackageManifest,
  loadMetadata,
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
    artifacts: {
      exportsHeader: 'sample.h',
      wasmModule: 'sample.capi.wasm',
      wit: 'sample.wit',
    },
  });
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
      artifacts: {
        exportsHeader: 'sample.h',
        wasmModule: 'sample.capi.wasm',
        wit: 'sample.wit',
      },
    }),
  );

  const resolvedManifestPath = discoverBindingPackageManifestPath(tempRoot);
  assert.equal(resolvedManifestPath, manifestPath);

  const manifest = loadBindingPackageManifest(resolvedManifestPath);
  const metadata = loadMetadata(metadataPath);

  assert.deepEqual(manifest, {
    schemaVersion: 1,
    kind: 'binding-package',
    moduleName: 'sample',
    hostAbiVersion: HOST_ABI_VERSION,
    minHostAbiVersion: HOST_ABI_VERSION,
    artifacts: {
      exportsHeader: 'sample.h',
      glue: ['a.js', 'z.js'],
      library: 'sample.capi.wasm',
      metadata: 'sample.cabi.json',
    },
  });
  assert.deepEqual(ensureCompatibleBindingPackageManifest(manifest), manifest);
  assert.deepEqual(ensureCompatibleMetadata(metadata), metadata);

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

test('node binding helper module is importable from the package root', () => {
  const packageJson = JSON.parse(
    readFileSync(new URL('../package.json', import.meta.url), 'utf8'),
  );
  assert.equal(packageJson.type, 'module');
  assert.equal(packageJson.exports, './kali_capi.mjs');
});
