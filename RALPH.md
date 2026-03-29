# RALPH

- Iteration: 38 of 50
- Prompt file: /workspace/prompts/iterate-spec.md
- Updated: 2026-03-29T19:13:41.555Z

## Last emitted message

```text
Done.

Updated:
- `SPEC.md`
- `specs/09-sandboxing.md`
- `specs/12-cli.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

What I improved:
- clarified the sandbox model so `kali effects` is a pure reporting command, not a second policy-validation path
- made the policy-validation workflow canonical: use `check/build --sandbox`, not `effects --sandbox`
- tightened config/schema wording so top-level `sandbox` applies only to sandbox-aware commands: `run`, `test`, `check`, `build`
- added an explicit maturity-matrix entry for rejecting `kali effects --sandbox ...`
- simplified the docs to avoid overlapping semantics around effect reporting vs policy enforcement

Git:
- committed as: `38217a3` — **Clarify sandbox policy validation workflow**
- pushed to: `origin/dev`

Note:
- `RALPH.md` had pre-existing local modifications and I left it untouched.
```
