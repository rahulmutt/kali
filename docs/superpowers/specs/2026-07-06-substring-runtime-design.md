# Runtime `substring` + `.length`, F1 store gate, F2 ternary — design

**Date:** 2026-07-06
**Status:** Approved (design)
**Series:** Runtime strings & dynamic tables for verbatim fasta — **Spec 2 of 6**
(Spec 1: `docs/superpowers/specs/2026-07-06-runtime-string-value-flow-design.md`,
shipped as PR #9, merge `09d003cb7`.)

## Series context

The CLBG fasta target compiles Ian Osgood's upstream `fasta-node-1` **verbatim**.
Spec 1 shipped the linchpin: string-typed vars/params/returns carry their tagged
linear-memory handle (`STRING_HANDLE_TAG | offset << 32 | len`) through `+` and
`console.log`, with fail-closed rejection everywhere else. This spec is the next
dependency in order: upstream `fastaRepeat` is

```js
function fastaRepeat(n, seq) {
  var seqi = 0, lenOut = 60;
  while (n > 0) {
    if (n < lenOut) lenOut = n;
    if (seqi + lenOut < seq.length) {
      ret(seq.substring(seqi, seqi + lenOut));
      seqi += lenOut;
    } else {
      var s = seq.substring(seqi);
      seqi = lenOut - s.length;
      ret(s + seq.substring(0, seqi));
    }
    n -= lenOut;
  }
}
```

which needs runtime `substring` (all three arg shapes), **runtime `.length`**
(`seq.length` on a string param, `s.length` on a substring result — no other
spec in the series owns `.length`, so it lands here), string `+` on substring
results (Spec 1), and `console.log` of runtime strings (Spec 1).

## Scope (user-approved)

Bundled into this spec:

1. **Runtime `substring`** — relax E5506 for string-valued receivers with
   runtime integer bounds.
2. **Runtime `.length`** on string-valued receivers.
3. **F1 (Spec 1 inventory)** — fail-closed gate for runtime-string rhs at
   element/field stores and array-literal materialization (closes the
   `arr[0]=taintedConcat; arr[0]=="xy"` silent-wrong launder).
4. **F2 (Spec 1 inventory)** — **full** ternary (`?:`) support: parse +
   resolve + codegen. Today `kali_parser` never builds `ConditionalExpression`
   and `b ? 1 : 2` silently drops `? 1 : 2` for ALL types — the
   highest-severity pre-existing miscompile in the inventory.

Out of scope: runtime `slice`/`charAt`/`repeat` receivers (same pattern, no
fasta need — YAGNI), `Array.prototype.join` (Spec 3), `for..in` dynamic keys
(Spec 4), `process.argv`/string→number coercion (Spec 5), UTF-16 semantics for
non-ASCII strings (rejected fail-closed instead, see below).

## Approach decision

Three candidates for producing the slice value at runtime; the `kali_types`
gate relaxation and repr plumbing are identical in all three.

- **A — zero-copy re-tag, pure wasm ALU (CHOSEN).** A substring of
  `TAG | off<<32 | len` with clamped bounds `[s, e)` is
  `TAG | (off+s)<<32 | (e−s)`: a few inline i64 ops. No new host import (the 4
  hand-mirrored browser JS import lists stay untouched — a known LinkError
  footgun), no allocation (fasta at N=25M does hundreds of thousands of
  substrings), O(1) regardless of slice length. Sound because guest string
  memory is immutable (the interned pool and `string_concat` outputs are never
  mutated in place). Verified: host-side decoding
  (`kali_runtime/src/host/memory.rs` — `decode_string_handle_bytes`,
  `read_guest_string_handle`) reads any `(offset, len)` generically, so slice
  handles flow through `string_concat`/`console.log` with zero host changes.
  The one hard part: the slice **aliases the receiver's memory**, so escape
  analysis must model it (below).
- **B — host-call substring** (fresh global-arena allocation per call):
  escape-independent, but alloc+copy in the hot loop plus a new import
  mirrored across 4 JS glue lists. Rejected.
- **C — guest-side arena copy** (`__alloc` + `memory.copy` inline): no import,
  but pays both the allocation churn AND the escape reasoning. Rejected.

## Design

### 1. Repr/type layer (`kali_types` repr_infer + `kali_common` ReprTable)

Two additions to Spec 1's string-seed BFS over the shared value-flow graph
(`solve_reach`/`build_adjacency`):

- **Substring nodes as string sources.** A `.substring(...)` call whose
  receiver is string-reachable becomes a `Repr::String` node, flowing onward
  through `+`, `console.log`, assignments, params, returns exactly like Spec 1
  vars. It also joins the **concat-taint set**
  (`ReprTable::is_string_concat_tainted*`): a substring result is a runtime,
  non-interned string, so `==`/`!=`/`!`/condition positions reject it instead
  of performing a wrong handle-identity comparison. Interned-literal `==`
  stays byte-identical to base.
- **ASCII-provenance bit.** Guest strings are raw UTF-8 bytes and the handle
  `len` is a *byte* count; JS `substring`/`.length` count UTF-16 code units.
  The two agree only for pure-ASCII strings (why the old gate demanded ASCII
  literals). A second reachability pass is seeded at **non-ASCII string
  literals** only — concat/template/`int_to_string`/`float_to_fixed` outputs
  preserve ASCII-ness, so literals are the sole seeds. A node is
  **ASCII-provable** iff string-reachable and NOT reached by a non-ASCII seed.
  `substring`/`.length` compile only on ASCII-provable receivers. Merge points
  (param/return/phi joins) union the non-ASCII taint — fail-closed by
  construction. fasta is all-ASCII and stays green.

### 2. Gate relaxation (`kali_types/src/static_analysis/string.rs`)

`resolve_string_substring_member_call` gains a runtime lane, checked AFTER the
existing static-literal fold lane (which stays byte-identical):

- receiver ASCII-provable string-valued, AND
- 0–2 args, each **int-repr** — a float-repr bound rejects (JS `ToInteger`
  NaN/fraction/±Infinity semantics deliberately unimplemented; fasta bounds
  are integer arithmetic), AND
- otherwise the existing E5506 diagnostic, message extended to name the
  failing condition (non-ASCII provenance / non-integer bounds).

`.length` on a string-valued receiver gets the same explicit accept
(ASCII-provable) / reject treatment. Today `.length` is **array-biased**:
codegen interprets `x.length` as an array element-count read
(`kali_codegen/src/emit/object.rs`, the one-child `Value("length")` shape), so
a string-valued receiver reaching that lane is a potential silent-wrong. The
string rule must fire first: string-valued receiver → handle low-32 extract
when ASCII-provable, reject otherwise — never the array lane.

### 3. Codegen (`kali_codegen` emit)

`is_string_valued` (emit/operators.rs) recognizes substring calls via the
existing scope-chain walk (locals-first, `_start` only for true free refs).
Lowering is inline i64 ALU on the handle — no host import, no allocation:

```
len = handle & 0xFFFF_FFFF
off = (handle >> 32) & 0x7FFF_FFFF
s   = min(max(start, 0), len)          // start defaults to 0
e   = min(max(end,   0), len)          // end defaults to len (0/1-arg forms)
if s > e: swap(s, e)                   // full JS substring clamp semantics
result  = TAG | (off + s) << 32 | (e − s)
.length = handle & 0xFFFF_FFFF
```

A zero-length slice yields a `len == 0` handle; the host reads it as `""`.

### 4. Escape flow (`kali_mir/src/analysis/escape_flow.rs`)

The slice aliases the receiver's memory. One new edge kind in the existing
interprocedural fixpoint (PR #7): **substring-result → receiver**. The result
inherits the receiver's may-heap classification, and the result escaping
implies the receiver escapes (so a slice of an arena-allocated concat string
can never outlive that arena's `__arena_reset` — the use-after-reset class the
PR #8 whole-branch review caught). A slice of an interned literal stays
non-heap: always safe, no arena interaction. This edge is the spec's soundness
keystone.

### 5. F1 — fail-closed store gate

Gate a **runtime** string-valued rhs (string-reachable, not the static fold
lane) at four sites in `kali_types`:

- the two element-store lowering paths (indexed assignment),
- array-literal materialization containing runtime string-valued elements,
- field stores on objects.

Each rejects with a targeted E5506-family diagnostic ("string element/field
stores are unavailable in the current direct-runtime path…"). Fold-lane-aware:
`const a = ["x","y"]; a.join(",")` is fully static and MUST stay green — the
gate tests runtime string-reachability, not literal-ness. Substring results
widen exactly this launder surface, which is why F1 ships in this spec. Real
string-element store support arrives with Spec 3; this is
reject-don't-miscompile only.

### 6. F2 — full ternary support

- **Parser (`kali_parser`):** build `ConditionalExpression` — right-
  associative, precedence between assignment and `??`/`||`; arms parse as
  assignment-expressions. (Today the parser stops after the condition and
  silently drops `? consequent : alternate`.)
- **Resolver/repr:** Spec 1's existing-but-dead ternary gate goes live. Arms
  merge like a phi: same repr → that repr; both string → `Repr::String` with
  taint and non-ASCII bits unioned; mixed string/number arms → fail-closed
  conflict diagnostic (the Spec 1 monotone-conflict pattern for mixed
  returns). The condition follows existing truthiness rules, including Spec
  1's rule that concat-tainted strings reject in condition position. A ternary
  in a never-called function must still compile (`kali check`-only benchmark
  fixtures depend on the Spec 1 precedent).
- **Codegen:** wasm `if`/`else` with a typed result (i64 or f64 by repr) — NOT
  `select`, because JS evaluates only the taken arm (laziness is observable
  via side effects).
- **Escape flow:** the ternary result aliases BOTH arms — two edges of the
  same alias kind as §4.

### 7. Error handling — fail-closed matrix

| Case | Behavior |
|---|---|
| `substring`/`.length` on non-ASCII-provable receiver | E5506, names non-ASCII provenance |
| `substring` bound is float-repr / non-int | E5506 (ToInteger unimplemented) |
| substring result in `==`/`!=`/`!`/condition | reject via concat-taint (Spec 1 machinery) |
| substring result in relational / bitwise / non-`+` arith | reject (Spec 1 proven-string rule) |
| runtime string rhs at element/field store or array literal | F1 reject |
| ternary arm repr conflict (string vs number) | conflict diagnostic |
| ternary in never-called function | still compiles |
| slice of arena string escaping its scope | alias edge escalates receiver (correct compile, not an error) |

Base-behavior invariants: static-literal substring fold lane byte-identical;
interned-literal `==` byte-identical; float axis untouched; existing E5506
static lane untouched for non-string-valued receivers.

### 8. Testing

- **Unit — `kali_types`:** gate accept/reject matrices per the table above;
  ASCII-provenance propagation including merge points (param joins, ternary
  arms, concat chains); F1 site coverage (both element-store paths, array
  literal, field store; fold lane stays green). **`kali_codegen`:** retag
  arithmetic (clamp, swap, 0/1/2-arg forms, empty slice, chained
  `s.substring(1).substring(1)`); ternary `if`/`else` emission and laziness.
  **escape_flow:** slice-escapes ⇒ receiver-escalates; interned-literal slice
  stays non-heap; the slice-outliving-arena shape from the PR #8 review class;
  ternary both-arm aliasing.
- **Integration — `kali_cli` fixtures (node host):** a **fastaRepeat-shaped
  fixture** — pinned `n` (argv is Spec 5), upstream's exact
  substring/`+`/`.length` code shape, golden byte-for-byte vs `node`;
  substring edge fixtures (bound swap, out-of-range clamp); ternary fixtures
  (int/float/string arms, nested, side-effecting arms proving laziness); F1
  reject fixtures pinning the `arr[0]==` launder to a compile error; xfail
  pins for every formerly-silent-wrong case this spec converts to a
  diagnostic.
- **Gates:** the standing 5-crate green gate, `cargo clippy --workspace -- -D
  warnings`, `cargo fmt` — all CI-exact. **Browser:** no new imports, so no
  JS-glue import-list sync; one browser smoke via the existing harness
  confirming substring output parity.

## Success criteria

1. The fastaRepeat-shaped fixture compiles and runs byte-for-byte against
   `node` under the node host.
2. Every row of the fail-closed matrix has a test; no formerly-silent-wrong
   case remains silent.
3. `b ? x : y` works for int/float/string arms with lazy evaluation; the
   parser drop-miscompile is gone.
4. The `arr[0]=taintedConcat; arr[0]=="xy"` launder is a compile error.
5. Full workspace gate green (tests + clippy `-D warnings` + fmt).
