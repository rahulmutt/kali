// tools/blast-radius/count.mjs
//
// Emits raw and reachable counts per register entry. Both are published: a
// reader can then see how much the reachability gate moved each entry instead
// of taking the gated number on faith.
//
// Ruling 2: counts are reported PER STRATUM as well as pooled. The anchor is
// 131 micro-snippets plus 6 real programs -- 4.4% of programs but 56.7% of
// bytes, non-CLBG median 52 bytes -- so a pooled count is dominated by the
// anchor's shape. The spec already forbids pooling accept RATES for exactly
// this reason; the same reasoning applies to counts.
//
// Run `accepts.mjs` first: this consumes `accepts.json`.

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { CORPUS, ROOT, loadVerifiedManifest } from "./corpus.mjs";
import { countAll, MATCHERS } from "./matchers.mjs";

// Ruling 3: the freeze is verified here too, not inherited from accepts.mjs.
const manifest = loadVerifiedManifest();
const catalogue = JSON.parse(fs.readFileSync(path.join(ROOT, "predicates.json"), "utf8"));

const acceptsPath = path.join(ROOT, "accepts.json");
if (!fs.existsSync(acceptsPath)) {
  throw new Error("accepts.json is missing -- run `node accepts.mjs` first; counts are gated on reachability");
}
const accepts = JSON.parse(fs.readFileSync(acceptsPath, "utf8"));

if (accepts.corpusHash !== manifest.corpus_hash) {
  throw new Error(
    `accepts.json was generated against corpus ${accepts.corpusHash} but the manifest is ` +
      `${manifest.corpus_hash} -- re-run accepts.mjs`,
  );
}
if (accepts.programs.length !== manifest.files.length) {
  throw new Error(
    `accepts.json covers ${accepts.programs.length} programs but the manifest lists ${manifest.files.length}`,
  );
}

// The catalogue and the matcher module must agree, in both directions. A
// catalogue naming a matcher that does not exist would silently contribute
// nothing; a matcher with no catalogue record would be counted for no entry.
const countable = catalogue.entries.filter((entry) => entry.kind === "countable");
for (const entry of countable) {
  if (!(entry.matcher in MATCHERS)) {
    throw new Error(
      `predicates.json names matcher \`${entry.matcher}\` (${entry.id}), which matchers.mjs does not export`,
    );
  }
}
for (const name of Object.keys(MATCHERS)) {
  if (!countable.some((entry) => entry.matcher === name)) {
    throw new Error(`matchers.mjs exports \`${name}\`, which no catalogue record names`);
  }
}

const acceptedPaths = new Set(accepts.programs.filter((p) => p.accepted).map((p) => p.path));
const strata = [...new Set(manifest.files.map((file) => file.stratum))].sort();

const zeros = () => Object.fromEntries(Object.keys(MATCHERS).map((name) => [name, 0]));
const totals = {
  pooled: { raw: zeros(), reachable: zeros(), programs: 0, accepted: 0 },
};
for (const stratum of strata) {
  totals[stratum] = { raw: zeros(), reachable: zeros(), programs: 0, accepted: 0 };
}

for (const file of manifest.files) {
  const source = fs.readFileSync(path.join(CORPUS, file.path), "utf8");
  // A parse failure throws out of countAll rather than reading as zero.
  const counts = countAll(source);
  const reachable = acceptedPaths.has(file.path);
  for (const bucket of [totals.pooled, totals[file.stratum]]) {
    bucket.programs += 1;
    if (reachable) bucket.accepted += 1;
    for (const [name, value] of Object.entries(counts)) {
      bucket.raw[name] += value;
      if (reachable) bucket.reachable[name] += value;
    }
  }
}

const entries = catalogue.entries.map((entry) => {
  if (entry.kind !== "countable") {
    return { id: entry.id, matcher: null, raw: null, reachable: null, strata: null };
  }
  const perStratum = {};
  for (const stratum of strata) {
    perStratum[stratum] = {
      raw: totals[stratum].raw[entry.matcher],
      reachable: totals[stratum].reachable[entry.matcher],
    };
  }
  return {
    id: entry.id,
    matcher: entry.matcher,
    // Pooled, kept as the documented `raw`/`reachable` fields.
    raw: totals.pooled.raw[entry.matcher],
    reachable: totals.pooled.reachable[entry.matcher],
    strata: perStratum,
  };
});

const nodeVersion = execFileSync(process.execPath, ["--version"], { encoding: "utf8" }).trim();
fs.writeFileSync(
  path.join(ROOT, "counts.json"),
  `${JSON.stringify(
    {
      corpusHash: manifest.corpus_hash,
      nodeVersion,
      acornVersion: JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8")).dependencies.acorn,
      programs: Object.fromEntries(
        ["pooled", ...strata].map((key) => [
          key,
          { programs: totals[key].programs, accepted: totals[key].accepted },
        ]),
      ),
      entries,
    },
    null,
    2,
  )}\n`,
);

console.log(`counted ${manifest.files.length} programs, ${countable.length} countable predicates`);
for (const key of ["pooled", ...strata]) {
  console.log(`${key}: ${totals[key].programs} programs, ${totals[key].accepted} reachable`);
}
const nonzero = entries.filter((entry) => entry.raw !== null && entry.raw > 0).length;
console.log(`${nonzero}/${countable.length} countable predicates have a nonzero raw count`);
for (const entry of entries) {
  if (entry.raw === 0) console.log(`zero raw count: ${entry.id} (${entry.matcher})`);
}
