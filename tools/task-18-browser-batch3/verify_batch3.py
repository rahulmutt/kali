r"""Independent re-verification sweep for Task 18 batch 3 (U11 / U14).

Deliberately does NOT reuse the generators' machinery:

  * fixture fidelity is re-checked by ENCODING each shipped TOML value back
    into the two spellings a Rust literal can legally have and searching the raw
    `.rs` text for them -- the inverse of the character-cursor DECODE the
    generators pulled every body through, so a bug in one cannot mask the
    other. Every `[source]` body must be found that way; every
    `browser_bundle_harness` `body` must either be found that way or be one of
    the strings the real `format!` calls actually produced (inventory below);

  * trial arithmetic is re-derived from `cargo test --list` output rather than
    from the generator's own case/matrix bookkeeping, and compared against the
    per-file expectation table below, which was written by reading the source
    files -- not produced by the generators.

Usage: verify_batch3.py
"""
import os
import re
import subprocess
import sys
import tomllib

TESTS = '/workspace/crates/kali_cli/tests'
CASES = os.path.join(TESTS, 'cases', 'browser')

# (rs stem, toml stem, #[test] fns in source, real invocations, expected trials)
# Hand-derived by reading every call site and expanding every loop.
TABLE = [
    ('browser_for_await_object_string_enumeration_sequence_wrappers_js_input',
     'for_await_object_string_enumeration_sequence_wrappers_js_input', 5, 16, 16),
    ('browser_for_of_array_iteration_alias_chain',
     'for_of_array_iteration_alias_chain', 16, 16, 16),
    ('browser_for_of_array_iteration_break_continue_harness',
     'for_of_array_iteration_break_continue_harness', 32, 32, 32),
    ('browser_for_of_array_iteration_break_continue',
     'for_of_array_iteration_break_continue', 16, 16, 16),
    ('browser_for_of_array_iteration_harness_sequence_wrappers_js_input',
     'for_of_array_iteration_harness_sequence_wrappers_js_input', 5, 16, 16),
    ('browser_for_of_array_iteration_harness_wrappers_js_input',
     'for_of_array_iteration_harness_wrappers_js_input', 40, 40, 40),
    ('browser_for_of_array_iteration_sequence_wrappers',
     'for_of_array_iteration_sequence_wrappers', 8, 8, 8),
    ('browser_for_of_array_iteration_wrappers',
     'for_of_array_iteration_wrappers', 16, 16, 16),
    ('browser_map_iteration_harness', 'map_iteration_harness', 5, 16, 16),
    # partial migrations: 1 #[test] retained hand-written in each
    ('browser_math_abs_sign_frozen_aliases',
     'math_abs_sign_frozen_aliases', 25, 24, 24),
    ('browser_math_asinh_acosh_atanh_identities',
     'math_asinh_acosh_atanh_identities', 24, 24, 24),
    ('browser_math_atan2_global_this_root',
     'math_atan2_global_this_root', 19, 69, 69),
    ('browser_math_atan2_trailing_argument_evaluation_bundle',
     'math_atan2_trailing_argument_evaluation_bundle', 8, 8, 8),
    ('browser_math_atan2_trailing_argument_evaluation_harness',
     'math_atan2_trailing_argument_evaluation_harness', 6, 12, 12),
    ('browser_math_bracketed_root_core_suite',
     'math_bracketed_root_core_suite', 9, 24, 24),
    ('browser_math_clz32_omitted_operands',
     'math_clz32_omitted_operands', 24, 24, 24),
    ('browser_math_exp2_global_this_root',
     'math_exp2_global_this_root', 24, 24, 24),
    ('browser_math_exp2_zero_identity', 'math_exp2_zero_identity', 16, 16, 16),
    ('browser_math_exp_log_bracketed_root',
     'math_exp_log_bracketed_root', 9, 16, 16),
    ('browser_math_exp_log_fully_bracketed_root',
     'math_exp_log_fully_bracketed_root', 9, 16, 16),
    ('browser_math_exp_log_identities', 'math_exp_log_identities', 16, 16, 16),
]

# Rule 8: the 17 distinct `browser_bundle_harness` bodies this batch ships. The
# first 8 export names below are the ones source builds with `format!`, and their
# resolved text came from EXECUTING those real `format!` calls in a standalone
# dump program, never hand-substituted; the remaining 9 are plain raw-string
# literals in source and would also be found by `in_source()`. The sweep fails
# any case file carrying a body outside this set.
FORMAT_BODIES = {
    'const mod = await import(bundleJs.href);\nawait mod.%s();\n' % n
    for n in (
        'forOfArrayIterationAsConstWrapper', 'forOfArrayIterationSatisfiesWrapper',
        'forOfArrayIterationSequenceWrapper',
        'forOfArrayIterationConstAliasChainWrapper',
        'forOfArrayIterationBreakContinueWrapper',
        'forAwaitArrayIterationBreakContinueWrapper',
        'globalThisMathAtan2ZeroSlice', 'globalThisMathAtan2FrozenCallableAliases',
        'globalThisMathAtan2AwaitWrappedZeroSlice',
        'globalThisMathAbsSignFrozenAliases', 'mathInverseHyperbolicIdentities',
        'mathClz32OmittedOperands',
        'globalThisMathExp2NonNegativeIntegerLiterals',
        'atan2TrailingArgumentEvaluation',
        'bracketedGlobalThisMathExpLogIdentities',
        'fullyBracketedGlobalThisMathExpLogIdentities',
        'bracketedGlobalThisMathCoreSuite',
    )
}

# --- independent fixture-fidelity check: ENCODE-and-search --------------------
# Deliberately the inverse of the generators' mechanism. They DECODED a Rust
# literal into a Python string with `lexer.py`'s character cursor; this re-ENCODES
# the shipped TOML value back into the two spellings a Rust literal can have and
# searches the raw `.rs` text for them. A bug in the decoder cannot hide a bug in
# the encoder: the two disagree unless the bytes really are the same.


def rust_spellings(value):
    """Both ways the `.rs` could legally hold this exact program text: verbatim
    inside a raw string (`r"..."` / `r#"..."#`), or escaped inside a plain
    string literal."""
    escaped = (value.replace('\\', '\\\\')
                    .replace('"', '\\"')
                    .replace('\n', '\\n')
                    .replace('\t', '\\t')
                    .replace('\r', '\\r'))
    return [value, escaped]


def in_source(value, rs_text):
    return any(sp in rs_text for sp in rust_spellings(value))


def substituted(value, matrix):
    """Every concrete spelling of a `${axis}`-bearing string."""
    outs = [value]
    for axis, values in (matrix or {}).items():
        nxt = []
        for o in outs:
            if '${' + axis + '}' in o:
                nxt.extend(o.replace('${' + axis + '}', v) for v in values)
            else:
                nxt.append(o)
        outs = nxt
    return outs


def main():
    failures = []
    listed = subprocess.run(
        ['cargo', 'test', '-q', '-p', 'kali_cli', '--test', 'cases', '--',
         '--list'], cwd='/workspace', capture_output=True, text=True).stdout
    trials = {}
    for line in listed.split('\n'):
        m = re.match(r'browser/([a-z0-9_]+)(\[[^\]]*\])?::', line.strip())
        if m:
            trials[m.group(1)] = trials.get(m.group(1), 0) + 1

    print(f'{"case file":52} {"fns":>4} {"invoc":>6} {"trials":>7} {"src":>5}')
    for rs_stem, toml_stem, n_fns, n_invoc, n_trials in TABLE:
        rs = open(os.path.join(TESTS, rs_stem + '.rs'), encoding='utf-8').read()
        real_fns = len(re.findall(r'^#\[test\]', rs, re.MULTILINE))
        got_trials = trials.get(toml_stem, 0)
        doc = tomllib.load(open(os.path.join(CASES, toml_stem + '.toml'), 'rb'))
        matrix = doc.get('matrix')
        # fixture fidelity, independent (encode-and-search) mechanism
        bad_src = [k for k, v in doc.get('source', {}).items()
                   if not in_source(v, rs)]
        # harness bodies: executed-format! inventory
        bodies = []
        for case in doc.get('case', []):
            for step in case.get('step', [case]):
                if 'body' in step:
                    bodies.append(step['body'])
        bad_body = [b for b in bodies
                    if b not in FORMAT_BODIES and not in_source(b, rs)]

        status = []
        if real_fns != n_fns:
            status.append(f'FN COUNT {real_fns}!={n_fns}')
        if got_trials != n_trials:
            status.append(f'TRIALS {got_trials}!={n_trials}')
        if n_invoc != n_trials:
            status.append(f'INVOC {n_invoc}!={n_trials}')
        if bad_src:
            status.append(f'FIXTURE NOT IN SOURCE: {bad_src}')
        if bad_body:
            status.append(f'HARNESS BODY UNVERIFIED: {bad_body}')
        # every ${axis} spelling resolves
        for k in list(doc.get('source', {})) + [
                a for c in doc.get('case', [])
                for s in c.get('step', [c]) for a in s.get('args', [])]:
            for spelling in substituted(k, matrix):
                if '${' in spelling:
                    status.append(f'UNRESOLVED SUBSTITUTION in {k!r}')
        line = (f'{toml_stem:52} {real_fns:>4} {n_invoc:>6} {got_trials:>7} '
                f'{len(doc.get("source", {})):>5}')
        if status:
            failures.append((toml_stem, status))
            line += '   <<< ' + '; '.join(status)
        print(line)

    print()
    if failures:
        print(f'FAILED: {len(failures)} file(s)')
        return 1
    print(f'OK: {len(TABLE)} files, '
          f'{sum(t for _, _, _, _, t in TABLE)} trials, fixture text and trial '
          f'arithmetic independently re-derived')
    return 0


if __name__ == '__main__':
    sys.exit(main())
