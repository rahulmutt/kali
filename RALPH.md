# RALPH

- Iteration: 509 of 1000
- Prompt file: /workspace/prompts/iterate-spec.md
- Updated: 2026-03-31T00:05:00.000Z

## Last emitted message

```text
Done.

What I changed
- `SPEC.md`
  - tightened the canonical definition of **Phase-1 static policy-validation surface**
  - made it explicit that this surface covers:
    - `kali check --sandbox ...` in both project-discovery and explicit-file-set forms
    - the default executable-oriented `kali build --sandbox <policy> <file>` path in the shared **Deno-oriented build context (schema v1)**
    - `kali build --lib --sandbox <policy> <file>`
    - browser-targeted `kali build --bundle --sandbox <policy> <file>`, including inherited-config browser equivalents
  - this closes an ambiguity where the old shorthand could be overread as "all build + sandbox shapes"

- `specs/19-feature-maturity.md`
  - aligned the Phase-1 summary note with that refined definition
  - spelled out that the static-policy surface is exactly `check --sandbox` plus the three supported build-intent lanes (`build`, `build --lib`, and browser `build --bundle`)

Why these changes
- I found a cross-spec ambiguity risk:
  - the old shared term was correct, but still broad enough to invite a wrong reading for inherited browser contexts and unsupported build shapes
  - the maturity chapter relied on the term without restating the exact Phase-1 build lanes
- tightening the canonical definition in one place and aligning the summary note makes the Phase-1 sandbox/build boundary easier to read and harder to misapply

Git
- Commit: pending
- Message: pending

Push
- Pending.
```
