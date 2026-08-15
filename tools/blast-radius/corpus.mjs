// tools/blast-radius/corpus.mjs
//
// What `accepts.mjs` and `count.mjs` must both do before they measure anything:
// verify the freeze, and resolve the kali binary.
//
// Ruling 3 (verify the freeze at measurement time). Both tools read
// `manifest.json`. The Rust side enforces the freeze in tests, but the tools
// that produce the PUBLISHED numbers must enforce it themselves -- a number
// stamped with a corpus hash it never checked is a number a reader cannot
// audit. The checks here mirror `crates/kali_blast_radius/src/manifest.rs`:
// non-empty, unambiguously encodable, self-consistent with its own
// `corpus_hash`, every recorded file present with the recorded digest, and no
// untracked `.js` file under the corpus root.
//
// Ruling 1 (resolve the kali binary properly, and fail loudly). The plan
// hardcoded `../../target/debug/kali`, which does not exist here -- the cargo
// target directory is `.cache/cargo-target`. Combined with a `try/catch` that
// records a spawn failure as `accepted = false`, a wrong path would have marked
// every program unreachable and produced an all-zero reachable count that looks
// like data.

import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const ROOT = path.dirname(fileURLToPath(import.meta.url));
export const CORPUS = path.join(ROOT, "corpus");
export const REPO = path.resolve(ROOT, "..", "..");

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

/** The order-independent hash over the whole file list, as the Rust side computes it. */
export function corpusHashOf(files) {
  const lines = files.map((file) => `${file.stratum} ${file.path} ${file.sha256}`).sort();
  return sha256(Buffer.from(lines.join("\n"), "utf8"));
}

function checkToken(field, value, where) {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${where}: empty \`${field}\``);
  }
  for (const character of value) {
    if (/\s/u.test(character) || character.codePointAt(0) < 0x20 || character === "�") {
      throw new Error(
        `${where}: \`${field}\` \`${value}\` contains a character the corpus_hash encoding cannot separate`,
      );
    }
  }
}

/** Every `.js` file under `root`, as corpus-root-relative paths. */
function jsFilesUnder(root, prefix = "") {
  const out = [];
  for (const entry of fs.readdirSync(path.join(root, prefix), { withFileTypes: true })) {
    const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) out.push(...jsFilesUnder(root, relative));
    else if (entry.isFile() && entry.name.endsWith(".js")) out.push(relative);
  }
  return out;
}

/**
 * Read `manifest.json` and verify the freeze against the bytes on disk.
 *
 * Throws on any disagreement. A measurement taken over a corpus that does not
 * match its manifest is not a measurement of the frozen corpus.
 */
export function loadVerifiedManifest() {
  const manifestPath = path.join(CORPUS, "manifest.json");
  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));

  if (!Array.isArray(manifest.files) || manifest.files.length === 0) {
    throw new Error("the manifest is empty -- refusing to measure over nothing");
  }

  const seen = new Set();
  for (const file of manifest.files) {
    const where = `manifest entry \`${file.path}\``;
    checkToken("path", file.path, where);
    checkToken("stratum", file.stratum, where);
    if (!/^[0-9a-f]{64}$/.test(file.sha256 ?? "")) {
      throw new Error(`${where}: sha256 \`${file.sha256}\` is not 64 lowercase hex digits`);
    }
    const segment = file.path.split("/")[0];
    if (segment !== file.stratum || segment === file.path) {
      throw new Error(`${where}: stratum \`${file.stratum}\` is not its leading path segment`);
    }
    if (seen.has(file.path)) throw new Error(`${where}: listed twice -- it would be counted twice`);
    seen.add(file.path);
  }

  const recomputedList = corpusHashOf(manifest.files);
  if (manifest.corpus_hash !== recomputedList) {
    throw new Error(
      `manifest corpus_hash ${manifest.corpus_hash} does not match its own file list (${recomputedList})`,
    );
  }

  // Both directions on disk. Checking only the recorded files would let an
  // untracked program be added to the corpus and counted while the frozen hash
  // still verified.
  for (const file of manifest.files) {
    const full = path.join(CORPUS, file.path);
    if (!fs.existsSync(full)) throw new Error(`manifest lists \`${file.path}\`, which is not on disk`);
    const actual = sha256(fs.readFileSync(full));
    if (actual !== file.sha256) {
      throw new Error(`\`${file.path}\` hashes ${actual}, but the manifest froze ${file.sha256}`);
    }
  }
  for (const found of jsFilesUnder(CORPUS)) {
    if (!seen.has(found)) throw new Error(`\`${found}\` is under the corpus root but not in the manifest`);
  }

  return manifest;
}

/**
 * The kali binary to measure with.
 *
 * `KALI_BIN` if set, else `<cargo target_directory>/debug/kali` read from
 * `cargo metadata` -- never a hardcoded `target/` guess. Verified to exist and
 * be executable here, before anything is measured, so a missing binary is a
 * loud abort rather than an all-zero accept table.
 */
export function resolveKaliBinary() {
  let candidate = process.env.KALI_BIN;
  let source = "KALI_BIN";
  if (!candidate) {
    let metadata;
    try {
      metadata = JSON.parse(
        execFileSync("cargo", ["metadata", "--format-version", "1", "--no-deps"], {
          cwd: REPO,
          encoding: "utf8",
          maxBuffer: 64 * 1024 * 1024,
          stdio: ["ignore", "pipe", "pipe"],
        }),
      );
    } catch (cause) {
      throw new Error(
        `cannot locate the kali binary: \`cargo metadata\` failed (${cause.message}). ` +
          `Set KALI_BIN to the binary built by \`cargo build -p kali_cli\`.`,
        { cause },
      );
    }
    if (!metadata.target_directory) {
      throw new Error("`cargo metadata` reported no target_directory -- set KALI_BIN instead");
    }
    candidate = path.join(metadata.target_directory, "debug", "kali");
    source = "cargo metadata target_directory";
  }

  if (!fs.existsSync(candidate)) {
    throw new Error(
      `the kali binary is not at \`${candidate}\` (resolved from ${source}). ` +
        `Run \`cargo build -p kali_cli\`, or set KALI_BIN. Refusing to measure: a missing binary ` +
        `would mark every program unreachable and publish an all-zero reachable count that looks like data.`,
    );
  }
  try {
    fs.accessSync(candidate, fs.constants.X_OK);
  } catch (cause) {
    throw new Error(`\`${candidate}\` (resolved from ${source}) is not executable`, { cause });
  }

  const version = spawnSync(candidate, ["--version"], { encoding: "utf8" });
  if (version.error || version.status !== 0) {
    throw new Error(
      `\`${candidate} --version\` did not run cleanly (${version.error?.message ?? `exit ${version.status}`}) ` +
        `-- refusing to measure with a binary that cannot report its own version`,
    );
  }

  return { path: candidate, source, version: version.stdout.trim() };
}

/**
 * Does `kali check <file>` exit 0?
 *
 * Distinguishes "kali ran and rejected the program" from "kali could not be
 * run". The second is never a measurement, so it throws instead of returning
 * `false`.
 */
export function kaliAccepts(binary, file) {
  const result = spawnSync(binary, ["check", file], { stdio: "pipe", timeout: 120_000, encoding: "utf8" });
  if (result.error) {
    throw new Error(`could not run \`${binary} check ${file}\`: ${result.error.message}`, { cause: result.error });
  }
  if (result.signal) {
    throw new Error(`\`${binary} check ${file}\` was killed by ${result.signal} -- that is not a rejection`);
  }
  if (result.status === null) {
    throw new Error(`\`${binary} check ${file}\` produced no exit status -- that is not a rejection`);
  }
  return result.status === 0;
}
