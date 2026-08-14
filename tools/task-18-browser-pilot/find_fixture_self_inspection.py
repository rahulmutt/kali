#!/usr/bin/env python3
"""Find controller ruling 4's fixture-self-inspection blind spot in `browser_*.rs`.

WHAT THIS REPLACES, AND WHY. Batch 5 proposed the predicate "which `#[test]` fns
never construct a `Command`", and it was briefly promoted to ruling 10. It is
WRONG, in the direction that matters. When the self-inspecting
`assert!(source.contains(...))` lives inside the same assert helper that also
builds the `Command` -- which is the commoner arrangement -- every `#[test]`
transitively constructs one and the predicate returns nothing. It misses
`browser_promise_any_bundle.rs` and `browser_promise_any_harness.rs`, and it
fails to re-find FOUR already-adjudicated retentions
(`browser_array_from_set_map_bundle.rs`, `browser_array_from_set_map_harness.rs`,
`browser_generator_default_export_rejection.rs`,
`browser_math_pow_exponent_one.rs`). Batch 5's three hits were real; they just
happened to put the self-inspection in a standalone Command-free test, and
"it returned exactly the three I already knew about" bounds OVER-reporting only.
A predicate validated only against its own sample is untested in the direction
that matters, which is why `--selftest` below asserts against KNOWN, listing
every previously-adjudicated instance as ground truth.

THE CORRECTED PREDICATE (ruling 10, as amended): a `.contains()` whose RECEIVER
is a fixture-builder's return value, reachable from any `#[test]`, regardless of
whether a `Command` is constructed. The receiver is resolved three ways, because
the corpus uses all three:

  1. direct        `browser_x_source().contains(...)`
  2. local binding `let source = browser_x_source(); ... source.contains(...)`
     -- including a CONDITIONAL initializer,
     `let source = if command == "test" { a() } else { b() };`, which two files
     use and which a "call immediately after `=`" match does not see
  3. parameter     `fn helper(source: &str) { ... source.contains(...) }`,
                   where SOME call site passes a fixture builder's value, a
                   fixture-valued local, OR AN INLINE STRING LITERAL

(3) is the one the old predicate could not see and is why the four adjudicated
retentions were missed: their self-inspection reads a parameter. The
string-literal clause of (3) is required by
`browser_generator_default_export_rejection.rs`, whose callers pass the program
under test inline rather than through a builder -- the receiver is still fixture
text, and it is still invisible to the audit for the same reason. It cannot
misfire on an output assertion, because the predicate only triggers when the
parameter IS the `.contains` receiver; a helper taking an expected-output
literal reads it as the ARGUMENT (`stdout.contains(expected)`), never the
receiver.

A "fixture builder" is a fn returning `&'static str` or `String` that neither
spawns a process nor reads the environment -- that exclusion is what keeps
`kali_bin()` out, and with it every `stdout.contains(...)` in the corpus, which
is an assertion about OUTPUT and precisely not this shape.

KNOWN-UNHANDLED RECEIVER SHAPES. This is a heuristic over masked text, not a
Rust front end, and the following shapes are NOT resolved. Each was probed
synthetically and then searched for in the corpus; all but one are absent from
`browser_*.rs` today, so there is no live loss, but a future file using one is
invisible here and the list is what a later reader needs in order to judge that.

CORRECTED, batch 6 (recorded instrument defect 2). This used to read "None
occurs in `browser_*.rs` today", and that is FALSE for the intervening-attribute
bullet: `browser_cdp_smoke.rs` has `#[test]` at `:103`, `#[ignore = "..."]` at
`:104` and the `fn` at `:105`, which is exactly the shape. It is still not a
live loss -- that file's only `.contains` is `relative.contains("..")` at `:82`,
whose receiver is a path component and not fixture text -- but the sentence was
wrong, and a blanket "none occurs" is precisely the kind of claim a later reader
would rely on instead of re-measuring. The corpus is swept with the exact list
below, not with this sentence.

  * a fixture held in a struct field
  * a method chain through `as_str()` (or any non-string-slicing adapter)
  * a `let` RE-binding of an already-fixture-valued local
  * a fixture reached through a `Vec` index
  * further members of the string-slicing family beyond lines/matches/
    starts_with (deliberately excluded -- see the site block for the sweep)
  * a builder returning `Cow<'static, str>`, or a non-`'static` `&str`
  * a builder declared with a `where` clause, or as `pub fn`
  * `#[test]` with another attribute between it and the `fn`
  * `#[test]` inside an inline `mod`

SCOPE. The default scan is `crates/kali_cli/tests/browser_*.rs` only. The seven
`browser_*` `#[path]` submodule DIRECTORIES are outside it entirely. Measured
rather than assumed: running this predicate over them returns 0 hits, so nothing
is lost today.

CORRECTED, batch 6 (recorded instrument defect 1). This paragraph used to say
"all 59 `.rs` files in those directories", and to give as the REASON that
receivers there "include `source`, `script`, `js` and `body`" but are not bound
to fixture builders. Both halves were measured over the wrong file set, and the
stated reason is simply false. State the set with every figure:

  * The seven `browser_*` `#[path]` directories hold **18** `.rs` files
    (non_literal_iterator_sources 4, object_keys_iteration 2, reflect_own_keys
    4, and runtime_summary_fallback_{js,jsx,ts,tsx}_input 2 each).
  * **59** is the count of `.rs` files under EVERY `tests/` subdirectory --
    which also sweeps in `inprocess/`, `runtime_smoke/`, `package_corpus/`,
    `schema_docs/`, `node_api_surface/` and the `late_compat_browser_*`
    directories. Those are not `browser_*` `#[path]` targets and were never in
    scope for this predicate.
  * The `.contains` RECEIVER census over the 18 is `{'stdout': 40}` -- 40 of 40,
    every one of them process output. So the true reason nothing is lost is the
    strongest one available, and the opposite of what was written: there is no
    fixture-text receiver in those files at all, not merely no fixture-BOUND
    one.

The substantive claim (0 hits) survives over both the 18 and the 59; only the
rationale was wrong, and it sat in a load-bearing docstring. If those
directories are ever migrated, scan them explicitly.

Usage:
  find_fixture_self_inspection.py --selftest        # ground-truth check, exit 1 on miss
  find_fixture_self_inspection.py [FILE.rs ...]     # default: every browser_*.rs
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")

from enumerate_invocations import strip_block_comments_and_strings  # noqa: E402

# Every instance adjudicated before this tool existed. `--selftest` requires the
# predicate to re-find ALL of them. Four were invisible to the superseded
# predicate; three are batch 5's own; two were named by the batch-5 review as
# unmigrated targets carrying the shape.
KNOWN = {
    # adjudicated retentions the SUPERSEDED predicate could not see (receiver is
    # a param, or the helper also builds the Command)
    "browser_array_from_set_map_bundle.rs",
    "browser_array_from_set_map_harness.rs",
    "browser_generator_default_export_rejection.rs",
    "browser_math_pow_exponent_one.rs",
    # batch 5's three (self-inspection in a standalone Command-free test)
    "browser_math_max_min_frozen_aliases.rs",
    "browser_math_pow_bracketed_frozen_wrapper.rs",
    "browser_math_pow_bracketed_frozen_wrapper_harness.rs",
    # batch 3 / batch 4 adjudicated retentions. OMITTED from the first version of
    # this set, which is why its selftest passed 9/9 while the predicate was blind
    # to a fourteenth instance: the one file it could not find was also the one
    # left out of the ground truth. Ground truth chosen after the fact grades
    # nothing.
    "browser_math_abs_sign_frozen_aliases.rs",
    "browser_math_atan2_global_this_root.rs",
    "browser_math_floor_trunc_ceil_aliases.rs",
    "browser_math_floor_trunc_ceil_bundle.rs",
    # the fourteenth: its `.contains` sits on a closure parameter, with `.lines()`
    # between the fixture binding and the call. Found only once the site match was
    # widened past a direct `.contains` on the bound identifier.
    "browser_array_iteration_spread.rs",
    # unmigrated batch 6-8 targets carrying the shape
    "browser_promise_any_bundle.rs",
    "browser_promise_any_harness.rs",
    # Task 19 batch 5's U4 trim, and the instance that proved the predicate had a
    # third blind spot: its self-inspection is `assert_eq!(source.matches(alias)
    # .count(), 2, ..)` with no `.contains` anywhere, which both existing arms
    # required. Ruling 10 says every newly adjudicated instance goes in here or
    # the selftest silently weakens as the corpus grows.
    "for_of_array_iteration_spread.rs",
}


def fn_spans(masked):
    """(name, start, body_start, end) for every top-level `fn`."""
    out = []
    for m in re.finditer(r"^fn\s+(\w+)", masked, re.M):
        brace = masked.find("{", m.end())
        if brace == -1:
            continue
        depth, i, n = 0, brace, len(masked)
        while i < n:
            if masked[i] == "{":
                depth += 1
            elif masked[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        out.append((m.group(1), m.start(), brace, i + 1))
    return out


def signature(text, start, body_start):
    return text[start:body_start]


def fixture_builders(masked, spans):
    """Fns whose return value is program text, not process output."""
    out = set()
    for name, start, body_start, end in spans:
        sig = signature(masked, start, body_start)
        if not re.search(r"->\s*(&'static\s+str|String)\s*$", sig.strip()):
            continue
        body = masked[body_start:end]
        if "Command::new" in body or "env::var" in body:
            continue          # kali_bin() and friends: not fixture text
        out.add(name)
    return out


def split_args(text):
    """Top-level comma split of a call's argument text."""
    args, depth, cur = [], 0, ""
    for ch in text:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == "," and depth == 0:
            args.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        args.append(cur.strip())
    return args


def call_args(masked, fn_name):
    """Argument lists of every call to `fn_name`, with the caller's offset."""
    out = []
    for m in re.finditer(r"\b" + re.escape(fn_name) + r"\s*\(", masked):
        i, depth = m.end() - 1, 0
        n = len(masked)
        while i < n:
            if masked[i] == "(":
                depth += 1
            elif masked[i] == ")":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        out.append((m.start(), split_args(masked[m.end():i])))
    return out


def params(sig):
    """Parameter names, in order, from a fn signature."""
    inner = sig[sig.find("(") + 1:sig.rfind(")")] if "(" in sig else ""
    names = []
    for a in split_args(inner):
        m = re.match(r"(?:mut\s+)?(\w+)\s*:", a)
        names.append(m.group(1) if m else None)
    return names


def analyse(path):
    raw = open(path).read()
    masked = strip_block_comments_and_strings(raw)
    spans = fn_spans(masked)
    if not spans:
        return None
    builders = fixture_builders(masked, spans)
    by_name = {n: (s, b, e) for n, s, b, e in spans}

    def owner(off):
        for n, s, b, e in spans:
            if s <= off < e:
                return n
        return None

    # fixture-valued locals, then fixture-valued params, to a fixed point
    fixture_vals = {n: set() for n in by_name}
    for _ in range(4):
        changed = False
        # locals: scan the WHOLE initializer, to its depth-0 `;`, not just the
        # token right after `=`. Two files bind the fixture through an
        # `if`/`else`, which the narrow form silently skipped.
        for name, (s, b, e) in by_name.items():
            body = masked[b:e]
            for m in re.finditer(r"\blet\s+(?:mut\s+)?(\w+)[^=;]*=", body):
                var = m.group(1)
                if var in fixture_vals[name]:
                    continue
                i, depth = m.end(), 0
                while i < len(body):
                    c = body[i]
                    if c in "([{":
                        depth += 1
                    elif c in ")]}":
                        depth -= 1
                    elif c == ";" and depth == 0:
                        break
                    i += 1
                init = body[m.end():i]
                if any(re.search(r"\b" + re.escape(fb) + r"\s*\(", init) for fb in builders):
                    fixture_vals[name].add(var)
                    changed = True
        # params, from call sites
        for callee, (s, b, e) in by_name.items():
            pnames = params(signature(masked, s, b))
            for off, args in call_args(masked, callee):
                if s <= off < e:
                    continue                      # the definition itself
                caller = owner(off)
                for idx, arg in enumerate(args):
                    if idx >= len(pnames) or pnames[idx] is None:
                        continue
                    head = re.match(r"&?\s*([A-Za-z_][\w:]*)", arg)
                    if head:
                        tok = head.group(1).split("::")[-1]
                        is_fix = tok in builders or (
                            caller is not None and tok in fixture_vals.get(caller, ()))
                    else:
                        # No identifier at all. In the masked text a string
                        # literal's body is blanked, so an argument that is
                        # entirely whitespace IS an inline literal -- the
                        # program under test passed by hand rather than built.
                        is_fix = arg.strip() == ""
                    if is_fix and pnames[idx] not in fixture_vals[callee]:
                        fixture_vals[callee].add(pnames[idx])
                        changed = True
        if not changed:
            break

    # call graph reachability from #[test] fns
    tests = []
    for m in re.finditer(r"#\[test\]\s*\nfn\s+(\w+)", masked):
        tests.append(m.group(1))
    reach, frontier = set(tests), list(tests)
    while frontier:
        cur = frontier.pop()
        if cur not in by_name:
            continue
        s, b, e = by_name[cur]
        body = masked[b:e]
        for other in by_name:
            if other != cur and other not in reach and re.search(
                    r"\b" + re.escape(other) + r"\s*\(", body):
                reach.add(other)
                frontier.append(other)

    # The sites. Two shapes: `fixture.contains(...)` directly, and a fixture
    # receiver that takes a string-slicing method FIRST and reaches `.contains`
    # further along the chain -- `source.lines().filter(|l| l.contains(..))`,
    # where the `.contains` receiver is a closure parameter and no amount of
    # receiver-name matching finds it.
    #
    # The chain set is closed at lines/matches/starts_with, and that boundary was
    # swept rather than guessed: `+lines` adds exactly one file
    # (browser_array_iteration_spread.rs), `+matches` and `+starts_with` add
    # nothing on today's corpus but are near-miss shapes of the same idea, and
    # extending to ends_with/split/find admits a false positive --
    # `let source = if filename.ends_with(".js")` in
    # browser_object_enumeration_wrapped_bundle.rs, where the receiver is a
    # literal-fed parameter and the `.contains` further down is about stderr.
    sites = []
    CHAIN = r"lines\s*\(\s*\)|matches\s*\([^)]*\)|starts_with\s*\([^)]*\)"
    # THE THIRD SHAPE, added in Task 19 batch 5, and it was a LIVE FALSE
    # NEGATIVE rather than a hardening. Both arms below require a `.contains`
    # somewhere, so a self-inspection that counts instead of testing membership
    # -- `assert_eq!(source.matches(alias).count(), 2, ..)` -- returned nothing.
    # `for_of_array_iteration_spread.rs` spells it exactly that way, and this
    # tool reported 0 hits on it while its browser twin
    # (`browser_array_iteration_spread.rs`, which spells the same shape with
    # `.lines().filter(|l| l.contains(..))`) was found and adjudicated. The gap
    # let a dispatch list that target as migratable-whole. The receiver gate is
    # unchanged and is what keeps this off process output: `stdout.matches(x)
    # .count()` cannot match, because `stdout` is not a fixture builder's value.
    TERMINAL_COUNT = re.compile(
        r"([A-Za-z_][\w:]*)\s*(?:\(\s*\))?\s*\.\s*matches\s*\([^)]*\)"
        r"\s*\.\s*count\s*\(\s*\)")
    direct = re.compile(r"([A-Za-z_][\w:]*)\s*(?:\(\s*\))?\s*\.contains\s*\(")
    via_chain = re.compile(
        r"([A-Za-z_][\w:]*)\s*(?:\(\s*\))?\s*((?:\.\s*(?:" + CHAIN + r")\s*)+)")
    seen_offsets = set()
    for m in direct.finditer(masked):
        recv = m.group(1).split("::")[-1]
        host = owner(m.start())
        if host is None or host not in reach:
            continue
        if not (recv in builders or recv in fixture_vals.get(host, ())):
            continue
        seen_offsets.add(m.start())
        sites.append((raw[:m.start()].count("\n") + 1, host, recv))
    for m in via_chain.finditer(masked):
        recv = m.group(1).split("::")[-1]
        host = owner(m.start())
        if host is None or host not in reach or m.start() in seen_offsets:
            continue
        if not (recv in builders or recv in fixture_vals.get(host, ())):
            continue
        # bounded lookahead: the `.contains` must belong to this chain
        if ".contains" not in masked[m.end():m.end() + 200]:
            continue
        sites.append((raw[:m.start()].count("\n") + 1, host,
                      recv + " (via " + m.group(2).strip().split("(")[0].lstrip(". ") + ")"))

    for m in TERMINAL_COUNT.finditer(masked):
        recv = m.group(1).split("::")[-1]
        host = owner(m.start())
        if host is None or host not in reach or m.start() in seen_offsets:
            continue
        if not (recv in builders or recv in fixture_vals.get(host, ())):
            continue
        seen_offsets.add(m.start())
        sites.append((raw[:m.start()].count("\n") + 1, host,
                      recv + " (via matches().count())"))

    if not sites:
        return None
    hosts = {h for _, h, _ in sites}
    reaching = []
    for t in tests:
        if t in hosts:
            reaching.append(t)
            continue
        s, b, e = by_name[t]
        body = masked[b:e]
        # transitive: does this test reach a hosting fn?
        seen, stack = set(), [x for x in by_name if re.search(
            r"\b" + re.escape(x) + r"\s*\(", body)]
        hit = False
        while stack:
            cur = stack.pop()
            if cur in seen:
                continue
            seen.add(cur)
            if cur in hosts:
                hit = True
                break
            if cur in by_name:
                s2, b2, e2 = by_name[cur]
                stack += [x for x in by_name if x not in seen and re.search(
                    r"\b" + re.escape(x) + r"\s*\(", masked[b2:e2])]
        if hit:
            reaching.append(t)
    return {"sites": sites, "tests": len(tests), "reaching": len(reaching),
            "hosts": sorted(hosts)}


def superseded_hits(paths):
    """The predicate this tool replaced: files with a Command-free `#[test]`.

    Kept, and run as a selftest case, for one reason: a replacement detector must
    be a SUPERSET of what it replaces, and nobody checked that the first time. A
    predicate was retired for a false-negative class and one with a DIFFERENT
    false-negative class accepted in its place -- the superseded predicate would
    have found browser_array_iteration_spread.rs, which the replacement missed.
    Containment is now asserted rather than assumed.
    """
    out = set()
    for path in paths:
        masked = strip_block_comments_and_strings(open(path).read())
        spans = fn_spans(masked)
        by = {n: (s, b, e) for n, s, b, e in spans}
        tests = [m.group(1) for m in re.finditer(r"#\[test\]\s*\nfn\s+(\w+)", masked)]
        for tname in tests:
            if tname not in by:
                continue
            seen, stack, spawns = set(), [tname], False
            while stack:
                cur = stack.pop()
                if cur in seen or cur not in by:
                    continue
                seen.add(cur)
                s, b, e = by[cur]
                body = masked[b:e]
                if "Command::new" in body:
                    spawns = True
                    break
                stack += [o for o in by if o not in seen and
                          re.search(r"\b" + re.escape(o) + r"\s*\(", body)]
            if not spawns:
                out.add(os.path.basename(path))
                break
    return out


def main(argv):
    selftest = "--selftest" in argv
    argv = [a for a in argv[1:] if not a.startswith("--")]
    paths = ([os.path.join(TESTS, f) for f in sorted(os.listdir(TESTS))
              if f.startswith("browser_") and f.endswith(".rs")]
             if not argv else [os.path.abspath(a) for a in argv])
    if selftest:
        # KNOWN is no longer a subset of `browser_*.rs`: batch 5's instance is a
        # non-browser target. A selftest that scans only the browser glob would
        # report it "gone" and pass by exclusion, which is the ground-truth
        # failure this whole module exists to avoid.
        paths = sorted(set(paths) | {os.path.join(TESTS, n) for n in KNOWN
                                     if os.path.exists(os.path.join(TESTS, n))})
    found, paths_by_name = {}, {}
    for p in paths:
        r = analyse(p)
        if r:
            found[os.path.basename(p)] = r
            paths_by_name[os.path.basename(p)] = p

    # "browser_*.rs" was true while `paths` was only the browser glob. `KNOWN`
    # is no longer a subset of it (batch 5 adjudicated a non-browser instance)
    # and `--selftest` unions `KNOWN` in, so the count is over FILES SCANNED --
    # which is also what it always was when the tool is given paths on argv.
    print(f"{len(found)} of {len(paths)} file(s) scanned carry the "
          f"fixture-self-inspection shape\n")
    todo = []
    for name in sorted(found):
        r = found[name]
        disp = ("WHOLE-FILE retention" if r["reaching"] == r["tests"]
                else f"U4 TRIM-AND-KEEP: retain {r['reaching']}, migrate "
                     f"{r['tests'] - r['reaching']}")
        rs_path = paths_by_name[name]
        adjudicated = open(rs_path).read().startswith("//!")
        stem = name[len("browser_"):-3]
        has_toml = os.path.exists(os.path.join(TESTS, "cases/browser", stem + ".toml"))
        status = ("ALREADY ADJUDICATED (carries a //! retention header"
                  + (", migrated remainder shipped)" if has_toml else ")")
                  if adjudicated else "NOT YET ADJUDICATED -- scope this into a batch")
        if not adjudicated:
            todo.append(name)
        print(f"{name}")
        print(f"    {len(r['sites'])} site(s) in {', '.join(r['hosts'])}; "
              f"{r['reaching']} of {r['tests']} #[test] fns reach it -> {disp}")
        print(f"    first site line {r['sites'][0][0]}, receiver `{r['sites'][0][2]}`")
        print(f"    {status}")
    print(f"\nUNADJUDICATED: {len(todo)} -> {todo}")
    print(
        "\nCAVEAT ON THE PRINTED DISPOSITION: it reflects THIS shape only. A file may\n"
        "also be blocked by an unrelated design-spec 5.11 ground, in which case the trim\n"
        "this tool suggests is not available. browser_generator_default_export_rejection.rs\n"
        "is the standing example -- 16 of its 28 tests reach the fixture self-inspection,\n"
        "so this tool proposes a 16/12 trim, but all 28 reach an errors-array quantifier\n"
        "as well and its adjudicated disposition is whole-file retention. Run the other\n"
        "5.11 scans before acting on a trim proposed here.")

    if selftest:
        names = set(found)
        missing = sorted(n for n in KNOWN if n not in names and
                         os.path.exists(os.path.join(TESTS, n)))
        gone = sorted(n for n in KNOWN if not os.path.exists(os.path.join(TESTS, n)))
        print("\n--- SELFTEST against previously adjudicated ground truth ---")
        for n in sorted(KNOWN):
            if n in gone:
                print(f"  SKIP  {n} (no longer in tree)")
            elif n in names:
                print(f"  FOUND {n}")
            else:
                print(f"  MISS  {n}")
        sup = superseded_hits(paths)
        not_contained = sorted(sup - names)
        print("\n--- CONTAINMENT against the superseded predicate ---")
        print(f"  superseded predicate hits {len(sup)} file(s); "
              f"{len(sup & names)} of them are also found here")
        for n in not_contained:
            print(f"  NOT CONTAINED: {n}")
        if missing or not_contained:
            if missing:
                print(f"\nSELFTEST FAILED — {len(missing)} known instance(s) not found: "
                      f"{missing}")
            if not_contained:
                print(f"SELFTEST FAILED — the replacement is not a superset of the "
                      f"predicate it replaced: {not_contained}")
            return 1
        print(f"\nSELFTEST OK — all {len(KNOWN) - len(gone)} known instances re-found, "
              f"and the superseded predicate's hits are a strict subset")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
