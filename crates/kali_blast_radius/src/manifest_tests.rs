use super::*;

/// A per-process, per-call-unique scratch directory. The brief's tests used
/// fixed names under `std::env::temp_dir()`; two concurrent runs of this test
/// binary (two shells, or a CI job sharing a machine) would then race on the
/// same directory and flake. Uniqueness must not rest on the wall clock alone
/// -- a coarse `SystemTime` can hand two threads the same nanosecond -- so a
/// process-wide counter carries it, mirroring the precedent in
/// `crates/kali_cli/tests/clbg_binary_trees_runtime.rs`.
fn scratch_dir(label: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir =
        std::env::temp_dir().join(format!("blast-radius-{label}-{}-{seq}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn the_corpus_hash_is_order_independent_and_content_sensitive() {
    let a = ManifestFile {
        path: "b.js".into(),
        stratum: "anchor".into(),
        sha256: "22".into(),
    };
    let b = ManifestFile {
        path: "a.js".into(),
        stratum: "anchor".into(),
        sha256: "11".into(),
    };
    let forward = corpus_hash(&[a.clone(), b.clone()]);
    let reversed = corpus_hash(&[b.clone(), a.clone()]);
    assert_eq!(forward, reversed, "hash must not depend on listing order");

    let changed = ManifestFile {
        sha256: "33".into(),
        ..a
    };
    assert_ne!(
        forward,
        corpus_hash(&[changed, b]),
        "a changed file must change the corpus hash"
    );
}

#[test]
fn verify_rejects_a_file_whose_content_changed() {
    let dir = scratch_dir("manifest-test");
    std::fs::create_dir_all(dir.join("anchor")).expect("mkdir");
    std::fs::write(dir.join("anchor/x.js"), "console.log(1);\n").expect("write");

    let good = ManifestFile {
        path: "anchor/x.js".into(),
        stratum: "anchor".into(),
        sha256: sha256_of("console.log(1);\n".as_bytes()),
    };
    let manifest = Manifest {
        corpus_hash: corpus_hash(std::slice::from_ref(&good)),
        files: vec![good],
    };
    verify_manifest(&dir, &manifest).expect("an unmodified corpus verifies");

    std::fs::write(dir.join("anchor/x.js"), "console.log(2);\n").expect("write");
    let error = verify_manifest(&dir, &manifest).expect_err("a modified corpus must not verify");
    assert!(
        error.contains("anchor/x.js"),
        "error names the file: {error}"
    );

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn verify_rejects_an_untracked_file_in_the_corpus() {
    // A file present on disk but absent from the manifest would be counted by
    // the counter while the frozen hash still looked unchanged -- the exact
    // post-hoc corpus edit the freeze rule exists to prevent.
    let dir = scratch_dir("untracked-test");
    std::fs::create_dir_all(dir.join("anchor")).expect("mkdir");
    std::fs::write(dir.join("anchor/x.js"), "console.log(1);\n").expect("write");
    std::fs::write(dir.join("anchor/sneaky.js"), "console.log(2);\n").expect("write");

    let tracked = ManifestFile {
        path: "anchor/x.js".into(),
        stratum: "anchor".into(),
        sha256: sha256_of("console.log(1);\n".as_bytes()),
    };
    let manifest = Manifest {
        corpus_hash: corpus_hash(std::slice::from_ref(&tracked)),
        files: vec![tracked],
    };
    let error = verify_manifest(&dir, &manifest).expect_err("an untracked file must not verify");
    assert!(error.contains("sneaky.js"), "error names the file: {error}");

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn the_shipped_corpus_matches_its_manifest() {
    let root = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/blast-radius/corpus"
    ));
    let text = std::fs::read_to_string(root.join("manifest.json")).expect("manifest is readable");
    let manifest = parse_manifest(&text).expect("manifest parses");
    assert!(
        !manifest.files.is_empty(),
        "an empty manifest must not verify as frozen"
    );
    assert_eq!(
        manifest.corpus_hash,
        corpus_hash(&manifest.files),
        "the recorded corpus hash does not match its own file list"
    );
    verify_manifest(root, &manifest).expect("the shipped corpus matches its manifest");
}

/// The corpus was frozen at these exact values on 2026-08-15 (Task 12).
///
/// Everything downstream cites `FROZEN_CORPUS_HASH`, so it is a published
/// constant, not an implementation detail.
const FROZEN_FILE_COUNT: usize = 177;
const FROZEN_ANCHOR_COUNT: usize = 137;
const FROZEN_EXTENSION_COUNT: usize = 40;
const FROZEN_CORPUS_HASH: &str = "ca6f53339feb61b1ad988f5075c2648fd95a96b1796d67bcf2cd3af69090660f";

/// The other half of the same freeze, pinned the same way.
///
/// Spec §4.3 freezes `predicates.json` alongside the corpus, and
/// `corpus/README.md` says so in the same sentence -- but only the corpus half
/// was mechanical. The catalogue was checked for COMPLETENESS (46 records ↔ 46
/// entries as of 2026-09-08 -- the figure was written as 42 and left behind by
/// three later re-freezes, and is corrected here rather than left to rot, which
/// is the same failure mode the strike-throughs in `tier2.toml` exist to make
/// visible; matcher names agreeing with `matchers.mjs`), which is silent about
/// *which* matcher an entry maps to: swap R-13's matcher for R-14's and every
/// completeness check stays green while both counts change.
///
/// `matchers.mjs` is pinned beside it because the catalogue only NAMES the
/// counting semantics; the semantics themselves are the matcher bodies. A
/// count is a function of (corpus, matcher body), so pinning the map without
/// the territory leaves the arithmetic behind every published frequency
/// unpinned.
///
/// **Re-frozen 2026-08-16**, and this is the first time either constant has
/// moved since the ranking project froze them. What moved and why: register
/// entry **R-56** (§2, Tier 2 — the quoted-numeric-string-key collision) was
/// filed, a §2 entry needs a catalogue record for `check_completeness` to pass,
/// and the record is honestly COUNTABLE — the triggering shape is a property key
/// spelled a particular way, which is syntax an acorn AST can see. The four
/// existing `uncountable` reasons are **R-17, R-21, R-22 and R-54**, and none of
/// them applies here: R-17 and R-21 are representation conditions, R-22 is a
/// runtime-type one, and R-54's is a PARSEABILITY condition — only invalid
/// JavaScript triggers it, so acorn rejects the shape and no conforming corpus
/// file can carry it. `{'"5"': 1}` is legal, parseable JavaScript, so it fails
/// every one of those tests for uncountability. A countable record names a
/// matcher, so `matchers.mjs` gained `objectLiteralQuotedNumericStringKey`. Both
/// files therefore moved together, by construction.
///
/// `counts.json` was regenerated with them (`node count.mjs`), and the diff is
/// the evidence that this was an ADDITION and not a change: the regeneration
/// added exactly one entry (R-56: raw 0, reachable 0, `unsampled`) and left
/// every other entry's every field byte-identical.
///
/// **Why the `countable`/`uncountable` word is worth this much comment.**
/// `score::aggregate` makes a cluster uncountable if ANY member is, and an
/// uncountable cluster is never dominated, so it lands in band 1 by
/// construction. Filing R-56 `uncountable` would have been strictly easier — no
/// matcher, one frozen SHA to re-pin instead of two — and would have put a
/// brand-new zero-frequency entry on the published frontier, with **no gate
/// objecting**. Nothing downstream can check that an `uncountable` reason is
/// true. The next author to add an entry faces the same incentive; the record is
/// countable because the shape is countable, and the band placement is a
/// consequence rather than a motive.
///
/// **Re-frozen 2026-09-08**, the second movement of these constants, by the
/// register-property-key-followups branch filing **R-57** (§2, Tier 2 — a key
/// spelled with an escape sequence is stored undecoded) and **R-58** (§2,
/// Tier 2 — a legacy-octal numeric key is read as decimal). Two §2 entries need
/// two catalogue records for `check_completeness`, and both records are honestly
/// COUNTABLE for the same reason R-56's is: each triggering shape is a property
/// key spelled a particular way, which an acorn AST hands you directly. The
/// four `uncountable` reasons (R-17, R-21, R-22, R-54 — representation,
/// representation, runtime-type, parseability) reach neither: `{"a\"b": 1}` and
/// `{042: 1}` are both legal, parseable JavaScript, and both matchers read
/// `key.raw` — the SOURCE SPELLING — which is the one thing acorn preserves
/// exactly. So `matchers.mjs` gained `objectLiteralEscapedStringKey` and
/// `objectLiteralLegacyOctalNumericKey`, and both files moved together again.
///
/// `counts.json` was regenerated with them (`node count.mjs`), and the diff is
/// again the evidence that this was an ADDITION: it added exactly two entries
/// (R-57 and R-58, each raw 0 / reachable 0, `unsampled`) and left every other
/// entry's every field byte-identical — with ONE exception that is a genuine
/// reading and not a drift, disclosed here rather than smoothed over: the
/// file's `nodeVersion` cell moved `v26.7.0` -> `v26.8.1`, because that is what
/// `node --version` prints on this machine now. No count depends on it (the
/// matchers run on acorn, pinned at 8.18.0, over the unchanged frozen corpus,
/// and every other number in the file is byte-identical), but it is published
/// provenance and it propagates into the ranking's provenance table.
///
/// The `countable` warning above applies unchanged to both new records, and the
/// band placement is again a consequence: both measure zero, so both land among
/// the tier-2 zeros rather than on the frontier.
///
/// **Re-frozen 2026-09-08 a second time**, the third movement of these
/// constants, by the same branch filing **R-59** (§2, Tier 2 — a computed
/// member index that is not a literal is fabricated into a property name) and
/// **R-60** (§2, Tier 2 — a present property on an `Object.fromEntries` object
/// reads `0`). Two more §2 entries, two more records, and `matchers.mjs` gained
/// `computedMemberFabricatedPropertyName` and
/// `memberReadOnObjectFromEntriesResult`.
///
/// **BOTH RECORDS ARE COUNTABLE, AND R-60's WAS THE CLOSE CALL.** R-59's shape
/// is pure syntax — which arms of `expression_to_property_name` an index
/// expression reaches — so it never came near the four `uncountable` reasons.
/// R-60's did: its triggering construct is a member read whose RECEIVER is a
/// `fromEntries` result, and a receiver is a value, which sounds like R-17's and
/// R-21's representation condition. It is not one. The receiver here is
/// identified by the SPELLING of the call that produced it, through at most one
/// binding, and this module already resolves bindings for R-02, R-10, R-12,
/// R-29, R-30 and R-47 — a binding walk reads nothing the source does not say.
/// What WOULD have been uncountable is the wider family R-60's own
/// `UPPER_BOUNDS` note discloses (any unresolvable static member read), and that
/// is exactly why the record is stated at the narrow, decidable shape and the
/// family is disclosed beside the number instead of smuggled into the matcher.
///
/// `counts.json` was regenerated with them (`node count.mjs`) over the unchanged
/// frozen corpus, and `accepts.mjs` was re-run FIRST and wrote a byte-identical
/// `accepts.json` (anchor 126/137, extension 1/40) as the evidence that corpus
/// and binary are where they were. The `counts.json` diff adds exactly two
/// entries and leaves every other entry's every field byte-identical — including
/// `nodeVersion`, which stays `v26.8.1`, and the corpus hash, which stays
/// `ca6f5333…`.
///
/// **THE ONE FIGURE THAT IS NOT A ZERO, AND THE ONE READING THAT CHANGES HOW IT
/// MUST BE PUBLISHED.** R-59 measures **raw 302 / reachable 45** — the first
/// entry filed by this branch with any frequency behind it, and the first new
/// record since the freeze to enter a nonzero band. Those are, to the digit, the
/// four numbers R-13's `computedMemberNonLiteralKey` already prints (anchor
/// 47/43, extension 255/2). That is measured, not assumed, and it is not a
/// duplicate matcher — but ~~R-59's is strictly narrower, excluding the
/// parenthesized, sequence and folded-unary index spellings this parser reads
/// CORRECTLY, and the corpus simply contains none of them~~ **WAS THE WRONG
/// REASON, corrected below at the fourth re-freeze.**
///
/// **Re-frozen 2026-09-08 a THIRD time, the FOURTH movement of these constants,
/// to withdraw a containment claim the instrument itself disproves.** Review
/// round 1 found that "R-59's shape is a strict SUBSET of R-13's" is false, and
/// the check is one line against the shipped module: `var o={}; o[true];
/// o[null]; o[/x/]; o[1n];` counts **0** under `computedMemberNonLiteralKey` and
/// **4** under `computedMemberFabricatedPropertyName`, while
/// `var o={1:"one"}; o[(1)]; o[(0,1)]; o[+1]; o[-1];` counts **3** under the
/// first and **0** under the second. R-13's shape is
/// `computed && property.type != "Literal"`; R-59's asks whether
/// `expression_to_property_name` can READ the index, and a boolean, `null`, a
/// BigInt and a regex are all `Literal` nodes that it cannot read. **The two
/// overlap and neither contains the other**, and both directions are counted
/// correctly: measured at `35e9ef4ef6` against node v26.8.1 in both scopes,
/// `o[true]`, `o[null]` and `o[1n]` each read the fabricated `index` property
/// (`5` against node's `7`) at exit 0, and the four readable spellings each read
/// the CORRECT name.
///
/// **The four figures are unchanged and so is every other number in
/// `counts.json`; what moved is the EXPLANATION.** The corpus prints the same
/// raw 302 / reachable 45 for both matchers because it contains NEITHER
/// separating family — not because one shape contains the other. That is the
/// distinction this re-freeze exists to publish, and it cost a record
/// `description`, a `count.mjs` note, five documents and both SHAs to fix,
/// because the false version had been written into all of them.
///
/// **A SECOND UPPER BOUND WAS FOUND IN THE SAME PASS AND IS NOW IN THE RECORD.**
/// The regex spelling `o[/x/]` is counted by R-59's matcher and diverges
/// **LOUDLY**, not silently: kali's lexer has no regex-literal token, so `/x/`
/// lexes as a division by the identifier `x` and the program is refused with
/// `error[E3100]: undefined identifier 'x'` at exit 1 where node reads the
/// property (measured at `35e9ef4ef6`, both scopes). Unlike the caveat
/// `2340b85335` deliberately kept OUT of R-58's record, this one IS about what
/// the matcher counts, so it belongs in the `description` — and the record was
/// being reopened anyway.
///
/// The pair `computedMemberFabricatedPropertyName counts READABLE-but-not-literal
/// indices as R-13 does not` and `... counts LITERAL-but-unreadable indices as
/// R-13 does not` in `matchers.test.mjs` pins both directions, so the withdrawn
/// claim cannot be made again without a red test.
///
/// **Re-frozen 2026-09-08 a FOURTH time, the FIFTH movement of these constants,
/// because R-59's matcher was counting the WRITE half of a READ-lane entry.**
/// The final whole-branch review found that
/// `computedMemberFabricatedPropertyName` counted assignment and update TARGETS
/// as well as reads: of its raw 302, **67 were store targets**; of its reachable
/// 45, **18 were** — 40% of the headline. R-59's own entry measures that a store
/// does not fabricate, so those sites are not the defect the record names. The
/// two measurements, re-run at `6f0df2c3db` against node v26.8.1 in **both**
/// scopes: `const o = {index:9, i:7}; let i = 1; o[i] = 8;` then `o.i` prints
/// `7` and `o.index` prints `9` on BOTH engines, exit 0, 0 bytes of stderr (the
/// store lands nowhere — that is R-13's write half); and `o[i]++` over the same
/// object is refused **LOUDLY** in both scopes (`error[E5506]: update expression
/// lowering is unavailable unless the target is a mutable local binding`, exit
/// 1) where node prints `7` and `9`. Neither is R-59's silent read-lane class.
/// The matcher now excludes both, exactly as `memberReadOnObjectFromEntriesResult`
/// already did, and the fix follows the standing constraint that a matcher is
/// written from the entry's triggering construct rather than tuned to a count.
///
/// **THIS ONE MOVED FIGURES, WHICH THE THREE RE-FREEZES BEFORE IT DID NOT.**
/// R-59 goes **raw 302 -> 235** and **reachable 45 -> 27** (anchor 47/43 ->
/// 27/25, extension 255/2 -> 208/2). The deltas are exactly R-13's record's own
/// `breakdown` storeTarget figures (raw 67, reachable 18; anchor 20/18,
/// extension 47/0), which is the cross-check that the exclusion removed store
/// targets and nothing else. `counts.json`'s whole diff is those five numbers
/// plus R-59's `note`. R-13's record is **not** reopened: R-13's matcher counts
/// store targets by its own description and classifies them in its breakdown.
///
/// **So the sentence above — that the corpus prints the same four numbers for
/// both matchers — is now history rather than fact, and is left standing as the
/// record of what the fourth re-freeze published.** The relationship was
/// re-measured over the frozen corpus file by file rather than inferred: of the
/// **51** files with a nonzero count under either matcher, **30** now differ (8
/// of the **14** reachable ones), and R-59's count exceeds R-13's in **zero**
/// files. That ordering holds on THIS corpus only because the corpus contains
/// neither family that runs the other way; the containment withdrawal above
/// stands unchanged, and store targets are simply the one separating family the
/// corpus does exercise. `computedMemberFabricatedPropertyName counts reads
/// only, not assignment or update targets` in `matchers.test.mjs` pins the third
/// direction (9 under R-13's matcher, 4 under R-59's, over the same program).
/// **Re-frozen 2026-09-10**, the sixth movement of these constants, by the
/// release-tier-allocation-identity project filing **R-61** (§2, Tier 2 — the
/// release tiers substitute an allocating array initializer for its own
/// name, so a literal-index read off it returns the initializer's raw form).
/// One more §2 entry, one more catalogue record, and **`matchers.mjs` did
/// NOT move** — the record is honestly `uncountable`, the first entry to be
/// filed that way since R-54, and for a NEW reason none of the four existing
/// `uncountable` records carry (R-17/R-21: representation; R-22: runtime-
/// type; R-54: parseability). This one is a **build-tier condition**: the
/// triggering source is identical text whether it compiles correctly at
/// `--fast` or wrongly at `--release`/`--release-advanced`, so no acorn AST
/// shape distinguishes a corpus file this matters for from one it does not —
/// the corpus matchers run once, over source text, with no concept of build
/// tier at all. Writing a matcher for it would silently claim a frequency
/// this instrument cannot see.
///
/// A second §7 entry, **R-62**, was filed alongside R-61 (the refusing half
/// of the same substitution). It needs no catalogue record: `parse_register`
/// excludes every `### R-` header that appears after a non-tier `## `
/// heading, exactly as it already does for R-50 and R-55, so §7 entries never
/// reach `check_completeness` at all.
///
/// `counts.json` was **not** regenerated — R-61 has no matcher to run and no
/// frequency to publish, and no other entry's matcher body changed, so every
/// figure already in that file is unaffected. This is the first re-freeze in
/// this series that touches `predicates.json` without touching
/// `counts.json`.
const FROZEN_PREDICATES_SHA256: &str =
    "ba31ac0d4a5a480f8e0cfb17949f6e68352909d2d4e8834b23512c0fa3083bd2";
const FROZEN_MATCHERS_SHA256: &str =
    "19d2520978c3fa3afb96f3ca8a2c5035107e623f0925a5e976212fd86cefd701";

#[test]
fn the_frozen_corpus_still_holds_the_programs_it_was_frozen_with() {
    // `the_shipped_corpus_matches_its_manifest` proves the manifest and the
    // directory agree with each other. It cannot notice them being changed
    // *together*: delete programs, re-run the generator, and a self-consistent
    // manifest with a different hash verifies happily. That is exactly the
    // post-hoc corpus edit the freeze exists to prevent, so the frozen numbers
    // are pinned here as constants.
    //
    // If this test fails, the corpus changed after the freeze. The fix is not
    // to update these constants in passing: §4.3 requires a separate,
    // explicitly-justified commit that says why the corpus moved, and the
    // published ranking's `corpus_hash` must be republished with it.
    let root = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/blast-radius/corpus"
    ));
    let text = std::fs::read_to_string(root.join("manifest.json")).expect("manifest is readable");
    let manifest = parse_manifest(&text).expect("manifest parses");

    assert_eq!(
        manifest.files.len(),
        FROZEN_FILE_COUNT,
        "the frozen corpus holds {FROZEN_FILE_COUNT} files; changing that needs its own \
         justified commit"
    );
    let anchor = manifest
        .files
        .iter()
        .filter(|file| file.stratum == "anchor")
        .count();
    let extension = manifest
        .files
        .iter()
        .filter(|file| file.stratum == "extension")
        .count();
    assert_eq!(
        (anchor, extension),
        (FROZEN_ANCHOR_COUNT, FROZEN_EXTENSION_COUNT),
        "the frozen strata are {FROZEN_ANCHOR_COUNT} anchor and {FROZEN_EXTENSION_COUNT} \
         extension programs; a total that still adds up does not make a swap between strata \
         acceptable"
    );
    assert_eq!(
        manifest.corpus_hash, FROZEN_CORPUS_HASH,
        "the frozen corpus_hash is the token every downstream result cites; changing it needs \
         its own justified commit"
    );
}

#[test]
fn the_frozen_catalogue_and_its_matchers_are_the_ones_the_counts_were_taken_with() {
    // If this fails, `predicates.json` or `matchers.mjs` moved after the
    // freeze. The fix is not to update the constant in passing: §4.3 requires
    // a separate, explicitly-justified commit that says why the instrument
    // moved, and `counts.json` and the published ranking must be regenerated
    // with it -- a changed matcher body changes every figure downstream of it
    // while every completeness check stays green.
    let tools = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/blast-radius"
    ));
    for (file, frozen) in [
        ("predicates.json", FROZEN_PREDICATES_SHA256),
        ("matchers.mjs", FROZEN_MATCHERS_SHA256),
    ] {
        let bytes = std::fs::read(tools.join(file))
            .unwrap_or_else(|error| panic!("{file} is readable: {error}"));
        assert_eq!(
            sha256_of(&bytes),
            frozen,
            "{file} is not the file the published counts were taken with; changing it needs \
             its own justified commit, and `counts.json` regenerated with it"
        );
    }
}

#[test]
fn every_frozen_anchor_file_has_committed_provenance() {
    // `manifest.json` records path and hash but not where a program came from,
    // so a reader holding the published corpus_hash cannot audit what was
    // measured from it alone. `anchor-provenance.json` closes that, and this
    // pins the two together so the audit path cannot rot: it is regenerated by
    // `tools/blast-radius/extract_anchor_corpus.py --write-provenance`.
    //
    // Scoped to the `anchor` stratum, because provenance means "the upstream
    // fixture this was extracted from" and only the anchor has one. The
    // extension programs were written for this measurement, so their audit
    // trail is `corpus/README.md`'s curation rule plus the commit that froze
    // them -- not an extraction source. Both directions still hold for the
    // anchor: no anchor file without a provenance row, no provenance row
    // without an anchor file.
    let tools = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/blast-radius"
    ));
    let manifest = parse_manifest(
        &std::fs::read_to_string(tools.join("corpus/manifest.json")).expect("manifest is readable"),
    )
    .expect("manifest parses");
    let provenance: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(tools.join("anchor-provenance.json"))
            .expect("provenance is readable"),
    )
    .expect("provenance parses");
    let rows = provenance.as_array().expect("provenance is an array");
    assert!(
        !rows.is_empty(),
        "an empty provenance table records nothing"
    );

    for row in rows {
        let source = row["source"].as_str().expect("every row names its source");
        assert!(
            tools.join("../..").join(source).exists(),
            "provenance names a source that does not exist: {source}"
        );
    }

    let mut recorded: Vec<&str> = rows
        .iter()
        .map(|row| row["file"].as_str().expect("every row names its file"))
        .collect();
    recorded.sort_unstable();
    let mut frozen: Vec<&str> = manifest
        .files
        .iter()
        .filter(|file| file.stratum == "anchor")
        .map(|file| {
            file.path
                .split('/')
                .next_back()
                .expect("a manifest path has a last segment")
        })
        .collect();
    frozen.sort_unstable();
    assert_eq!(
        recorded, frozen,
        "the provenance table and the frozen anchor list different programs"
    );
    assert!(
        manifest
            .files
            .iter()
            .any(|file| file.stratum == "extension"),
        "the frozen corpus has lost its extension stratum -- the anchor alone \
         cannot carry an accept rate that means anything"
    );
}

// --- The freeze token must not be ambiguous, unchecked, or empty. -----------

/// 64 lowercase hex digits, distinct per `seed`, so a fixture manifest can be
/// built without hashing a real file.
fn digest(seed: u8) -> String {
    sha256_of(&[seed])
}

fn manifest_json(files: &[(&str, &str, String)]) -> String {
    let entries: Vec<ManifestFile> = files
        .iter()
        .map(|(path, stratum, sha)| ManifestFile {
            path: (*path).into(),
            stratum: (*stratum).into(),
            sha256: sha.clone(),
        })
        .collect();
    // Built through `serde_json` rather than `format!`, so control characters
    // and quotes are encoded as JSON demands and the parser sees exactly the
    // bytes the fixture intends.
    let body: Vec<serde_json::Value> = entries
        .iter()
        .map(|file| {
            serde_json::json!({
                "path": file.path,
                "stratum": file.stratum,
                "sha256": file.sha256,
            })
        })
        .collect();
    serde_json::json!({ "corpus_hash": corpus_hash(&entries), "files": body }).to_string()
}

#[test]
fn the_demonstrated_corpus_hash_collision_is_rejected_at_parse_time() {
    // `corpus_hash` joins "{stratum} {path} {sha256}" records with '\n' and
    // escapes nothing, so the encoding is only injective while no field can
    // hold a space or a newline. Two honest files collide with one file whose
    // *name* replays the separator bytes -- and that name is a legal filename
    // ending in `.js`, so `collect_js` finds it and both directions of
    // `verify_manifest` pass. Reaching it needs a hostile filename; the freeze
    // is worth nothing if it relies on a reviewer noticing that.
    let (h1, h2) = (digest(1), digest(2));
    let honest = manifest_json(&[
        ("anchor/x.js", "anchor", h1.clone()),
        ("anchor/y.js", "anchor", h2.clone()),
    ]);
    let smuggled = format!("anchor/x.js {h1}\nanchor anchor/y.js");
    let collision = manifest_json(&[(&smuggled, "anchor", h2)]);

    // The collision is real: the two manifests are different corpora (two
    // files versus one) with byte-identical freeze tokens.
    let of = |text: &str| {
        let value: serde_json::Value = serde_json::from_str(text).expect("json");
        value["corpus_hash"].as_str().expect("hash").to_string()
    };
    assert_eq!(
        of(&honest),
        of(&collision),
        "the collision this test exists to close must actually collide"
    );

    parse_manifest(&honest).expect("the honest two-file manifest still parses");
    let error = parse_manifest(&collision).expect_err("the colliding manifest must be rejected");
    assert!(
        error.contains("path") && error.contains("corpus_hash encoding"),
        "error explains the ambiguity: {error}"
    );
}

#[test]
fn parse_rejects_whitespace_control_and_empty_tokens() {
    // Every one of these is a field the `corpus_hash` encoding cannot separate
    // from its neighbours, so none may reach the digest.
    for (label, path, stratum, expected) in [
        (
            "space in path",
            "anchor/a b.js",
            "anchor",
            "corpus_hash encoding",
        ),
        (
            "newline in path",
            "anchor/a\nb.js",
            "anchor",
            "corpus_hash encoding",
        ),
        (
            "tab in path",
            "anchor/a\tb.js",
            "anchor",
            "corpus_hash encoding",
        ),
        (
            "nul in path",
            "anchor/a\0b.js",
            "anchor",
            "corpus_hash encoding",
        ),
        (
            "space in stratum",
            "anchor/a.js",
            "anch or",
            "corpus_hash encoding",
        ),
        ("empty path", "", "anchor", "empty `path`"),
        ("empty stratum", "anchor/a.js", "", "empty `stratum`"),
    ] {
        let json = manifest_json(&[(path, stratum, digest(1))]);
        let error = parse_manifest(&json).expect_err(label);
        assert!(error.contains(expected), "{label}: {error}");
    }
}

#[test]
fn parse_rejects_a_sha256_that_is_not_64_lowercase_hex() {
    for (label, sha) in [
        ("too short", "abc123".to_string()),
        ("uppercase", digest(1).to_uppercase()),
        ("non-hex", "z".repeat(64)),
        ("too long", format!("{}0", digest(1))),
        ("empty", String::new()),
    ] {
        let json = manifest_json(&[("anchor/a.js", "anchor", sha)]);
        let error = parse_manifest(&json).expect_err("must be rejected");
        assert!(
            error.contains("64 lowercase hex digits"),
            "{label}: {error}"
        );
    }
}

#[test]
fn parse_rejects_a_stratum_that_is_not_the_leading_path_segment() {
    // Accept rates and counts are reported per stratum and never pooled, so a
    // mislabelled file silently moves a program between the two populations.
    let error = parse_manifest(&manifest_json(&[("anchor/a.js", "extension", digest(1))]))
        .expect_err("a mislabelled stratum must be rejected");
    assert!(
        error.contains("leading path segment"),
        "error explains the mismatch: {error}"
    );

    let error = parse_manifest(&manifest_json(&[("a.js", "a.js", digest(1))]))
        .expect_err("a file outside any stratum directory must be rejected");
    assert!(
        error.contains("leading path segment"),
        "error explains the mismatch: {error}"
    );
}

#[test]
fn parse_rejects_a_path_listed_twice() {
    // A duplicate entry verifies clean against disk -- the same file is read
    // and hashed twice -- but the counter would count the program twice.
    let error = parse_manifest(&manifest_json(&[
        ("anchor/a.js", "anchor", digest(1)),
        ("anchor/a.js", "anchor", digest(1)),
    ]))
    .expect_err("a duplicate path must be rejected");
    assert!(error.contains("listed twice"), "error names it: {error}");
}

#[test]
fn parse_rejects_an_empty_file_list() {
    let error = parse_manifest("{\"corpus_hash\": \"\", \"files\": []}")
        .expect_err("an empty corpus must not read as frozen");
    assert!(
        error.contains("empty corpus is not a freeze"),
        "error says why: {error}"
    );
}

#[test]
fn verify_rejects_a_manifest_whose_recorded_corpus_hash_is_wrong() {
    // The freeze token is the whole provenance claim of the published ranking.
    // A manifest that agrees with disk in both directions but carries the wrong
    // corpus_hash would stamp a value that measured something else.
    let dir = scratch_dir("wrong-hash-test");
    std::fs::create_dir_all(dir.join("anchor")).expect("mkdir");
    std::fs::write(dir.join("anchor/x.js"), "console.log(1);\n").expect("write");

    let file = ManifestFile {
        path: "anchor/x.js".into(),
        stratum: "anchor".into(),
        sha256: sha256_of("console.log(1);\n".as_bytes()),
    };
    let honest = Manifest {
        corpus_hash: corpus_hash(std::slice::from_ref(&file)),
        files: vec![file.clone()],
    };
    verify_manifest(&dir, &honest).expect("the honest manifest verifies");

    let lying = Manifest {
        corpus_hash: digest(9),
        files: vec![file],
    };
    let error =
        verify_manifest(&dir, &lying).expect_err("a wrong freeze token must not verify clean");
    assert!(
        error.contains("does not match its own file list"),
        "error names the freeze token: {error}"
    );

    std::fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn verify_rejects_an_empty_manifest_over_an_empty_directory() {
    // Both directions of the on-disk check are vacuously satisfied here: no
    // recorded file is missing, and no untracked file is present. Without the
    // non-empty guard *inside* `verify_manifest`, a corpus of nothing verifies
    // as frozen -- ran-nothing-green at the level of the instrument itself.
    let dir = scratch_dir("empty-corpus-test");
    std::fs::create_dir_all(&dir).expect("mkdir");

    let empty = Manifest {
        corpus_hash: corpus_hash(&[]),
        files: Vec::new(),
    };
    let error = verify_manifest(&dir, &empty).expect_err("an empty corpus must not verify");
    assert!(
        error.contains("empty corpus is not a freeze"),
        "error says why: {error}"
    );

    std::fs::remove_dir_all(&dir).expect("cleanup");
}
