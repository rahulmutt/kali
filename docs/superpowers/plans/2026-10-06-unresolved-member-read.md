# Unresolved Member Read Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A member read, or a plain `=` store, that reaches codegen's silent-`0` fallback is refused with E5506 when its receiver (or store target) is rooted at a value the program built. Host-rooted receivers keep today's warn+0 behaviour.

**Architecture:** One new gate, `unresolved_member_read_refuses`, sits in `crates/kali_codegen/src/emit/member_provenance.rs`. It shares a root-verdict helper with the existing `unresolved_member_call_refuses` and adds a call-root arm. It is consulted at two fallback sites in `emit/operators.rs`. A narrow `check` mirror in `kali_types/src/resolve/member.rs` refuses absent-field reads on `const` object-literal and program-class bindings. Messages live in `kali_common/src/messages.rs`.

**Tech Stack:** Rust workspace (`kali_common`, `kali_codegen`, `kali_types`, `kali_cli`); black-box `.toml` case files run by `kali_case_runner`; node v26.10.0 as the oracle.

**Spec:** `docs/superpowers/specs/2026-10-06-unresolved-member-read-design.md` (read §3, §3.7 amendments A-1..A-4, §5 and §6 before starting).

## Global Constraints

- Baseline commit `28c97e4ed`; branch `unresolved-member-read`.
- Oracle: `node v26.10.0`, always run as `env -u FORCE_COLOR node`.
- Diagnostic code: E5506 (`e5::FEATURE_UNAVAILABLE`). No new code, flag, schema or maturity claim.
- Every message contains `the receiver is a value this program built`. Distinct substrings: `no lowering for that read`, `no lowering for that store`.
- Rust unit tests go in sibling `*_tests.rs` files, never inline `#[cfg(test)]` modules.
- Black-box tests are `.toml` cases under `crates/kali_cli/tests/cases/`, never new `tests/*.rs` targets.
- **Resource caps (the pod has a 32Gi memory limit and a 20Gi `/tmp` limit; an earlier run was killed):** one shared `/workspace/target`; never set `CARGO_TARGET_DIR`; never build in a worktree with its own target dir; `cargo build -j 8`; `cargo test -j 8 … -- --test-threads=8`; never run two cargo commands concurrently; wrap every full `cases` run and `cargo test --workspace` in the watchdog from Task 1.
- **Stop rule (spec §6.1):** a case that moves and is not in Task 7's table stops execution. Report it to the human partner; never re-pin it silently.
- Commit messages: `feat|fix|test|docs(unresolved-member-read): …`.

## Review Focus

1. **A host-rooted receiver must keep working.** `const f = Object.freeze(Math.log2); f(8)`, `Number.isNaN` aliases and `globalThis.performance` all print node's output today. The gate must not refuse them. Pinned in Task 3 (`a_host_root_keeps_its_read`) and Task 6 (exemption-control cases), and verified by the full run in Task 7 (all ~30 group-B cases stay green).
2. **One defect, one diagnostic in `check`.** `const o={k:1}; o.zork()` already gets the call mirror's diagnostic. The read mirror must not add a second one for the callee member, including `o.zork?.()`. Pinned in Task 5 (`check_read_mirror_skips_callees`).
3. **`check` must not refuse what `run` accepts.** `typeof o.z` (A-3) goes through a different placeholder, so the mirror skips it. Pinned in Task 5 and Task 6.
4. **Spread and comma are not property reads.** R-25 (`[...a]`) and R-27 (`(1, 2)`) reach the same gate. They get the neutral message, never "reading `.spread`" or "reading `.`". Pinned in Task 2 (message unit tests) and Task 6.
5. **A store to a free global keeps working**, while a store to a `const` refuses. Pinned in Task 4 (`a_store_to_a_free_global_keeps_its_lowering`) and Task 6.

---

## File structure

| file | change | responsibility |
|---|---|---|
| `crates/kali_common/src/messages.rs` | modify | the two message helpers |
| `crates/kali_common/src/messages_tests.rs` | modify | their exact-text tests |
| `crates/kali_codegen/src/emit/member_provenance.rs` | modify | shared root verdict, the read gate, the store refusal |
| `crates/kali_codegen/src/emit/member_provenance_tests.rs` | modify | gate unit tests |
| `crates/kali_codegen/src/emit/operators.rs` | modify | the two call sites (read fallback about line 794, binary `_` arm about line 2713) |
| `crates/kali_types/src/context.rs` | modify | `read_mirror_skipped_members` field |
| `crates/kali_types/src/resolve/member.rs` | modify | `reject_unresolved_member_read`, callee skip |
| `crates/kali_types/src/resolve/expression.rs` | modify | `typeof` operand skip (about line 1847) |
| `crates/kali_types/src/resolve/member_tests.rs` | modify | mirror unit tests |
| `crates/kali_cli/tests/cases/soundness/unresolved_member_read.toml` | create | black-box cases |
| `tools/array-return-probes/probes/umr_*.js`, `tools/array-return-probes/baseline-umr.tsv` | create | probes and baseline sweep |
| `tools/watchdog.sh` | create | resource watchdog |
| case files listed in Task 7 | modify | re-pins |
| register, ranking, `clusters.json`, followups, `specs/15-errors.md` | modify | Task 8 |

---

### Task 1: Watchdog, probes and baseline measurement

**Files:**
- Create: `tools/watchdog.sh`
- Create: `tools/array-return-probes/probes/umr_*.js` (the 15 files below)
- Create: `tools/array-return-probes/baseline-umr.tsv`

**Interfaces:**
- Produces: `tools/watchdog.sh <log>`, used by Tasks 7 and 9. It runs until killed and kills `cargo` and the case binary if memory exceeds 24G or `/tmp` exceeds 14G. It also produces the `umr_` probe set, used in Task 7.

- [ ] **Step 1: Write the watchdog**

```bash
#!/usr/bin/env bash
# Resource watchdog for long cargo runs in the dev pod (32Gi memory, 20Gi /tmp).
# Usage: tools/watchdog.sh LOG & WD=$!; <run>; kill $WD
log="$1"
memcap=$((24 * 1024 * 1024 * 1024))
tmpcap=$((14 * 1024 * 1024 * 1024))
while true; do
  mem=$(cat /sys/fs/cgroup/memory.current)
  peak=$(cat /sys/fs/cgroup/memory.peak)
  tmpb=$(du -sb /tmp 2>/dev/null | cut -f1)
  tgt=$(du -sb "$(dirname "$0")/../target" 2>/dev/null | cut -f1)
  echo "$(date +%T) mem=$((mem >> 20))M peak=$((peak >> 20))M tmp=$((tmpb >> 20))M target=$((tgt >> 20))M" >>"$log"
  if [ "$mem" -gt "$memcap" ] || [ "$tmpb" -gt "$tmpcap" ]; then
    echo "$(date +%T) THRESHOLD HIT - killing" >>"$log"
    pkill -f 'deps/cases-'
    pkill cargo
    exit 1
  fi
  sleep 15
done
```

Run: `chmod +x tools/watchdog.sh`

- [ ] **Step 2: Rebuild the baseline binary**

Run: `git stash list; git status --short` (expect clean), then `cargo build -j 8 -p kali_cli`.
Expected: `Finished`. `target/` may hold a build of a throwaway patch from the design session, so this rebuild is required.

- [ ] **Step 3: Write the probes** (one program per file, each ending in a newline)

| file | program |
|---|---|
| `umr_c8.js` | `function mk(){ return {a:1}; } function f(){ let o=mk(); const g=()=>o; return g().a; } console.log(f());` |
| `umr_c9.js` | `function mk(){ return {a:1}; } function show(z){ console.log(z.a); } function f(){ let o=mk(); const g=()=>{ show(o); }; g(); } f();` |
| `umr_e1.js` | `function show(z){ return z.n * 10; } function outer(p){ const obj = p; function rd(){ return show(obj); } console.log(rd()); } const x={n:4}; outer(x);` |
| `umr_m6.js` | `function f(p){ const o=p; const g=()=>o["a"]; return g(); } const x={a:1}; console.log(f(x));` |
| `umr_q8.js` | `function mk(){ return {a:1, s:"xy", arr:[1,2]}; } function f(){ let o=mk(); const g=()=>o["a"]; return g(); } console.log(f());` |
| `umr_mkcall.js` | `function mk(){ return {a:1}; } console.log(mk().a);` |
| `umr_idcall.js` | `function id(o){ return o; } const x={a:1}; console.log(id(x).a);` |
| `umr_absent.js` | `const o={a:1}; console.log("z="+o.z);` |
| `umr_absent_bracket.js` | `const o={a:1}; console.log("z="+o["z"]);` |
| `umr_absent_nullish.js` | `const o={a:1}; console.log(o.z ?? 5);` |
| `umr_absent_optional.js` | `const o={a:1}; console.log("z="+o?.z);` |
| `umr_typeof.js` | `const o={a:1}; console.log(typeof o.z);` |
| `umr_const_store.js` | `const x = 1; x = 2; console.log("r=" + x);` |
| `umr_spread.js` | `const a=[1,2]; console.log([...a]);` |
| `umr_comma.js` | `function main(){ let n = 0; function bump() { n = n + 1; return 5; } let b = (bump(), 7); console.log("b=" + b); } main();` |

- [ ] **Step 4: Record the baseline sweep**

Run: `tools/array-return-probes/run.sh /tmp/claude-umr-all.tsv && grep '^umr_' /tmp/claude-umr-all.tsv > tools/array-return-probes/baseline-umr.tsv; rm /tmp/claude-umr-all.tsv`
Expected, matching spec §2.1 and §3.7: c8, c9, m6, q8, mkcall and idcall print `0` where node prints `1`; e1 prints `0` (node `40`); absent and absent_bracket print `z=0`; absent_nullish prints `5` (node-equal, A-4); absent_optional prints `z=0`; typeof prints `0`; const_store prints `r=1` (node throws); spread prints `0`; comma prints `b=0`.

Then run each probe under `target/debug/kali check` and confirm every one exits 0.
**If any row differs from this list, stop and report it.** The design rests on these values.

- [ ] **Step 5: Commit**

```bash
git add tools/watchdog.sh tools/array-return-probes/probes/umr_*.js tools/array-return-probes/baseline-umr.tsv
git commit -m "test(unresolved-member-read): watchdog, umr probes and baseline sweep"
```

---

### Task 2: Message helpers

**Files:**
- Modify: `crates/kali_common/src/messages.rs` (after `unresolved_member_call_unavailable_message`, about line 236)
- Test: `crates/kali_common/src/messages_tests.rs`

**Interfaces:**
- Produces: `pub fn unresolved_member_read_unavailable_message(name: &str) -> String` and `pub fn unresolved_store_unavailable_message(target: &str) -> String`, both reachable as `kali_common::…`.

- [ ] **Step 1: Write the failing tests** (append to `messages_tests.rs`)

```rust
#[test]
fn unresolved_member_read_message_names_the_property() {
    assert_eq!(
        unresolved_member_read_unavailable_message("a"),
        "reading `.a` is unavailable in the current phase: the receiver is a value this program built, and kali has no lowering for that read; node would read a property or `undefined`, so kali refuses rather than read 0"
    );
}

#[test]
fn unresolved_member_read_message_is_neutral_for_spread_and_sequence_text() {
    // LIR spells a spread `...a` as text "spread" and a comma expression as
    // text "" (spec A-1); neither is a property read.
    let neutral = "this expression is unavailable in the current phase: the receiver is a value this program built, and kali has no lowering for that read; kali refuses rather than evaluate it to 0";
    assert_eq!(unresolved_member_read_unavailable_message("spread"), neutral);
    assert_eq!(unresolved_member_read_unavailable_message(""), neutral);
}

#[test]
fn unresolved_store_message_names_the_target() {
    assert_eq!(
        unresolved_store_unavailable_message("x"),
        "assigning to `x` is unavailable in the current phase: the receiver is a value this program built, and kali has no lowering for that store; node would store the value or throw a TypeError, so kali refuses rather than drop the store"
    );
    assert!(unresolved_store_unavailable_message(".a").starts_with("assigning to `.a` is"));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -j 8 -p kali_common unresolved_ -- --test-threads=8`
Expected: compile error, `cannot find function unresolved_member_read_unavailable_message`.

- [ ] **Step 3: Implement** (in `messages.rs`)

```rust
/// A member read that reached codegen's placeholder fallback on a receiver
/// this program built (unresolved-member-read spec §3.5). LIR spells a spread
/// (`"spread"`) and a comma expression (`""`) like a member read (spec A-1),
/// so those texts get a message that does not claim a property.
pub fn unresolved_member_read_unavailable_message(name: &str) -> String {
    if name.is_empty() || name == "spread" {
        return "this expression is unavailable in the current phase: the receiver is a value this program built, and kali has no lowering for that read; kali refuses rather than evaluate it to 0".to_string();
    }
    format!(
        "reading `.{name}` is unavailable in the current phase: the receiver is a value this program built, and kali has no lowering for that read; node would read a property or `undefined`, so kali refuses rather than read 0"
    )
}

/// A plain `=` that reached codegen's final binary fallback with a target
/// rooted at a value this program built (spec §3.3, A-2). `target` is
/// `.name` for a member and `name` for an identifier.
pub fn unresolved_store_unavailable_message(target: &str) -> String {
    format!(
        "assigning to `{target}` is unavailable in the current phase: the receiver is a value this program built, and kali has no lowering for that store; node would store the value or throw a TypeError, so kali refuses rather than drop the store"
    )
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -j 8 -p kali_common unresolved_ -- --test-threads=8`
Expected: 3 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_common/src/messages.rs crates/kali_common/src/messages_tests.rs
git commit -m "feat(unresolved-member-read): read and store refusal messages"
```

---

### Task 3: The read gate and the read site

**Files:**
- Modify: `crates/kali_codegen/src/emit/member_provenance.rs:51-100` (`unresolved_member_call_refuses`)
- Modify: `crates/kali_codegen/src/emit/operators.rs` (`emit_unary`'s `_` arm, after the `capture_member_fallback_refusal` check at about line 794)
- Test: `crates/kali_codegen/src/emit/member_provenance_tests.rs`

**Interfaces:**
- Consumes: `kali_common::unresolved_member_read_unavailable_message` (Task 2); the existing `receiver_chain_root`, `lexical_scopes`, `root_has_host_provenance`, `init_has_host_provenance`, `emitting_method_of_host_derived_class`, `deny_e5506`.
- Produces: `pub(crate) fn unresolved_member_read_refuses(&self, receiver: LirNodeId) -> bool`. Task 4 uses it.

- [ ] **Step 1: Write the failing tests** (append to `member_provenance_tests.rs`)

```rust
const UNRES_READ: &str = "no lowering for that read";

fn read_refusals(source: &str) -> usize {
    diagnostics_for(source)
        .iter()
        .filter(|d| d.code == Some(5506) && d.message.contains(UNRES_READ))
        .count()
}

#[test]
fn a_member_read_on_a_program_built_root_refuses() {
    for source in [
        // captured-bindings followups §5.10
        "function mk(){ return {a:1}; } function f(){ let o=mk(); const g=()=>o; return g().a; } console.log(f());",
        "function mk(){ return {a:1}; } function show(z){ console.log(z.a); } function f(){ let o=mk(); const g=()=>{ show(o); }; g(); } f();",
        "function show(z){ return z.n * 10; } function outer(p){ const obj = p; function rd(){ return show(obj); } console.log(rd()); } const x={n:4}; outer(x);",
        "function f(p){ const o=p; const g=()=>o[\"a\"]; return g(); } const x={a:1}; console.log(f(x));",
        "function mk(){ return {a:1, s:\"xy\", arr:[1,2]}; } function f(){ let o=mk(); const g=()=>o[\"a\"]; return g(); } console.log(f());",
        // call roots and absent fields
        "function mk(){ return {a:1}; } console.log(mk().a);",
        "function id(o){ return o; } const x={a:1}; console.log(id(x).a);",
        "const o={a:1}; console.log(\"z=\"+o.z);",
        "const o={a:1}; console.log(\"z=\"+o[\"z\"]);",
        "const o={a:1}; console.log(o.z ?? 5);",
        "const o={a:1}; console.log(\"z=\"+o?.z);",
        // spread and comma reach the same fallback (spec A-1)
        "const a=[1,2]; console.log([...a]);",
        "function main(){ let n = 0; function bump() { n = n + 1; return 5; } let b = (bump(), 7); console.log(\"b=\" + b); } main();",
    ] {
        assert_e5506(&diagnostics_for(source), UNRES_READ, source);
    }
}

#[test]
fn spread_and_comma_get_the_neutral_message() {
    for source in [
        "const a=[1,2]; console.log([...a]);",
        "function main(){ let b = (1, 7); console.log(\"b=\" + b); } main();",
    ] {
        let diagnostics = diagnostics_for(source);
        assert_e5506(&diagnostics, "this expression is unavailable", source);
        assert!(
            !diagnostics.iter().any(|d| d.message.contains("reading `.")),
            "{source}: {diagnostics:?}"
        );
    }
}

#[test]
fn a_host_root_keeps_its_read() {
    for source in [
        "const f = Object.freeze(Math.log2); console.log(f(8));",
        "const finite = Number.isFinite; console.log(finite(1));",
        "const n = Object.freeze(Number[\"isNaN\"]); console.log(n(1));",
        "let t=globalThis.performance; t.now(); console.log(\"ok\");",
        "const p = Object.freeze(globalThis.String.fromCharCode); console.log(p(72));",
        "const o = Object.fromEntries([[\"a\", 1]]); console.log(o.a);",
        "let a = (console.log(\"x\"), 7); console.log(\"a=\" + a);",
    ] {
        assert_eq!(read_refusals(source), 0, "{source}: {:?}", diagnostics_for(source));
    }
}

#[test]
fn a_resolved_read_is_not_refused() {
    for source in [
        "const o={a:1}; console.log(o.a);",
        "class C{ constructor(){ this.a = 3; } } const c=new C(); console.log(c.a);",
        "function outer(){ const obj={n:4}; function rd(){ return obj.n; } return rd(); } console.log(outer());",
    ] {
        assert_eq!(read_refusals(source), 0, "{source}: {:?}", diagnostics_for(source));
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -j 8 -p kali_codegen member_provenance -- --test-threads=8`
Expected: `a_member_read_on_a_program_built_root_refuses` and `spread_and_comma_get_the_neutral_message` FAIL ("expected an E5506 containing …"). The other two pass.

- [ ] **Step 3: Extract the shared root verdict** (in `member_provenance.rs`)

Replace the `let root = …; let root_node = …; match root_node.kind { … }` tail of `unresolved_member_call_refuses` with:

```rust
        let root = self.receiver_chain_root(receiver);
        self.program_built_root_verdict(root).unwrap_or(false)
    }

    /// The root verdict the member-call and member-read gates share
    /// (unresolved-member-read spec §3.1): `Some(true)` refuse, `Some(false)`
    /// keep warn+0, `None` for a call or any other root, which each gate
    /// decides for itself.
    fn program_built_root_verdict(&self, root: LirNodeId) -> Option<bool> {
        let root_node = self.node(root);
        match root_node.kind {
            LirNodeKind::Literal => Some(true),
            LirNodeKind::Value if root_node.children.is_empty() => {
                Some(match root_node.text.as_deref() {
                    Some(name) if !name.is_empty() => {
                        // No lexical scope chain (the current body is not
                        // reachable from the module root): not proven, refuse.
                        let Some(scopes) = self.lexical_scopes() else {
                            return Some(true);
                        };
                        !self.root_has_host_provenance(name, &scopes, 0, &mut HashSet::new())
                    }
                    // `this`, `{}` or `[]`: a text-less childless Value.
                    // `this` in a method or constructor of a host-derived
                    // class is that class's instance (ruling R8). LIR spells
                    // `this` like `{}` / `[]`, so an empty literal start in
                    // such a method keeps warn+0 too.
                    _ => !self.emitting_method_of_host_derived_class(),
                })
            }
            // An array or object literal with two or more children.
            LirNodeKind::Value if root_node.text.is_none() && root_node.children.len() >= 2 => {
                Some(true)
            }
            _ => None,
        }
    }

    /// Unresolved-member-read spec §3.1: a member read that reached the
    /// placeholder fallback. Same roots as the call gate, plus a call root
    /// (`g().a`, `mk().a`), which refuses unless its callee is host.
    pub(crate) fn unresolved_member_read_refuses(&self, receiver: LirNodeId) -> bool {
        let root = self.receiver_chain_root(receiver);
        if let Some(verdict) = self.program_built_root_verdict(root) {
            return verdict;
        }
        if self.node(root).kind != LirNodeKind::Call {
            return false;
        }
        let Some(scopes) = self.lexical_scopes() else {
            return true;
        };
        !self.init_has_host_provenance(root, &scopes, 0, &mut HashSet::new())
    }
```

Keep `unresolved_member_call_refuses`'s doc comment, its early `receiver` extraction and its `.call` / `.apply` arm unchanged.

- [ ] **Step 4: Wire the read site** (in `operators.rs`, directly after the `capture_member_fallback_refusal` block that ends with `return self.deny_e5506(function, &message); }`)

```rust
                // Unresolved-member-read spec §3.2: a read off a value this
                // program built refuses rather than read the placeholder.
                // Host-rooted receivers keep warn+0 (builtin aliases such as
                // `Object.freeze(Math.log2)` store this `0` and never read it).
                if !is_unary_operator_text(op) && self.unresolved_member_read_refuses(arg) {
                    return self.deny_e5506(
                        function,
                        &kali_common::unresolved_member_read_unavailable_message(op),
                    );
                }
```

Then update the R-60 comment block above the capture check: keep it, and add one line saying that R-60's `fromEntries` receiver is host-rooted and still reaches the warning.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -j 8 -p kali_codegen member_provenance -- --test-threads=8`
Expected: all `member_provenance` tests pass, including the existing call-gate tests.

- [ ] **Step 6: Run the codegen crate**

Run: `cargo test -j 8 -p kali_codegen -- --test-threads=8`
Expected: green. If an existing codegen test fails, read it. If it pinned a silent `0` on a program-built root, that is an expected move: record it for Task 7's table and update it to assert the refusal. Any other failure: stop and report.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_codegen/src/emit/member_provenance.rs crates/kali_codegen/src/emit/member_provenance_tests.rs crates/kali_codegen/src/emit/operators.rs
git commit -m "feat(unresolved-member-read): refuse a fallback member read on a program-built root"
```

---

### Task 4: The store site

**Files:**
- Modify: `crates/kali_codegen/src/emit/member_provenance.rs` (add `unresolved_store_refusal`)
- Modify: `crates/kali_codegen/src/emit/operators.rs` (`emit_binary`'s final `_` arm, about line 2713)
- Test: `crates/kali_codegen/src/emit/member_provenance_tests.rs`

**Interfaces:**
- Consumes: `unresolved_member_read_refuses` (Task 3), `kali_common::unresolved_store_unavailable_message` (Task 2).
- Produces: `pub(crate) fn unresolved_store_refusal(&self, target: LirNodeId) -> Option<String>`.

- [ ] **Step 1: Write the failing tests**

```rust
const UNRES_STORE: &str = "no lowering for that store";

#[test]
fn a_store_to_a_const_binding_refuses() {
    for source in [
        "const x = 1; x = 2; console.log(\"r=\" + x);",
        "function main() { const x = 1; x = 2; console.log(\"r=\" + x); } main();",
    ] {
        let diagnostics = diagnostics_for(source);
        assert_e5506(&diagnostics, UNRES_STORE, source);
        assert_e5506(&diagnostics, "assigning to `x`", source);
    }
}

#[test]
fn a_store_to_a_free_global_keeps_its_lowering() {
    for source in [
        "globalThis.zz = 3; console.log(\"ok\");",
        "function mk(){ return {a:1}; } const o=mk(); o[\"a\"] = 5; console.log(o.a);",
        "class C{ constructor(){ this.a = 3; } set(v){ this.a = v; } } const c=new C(); c.set(5); console.log(c.a);",
    ] {
        let diagnostics = diagnostics_for(source);
        assert!(
            !diagnostics.iter().any(|d| d.message.contains(UNRES_STORE)),
            "{source}: {diagnostics:?}"
        );
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -j 8 -p kali_codegen member_provenance -- --test-threads=8`
Expected: `a_store_to_a_const_binding_refuses` FAILS. `a_store_to_a_free_global_keeps_its_lowering` passes.

- [ ] **Step 3: Implement the helper** (in `member_provenance.rs`, after `unresolved_member_read_refuses`)

```rust
    /// Unresolved-member-read spec §3.3 and A-2: a plain `=` that reached
    /// `emit_binary`'s final fallback. The target's chain root decides, as
    /// for a read; an identifier target is its own root, so a store to a
    /// `const` refuses and a store to a free global keeps warn+0.
    pub(crate) fn unresolved_store_refusal(&self, target: LirNodeId) -> Option<String> {
        let target = self.unwrap_transparent(target);
        let node = self.node(target);
        if node.kind != LirNodeKind::Value {
            return None;
        }
        let name = node.text.as_deref().filter(|name| !name.is_empty())?;
        let shown = match node.children.len() {
            0 => name.to_string(),
            1 | 2 => format!(".{name}"),
            _ => return None,
        };
        self.unresolved_member_read_refuses(target)
            .then(|| kali_common::unresolved_store_unavailable_message(&shown))
    }
```

- [ ] **Step 4: Wire the store site** (in `operators.rs`, the first lines of `emit_binary`'s final `_ => {` arm, before the "unsupported binary operator" warning)

```rust
                // Unresolved-member-read spec §3.3: a plain `=` that no lane
                // stored refuses when its target is rooted at a value this
                // program built (R-29's `const x = 1; x = 2` is the measured
                // case), instead of evaluating `left + right`.
                if op == "=" {
                    if let Some(message) = self.unresolved_store_refusal(left) {
                        return self.deny_e5506(function, &message);
                    }
                }
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -j 8 -p kali_codegen -- --test-threads=8`
Expected: green. An unexpected failure is handled as in Task 3 Step 6.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_codegen/src/emit/member_provenance.rs crates/kali_codegen/src/emit/member_provenance_tests.rs crates/kali_codegen/src/emit/operators.rs
git commit -m "feat(unresolved-member-read): refuse a fallback store whose target is program-built"
```

---

### Task 5: The `check` mirror for absent fields

**Files:**
- Modify: `crates/kali_types/src/context.rs` (struct field about line 48, initializer about line 113)
- Modify: `crates/kali_types/src/resolve/member.rs` (`resolve_member_expression` at line 7, `reject_member_call_gates`, and a new `reject_unresolved_member_read`)
- Modify: `crates/kali_types/src/resolve/expression.rs` (the `Expression::UnaryExpression` arm, about line 1847)
- Test: `crates/kali_types/src/resolve/member_tests.rs`

**Interfaces:**
- Consumes: `kali_common::unresolved_member_read_unavailable_message` (Task 2), and the existing `known_member_set`, `assigned_property_names`, `OBJECT_PROTOTYPE_NAMES` and `super::expression::unwrap_transparent`.
- Produces: `pub(crate) read_mirror_skipped_members: HashSet<usize>` on `TypeContext`, and `pub(crate) fn reject_unresolved_member_read(&mut self, member: &MemberExpression)`.

- [ ] **Step 1: Write the failing tests** (append to `member_tests.rs`, after `check_stays_quiet_where_it_cannot_know`)

```rust
const UNRES_READ: &str = "no lowering for that read";

fn read_mirror_count(source: &str) -> usize {
    e5506_messages(source)
        .iter()
        .filter(|m| m.contains(UNRES_READ))
        .count()
}

#[test]
fn check_refuses_an_absent_field_read_on_a_known_member_set() {
    for source in [
        "const o={a:1}; console.log(\"z=\"+o.z);",
        "const o={a:1}; console.log(\"z=\"+o[\"z\"]);",
        "function main(){ const o={a:1}; console.log(\"z=\"+o.z); } main();",
        "const o={a:1}; console.log(o.z ?? 5);",
        "const o={a:1}; console.log(\"z=\"+o?.z);",
    ] {
        assert_eq!(read_mirror_count(source), 1, "{source}");
    }
}

#[test]
fn check_read_mirror_skips_callees_typeof_and_unknown_sets() {
    for source in [
        // callees belong to the call mirror (one defect, one diagnostic)
        "const o={k:1}; console.log(o.zork(4));",
        "const o={k:1}; o.zork?.();",
        // typeof goes through a different placeholder (spec A-3)
        "const o={a:1}; console.log(typeof o.z);",
        "const o={a:1}; console.log(typeof (o.z));",
        // present, prototype, assigned
        "const o={a:1}; console.log(o.a);",
        "const o={a:1}; console.log(o.toString());",
        "const o={a:1}; o.z = 5; console.log(o.z);",
        // unknown member sets: run-only (spec §3.4)
        "let o={a:1}; console.log(o.z);",
        "function f(p){ return p.z; }",
        "function mk(){ return {a:1}; } console.log(mk().a);",
    ] {
        assert_eq!(read_mirror_count(source), 0, "{source}");
    }
}

#[test]
fn check_read_mirror_counts_one_diagnostic_per_callee_with_the_call_mirror() {
    // The call mirror's count stays 1 with the read mirror present.
    for source in [
        "const o={k:1}; console.log(o.zork(4));",
        "const o={k:1}; o.zork?.();",
    ] {
        assert_eq!(unres_count(source), 1, "{source}");
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -j 8 -p kali_types check_ -- --test-threads=8`
Expected: `check_refuses_an_absent_field_read_on_a_known_member_set` FAILS (count 0). The other two pass.

- [ ] **Step 3: Add the field** (in `context.rs`)

Struct, after `assigned_property_names`:

```rust
    /// Members the absent-field read mirror must not judge, by address:
    /// call callees (the call mirror owns them) and `typeof` operands
    /// (unresolved-member-read spec §3.4, A-3).
    pub(crate) read_mirror_skipped_members: HashSet<usize>,
```

Initializer, after `assigned_property_names: …,`:

```rust
            read_mirror_skipped_members: HashSet::new(),
```

- [ ] **Step 4: Record the skips**

In `member.rs`, `reject_member_call_gates`, add as its first line:

```rust
        self.read_mirror_skipped_members
            .insert(member as *const MemberExpression as usize);
```

In `expression.rs`, at the top of the `Expression::UnaryExpression(expr) => {` arm:

```rust
                if expr.operator == "typeof" {
                    if let Expression::MemberExpression(member) = unwrap_transparent(&expr.argument) {
                        self.read_mirror_skipped_members
                            .insert(member.as_ref() as *const MemberExpression as usize);
                    }
                }
```

(`MemberExpression` is boxed inside `Expression`. If the `as_ref()` spelling does not type-check, use `&**member as *const MemberExpression as usize`, which yields the same address that `resolve_member_expression` receives.)

- [ ] **Step 5: Implement the mirror** (in `member.rs`)

Call it first in `resolve_member_expression`:

```rust
    pub(crate) fn resolve_member_expression(&mut self, expr: &MemberExpression) {
        self.reject_unresolved_member_read(expr);
        self.reject_unprovable_string_length(expr);
```

Add next to `reject_unresolved_member_call`:

```rust
    /// The `check` mirror of the unresolved-member-read gate (spec §3.4): a
    /// property name missing from a `const` object literal's keys or a
    /// program class chain's members. Callees and `typeof` operands are
    /// skipped (A-3); wherever the member set is unknown `kali run` alone
    /// refuses.
    pub(crate) fn reject_unresolved_member_read(&mut self, member: &MemberExpression) {
        if self
            .read_mirror_skipped_members
            .contains(&(member as *const MemberExpression as usize))
        {
            return;
        }
        let Some(name) = member.property.as_deref() else {
            return;
        };
        if kali_common::OBJECT_PROTOTYPE_NAMES.contains(&name)
            || self.assigned_property_names.contains(name)
        {
            return;
        }
        let Some(members) = self.known_member_set(&member.object) else {
            return;
        };
        if members.contains(name) {
            return;
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            kali_common::unresolved_member_read_unavailable_message(name),
        ));
    }
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -j 8 -p kali_types -- --test-threads=8`
Expected: green, including the existing `check_refuses_a_missing_method…` and `check_stays_quiet_where_it_cannot_know`. If an existing test now sees an extra read diagnostic, the mirror is judging a callee or a `typeof` operand it should skip. Fix the skip; never weaken the test.

- [ ] **Step 7: Verify `run` refuses every mirror row**

Run: `cargo build -j 8 -p kali_cli`, then run each row of `check_refuses_an_absent_field_read_on_a_known_member_set` through `target/debug/kali run` and `target/debug/kali check` (write each to a scratch file under the session scratchpad).
Expected: both commands exit 1 with E5506 on every row. **A row where `run` exits 0 is a `check` / `run` disagreement: stop and report it.** Do not widen `run` to match.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_types/src/context.rs crates/kali_types/src/resolve/member.rs crates/kali_types/src/resolve/expression.rs crates/kali_types/src/resolve/member_tests.rs
git commit -m "feat(unresolved-member-read): check mirror for absent-field reads on known member sets"
```

---

### Task 6: Black-box cases

**Files:**
- Create: `crates/kali_cli/tests/cases/soundness/unresolved_member_read.toml`

**Interfaces:**
- Consumes: the binary built in Task 5 Step 7.
- Produces: the case file that Task 7's full run includes.

- [ ] **Step 1: Write the case file**

Header and constants:

```toml
# Cases for the unresolved-member-read project (spec
# docs/superpowers/specs/2026-10-06-unresolved-member-read-design.md).
#
# A member read, or a plain `=` store, that reaches codegen's silent-`0`
# fallback refuses with E5506 under `run` when its receiver (or target) is
# rooted at a value this program built. `check` refuses only the absent-field
# shape on a `const` object literal (spec §3.4); every other refused row exits
# 0 under `check`, which pins the gap. Host-rooted receivers keep node's
# output. Each rationale gives kali's `run` output at the baseline `28c97e4ed`
# (tools/array-return-probes/baseline-umr.tsv) and node v26.10.0's.
#
# `[source]` keys are one file per program, because `[source]` is file-wide.

[constants]
UNREAD = "no lowering for that read"
UNSTORE = "no lowering for that store"
NEUTRAL = "this expression is unavailable"
```

`[source]`: one key per program, using the Task 1 probe programs (`c8.js`, `c9.js`, `e1.js`, `m6.js`, `q8.js`, `mkcall.js`, `idcall.js`, `absent.js`, `absent_bracket.js`, `absent_nullish.js`, `absent_optional.js`, `typeof.js`, `const_store.js`, `spread.js`, `comma.js`) plus:

```toml
"shadow_math.js" = '''
let Math = {a:1}; console.log(Math.b);
'''
"ok_math_alias.js" = '''
const f = Object.freeze(Math.log2); console.log(f(8));
'''
"ok_number_alias.js" = '''
const n = Object.freeze(Number["isNaN"]); console.log(n(1));
'''
"ok_performance.js" = '''
let t=globalThis.performance; t.now(); console.log("ok");
'''
"fromentries.js" = '''
const o = Object.fromEntries([["a", 1]]); console.log(o.a);
'''
"ok_present.js" = '''
const o={a:1}; console.log(o.a);
'''
"ok_class_field.js" = '''
class C{ constructor(){ this.a = 3; } } const c=new C(); console.log(c.a);
'''
"ok_bracket_store.js" = '''
function mk(){ return {a:1}; } const o=mk(); o["a"] = 5; console.log(o.a);
'''
"ok_global_store.js" = '''
globalThis.zz = 3; console.log("ok");
'''
"ok_capture_decl.js" = '''
function outer(){ const obj={n:4}; function rd(){ return obj.n; } return rd(); } console.log(outer());
'''
```

Cases. Every rationale follows this shape: "At `28c97e4ed` kali printed `<baseline>` at exit 0 where node v26.10.0 prints `<node>`." Take the values from `baseline-umr.tsv`, or measure them for the extra programs.

- `run` refusals (`exit = "failure"`, `stdout = ""`, `stderr_contains = ["E5506", "${UNREAD}"]`): c8, c9, e1, m6, q8, mkcall, idcall, absent, absent_bracket, absent_nullish (its rationale names the A-4 capability loss: kali printed node's `5` by coincidence), absent_optional.
- `run` refusal with the neutral text (`stderr_contains = ["E5506", "${NEUTRAL}"]`): spread, comma.
- `run` store refusal (`stderr_contains = ["E5506", "${UNSTORE}", "assigning to `x`"]`): const_store.
- `check` refusals (`args = ["check", …]`, `exit = "failure"`, `stderr_contains = ["E5506", "${UNREAD}"]`): absent, absent_bracket, absent_nullish, absent_optional.
- `check` gap (`args = ["check", …]`, `exit = "success"`): c8, c9, e1, m6, q8, mkcall, idcall, const_store, spread, comma, typeof. Each rationale says "the `check` / `run` gap of spec §3.4 (A-3 for typeof): `run` refuses, `check` cannot see the fallback".
- typeof under `run`: `exit = "success"`, `stdout = "0\n"`, with a rationale stating WRONG ON PURPOSE (node prints `undefined`; spec §5 residue A-3).
- shadow_math: `run` and `check` both `exit = "failure"`, `stderr_contains = ["E5506"]` only (it refuses through the fixed-shape gate at the baseline).
- Exemption controls under `run`, `exit = "success"`: ok_math_alias `3\n`, ok_number_alias `false\n`, ok_performance `ok\n`.
- fromentries under `run`: `exit = "success"`, `stdout = "0\n"`, with a rationale stating WRONG ON PURPOSE (R-60; the host-rooted receiver is spec §5 residue).
- No regression under `run`, `exit = "success"`: ok_present `1\n`, ok_class_field `3\n`, ok_bracket_store `5\n`, ok_global_store `ok\n`, ok_capture_decl `4\n`.

Name cases after their claim, as in `soundness/unresolved_member_call.toml` (for example `a_member_read_on_a_call_result_refuses_under_run`, `an_absent_field_read_on_a_const_object_literal_refuses_under_check`).

- [ ] **Step 2: Run the file**

Run: `cargo test -j 8 -p kali_cli --test cases -- --test-threads=8 soundness/unresolved_member_read`
Expected: all pass. A failing row means the case or the measured value is wrong. Re-measure against node and the binary; never edit an expected value to make a red case pass without a measurement.

- [ ] **Step 3: Commit**

```bash
git add crates/kali_cli/tests/cases/soundness/unresolved_member_read.toml
git commit -m "test(unresolved-member-read): black-box cases for refusals, the check gap and host controls"
```

---

### Task 7: Full run and re-pins

**Files:**
- Modify: the case files named in the table below
- Create: `$SCRATCH/umr-cases.log` (outside the repo; `$SCRATCH` is the session scratchpad)

**Interfaces:**
- Consumes: everything above.
- Produces: a green `cases` target, plus the list of moved cases that Task 8 records.

- [ ] **Step 1: Run the full case target under the watchdog**

```bash
tools/watchdog.sh $SCRATCH/watch.log & WD=$!
cargo test -j 8 -p kali_cli --test cases -- --test-threads=8 > $SCRATCH/umr-cases.log 2>&1; echo "exit=$?"
kill $WD; grep "test result" $SCRATCH/umr-cases.log; tail -2 $SCRATCH/watch.log
```

Expected: a failure count that matches the table below and nothing else. The design-time measurement (blanket refusal) failed 372 trials in 50 cases. This gate keeps the ~30 group-B cases, so expect roughly 15 cases.

- [ ] **Step 2: List the moved cases**

Run: `grep -E '\.\.\. FAILED' $SCRATCH/umr-cases.log | awk '{print $2}' | sed 's/::[^:]*$//;s/\[ext=.*\]//' | sort -u`

Compare against this table (spec §6.1):

| case(s) | re-pin |
|---|---|
| `oracle/tier2` `r21f_absent_object_field_const_receiver_{module_scope,in_function}` | `verdict = "fail_closed"`; rationale: strike the SILENT measurement through, then add "RE-MEASURED 2026-10-xx at `<sha>`: FAIL_CLOSED (unresolved-member-read §3.2)" |
| `oracle/tier2` `r25l_array_spread_console_log_residual_*` | as above (A-1) |
| `oracle/tier2` `r27_comma_operator_value_*` | as above (A-1) |
| `oracle/tier3` `r29_assignment_to_a_const_is_ignored_*` | `verdict = "fail_closed"` (A-2); the rationale records that node exits 1 with a TypeError and kali now exits 1 with E5506 |
| `object/property_key_identity` escaped-quote and member-probe rows (four, not the from-entries rows) | `exit = "failure"`, `stdout = ""`, `stderr_contains = ["E5506", "no lowering for that read"]`; rationale: strike the pinned `0` through |
| `object/computed_member_static_name` `an_absent_property_after_a_successful_fold_is_r21s_lane_not_this_projects`, `the_argv_index_lane_disagrees_…` | as above |
| `soundness/r06_object_init::returned_object_member_read_no_worse` | as above; the case's own rationale already allows fail-closed |
| `soundness/bitwise_compound` (the two rows) | assert the new E5506 text in place of E4201 or the dropped write |
| `misc/arena_reclamation_runtime_sandboxed::function_scratch_is_reclaimed` | change `console.log(total + taints[0].v - taints[0].v);` to `console.log(total);` in the `.ts` fixture; same expected stdout. The rationale records why: the field read was a cancelling pair of zeros |

**If any other case moves:** stop. Report the case, its program and the old and new output to the human partner. That includes `browser/promise_all_settled_bundle` and `browser/template_literal_dynamic_import_harness`, and any group-B builtin-alias case.

- [ ] **Step 3: Re-pin, one file at a time**

For each file, edit the cases and re-run only that file: `cargo test -j 8 -p kali_cli --test cases -- --test-threads=8 <dir>/<file>`.
For the oracle files, the runner compares against node at test time, so a verdict flip must match a live measurement.

- [ ] **Step 4: Re-run the full target under the watchdog** (Step 1's command)

Expected: `test result: ok`.

- [ ] **Step 5: Re-run the probe sweep**

Run: `tools/array-return-probes/run.sh $SCRATCH/umr-after.tsv; diff <(grep '^umr_' $SCRATCH/umr-after.tsv) tools/array-return-probes/baseline-umr.tsv`
Expected: every `umr_` row except `umr_typeof` moves from a printed value to E5506; `umr_typeof` is unchanged.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_cli/tests/cases
git commit -m "test(unresolved-member-read): re-pin cases the refusal moved (spec §6.1)"
```

---

### Task 8: Register, ranking, followups and errors doc

**Files:**
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md` (§0.2 rows R-21 at about line 282, R-25 at about 286, R-27 at about 288, R-29 at about 290, and the §0.2 changelog)
- Modify: `tools/blast-radius/clusters.json` and `docs/superpowers/followups/blast-radius-ranking.md`, only if a SILENT lane left the filter
- Modify: `docs/superpowers/followups/captured-bindings-discovered-defects.md` (§5.10, and §4's "Objects passed through a `const` copy of a parameter")
- Create: `docs/superpowers/followups/unresolved-member-read-discovered-defects.md`
- Modify: `specs/15-errors.md` (under E5506)

**Interfaces:**
- Consumes: Task 7's list of moved cases.

- [ ] **Step 1: Confirm the register gate fails first**

Run: `cargo test -j 8 -p kali_blast_radius -- --test-threads=8 every_zero_two_row`
Expected: FAIL, naming R-21, R-25, R-27 and R-29: the rows disagree with the cases they assert.

- [ ] **Step 2: Re-derive the rows**

Follow the R-12 / R-13 precedent already in §0.2, and keep old text struck through:

- **R-21:** the `r21f` lane moves SILENT → FAIL_CLOSED. The other lanes stay SILENT, so the entry is not retired.
- **R-25:** `r25l` moves to FAIL_CLOSED. Both lanes are now FAIL_CLOSED, but node's `[ 1, 2 ]` is still not produced. Like R-12, the entry leaves the SILENT filter **without being fixed** and is **not retired**.
- **R-27:** both scopes move to FAIL_CLOSED, with the same "leaves the filter, not fixed" wording.
- **R-29:** ACCEPTS_INVALID → FAIL_CLOSED (A-2).

Add a dated changelog bullet at the end of §0.2 naming this project, the commit and the four rows.

- [ ] **Step 3: Re-run the gate**

Run: `cargo test -j 8 -p kali_blast_radius -- --test-threads=8`
Expected: green. If the ranking's clusters test fails because R-25 or R-27 left the SILENT filter, go to Step 4. Otherwise skip Step 4.

- [ ] **Step 4: Ranking amendment (only if Step 3 requires it)**

Remove the departed entries from `tools/blast-radius/clusters.json`, following the R-12 / R-13 precedent. If a cluster loses all its members, its definition goes too, as forced by `crates/kali_blast_radius/src/ranking.rs`.
Run: `cargo run -j 8 -p kali_blast_radius --example rank > $SCRATCH/rank.txt`. Replace the GENERATED block in `blast-radius-ranking.md` with that stdout, verbatim, and add the next numbered §6 amendment, written like the existing ones (what left, why, and what did not move: R-60).
Re-run: `cargo test -j 8 -p kali_blast_radius -- --test-threads=8`. Expected: green.

- [ ] **Step 5: Followups**

In `captured-bindings-discovered-defects.md`, §5.10 gets a dated note at its top: "CLOSED (refused) 2026-10-xx by unresolved-member-read (`docs/superpowers/specs/2026-10-06-unresolved-member-read-design.md`): c8, c9, e1, m6 and q8 refuse with E5506 under `run`; `check` exits 0 (spec §3.4 gap)." Add the same note to §4's "Objects passed through a `const` copy of a parameter" item.

Create `unresolved-member-read-discovered-defects.md` on the convention of the other `*-discovered-defects.md` files. Sections:

1. Residue: R-60 / `fromEntries`; host-object reads; the `check` / `run` gap list; `typeof` on a program-built value; a comma expression with a host-rooted first operand.
2. The A-4 capability loss (`o.z ?? d`).
3. Anything Tasks 3–7 recorded.

- [ ] **Step 6: Errors doc**

In `specs/15-errors.md`, under the E5506 usage list, add: "- a member read or a plain `=` store that kali cannot lower, on a receiver or target the program built (an object or array literal, a parameter, a user-function result, or a binding of one); host-rooted receivers are not refused".

- [ ] **Step 7: Commit**

```bash
git add docs specs tools/blast-radius
git commit -m "docs(unresolved-member-read): register rows, ranking, followups and E5506 entry"
```

---

### Task 9: Workspace verification

**Files:** none new.

- [ ] **Step 1: Run the workspace tests under the watchdog**

```bash
tools/watchdog.sh $SCRATCH/watch-ws.log & WD=$!
cargo test -j 8 --workspace -- --test-threads=8 > $SCRATCH/ws.log 2>&1; echo "exit=$?"
kill $WD; grep -E "test result|FAILED" $SCRATCH/ws.log | sort | uniq -c | head; tail -2 $SCRATCH/watch-ws.log
```

Expected: exit 0, and no `FAILED` lines. A failure outside the Task 7 table follows the stop rule.

- [ ] **Step 2: Re-read the docs for consistency**

Check that `specs/15-errors.md`, the register rows, the followups and the spec's §5 residue all describe the same behaviour, and that `specs/19-feature-maturity.md` is unchanged.

- [ ] **Step 3: Commit any doc fixes**

```bash
git add -A docs specs
git commit -m "docs(unresolved-member-read): consistency pass"
```

(Skip this step if nothing changed.)
