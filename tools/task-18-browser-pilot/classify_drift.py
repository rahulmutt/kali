#!/usr/bin/env python3
"""Regenerate every generator, classify any drift, and GATE on it.

WHAT THIS EXISTS FOR (batch 7A gap 3). The finding that 25 of 26 drifting case
files differed from their generator's output in CITATION FORM ONLY, and exactly
one differed in content, lived only as prose in a report. Prose is not an
instrument, and one instrument had already got this exact measurement wrong:
its normaliser collapsed the citation construct but not the RE-WRAPPING that
deleting text causes, so pure reflow read as content drift and it reported 6/20
instead of 25/1. Adding a reflow control gave 25/1. A description that has
already been mis-implemented once must not be re-implemented from prose again,
so the measurement is committed as code, with the reflow control in its
selftest.

WHAT IT CHECKS. For every generator in the derived population:

  * it runs, exit 0 (a crashing generator is a gate failure, not a skip);
  * every case file it writes is byte-identical to the shipped one, or else the
    difference is classified;
  * the two enumerated sets -- citation-form-only, and content-drift -- equal
    their DECLARED enumerations.

The declarations are enumerations and not counts, deliberately: ruling 15
rejects a bare figure, and ruling 16 says a family-wide count has no gateable
home. `25/1` tells a later reader nothing they can act on; the two SETS tell
them exactly which files to look at, and the gate compares them against its own
output every run, so an unrelated edit that adds drift fails here by name.

TWO METHODS, AND THEY MUST AGREE. This is the structure that settled the
measurement the first time, and reproducing it is the cheapest defence against
another false-green harness:

  M1 uses NO normaliser. A lost reword can only DELETE text -- its whole effect
     is inserting `` `snippet` `` before a number -- so M1 asks whether the
     regenerated token stream is a SUBSEQUENCE of the shipped one, with zero
     insertions and every deleted run one or more complete backticked spans.
     Header tokens drop whitespace, because reflow legitimately moves the
     header's line breaks; BODY tokens keep it, because nothing re-wraps a
     `rationale` or a `[source]` fixture, so a whitespace edit there is a
     rewritten program under test (rule 9), not reflow.

  M2 is a from-scratch normaliser: delete every reword construct, then compare
     the header with its wrapping undone (this is the REFLOW CONTROL -- the step
     the first instrument lacked) and the body WITHOUT any whitespace collapse.

They share only `_deflow`, which undoes the `# ` markers and a hyphenated
re-wrap. That is not the reflow control; it is the precondition for comparing a
re-wrapped header at all, and both methods would be unable to run without it.

They are independent implementations of the same question. If they disagree the
run RAISES rather than picking one, per ruling 18's "make a non-match an error":
a classifier whose two halves silently diverge is exactly the instrument this
gate exists to replace.

Usage:
  classify_drift.py              regenerate everything, classify, gate
  classify_drift.py --selftest   poisoned probes for both methods (no tree writes)
  classify_drift.py --only NAME  one generator (still restores and proves)

Exit 0 only if every generator runs, the census matches both declarations, and
the tree is restored byte-for-byte afterwards.
"""

import hashlib
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
CASES = os.path.join(REPO, "crates/kali_cli/tests/cases/browser")
CASES_REL = "crates/kali_cli/tests/cases/browser"


# --------------------------------------------------------------------------
# The declarations. Ruling 15 answer 1: the figure is the gate's own output,
# compared against a declaration every run. Enumerations, never counts.
# --------------------------------------------------------------------------

# Case files whose regenerated form differs from the shipped one ONLY in
# citation form -- the lost `reword_ungated_citations` post-pass. Empty since
# batch 8-inst-1 folded the reword into `case_emit.write`, which is the whole
# point of that fold: a generator that emits the gated form leaves nothing here.
CITATION_FORM_ONLY_DECLARED = []

# Case files whose regenerated form differs in CONTENT. Empty since batch
# 8-inst-1's step 3 brought `gen_batch5_group_c` and `gen_batch4_group_a` up to
# the two shipped texts batch 7 fix round 1 (M8, 32fb3e3fab) had corrected in
# review and never folded back.
CONTENT_DRIFT_DECLARED = []

# Generators that are expected to fail. There are none, and there is no
# mechanism to add one: a generator that cannot run cannot describe the tree,
# which is the property this gate exists to hold.


# --------------------------------------------------------------------------
# Classification
# --------------------------------------------------------------------------

# That insertion in situ: the span, its separating space, then the number, with
# NOTHING between them. The narrowness is load-bearing and was measured, not
# assumed: a tolerant `(?=\(?:\d)` also matches a span a GENERATOR wrote in
# front of `(:N)` -- `math_expm1_log1p_frozen_aliases` carries
# `` `build_emits_..._input` (:224-229) `` -- and it matches it on the
# regenerated side only, because on the shipped side the reword has since put
# its own span between the two. Stripping one side and not the other reported a
# false content drift on a file whose only difference was the reword.
# `gen_batch7a.cite`'s `` `snippet` (:N) `` spelling needs no stripping at all:
# both sides carry it, because the generator writes it.
_REWORD_INSERT = re.compile(r"`[^`\n]*` (?=:\d)")


def _strip_marker(text):
    """Drop the `# ` comment marker from every header line.

    This is not a normaliser in M2's sense and M1 is entitled to it: reflow
    changes how many LINES the header occupies, so the marker count is a
    property of the wrapping rather than of the prose. Nothing else is touched.
    """
    return re.sub(r"(?m)^#[ \t]?", "", text)


def _deflow(text):
    """The `#` header block as one line: markers gone, wrapping undone.

    BOTH methods need this and it is not the reflow control -- it is what makes
    the reflow control possible. Two things must be undone before the header can
    be compared at all:

      * the `# ` markers, whose COUNT is a property of the wrapping;
      * a HYPHENATED re-wrap. `textwrap` breaks `live-captured` across lines when
        it lands on the margin, so the same words wrapped at two widths differ by
        a `-\n`. Ruling 18 records exactly this mutation silencing three gate
        arms, and a classifier that does not undo it reports a false content
        drift on a pure re-wrap -- which is the 6/20 bug in a second disguise.

    A backticked span containing a space is also re-joined by this, which matters
    because such a span IS split by wrapping and `_ATOM` only recognises a span
    that closes on its own line.
    """
    text = _strip_marker(text)
    # A SINGLE hyphen between two word characters, broken by the wrap. The
    # guards are load-bearing in both directions: without the left-hand
    # `[^\s-]` this eats the space in this family's `--` em-dash surrogate
    # (`-- rule 2` became `--rule 2` on whichever side happened to wrap there,
    # and only that side), and without the right-hand `\w` it would join a
    # trailing dash to punctuation.
    text = re.sub(r"(?<=[^\s-])-\n[ \t]*(?=\w)", "-", text)
    return " ".join(text.split())


def _header_split(text):
    """(leading `#` comment block, everything after it).

    Only the header block is ever re-wrapped: `textwrap` runs over header prose,
    while a `rationale` is one long line and a `[source]` body is program text.
    So reflow is tolerated on the left of this split and forbidden on the right.
    """
    lines = text.split("\n")
    i = 0
    while i < len(lines) and (lines[i].startswith("#") or not lines[i].strip()):
        i += 1
    return "\n".join(lines[:i]), "\n".join(lines[i:])


def _runs(a, b):
    """(deleted runs, inserted runs) between two token sequences.

    A real diff, not a greedy subsequence walk. Greedy matching re-aligns on the
    first token that happens to repeat and then reports run boundaries that are
    nonsense -- the first attempt at this reported
    `'`assert_browser_iterator_source_rejects'` as a deleted "run" and made
    every donor look like content drift. `difflib` is asked instead.
    """
    import difflib
    deleted, inserted = [], []
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(
            None, a, b, autojunk=False).get_opcodes():
        if tag in ("delete", "replace"):
            deleted.append(a[i1:i2])
        if tag in ("insert", "replace"):
            inserted.append(b[j1:j2])
    return deleted, inserted


# One or more whole reword spans back to back: a single deleted run can hold
# several, because two citations can sit adjacently in one sentence.
_SPAN_RUN = re.compile(r"(?:`[^`\n]*`\s+)+")


# The atom a token stream is made of. A backticked span is ONE atom and a `:N`
# citation is another, and both matter: with a plain whitespace split, the
# shipped `` (`json["command"]` :58), `` tokenises as two atoms and the
# un-reworded `(:58),` as one, so the diff reports a REPLACE where the only real
# change is a deletion -- which reads as content drift and is the same class of
# mistake as the 6/20 normaliser. Punctuation is split off so the parenthesis
# around a citation is not welded to it.
_ATOM = re.compile(r"`[^`\n]*`|:\d+(?:-\d+)?|[A-Za-z_][A-Za-z0-9_]*|\s+|\S")


def _tokens(text, keep_space):
    """Header tokens drop whitespace; body tokens keep it.

    That difference is the whole of M1's strictness. Reflow legitimately moves
    the header's line breaks once text of a different length is inserted, so
    whitespace there carries no information. Nothing re-wraps a `rationale` or a
    `[source]` fixture, so a whitespace edit in the body is a rewritten program
    under test (rule 9), not reflow, and must not be tokenised away.
    """
    atoms = _ATOM.findall(text)
    return atoms if keep_space else [a for a in atoms if a.strip()]


def m1(shipped, regen):
    """NO normaliser. Deletions only, and every deleted run a whole reword span.

    A lost reword can only DELETE text -- its entire effect is inserting
    `` `snippet` `` before a number -- so the question is whether the
    regenerated token stream is a SUBSEQUENCE of the shipped one, with zero
    insertions and with every deleted run one or more complete backticked spans.
    Anything else is content.
    """
    s_head, s_body = _header_split(shipped)
    r_head, r_body = _header_split(regen)
    total = 0
    for label, keep_space, a_text, b_text in (
            ("header", False, _deflow(s_head), _deflow(r_head)),
            ("body", True, s_body, r_body)):
        a, b = _tokens(a_text, keep_space), _tokens(b_text, keep_space)
        deleted, inserted = _runs(a, b)
        joiner = "" if keep_space else " "
        if inserted:
            return "CONTENT_DRIFT", [
                f"M1: {len(inserted)} INSERTION(s) in the {label} -- the reword only ever "
                f"deletes. First: {joiner.join(inserted[0])[:60]!r}"]
        bad = [joiner.join(r) for r in deleted
               if not _SPAN_RUN.fullmatch(joiner.join(r).strip() + " ")]
        if bad:
            return "CONTENT_DRIFT", [
                f"M1: {label} deletion is not a reword span: {x[:70]!r}" for x in bad[:4]]
        total += len(deleted)
    return "CITATION_FORM_ONLY", [
        f"M1: {total} deleted token-run(s), every one a complete backticked span, "
        f"0 insertions"]


# The same construct for the normaliser, tolerant only of the whitespace
# `_deflow` may have collapsed. See `_REWORD_INSERT` for why it must not reach
# across a `(`.
_REWORD = re.compile(r"`[^`\n]*` +(?=:\d)")


def m2(shipped, regen):
    """From-scratch normaliser, WITH the reflow control -- and only there.

    Whitespace collapse IS the reflow control: it is the step the first
    instrument lacked, and without it a pure re-wrap -- which deleting text
    inside a wrapped paragraph always causes -- reads as content drift. It is
    applied to the `#` header block ONLY. Collapsing whitespace in the body too
    would excuse a whitespace edit inside a `[source]` fixture, which is a
    rewritten program under test (rule 9), not reflow: nothing re-wraps a
    `rationale` or a fixture body, so there is no reflow there to control for.
    """
    def head_norm(text):
        return _REWORD.sub("", _deflow(text))

    def body_norm(text):
        return _REWORD.sub("", text)

    s_head, s_body = _header_split(shipped)
    r_head, r_body = _header_split(regen)
    if head_norm(s_head) != head_norm(r_head):
        return "CONTENT_DRIFT", ["M2: header differs after stripping reword constructs "
                                 "and collapsing reflow"]
    if body_norm(s_body) != body_norm(r_body):
        return "CONTENT_DRIFT", ["M2: body differs after stripping reword constructs "
                                 "(no whitespace collapse here -- nothing re-wraps a body)"]
    return "CITATION_FORM_ONLY", ["M2: equal after stripping reword constructs and "
                                  "collapsing header reflow"]


def classify(shipped, regen):
    """`IDENTICAL` / `CITATION_FORM_ONLY` / `CONTENT_DRIFT`, agreed by both methods."""
    if shipped == regen:
        return "IDENTICAL", ["byte-identical"]
    v1, e1 = m1(shipped, regen)
    v2, e2 = m2(shipped, regen)
    if v1 != v2:
        raise AssertionError(
            "THE TWO METHODS DISAGREE -- refusing to pick one.\n"
            f"  M1 -> {v1}: {e1}\n  M2 -> {v2}: {e2}")
    return v1, e1 + e2


# --------------------------------------------------------------------------
# The census: snapshot, run one generator, compare, restore, PROVE the restore
# --------------------------------------------------------------------------

def snapshot():
    return {f: hashlib.sha256(open(os.path.join(CASES, f), "rb").read()).hexdigest()
            for f in sorted(os.listdir(CASES)) if f.endswith(".toml")}


def changed(base, now):
    return sorted(f for f in set(base) | set(now) if base.get(f) != now.get(f))


def restore():
    subprocess.run(["git", "-C", REPO, "checkout", "--", CASES_REL], check=True)
    subprocess.run(["git", "-C", REPO, "clean", "-fdq", CASES_REL], check=True)


def prove_restored(base):
    """Every sha back to baseline AND git status empty for the cases tree."""
    diff = changed(base, snapshot())
    status = subprocess.run(
        ["git", "-C", REPO, "status", "--porcelain", "--", CASES_REL],
        capture_output=True, text=True).stdout.strip()
    return (not diff and not status), diff, status


def shipped_text(name):
    return subprocess.run(
        ["git", "-C", REPO, "show", f"HEAD:{CASES_REL}/{name}"],
        capture_output=True, text=True, check=True).stdout


def generators():
    """The derived population: every `gen_batch*.py` in this directory.

    Derived, not listed. The routed backlog item said "all seven" and the
    controller counted eight; both were reading a family (`gen_batch5*/6*/7*`)
    rather than applying the criterion, and neither figure included the six
    `gen_batch4*` generators -- which emit the same `#[test]`-fns/invocations
    shape, write 20 of the family's case files, and drifted exactly like the
    rest. A hand-maintained list is how a population figure goes stale; the
    directory is the population.
    """
    return sorted(f for f in os.listdir(HERE)
                  if f.startswith("gen_batch") and f.endswith(".py"))


def run_controls(base):
    """§3.4: a comparator that has not been shown to fire is not evidence."""
    print("CONTROLS (run FIRST -- a comparator that has not fired is not evidence)")
    victim = sorted(base)[0]
    path = os.path.join(CASES, victim)
    with open(path, "a") as fh:
        fh.write("# injected control probe\n")
    seen = changed(base, snapshot())
    c1 = seen == [victim]
    print(f"  control 1  append a line to {victim} -> comparator says {seen or 'NOTHING'}"
          f"   VERDICT: {'DRIFTED, as required' if c1 else 'FAILED -- comparator is blind'}")

    poisoned = open(path).read()
    v, _ = classify(shipped_text(victim), poisoned)
    c2 = v == "CONTENT_DRIFT"
    print(f"  control 2  classify that same injected line -> {v}"
          f"   VERDICT: {'correct' if c2 else 'FAILED -- classifier is blind'}")

    restore()
    ok, diff, status = prove_restored(base)
    print(f"  control 3  restore -> every sha back to baseline AND git status empty: {ok}"
          f"   VERDICT: {'restored' if ok else f'FAILED diff={diff} status={status!r}'}")
    return c1 and c2 and ok


def census():
    base = snapshot()
    print(f"BASELINE: {len(base)} case file(s) under {CASES_REL}\n")
    if not run_controls(base):
        print("\nCONTROLS FAILED -- every verdict below would be unevidenced.")
        return 2

    only = sys.argv[sys.argv.index("--only") + 1] if "--only" in sys.argv else None
    gens = [g for g in generators() if only is None or g == only]
    if not gens:
        print(f"no such generator: {only}")
        return 2

    print(f"\nCENSUS over {len(gens)} generator(s)")
    citation_form, content, crashed, failed_restore = [], [], [], []
    for gen in gens:
        proc = subprocess.run([sys.executable, os.path.join(HERE, gen)],
                              capture_output=True, text=True, cwd=HERE)
        drift = changed(base, snapshot())
        if proc.returncode != 0:
            crashed.append(gen)
            tail = " | ".join(proc.stderr.strip().split("\n")[-2:])
            print(f"  {gen:<26} CRASHED rc={proc.returncode}   {tail[:160]}")
        elif not drift:
            print(f"  {gen:<26} FIXED POINT")
        else:
            print(f"  {gen:<26} DRIFTED, {len(drift)} file(s)")
        for name in drift:
            verdict, why = classify(shipped_text(name), open(os.path.join(CASES, name)).read())
            (citation_form if verdict == "CITATION_FORM_ONLY" else content).append(name)
            print(f"      {name}: {verdict}")
            for line in why:
                print(f"          {line}")
        restore()
        ok, diff, status = prove_restored(base)
        if not ok:
            failed_restore.append(gen)
            print(f"      RESTORE FAILED after {gen}: diff={diff} status={status!r}")

    print("\nENUMERATED SETS (the gate's own output, compared with the declarations)")
    problems = []
    for label, got, want in (
            ("citation-form-only", sorted(set(citation_form)), sorted(CITATION_FORM_ONLY_DECLARED)),
            ("content-drift", sorted(set(content)), sorted(CONTENT_DRIFT_DECLARED))):
        print(f"  {label}: {got or 'EMPTY'}")
        if got != want:
            problems.append(f"{label} set is {got}, declared {want}")
    if crashed:
        problems.append(f"generator(s) failed to run: {crashed}")
    if failed_restore:
        problems.append(f"tree not restored after: {failed_restore}")

    if problems:
        print("\nGATE FAILED:")
        for p in problems:
            print(f"  * {p}")
        return 1
    print(f"\nCLASSIFIER OK -- {len(gens)} generator(s) ran, "
          f"{len(base)} case file(s) reproduced byte-for-byte, both declared sets empty "
          f"and matched against this run's own output, tree restored.")
    return 0


# --------------------------------------------------------------------------
# Selftest -- poisoned probes, each paired with a control differing only in the
# poison. A green arm is not evidence the arm is wired (ruling 18).
# --------------------------------------------------------------------------

def _reword_donor():
    """A shipped case file carrying reword constructs, chosen by SCANNING rather
    than named -- a hardcoded stem is a figure that a later batch's deletion
    silently invalidates. Raises if the family carries none, which would mean
    the construct this whole gate is about has vanished."""
    best = None
    for name in sorted(os.listdir(CASES)):
        if not name.endswith(".toml"):
            continue
        text = open(os.path.join(CASES, name)).read()
        n = len(_REWORD_INSERT.findall(text))
        # SMALLEST file carrying enough constructs to be a real probe: the
        # largest in this family is half a megabyte and diffing it repeatedly
        # costs far more than it proves.
        if n >= 20 and (best is None or len(text) < best[0]):
            best = (len(text), name, text)
    if best is None:
        raise AssertionError(
            "no shipped case file carries 20+ reword constructs -- the citation form "
            "this gate classifies no longer exists in the tree, so the gate is vacuous")
    return best[1], best[2]


def _strip_reword(text):
    """Undo the reword: exactly what a generator that never learned it emits.

    Removes the inserted span AND its one separating space, so `(`x` :77)` goes
    back to `(:77)` -- not to `( :77)`, which is a state no generator ever
    emitted and which would make this probe test a straw man.
    """
    return _REWORD_INSERT.sub("", text)


def _reflow(text, width=70):
    """Re-wrap the `#` header block. Pure reflow: no word is added or removed.

    This is THE control. The first instrument to attempt this measurement did
    not have it, read reflow as content drift, and reported 6/20 instead of
    25/1."""
    import textwrap
    head, body = _header_split(text)
    # `textwrap` defaults: hyphen breaking ON, exactly as the generators wrap.
    # A probe that turned it off would model a reflow the tree never produces.
    wrapped = textwrap.wrap(_deflow(head), width=width) or [""]
    return "\n".join("# " + l for l in wrapped) + "\n\n" + body


def selftest():
    name, shipped = _reword_donor()
    print(f"donor (scanned, not named): {name} "
          f"-- {len(_REWORD_INSERT.findall(shipped))} reword construct(s)")
    probes = []

    def probe(label, regen, want, text=None):
        base = shipped if text is None else text
        try:
            got, why = classify(base, regen)
        except AssertionError as exc:
            got, why = f"RAISED({exc.args[0].splitlines()[0]})", []
        ok = got == want
        probes.append(ok)
        print(f"  {'ok  ' if ok else 'FAIL'} {label}: want {want}, got {got}")
        return ok

    # control: the unpoisoned pair
    probe("identical (control)", shipped, "IDENTICAL")

    # poison 1: the lost reword, which is the whole 25-file class
    probe("reword lost", _strip_reword(shipped), "CITATION_FORM_ONLY")

    # poison 2: pure reflow, the control the first instrument lacked. Probed at
    # SEVERAL widths, per ruling 18: a wrap-width change is a gate change, and a
    # normaliser validated at one width is validated against one sample.
    for w in (58, 70, 86, 110):
        probe(f"pure reflow only, width {w}", _reflow(shipped, w), "CITATION_FORM_ONLY")

    # poison 3: both at once -- the real shape of the drift
    for w in (58, 86):
        probe(f"reword lost + reflow, width {w}",
              _reflow(_strip_reword(shipped), w), "CITATION_FORM_ONLY")

    # poison 4/5/6: content, in three different places
    probe("a citation NUMBER moved",
          re.sub(r":(\d+)", lambda m: f":{int(m.group(1)) + 1}", _strip_reword(shipped), count=1),
          "CONTENT_DRIFT")
    probe("a word added to header prose",
          shipped.replace("#", "# INVENTED", 1), "CONTENT_DRIFT")
    body_hit = re.search(r'(?m)^(stdout_contains|args|name) = .*$', shipped)
    if body_hit:
        probe("a body line changed",
              shipped[:body_hit.end() - 1] + "XYZZY" + shipped[body_hit.end() - 1:],
              "CONTENT_DRIFT")
    else:
        print("  FAIL body-line probe: donor has no assertion line to poison")
        probes.append(False)

    # poison 7: whitespace inside a body line must NOT be excused as reflow --
    # this is the direction a whitespace-blind classifier gets wrong.
    ws = re.search(r'(?m)^args = \[.*$', shipped)
    if ws:
        probe("whitespace changed in a BODY line",
              shipped[:ws.start()] + ws.group(0).replace(", ", ",  ") + shipped[ws.end():],
              "CONTENT_DRIFT")

    # poison 8: the two methods must be able to DISAGREE, and the disagreement
    # must RAISE rather than resolve. An INSERTED span is the case that splits
    # them: M2 strips reword constructs from both sides and sees nothing, M1
    # forbids insertions outright. Nothing else in this file exercises that
    # path, so without this probe the raise would be dead code -- and a green
    # arm is not evidence the arm is wired.
    hit = _REWORD_INSERT.search(shipped)
    forged = shipped[:hit.start()] + "`forged.snippet()` " + shipped[hit.start():]
    try:
        got, _ = classify(forged, shipped)
        print(f"  FAIL methods disagree -> RAISES: classify() returned {got} on an INSERTED span")
        probes.append(False)
    except AssertionError as exc:
        ok = "DISAGREE" in exc.args[0]
        print(f"  {'ok  ' if ok else 'FAIL'} methods disagree -> classify() raises "
              f"rather than picking one")
        probes.append(ok)

    if all(probes):
        print(f"\nSELFTEST OK -- {len(probes)} probes: the lost reword and pure reflow are both "
              "classified citation-form-only, content drift is caught in four places including "
              "a whitespace-only body edit, and a disagreement between the two methods raises.")
        return 0
    print(f"\nSELFTEST FAILED -- {probes.count(False)} of {len(probes)} probes")
    return 1


def main():
    if "--selftest" in sys.argv:
        return selftest()
    return census()


if __name__ == "__main__":
    sys.exit(main())
