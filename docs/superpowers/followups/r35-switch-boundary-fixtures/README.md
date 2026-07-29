# R-35 switch boundary — fixtures and raw transcripts

The reproducible record behind
`docs/superpowers/followups/r35-switch-boundary-rederived.md`. Stage 2's allowlist is sized
against that matrix, so the evidence is tracked here rather than left in a scratch directory.

**Measured on:** `5c9bbd051` (branch `r35-switch-lowering`; includes Task 2 `9db9150c0` and
Task 3 `5c9bbd051`), binary `./target/debug/kali` rebuilt at that commit.
**Oracle:** `node v26.5.0`. **Date:** 2026-07-28.

## Re-running

```bash
cd /workspace && cargo build --bin kali            # never trust a stale binary
./docs/superpowers/followups/r35-switch-boundary-fixtures/run.sh
```

`run.sh` runs every fixture under both `./target/debug/kali run` and `node`, capturing stdout,
stderr and the **unpiped** exit status of each. It writes a transcript in the same format as
the checked-in ones so a re-run can be diffed against them directly.

Three fixtures are expected to burn ~60 s each of CPU-fuel budget before kali gives up
(`c13B_*`, `c13C_*`, `c13D_mod`, `c13E_mod`, `c13F_mod` — R-09's hang); `run.sh` wraps every
invocation in `timeout 120`.

## Naming

| prefix | meaning |
|---|---|
| `sNN_fn.js`, `sNN_mod_<disc>.js` | matrix shape NN, in-function / module scope. The `mod` files carry the discriminant in the name because a module-scope `return` ends the module, so one discriminant needs one file. |
| `sNNr_*.js` | shape NN in its **`return`-terminated** form. These are the primary matrix cells for shapes 4, 5, 6, 9, 10, 11, 14, 15, 16; the un-suffixed `sNN_*` files are the `break`-terminated confound control, which is uniformly `E5506`. |
| `mNN_*.js` | supplementary probes — mechanism (`m01`-`m04`), `throw` (`m05`), Rule 5 / block scoping (`m06`-`m09`), loop-nested `break` (`m10`), falsy string (`m11`), boolean clause order (`m12`), coincidence controls (`m13`, `m14`). |
| `c13*.js` | fix round 1: cell 13's two constructions, its `c13G`/`c13H` discriminators, and the no-switch controls that prove the hang is R-09. |
| `disc/dNN_*.js` | fix round 2: the 13 switch-**discriminant** forms, which isolate the `E2000` regression (R-50). |
| `e2000.js`, `e2000b.js`, `seq1.js` | Task 3's `E2000` emission checks and R-50's repro. |

## Transcripts

| file | contents |
|---|---|
| `transcript-1-primary.txt` | the 70 primary + break-control fixtures |
| `transcript-2-isolates-and-mechanism.txt` | the `return`-terminated isolates and `m01`-`m14` |
| `transcript-3-rule45-probes.txt` | the Rule 4 / Rule 5 probes (`m08`-`m14`) |
| `transcript-4-cell13-fixround1.txt` | cell 13 re-measurement and the no-switch controls |
| `transcript-5-cell13-discriminators.txt` | the `c13E`-`c13H` discriminators |

## Instrument rules every fixture obeys

Exactly **one** argument per `console.log`, always literal-rooted concatenation (`"r=" + r`),
and **no default parameters anywhere** — a default parameter silently truncates the rest of
the module (register R-01) and would void the fixture without any diagnostic. No `console.log`
receives an array, an object or a bare boolean, so R-30 / R-31 rendering defects cannot
contaminate a cell.
