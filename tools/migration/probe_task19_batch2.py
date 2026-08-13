"""Three-armed injection probe for Task 19 batch 2.

A green gate that has never been made red is not evidence. For each pair this
poisons ONE real pin, then asks three independent gates whether they notice, and
records WHICH one did:

  A  audit-case-migration.py   (rule 3, absolute -- literal COVERAGE only)
  B  fidelity.py MISSING side  (U14 -- the poisoned literal leaves the TOML)
  C  the trial itself          (U9 -- the real binary does not emit the poison)

Arm C must fire for every pair. Arms A and B are recorded rather than required,
because the audit is a coverage tool by construction: it asks whether a source
literal appears SOMEWHERE in the case file, so poisoning one of two identical
claims leaves it green and is right to.
"""
import os, subprocess, sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "task-18-browser-pilot"))
from fidelity import diff

import os
REPO_ = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
T = os.path.join(REPO_, "crates/kali_cli/tests")
REPO = REPO_
AUDIT = os.path.join(REPO_, "scripts/audit-case-migration.py")

PAIRS = [
    ("exponentiation_operator", "misc/exponentiation_operator",
     'stdout_contains = ["8"]', 'stdout_contains = ["9"]'),
    ("runtime_fasta_capstone", "runtime/fasta_capstone",
     '>ONE Homo sapiens alu\\nGGCCGGGCGCGGTGGC', '>ONE Homo sapiens alu\\nGGCCGGGCGCGGTGGX'),
    ("runtime_fasta_output", "runtime/fasta_output",
     'tactDtDagc', 'tactDtDagX'),
    ("module_var_object_compound", "misc/module_var_object_compound",
     'stderr_contains = ["E5506"]', 'stderr_contains = ["E5507"]'),
    ("closure_return_isolation", "misc/closure_return_isolation",
     'stdout = "bump\\n0\\nafter\\n"', 'stdout = "bump\\n1\\nafter\\n"'),
    ("heap_grow_runtime", "misc/heap_grow_runtime",
     'stdout = "393216\\n"', 'stdout = "393217\\n"'),
    ("trap_diagnostics_runtime", "misc/trap_diagnostics_runtime",
     '"CPU fuel budget exhausted"', '"CPU fuel budget exhaustedX"'),
    ("float_console_runtime", "misc/float_console_runtime",
     'stdout = "1e-7\\n"', 'stdout = "1e-8\\n"'),
    ("number_predicates_runtime", "misc/number_predicates_runtime",
     '"ok 1"', '"ok 2"'),
    ("number_predicates_freeze_runtime", "misc/number_predicates_freeze_runtime",
     'payload.total = 1', 'payload.total = 2'),
    ("promise_any_sequencing", "misc/promise_any_sequencing",
     'stderr_contains = ["E4000"]', 'stderr_contains = ["E4001"]'),
    ("promise_race_sequencing", "misc/promise_race_sequencing",
     'exit = "failure"', 'exit = "success"'),
    ("runtime_monomorphize", "runtime/monomorphize",
     'stdout = "3\\n2\\n"', 'stdout = "3\\n4\\n"'),
    ("parse_int_static_ascii", "misc/parse_int_static_ascii",
     'stdout = "42\\n-16\\n255\\n5\\n63\\n2\\n"', 'stdout = "42\\n-16\\n255\\n5\\n63\\n3\\n"'),
]


def audit(rs, toml):
    return subprocess.run([sys.executable, AUDIT, rs + ".rs", "cases/" + toml + ".toml"],
                          cwd=T, capture_output=True, text=True).returncode


def missing_set(rs, toml):
    return diff([os.path.join(T, rs + ".rs")],
                [os.path.join(T, "cases", toml + ".toml")])[2]


def trials(toml):
    return subprocess.run(
        ["cargo", "test", "-p", "kali_cli", "--test", "cases", "--", toml],
        cwd=REPO, capture_output=True, text=True).returncode


bad = []
rows = []
for rs, toml, old, new in PAIRS:
    path = os.path.join(T, "cases", toml + ".toml")
    original = open(path).read()
    if old not in original:
        bad.append(f"{toml}: poison anchor {old!r} not present")
        continue
    a0, m0, c0 = audit(rs, toml), missing_set(rs, toml), trials(toml)
    open(path, "w").write(original.replace(old, new, 1))
    a1, m1, c1 = audit(rs, toml), missing_set(rs, toml), trials(toml)
    open(path, "w").write(original)
    a2, c2 = audit(rs, toml), trials(toml)
    restored = open(path).read() == original

    A = (a0 == 0 and a1 == 1)
    B = bool(m1 - m0)
    C = (c0 == 0 and c1 != 0)
    ok = C and restored and a2 == 0 and c2 == 0
    rows.append((toml, A, B, C, ok))
    print(f"  {'ok ' if ok else 'BAD'} {toml:<44} A(audit)={'RED' if A else '- '} "
          f"B(fidelity)={'RED' if B else '- '} C(trial)={'RED' if C else '- '}")
    if not ok:
        bad.append(f"{toml}: A={A} B={B} C={C} restored={restored} a2={a2} c2={c2}")

print()
print(f"arm A (audit) fired on {sum(1 for r in rows if r[1])}/{len(rows)} pairs")
print(f"arm B (fidelity MISSING) fired on {sum(1 for r in rows if r[2])}/{len(rows)} pairs")
print(f"arm C (the trial) fired on {sum(1 for r in rows if r[3])}/{len(rows)} pairs")
if bad:
    print("\nPROBE FAILED")
    for b in bad:
        print("  " + b)
    sys.exit(1)
print(f"\nPROBE OK -- {len(rows)} pairs, every poison caught by at least the trial arm, "
      f"every file restored byte-for-byte")
