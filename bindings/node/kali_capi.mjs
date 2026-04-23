import core from './kali_capi.core.cjs';

export const HOST_ABI_VERSION = core.HOST_ABI_VERSION;
export const KaliCAPI = core.KaliCAPI;
export const bindingPackageManifestSummary = core.bindingPackageManifestSummary;
export const discoverBindingPackageManifestPath = core.discoverBindingPackageManifestPath;
export const ensureCompatibleBindingPackageManifest = core.ensureCompatibleBindingPackageManifest;
export const ensureCompatibleMetadata = core.ensureCompatibleMetadata;
export const loadBindingPackageManifest = core.loadBindingPackageManifest;
export const loadBindingPackageManifestFromRoot = core.loadBindingPackageManifestFromRoot;
export const loadMetadata = core.loadMetadata;
export const loadMetadataSummary = core.loadMetadataSummary;
export const parseBindingPackageManifest = core.parseBindingPackageManifest;
export const cabiMetadataSummary = core.cabiMetadataSummary;
export const loadBindingPackageManifestSummary = core.loadBindingPackageManifestSummary;
export const loadBindingPackageManifestSummaryFromRoot = core.loadBindingPackageManifestSummaryFromRoot;
export const parseExports = core.parseExports;
export const parseMetadata = core.parseMetadata;

export default core;
