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
  "R-56": {
    disclosedInRecord: true,
    note:
      "Upper bound, per the record: the exact condition is that the key's inner text lies in " +
      "Rust's `Display for f64` image -- what `lower_property_name` could have written -- and an " +
      "acorn AST cannot reproduce that image. The matcher tests `Number.isFinite(Number(inner))` " +
      "instead, which is strictly wider, so `{'\"1e21\"': 1}`, `{'\"05\"': 1}` and `{'\"5.\"': 1}` " +
      "are counted here and are NOT this defect -- each is a string key the codegen predicate " +
      "correctly leaves alone.",
  },
  "R-57": {
    disclosedInRecord: true,
    note:
      "Upper bound, per the record: the matcher counts any string-literal key whose raw source " +
      "text carries a backslash, and acorn accepts escapes kali's lexer refuses outright -- " +
      "`\\u`, `\\x`, and everything outside the eleven at `crates/kali_lexer/src/string.rs:28`. " +
      "A key spelled with one of those is counted here and diverges LOUDLY (`error[E1004]: " +
      "unsupported string escape sequence`, exit 1), which is not this entry's silent class. " +
      "Every key spelled with one of the eleven IS this defect, computed or not -- all eleven " +
      "measured at `dde0f083c0` in both scopes: `Object.keys(o)[0] === \"<the same literal>\"` " +
      "is `false` in kali and `true` in node for every one of them.",
  },
  "R-58": {
    disclosedInRecord: true,
    note:
      "Upper bound, per the record: `0[0-7]+` is the legacy-octal SPELLING, and a spelling is " +
      "not always a divergence. A one-digit run reads the same in both radices, so `{00: 1}` " +
      "through `{07: 1}` are counted here and agree with node. Every longer run diverges " +
      "(`{010: 1}` is the key `8` in node and `10` in kali). `08`/`09` are " +
      "NonOctalDecimalIntegerLiteral, not octal, and are correctly excluded; `0o42` is excluded " +
      "because kali's lexer never tokenizes it as one number and it fails LOUDLY (`error[E3100]: " +
      "undefined identifier 'o42'`, exit 1). " +
      "THIS NUMBER IS ALSO A LOWER BOUND ON R-58, WHICH NO OTHER RECORD IN THIS CATALOGUE IS, " +
      "AND THE 0 MUST NOT BE READ AS THE FREQUENCY OF THE WHOLE DEFECT. The matcher counts " +
      "legacy octal in OBJECT-LITERAL KEY position only, and R-58's entry establishes by " +
      "measurement that the same misreading fires in ordinary expression position too: " +
      "`console.log(042)` prints `42` where node prints `34` (measured at `dde0f083c0` against " +
      "node v26.8.1, both scopes, and again through a `const` binding). That lane is a " +
      "different parser function (`expression/primary.rs`'s numeric-literal arm, not " +
      "`numeric_property_name`) and is counted by nothing here. The catalogue RECORD cannot " +
      "disclose this, because a record states what its matcher counts and the matcher counts " +
      "the key lane; the disclosure therefore lives here, beside the number a reader actually " +
      "meets.",
  },
  "R-59": {
    disclosedInRecord: true,
    note:
      "Upper bound, per the record: the shape is a computed index the parser cannot read " +
      "statically, and a receiver ALLOCATED WITH `new Array(n)` reaches a runtime-index lane " +
      "that evaluates the member node's structured index child instead of the fabricated name, " +
      "and does not diverge. Measured at `35e9ef4ef6` against node v26.8.1, both scopes: " +
      "`const a = new Array(3); for (let i = 0; i < 3; i++) { a[i] = i * 2; }` then reading " +
      "`a[i]` prints `0 2 4` on BOTH engines, while the same loop over an ARRAY LITERAL " +
      "(`const a = [5, 6, 7]`) prints `0 0 0` against node's `5 6 7`. An acorn AST cannot see " +
      "how a receiver was allocated, so the working lane is counted here. " +
      "READ THIS BESIDE R-13's NOTE, NOT INSTEAD OF IT. The two records overlap by " +
      "construction: R-59's shape is a strict SUBSET of R-13's, because R-13's `not a literal` " +
      "counts the sequence and folded-unary spellings this parser reads correctly and R-59's " +
      "arm-derived matcher excludes them. The two entries are not the same defect -- R-13 " +
      "records a computed read returning `0`, and R-59 records the same read returning the " +
      "WRONG PROPERTY'S VALUE when the fabricated name collides with a real property, which is " +
      "a different and worse observable -- but a reader who adds these two numbers together is " +
      "double-counting sites. ON THIS CORPUS THE TWO MATCHERS PRINT THE SAME FOUR NUMBERS " +
      "(raw 302, reachable 45, anchor 47/43, extension 255/2), which is a measurement and not " +
      "a coincidence worth hiding: the frozen corpus contains NO parenthesized, sequence or " +
      "folded-unary computed index at all, so the shapes that separate the two records do not " +
      "occur here. The narrowing is real in the language and unexercised in this corpus, and " +
      "the identical figures are the evidence for both halves of that sentence.",
  },
  "R-60": {
    disclosedInRecord: false,
    note:
      "Upper bound NOT disclosed by the record, and a LOWER bound as well; neither direction " +
      "is a defect in the matcher, and both are about what a member read's RECEIVER is. " +
      "Upper: the record counts every member read on a `fromEntries` result, and the " +
      "measurements behind R-60 are all reads of a property the object HAS or does not have " +
      "under the default `kali run` (Fast) build mode. Whether the same read diverges under " +
      "`--release`, where `fold_object_from_entries_call`'s binding path runs, was NOT " +
      "measured -- there is no runner for a built artifact on this machine -- so a corpus " +
      "counted here is counted for the mode the oracle cases pin and no other. " +
      "Lower: the fabricated `0` is what an unresolvable static member read emits generally, " +
      "and `Object.fromEntries` is one producer of an unresolvable receiver. The matcher " +
      "counts that producer alone and carries no evidence about the rest of the family " +
      "(R-21's, and the `o[1]` read in R-59's own repro). " +
      "The neighbouring ENUMERATION consumers of the same receiver are excluded correctly and " +
      "for a reason worth stating: `Object.keys(o)` / `Object.values(o)` / `Object.entries(o)` " +
      "on a `fromEntries` result fail LOUDLY (`error[E5506]: Object enumeration is only " +
      "supported where the object has a compile-time-known fixed shape`, exit 1, measured at " +
      "`35e9ef4ef6` in both scopes, directly and through a binding), which is a different " +
      "verdict class, and in any case the receiver is an ARGUMENT there rather than a member " +
      "receiver.",
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
    // Per stratum as well as pooled, for the same reason the counts are: a
    // pooled breakdown cannot answer "which stratum are the register-shaped
    // sites in?", and for R-13 that question is the difference between a
    // number about real programs and a number about test snippets.
    upperBound.breakdown = {
      of: BREAKDOWNS[entry.id].of,
      raw: totals.pooled.breakdownRaw[entry.id],
      reachable: totals.pooled.breakdownReachable[entry.id],
      strata: Object.fromEntries(
        strata.map((stratum) => [
          stratum,
          { raw: totals[stratum].breakdownRaw[entry.id], reachable: totals[stratum].breakdownReachable[entry.id] },
        ]),
      ),
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
