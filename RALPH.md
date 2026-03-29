# RALPH

- Iteration: 3 of 10
- Prompt file: /workspace/prompts/iterate-spec.md
- Updated: 2026-03-29T16:53:12.602Z

## Last emitted message

```text
Done.

Updated:
- `SPEC.md`
- `specs/09-sandboxing.md`
- `specs/12-cli.md`
- `specs/13-embedding.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

What I fixed/simplified:
- **Resolved API-surface vs runtime-profile drift**
  - Clarified that `deno/node/browser` are **API surfaces**
  - Clarified that `runtimeProfiles` are a separate axis, e.g. default single-threaded vs later `wasm-threads`
  - Fixed the maturity table where Deno was previously described like a runtime profile

- **Made config/CLI terminology more consistent**
  - `runtimeProfiles: []` now explicitly means the default single-threaded baseline
  - Added a rule that `apiSurface` and `runtimeProfiles` must not be conflated

- **Simplified embedding API shape**
  - Reworked the embedding example to use set-like methods:
    - `.runtime_profiles([])`
    - `.compat_features([])`
  - This keeps embedding aligned with `kali.json` and avoids awkward boolean pair APIs

- **Tightened sandbox policy structure**
  - Added a clear split:
    - `effects.*` = capability allow/deny + capability-local allowlists/caps
    - `resources.*` = cross-cutting runtime budgets
  - This reduces schema drift and avoids duplicate limit concepts in multiple places

- **Improved spec consistency around command gating**
  - Reinforced that `--api node` is phase-gated consistently across `check/build/run/test`
  - Kept browser support clearly framed as early **check/build profile**, not standalone runtime

A few good next cleanup opportunities I’d recommend:
1. **Trim experimental algebraic-effects surface further**
   - Keep parse support documented, but move most illustrative syntax/details behind a shorter “future appendix” so the early effect model stays focused on sandbox capabilities.

2. **Add one canonical “default execution tuple”**
   - Something like:
     - `apiSurface=deno`
     - `buildMode=fast`
     - `runtimeProfiles=[]`
   - Could live in `SPEC.md` and be referenced everywhere.

3. **Unify artifact examples**
   - A few specs still describe outputs informally; they should all point to the canonical artifact kinds in `18-schemas.md`.

If you want, I can do a second pass over the remaining specs and make them even more aggressively minimal/implementation-oriented.
```
