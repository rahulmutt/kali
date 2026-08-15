// tools/blast-radius/accepts.mjs
//
// Reachability (design spec §6): per corpus program, binary -- does
// `kali check` exit 0? Occurrences in rejected programs score zero, because a
// defect kali fails closed on does no damage.
//
// Run this BEFORE count.mjs: the counter consumes `accepts.json`.

import fs from "node:fs";
import path from "node:path";
import { CORPUS, ROOT, kaliAccepts, loadVerifiedManifest, resolveKaliBinary } from "./corpus.mjs";

// Ruling 3: the freeze is verified here, not assumed.
const manifest = loadVerifiedManifest();

// Ruling 1: resolve and verify the binary BEFORE measuring anything. A spawn
// failure is never recorded as `accepted = false`.
const kali = resolveKaliBinary();
console.log(`kali: ${kali.path} (${kali.source})`);
console.log(`version: ${kali.version}`);
console.log(`corpus: ${manifest.corpus_hash} (${manifest.files.length} programs)`);

const programs = manifest.files.map((file) => ({
  path: file.path,
  stratum: file.stratum,
  accepted: kaliAccepts(kali.path, path.join(CORPUS, file.path)),
}));

const rates = {};
for (const program of programs) {
  const bucket = (rates[program.stratum] ??= { accepted: 0, total: 0 });
  bucket.total += 1;
  if (program.accepted) bucket.accepted += 1;
}

fs.writeFileSync(
  path.join(ROOT, "accepts.json"),
  `${JSON.stringify(
    {
      corpusHash: manifest.corpus_hash,
      kaliBinary: kali.path,
      kaliVersion: kali.version,
      // Per stratum, never pooled -- see below.
      rates,
      programs,
    },
    null,
    2,
  )}\n`,
);

// Rates are printed per stratum and NEVER pooled: the anchor is passing tests,
// so a pooled rate would inherit its ~100% and mean nothing.
for (const [stratum, bucket] of Object.entries(rates)) {
  const percent = ((100 * bucket.accepted) / bucket.total).toFixed(1);
  console.log(`${stratum}: ${bucket.accepted}/${bucket.total} accepted (${percent}%)`);
}
