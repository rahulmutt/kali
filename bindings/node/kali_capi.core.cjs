const { readFileSync, existsSync, readdirSync } = require('node:fs');
const { join } = require('node:path');

const HOST_ABI_VERSION = 2;

const EXPORT_RE = /^extern\s+int32_t\s+(?<name>[A-Za-z_][A-Za-z0-9_]*)\((?<params>[^)]*)\);$/gm;

function isInt(value) {
  return Number.isInteger(value);
}

function requireInt(payload, key, context) {
  const value = payload[key];
  if (!isInt(value)) {
    throw new Error(`${context} field '${key}' must be an integer`);
  }
  return value;
}

function requireStr(payload, key, context) {
  const value = payload[key];
  if (typeof value !== 'string') {
    throw new Error(`${context} field '${key}' must be a string`);
  }
  return value;
}

function requireArtifacts(payload, context, requiredKeys) {
  if (payload === null || typeof payload !== 'object' || Array.isArray(payload)) {
    throw new Error(`${context} field 'artifacts' must be a JSON object`);
  }

  const artifacts = {};
  for (const key of requiredKeys) {
    const value = payload[key];
    if (typeof value !== 'string') {
      throw new Error(`${context} field 'artifacts.${key}' must be a string`);
    }
    artifacts[key] = value;
  }

  return Object.fromEntries(Object.entries(artifacts).sort(([left], [right]) => left.localeCompare(right)));
}

function requireStringList(payload, context, fieldName) {
  if (!Array.isArray(payload)) {
    throw new Error(`${context} field '${fieldName}' must be an array of strings`);
  }

  const items = [];
  for (const item of payload) {
    if (typeof item !== 'string') {
      throw new Error(`${context} field '${fieldName}' entries must be strings`);
    }
    items.push(item);
  }

  return Object.freeze(items.sort());
}

function parseExports(headerText) {
  const exports = [];
  for (const match of headerText.matchAll(EXPORT_RE)) {
    const params = match.groups.params.trim();
    const arity = !params || params === 'void'
      ? 0
      : params.split(',').filter((param) => param.trim().length > 0).length;
    exports.push({ name: match.groups.name, arity });
  }
  return exports;
}

function parseMetadata(metadataText) {
  const payload = JSON.parse(metadataText);
  if (payload === null || typeof payload !== 'object' || Array.isArray(payload)) {
    throw new Error('cabi metadata must be a JSON object');
  }

  const schemaVersion = requireInt(payload, 'schemaVersion', 'cabi metadata');
  if (schemaVersion !== 1) {
    throw new Error(`unsupported cabi metadata schemaVersion ${schemaVersion}`);
  }

  const kind = requireStr(payload, 'kind', 'cabi metadata');
  if (kind !== 'cabi-metadata') {
    throw new Error(`unsupported cabi metadata kind ${kind}`);
  }

  const hostAbiVersion = requireInt(payload, 'hostAbiVersion', 'cabi metadata');
  const minHostAbiVersion = Object.prototype.hasOwnProperty.call(payload, 'minHostAbiVersion')
    ? payload.minHostAbiVersion
    : hostAbiVersion;
  if (!isInt(minHostAbiVersion)) {
    throw new Error("cabi metadata field 'minHostAbiVersion' must be an integer");
  }

  const artifacts = requireArtifacts(payload.artifacts, 'cabi metadata', [
    'wasmModule',
    'wit',
    'exportsHeader',
  ]);

  return Object.freeze({
    schemaVersion,
    kind,
    hostAbiVersion,
    minHostAbiVersion,
    artifacts,
  });
}

function parseBindingPackageManifest(manifestText) {
  const payload = JSON.parse(manifestText);
  if (payload === null || typeof payload !== 'object' || Array.isArray(payload)) {
    throw new Error('binding package manifest must be a JSON object');
  }

  const schemaVersion = requireInt(payload, 'schemaVersion', 'binding package');
  if (schemaVersion !== 1) {
    throw new Error(`unsupported binding package schemaVersion ${schemaVersion}`);
  }

  const kind = requireStr(payload, 'kind', 'binding package');
  if (kind !== 'binding-package') {
    throw new Error(`unsupported binding package kind ${kind}`);
  }

  const moduleName = requireStr(payload, 'moduleName', 'binding package');
  const hostAbiVersion = requireInt(payload, 'hostAbiVersion', 'binding package');
  const minHostAbiVersion = Object.prototype.hasOwnProperty.call(payload, 'minHostAbiVersion')
    ? payload.minHostAbiVersion
    : hostAbiVersion;
  if (!isInt(minHostAbiVersion)) {
    throw new Error("binding package field 'minHostAbiVersion' must be an integer");
  }

  const maxSpecializations = Object.prototype.hasOwnProperty.call(payload, 'maxSpecializations')
    ? requireInt(payload, 'maxSpecializations', 'binding package')
    : undefined;

  if (payload.artifacts === null || typeof payload.artifacts !== 'object' || Array.isArray(payload.artifacts)) {
    throw new Error("binding package field 'artifacts' must be a JSON object");
  }

  const library = requireStr(payload.artifacts, 'library', 'binding package');
  const metadata = requireStr(payload.artifacts, 'metadata', 'binding package');
  const exportsHeader = requireStr(payload.artifacts, 'exportsHeader', 'binding package');
  const glue = requireStringList(payload.artifacts.glue, 'binding package', 'glue');

  const manifest = {
    schemaVersion,
    kind,
    moduleName,
    hostAbiVersion,
    minHostAbiVersion,
    artifacts: Object.freeze({
      exportsHeader,
      glue,
      library,
      metadata,
    }),
  };

  if (maxSpecializations !== undefined) {
    manifest.maxSpecializations = maxSpecializations;
  }

  return Object.freeze(manifest);
}

function loadMetadata(path) {
  return parseMetadata(readFileSync(path, 'utf8'));
}

function loadBindingPackageManifest(path) {
  return parseBindingPackageManifest(readFileSync(path, 'utf8'));
}

function discoverBindingPackageManifestPath(bundleRoot, manifestName = 'binding-package.json') {
  const explicitPath = join(bundleRoot, manifestName);
  if (existsSync(explicitPath)) {
    return explicitPath;
  }

  if (manifestName !== 'binding-package.json') {
    throw Object.assign(new Error(explicitPath), { code: 'ENOENT', path: explicitPath });
  }

  const candidates = readdirSync(bundleRoot)
    .filter((entry) => entry.endsWith('.binding-package.json'))
    .sort((left, right) => left.localeCompare(right));

  if (candidates.length === 0) {
    throw Object.assign(new Error(explicitPath), { code: 'ENOENT', path: explicitPath });
  }

  if (candidates.length > 1) {
    throw new Error('binding package manifest is ambiguous; pass manifestName explicitly');
  }

  return join(bundleRoot, candidates[0]);
}

function ensureCompatibleMetadata(metadata, availableHostAbiVersion = HOST_ABI_VERSION) {
  if (!isInt(availableHostAbiVersion)) {
    throw new Error('availableHostAbiVersion must be an integer');
  }

  if (availableHostAbiVersion < metadata.minHostAbiVersion) {
    throw new Error(
      `incompatible host ABI version ${availableHostAbiVersion}; expected at least ${metadata.minHostAbiVersion}`,
    );
  }

  return metadata;
}

function loadBindingPackageManifestFromRoot(bundleRoot, manifestName = 'binding-package.json') {
  return loadBindingPackageManifest(discoverBindingPackageManifestPath(bundleRoot, manifestName));
}

function ensureCompatibleBindingPackageManifest(
  manifest,
  availableHostAbiVersion = HOST_ABI_VERSION,
) {
  if (!isInt(availableHostAbiVersion)) {
    throw new Error('availableHostAbiVersion must be an integer');
  }

  if (availableHostAbiVersion < manifest.minHostAbiVersion) {
    throw new Error(
      `incompatible host ABI version ${availableHostAbiVersion}; expected at least ${manifest.minHostAbiVersion}`,
    );
  }

  return manifest;
}

class KaliCAPI {
  constructor(library, exports, maxSpecializations = null) {
    this._library = library;
    this._exports = Object.freeze([...exports]);
    this._maxSpecializations = maxSpecializations;
    this._bindExports();
  }

  static fromHeader(library, headerText) {
    return new KaliCAPI(library, parseExports(headerText));
  }

  static fromHeaderAndMetadata(
    library,
    headerText,
    metadataText,
    { availableHostAbiVersion = HOST_ABI_VERSION } = {},
  ) {
    ensureCompatibleMetadata(
      parseMetadata(metadataText),
      availableHostAbiVersion,
    );
    return new KaliCAPI(library, parseExports(headerText));
  }

  static fromBindingPackage(
    library,
    bundleRoot,
    {
      manifestName = 'binding-package.json',
      availableHostAbiVersion = HOST_ABI_VERSION,
    } = {},
  ) {
    const resolvedBundleRoot = typeof bundleRoot === 'string' ? bundleRoot : bundleRoot?.toString();
    if (typeof resolvedBundleRoot !== 'string' || resolvedBundleRoot.length === 0) {
      throw new Error('bundleRoot must be a non-empty path string');
    }
    const manifestPath = discoverBindingPackageManifestPath(resolvedBundleRoot, manifestName);
    const manifest = ensureCompatibleBindingPackageManifest(
      loadBindingPackageManifest(manifestPath),
      availableHostAbiVersion,
    );
    const headerText = readFileSync(join(resolvedBundleRoot, manifest.artifacts.exportsHeader), 'utf8');
    const metadataText = readFileSync(join(resolvedBundleRoot, manifest.artifacts.metadata), 'utf8');
    ensureCompatibleMetadata(
      parseMetadata(metadataText),
      availableHostAbiVersion,
    );
    return new KaliCAPI(
      library,
      parseExports(headerText),
      manifest.maxSpecializations ?? null,
    );
  }

  get exports() {
    return this._exports;
  }

  get maxSpecializations() {
    return this._maxSpecializations;
  }

  _bindExports() {
    for (const exportEntry of this._exports) {
      const functionValue = this._library[exportEntry.name];
      if (typeof functionValue !== 'function') {
        throw new Error(`library is missing export ${exportEntry.name}`);
      }
      this[exportEntry.name] = functionValue.bind(this._library);
    }
  }
}

module.exports = {
  HOST_ABI_VERSION,
  KaliCAPI,
  discoverBindingPackageManifestPath,
  ensureCompatibleBindingPackageManifest,
  ensureCompatibleMetadata,
  loadBindingPackageManifest,
  loadBindingPackageManifestFromRoot,
  loadMetadata,
  parseBindingPackageManifest,
  parseExports,
  parseMetadata,
};
