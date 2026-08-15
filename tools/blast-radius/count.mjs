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
import { ALTERNATE_READINGS, BREAKDOWNS, countAll, MATCHERS, parse } from "./matchers.mjs";

// --------------------------------------------------------------------------
// Disclosure, published beside the numbers rather than only in a report.
//
// A reader who opens counts.json alone must be able to tell an upper bound from
// a measurement, and a zero of one kind from a zero of another. Neither is
// inferable from a bare integer.
// --------------------------------------------------------------------------

/**
 * Per-entry semantics. `disclosedInRecord` says whether the record itself
 * carries the upper-bound clause, or whether this measurement found it.
 */
const UPPER_BOUNDS = {
  "R-08": {
    disclosedInRecord: true,
    note:
      "Upper bound, per the record: the 2026-07-19 fix closed the provable majority, and the " +
      "surviving residuals are gated on the compiler failing to prove the other operand's type " +
      "class -- a compiler-internal decision the AST cannot see. Note also that the `??` half of " +
      "the predicate is unsampled: the corpus dialect contains no `??` at all (corpus/README.md), " +
      "so this count carries no evidence about nullish coalescing.",
  },
  "R-16": {
    disclosedInRecord: true,
    note:
      "Upper bound, per the record: the AST cannot see that the receiver is a runtime string. A " +
      "`.slice(...)` on an array in a `+` position is counted here and is not this defect.",
  },
  "R-26": {
    disclosedInRecord: true,
    note:
      "Upper bound, per the record: the defect needs the operand to hold a non-numeric string at " +
      "run time. `+\"42\"`, `+\"-5\"`, `+\"1.5\"`, `+\"\"` and `+true` are correct in kali and are " +
      "counted here anyway, because the record lists a string literal among the counted shapes.",
  },
  "R-30": {
    disclosedInRecord: true,
    note:
      "Upper bound, per the record: for the call, parameter, `var`-binding and object-field " +
      "producers the AST cannot see whether the value is actually a boolean. The literal-selecting " +
      "`??` is not among them. Read the per-stratum split before this total: the anchor's share is " +
      "almost entirely `console.log(<comparison>)` in `f64_*` micro-snippets that observe " +
      "floating-point arithmetic through a comparison -- a test idiom, not a program idiom.",
  },
  "R-13": {
    disclosedInRecord: false,
    note:
      "Upper bound NOT disclosed by the record. The record is 'computed member access whose key " +
      "expression is not a literal', with no qualifying clause, so ordinary array indexing `a[i]` " +
      "counts -- and array indexing demonstrably works (much of the anchor stratum depends on it). " +
      "The register's R-13 repro is an OBJECT read with a variable key. See `breakdown`: only the " +
      "object-literal-receiver share is the register's shape, and `storeTarget` counts the write " +
      "half, which the register treats as the worse half but which is a different site class from " +
      "a read. Do not present this total as 'how often R-13's defect is triggered'.",
  },
  "R-14": {
    disclosedInRecord: false,
    note:
      "Upper bound NOT disclosed by the record. 'A member or computed read applied directly to a " +
      "call expression's result' also matches `\"a,b\".split(\",\")[0]` and `s.slice(1).length`, " +
      "while the register's repro is specifically an ARRAY returned from a function reading back " +
      "as zeros.",
  },
  "R-07": {
    disclosedInRecord: false,
    note:
      "The published count uses the record's main clause ('is not a literal'). Its appositive " +
      "dash-list, read as exhaustive, gives a materially different number and a different rank -- " +
      "see `alternateReading`. Independently of that choice, this count is broad by construction: " +
      "in the corpus's dialect nearly every `const` has a non-literal initializer.",
  },
};

/**
 * The one entry whose zero is not a frequency at all. Kept as an explicit,
 * justified table rather than a rule, because it is a claim about JavaScript
 * and about this corpus's curation rule, not something derivable from a count.
 */
const STRUCTURALLY_UNCOUNTABLE = {
  "R-29":
    "An assignment to a `const` binding is a TypeError at run time, so no program that runs " +
    "clean under node can execute one; the construct and this corpus's runnability requirement " +
    "are mutually exclusive (corpus/README.md). This zero is not a frequency and must never be " +
    "ranked as one.",
};

const ZERO_KINDS = {
  "structurally-uncountable":
    "The construct cannot appear in any conforming corpus program. Not a frequency; must not be " +
    "ranked against measured frequencies.",
  unsampled:
    "Countable and legal, but absent from this corpus. An ordinary zero over this population: it " +
    "says nothing about a larger or differently-shaped one.",
  "present-but-unreachable":
    "raw > 0 and reachable = 0: the construct DOES occur, but every program carrying it is " +
    "rejected by kali as a whole. This is the most misreadable of the three. It does NOT mean the " +
    "construct is rare, and it does NOT mean kali fails closed on this construct -- the carrying " +
    "program was usually rejected for an unrelated reason elsewhere in the file.",
};

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
const altZeros = () => Object.fromEntries(Object.keys(ALTERNATE_READINGS).map((id) => [id, 0]));
const breakdownZeros = () =>
  Object.fromEntries(
    Object.keys(BREAKDOWNS).map((id) => [id, { total: 0, objectLiteralReceiver: 0, arrayLikeReceiver: 0, storeTarget: 0 }]),
  );

const newBucket = () => ({
  raw: zeros(),
  reachable: zeros(),
  altRaw: altZeros(),
  altReachable: altZeros(),
  breakdownRaw: breakdownZeros(),
  breakdownReachable: breakdownZeros(),
  programs: 0,
  accepted: 0,
});
const totals = { pooled: newBucket() };
for (const stratum of strata) totals[stratum] = newBucket();

for (const file of manifest.files) {
  const source = fs.readFileSync(path.join(CORPUS, file.path), "utf8");
  // A parse failure throws rather than reading as zero.
  const counts = countAll(source);
  const ast = parse(source);
  const alternates = Object.fromEntries(
    Object.entries(ALTERNATE_READINGS).map(([id, reading]) => [id, reading.count(ast)]),
  );
  const breakdowns = Object.fromEntries(Object.entries(BREAKDOWNS).map(([id, each]) => [id, each.count(ast)]));
  const reachable = acceptedPaths.has(file.path);

  for (const bucket of [totals.pooled, totals[file.stratum]]) {
    bucket.programs += 1;
    if (reachable) bucket.accepted += 1;
    for (const [name, value] of Object.entries(counts)) {
      bucket.raw[name] += value;
      if (reachable) bucket.reachable[name] += value;
    }
    for (const [id, value] of Object.entries(alternates)) {
      bucket.altRaw[id] += value;
      if (reachable) bucket.altReachable[id] += value;
    }
    for (const [id, value] of Object.entries(breakdowns)) {
      for (const [field, each] of Object.entries(value)) {
        bucket.breakdownRaw[id][field] += each;
        if (reachable) bucket.breakdownReachable[id][field] += each;
      }
    }
  }
}

/** Which of the three kinds of zero this is, or null when the entry is not zero. */
function classifyZero(id, raw, reachable) {
  if (raw === 0) return id in STRUCTURALLY_UNCOUNTABLE ? "structurally-uncountable" : "unsampled";
  if (reachable === 0) return "present-but-unreachable";
  return null;
}

const entries = catalogue.entries.map((entry) => {
  if (entry.kind !== "countable") {
    return {
      id: entry.id,
      matcher: null,
      raw: null,
      reachable: null,
      strata: null,
      zero: null,
      upperBound: null,
      alternateReading: null,
    };
  }
  const perStratum = {};
  for (const stratum of strata) {
    perStratum[stratum] = {
      raw: totals[stratum].raw[entry.matcher],
      reachable: totals[stratum].reachable[entry.matcher],
    };
  }
  const raw = totals.pooled.raw[entry.matcher];
  const reachable = totals.pooled.reachable[entry.matcher];
  const zeroKind = classifyZero(entry.id, raw, reachable);

  const upperBound = UPPER_BOUNDS[entry.id] ? { ...UPPER_BOUNDS[entry.id] } : null;
  if (upperBound && BREAKDOWNS[entry.id]) {
    upperBound.breakdown = {
      of: BREAKDOWNS[entry.id].of,
      raw: totals.pooled.breakdownRaw[entry.id],
      reachable: totals.pooled.breakdownReachable[entry.id],
    };
  }

  const reading = ALTERNATE_READINGS[entry.id];
  const alternateReading = reading
    ? {
        publishedReading: reading.publishedReading,
        alternateReading: reading.alternateReading,
        whyPublishedReadingWasChosen: reading.whyPublishedReadingWasChosen,
        published: { raw, reachable },
        alternate: { raw: totals.pooled.altRaw[entry.id], reachable: totals.pooled.altReachable[entry.id] },
        alternateStrata: Object.fromEntries(
          strata.map((stratum) => [
            stratum,
            { raw: totals[stratum].altRaw[entry.id], reachable: totals[stratum].altReachable[entry.id] },
          ]),
        ),
      }
    : null;

  return {
    id: entry.id,
    matcher: entry.matcher,
    // Pooled, kept as the documented `raw`/`reachable` fields.
    raw,
    reachable,
    strata: perStratum,
    zero: zeroKind,
    upperBound,
    alternateReading,
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
      // What the reachable column is a frequency OVER. Published here because
      // it is not inferable from a bare count, and it changes what every
      // reachable number means.
      population: {
        reachableColumn:
          `${totals.pooled.accepted} of ${totals.pooled.programs} programs are reachable, and ` +
          `${totals.anchor.accepted} of those ${totals.pooled.accepted} are ANCHOR programs -- a stratum that is ` +
          `131 micro-snippets written to probe compiler behaviour plus 6 real CLBG programs. Every ` +
          `reachable ranking is therefore, in substance, a ranking over test snippets. Read the ` +
          `per-entry \`strata\` split before treating any reachable figure as a frequency in real code.`,
        extensionStratum:
          `${totals.extension.accepted}/${totals.extension.programs} extension programs are accepted ` +
          `(${((100 * totals.extension.accepted) / totals.extension.programs).toFixed(1)}%). The extension is the ` +
          `stratum written to do jobs rather than to probe the compiler, so almost everything it ` +
          `measures about real programs lands in the RAW column only. Its accept rate is a finding ` +
          `in its own right, not a defect of the corpus: curation was independent of acceptance.`,
        dialect:
          "The extension is written in the project's imperative-core dialect: no regex, no " +
          "destructuring, no template literals, no `??`, no class/Map/Set/async. See " +
          "corpus/README.md for which counts that biases and in which direction. A frequency here " +
          "is a frequency in *programs of that dialect*, not in JavaScript generally.",
      },
      zeroKinds: ZERO_KINDS,
      structurallyUncountable: STRUCTURALLY_UNCOUNTABLE,
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
for (const kind of Object.keys(ZERO_KINDS)) {
  const members = entries.filter((entry) => entry.zero === kind).map((entry) => entry.id);
  console.log(`${kind}: ${members.length ? members.join(" ") : "(none)"}`);
}
for (const entry of entries) {
  if (entry.alternateReading) {
    const { published, alternate } = entry.alternateReading;
    console.log(
      `${entry.id} rests on a reading: published ${published.raw}/${published.reachable}, ` +
        `alternate ${alternate.raw}/${alternate.reachable} (raw/reachable)`,
    );
  }
}
