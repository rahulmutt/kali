//! Stage C (closures) — dynamic-env safety gate for capture invocation.
//!
//! ## The falsified premise this module closes (stage-review CRITICAL)
//!
//! `emit_function_env_prologue` links a new env record's parent to the
//! INCOMING `current_env` — i.e. the DYNAMIC caller's env — and the capture
//! lowering (`resolve_capture_access`) addresses cells against `current_env`
//! directly. The capture analysis (`derive_env_plans`, `mir_depth`) is
//! LEXICAL. The two agree only when a capturing function is invoked while its
//! lexical owner's record is the active `current_env`. When a capturer is
//! invoked while a SIBLING env-owner's record is active (e.g. `inc` — which
//! captures `outer`'s `c` — called from inside `sib`, which owns its own `d`
//! cell), cell addressing resolves against the WRONG record: a silent
//! cross-binding memory corruption (design spec §3.4 was falsified by the
//! final stage review; see `stageC-closures-triage.md`).
//!
//! ## The gate (fail closed — the lexical-parent-links rewrite is out of scope)
//!
//! A function whose capture lowering is ENGAGED (>=1 `mir_depth == 1` ref
//! whose owner-keyed cell is promotable — exactly the
//! `resolve_capture_access` engagement predicate) may only be invoked from
//! call sites where `current_env` PROVABLY holds its owner's record.
//! Everything else is rejected E5506 at compile time, which un-lowers the
//! program to the pre-Stage-C reject (the same shapes were E5506 at base
//! `a57cd09d5`).
//!
//! Sound conservative approximation (the `escape_flow.rs` interprocedural-
//! fixpoint precedent): abstract, per function `F`, the value `current_env`
//! holds while `F`'s body runs —
//!
//! - `F` owns a promotable env → its prologue publishes its own record:
//!   `Record(F)`, independent of callers.
//! - otherwise `F` is TRANSPARENT (never touches `current_env`) → the join of
//!   its callers' body contexts; the module root `_start` is `NoEnv`.
//! - conflicting / unknown joins → `Top`.
//!
//! A call (or `Kali.test` registration — the host restores the `env_ptr`
//! captured AT the registration site, so the registration site inherits the
//! identical requirement) from `F` to capturer `C` with owner `O` is safe iff
//! `F`'s body context is exactly `Record(O)`: the owner's own body, or a
//! transparent intermediate reachable only from the owner's body. A call from
//! a different env-owning function (the sibling case), from module scope, or
//! any join to `Top` → E5506.
//!
//! ## Call-graph completeness (why "no edge" is sound)
//!
//! The verdict keys on edges that can actually INVOKE a function at runtime.
//! kali has no first-class invocation: an indirect call through a non-const
//! binding, array element, member, or call result lowers to the pre-existing
//! zero-placeholder no-op (pinned by the `exotic_*` / escaping-closure
//! tripwires) — it never reaches the callee. The lanes that DO invoke are
//! (1) a direct call whose callee resolves by name/const-binding provenance
//! and (2) the `Kali.test` registration (`test_register` + host
//! `invoke_callback`). Both are over-approximated here: every identifier text
//! under a callee (or registration-callback) subtree is resolved through a
//! name-keyed, whole-program alias map (declarator initializers and `for-of`
//! bindings, transitively), so any name that COULD denote a function
//! contributes an edge. Nested-function subtrees are opaque leaves — their
//! own calls are attributed to their own body walk — but their NAME still
//! escapes into the enclosing alias/edge scan (a function value embedded in
//! an initializer is trackable). Scheduling surfaces (`setTimeout` et al.)
//! never invoke (their callbacks are dropped; the default-deny guard in
//! `emit/call.rs` rejects everything not provably non-capturing), so they
//! contribute no edges. A capturer with NO edges is vacuously safe: nothing
//! ever reaches it (the escaping-closure pins m/n/s stay exactly as pinned).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::*;

/// Abstract value of `current_env` while a given function's body runs.
#[derive(Clone, Debug, PartialEq, Eq)]
enum EnvCtx {
    /// `current_env == 0` — the module root / no record published.
    NoEnv,
    /// `current_env` is (some activation of) this function's env record.
    Record(String),
    /// Conflicting joins or unknown — never safe.
    Top,
}

/// Lattice join: equal values keep themselves, anything else is `Top`.
fn join(a: &EnvCtx, b: &EnvCtx) -> EnvCtx {
    if a == b {
        a.clone()
    } else {
        EnvCtx::Top
    }
}

/// Every non-empty `text` in the subtree at `id`, treating nested
/// function-like nodes as opaque leaves (their own NAME is collected — a
/// function value embedded in an initializer/argument stays trackable — but
/// their interior calls/aliases belong to their own body walk, not the
/// enclosing one).
fn collect_texts<'a>(nodes: &'a [LirNode], id: LirNodeId, out: &mut BTreeSet<&'a str>) {
    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };
    if let Some(text) = node.text.as_deref() {
        if !text.is_empty() {
            out.insert(text);
        }
    }
    if crate::lower::is_function_like(nodes, id) {
        return;
    }
    for child in &node.children {
        collect_texts(nodes, *child, out);
    }
}

/// Static mirror of `FunctionEmitter::for_of_binding_name_from_node` (no
/// emitter available at lower time): the single binding name a `for-of` head
/// declares or assigns.
fn for_of_binding_name(nodes: &[LirNode], id: LirNodeId) -> Option<String> {
    let node = nodes.get(id.0 as usize)?;
    if node.children.is_empty() {
        return node.text.clone();
    }
    if matches!(node.text.as_deref(), Some("const" | "let" | "var")) {
        let declarator = node.children.first().copied()?;
        return nodes.get(declarator.0 as usize)?.text.clone();
    }
    if node.text.as_deref().is_some_and(|text| text.is_empty()) && !node.children.is_empty() {
        return for_of_binding_name(nodes, *node.children.last()?);
    }
    if (node.text.is_none() || node.text.as_deref() == Some("await")) && node.children.len() == 1 {
        return for_of_binding_name(nodes, node.children[0]);
    }
    None
}

/// True when `callee_id` is the `Kali.test` member callee shape
/// (`FunctionEmitter::is_kali_test_call`, mirrored statically).
fn is_kali_test_callee(nodes: &[LirNode], callee_id: LirNodeId) -> bool {
    let Some(callee) = nodes.get(callee_id.0 as usize) else {
        return false;
    };
    if callee.text.as_deref() != Some("test") {
        return false;
    }
    callee.children.first().is_some_and(|&obj| {
        nodes
            .get(obj.0 as usize)
            .is_some_and(|node| node.text.as_deref() == Some("Kali"))
    })
}

/// True when `callee` is a bare-identifier scheduling callee whose call
/// REGISTERS its callback argument (`children[1]`) for a later host-driven
/// invocation (`queueMicrotask` / `setTimeout` / `setInterval`, Stage D).
/// The env active at the registration site is what the host restores before
/// invoking the callback, so the registration site inherits the same
/// Record(owner) requirement as a direct call — the `Kali.test` precedent.
/// Shadowing is ignored here: a spurious edge from a user-shadowed name is a
/// safe over-approximation (this analysis only ever REJECTS more).
fn is_scheduling_registration_callee(nodes: &[LirNode], callee: LirNodeId) -> bool {
    nodes.get(callee.0 as usize).is_some_and(|node| {
        node.children.is_empty()
            && matches!(
                node.text.as_deref(),
                Some("queueMicrotask") | Some("setTimeout") | Some("setInterval")
            )
    })
}

/// True when `callee` is a MEMBER callee named `addEventListener` (it has a
/// receiver child). The callback argument (`children[2]`) is registered for a
/// later host-driven invocation with the env active at the registration site,
/// so it inherits the same Record(owner) requirement as `Kali.test` /
/// scheduling registrations (Stage D event lane). Receiver provenance is
/// deliberately ignored: a spurious edge from an out-of-lane receiver is a safe
/// over-approximation (this analysis only ever REJECTS more).
fn is_event_registration_callee(nodes: &[LirNode], callee: LirNodeId) -> bool {
    nodes.get(callee.0 as usize).is_some_and(|node| {
        !node.children.is_empty() && node.text.as_deref() == Some("addEventListener")
    })
}

/// Compute the dynamic-env safety diagnostics for the whole program: one
/// E5506 per (caller, capturer) edge whose call/registration site cannot be
/// proven to run with the capturer's owner record in `current_env`.
///
/// Empty (zero cost beyond the plan scan) whenever no function has its
/// capture lowering engaged — the common no-closure program.
pub(crate) fn env_capture_safety_diagnostics(
    lir: &LirProgram,
    all_functions: &[FunctionPlan],
    env_plans: &BTreeMap<String, kali_mir::EnvPlan>,
    repr_table: &kali_common::ReprTable,
) -> Vec<Diagnostic> {
    // 1. Capturers with ENGAGED lowering, mapped to the owner(s) of their
    //    lowered refs — the same predicate `resolve_capture_access_inner`
    //    lowers on (`mir_depth == 1` + owner-keyed `cell_is_promotable`).
    //    Refs that are NOT lowered (depth >= 2, non-promotable cells) keep
    //    their pre-Stage-C baseline behavior and impose no constraint here.
    let mut capture_owners: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (name, plan) in env_plans {
        if name.is_empty() {
            // Module-root plan key: the root never captures through the chain.
            continue;
        }
        let owners: BTreeSet<&str> = plan
            .captured
            .iter()
            .filter(|reference| {
                reference.depth == 1
                    && crate::closure::cell_is_promotable(
                        repr_table,
                        &reference.owner,
                        &reference.name,
                        reference.is_scalar,
                    )
            })
            .map(|reference| reference.owner.as_str())
            .collect();
        if !owners.is_empty() {
            capture_owners.insert(name.as_str(), owners);
        }
    }
    if capture_owners.is_empty() {
        return Vec::new();
    }

    // 2. Promotable env OWNERSHIP — mirrors the `lower.rs` promotion loop
    //    (the prologue publishes a record iff >=1 owned cell is promotable).
    let promotable_owner = |name: &str| -> bool {
        env_plans.get(name).is_some_and(|plan| {
            plan.cells.iter().any(|cell| {
                crate::closure::cell_is_promotable(repr_table, name, &cell.name, cell.is_scalar)
            })
        })
    };

    // 3. Source functions (synthetic page-pool/join/streq helpers carry an
    //    inert `body = lir.root` placeholder and must not be walked).
    let source_fns: Vec<&FunctionPlan> = all_functions
        .iter()
        .filter(|plan| !crate::lower::is_synthetic_function(&plan.name))
        .collect();

    // 4. Name-keyed alias map: which function(s) may a name denote? Seeded by
    //    every function name denoting itself; declarator initializers and
    //    `for-of` bindings propagate denotations transitively (whole-program,
    //    name-keyed — a deliberate over-approximation; spurious edges only
    //    ever ADD constraints, never remove them).
    let mut denotes: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for plan in &source_fns {
        if !plan.is_entry {
            denotes
                .entry(plan.name.as_str())
                .or_default()
                .insert(plan.name.as_str());
        }
    }
    let mut alias_pairs: Vec<(&str, BTreeSet<&str>)> = Vec::new();
    for node in &lir.nodes {
        if node.kind == LirNodeKind::Instruction
            && matches!(node.text.as_deref(), Some("const" | "let" | "var"))
        {
            for declarator_id in &node.children {
                let Some(declarator) = lir.nodes.get(declarator_id.0 as usize) else {
                    continue;
                };
                let (Some(name), Some(&init)) =
                    (declarator.text.as_deref(), declarator.children.get(1))
                else {
                    continue;
                };
                let mut texts = BTreeSet::new();
                collect_texts(&lir.nodes, init, &mut texts);
                alias_pairs.push((name, texts));
            }
        }
        if node.kind == LirNodeKind::Branch
            && matches!(node.text.as_deref(), Some("for-of" | "for-await-of"))
        {
            // `for (const f of [fn1, fn2]) f()` — the const-element for-of
            // intrinsic binds the loop name to each element and CAN emit real
            // calls to element function values; alias the loop name to every
            // function the iterable subtree mentions.
            let (Some(&head), Some(&iterable)) = (node.children.first(), node.children.get(1))
            else {
                continue;
            };
            if let Some(name_owned) = for_of_binding_name(&lir.nodes, head) {
                // Resolve the owned name back to a borrowed key via the node
                // that carries it (alias_pairs borrows from `lir`).
                let mut texts = BTreeSet::new();
                collect_texts(&lir.nodes, iterable, &mut texts);
                if let Some(name) = find_text(&lir.nodes, head, &name_owned) {
                    alias_pairs.push((name, texts));
                }
            }
        }
    }
    loop {
        let mut changed = false;
        for (name, sources) in &alias_pairs {
            let mut add: BTreeSet<&str> = BTreeSet::new();
            for source in sources {
                if let Some(set) = denotes.get(source) {
                    add.extend(set.iter().copied());
                }
            }
            if add.is_empty() {
                continue;
            }
            let entry = denotes.entry(name).or_default();
            for target in add {
                changed |= entry.insert(target);
            }
        }
        if !changed {
            break;
        }
    }

    // 5. Call + registration edges, attributed to the ENCLOSING function
    //    (nested function subtrees are opaque; their calls belong to their
    //    own plan's walk).
    let mut edges: BTreeSet<(&str, &str)> = BTreeSet::new();
    for plan in &source_fns {
        let caller = plan.name.as_str();
        let mut stack = vec![plan.body];
        let mut seen: HashSet<LirNodeId> = HashSet::new();
        while let Some(id) = stack.pop() {
            if !seen.insert(id) {
                continue;
            }
            let Some(node) = lir.nodes.get(id.0 as usize) else {
                continue;
            };
            if id != plan.body && crate::lower::is_function_like(&lir.nodes, id) {
                continue;
            }
            if node.kind == LirNodeKind::Call {
                if let Some(&callee) = node.children.first() {
                    // A `Kali.test` registration invokes its callback later
                    // with the env captured HERE — model it as a call edge
                    // from the registering function. Every other call: any
                    // function a callee-subtree name may denote is a
                    // potential direct-call target.
                    let target_root = if is_kali_test_callee(&lir.nodes, callee) {
                        node.children.get(2).copied()
                    } else if is_scheduling_registration_callee(&lir.nodes, callee) {
                        node.children.get(1).copied()
                    } else if is_event_registration_callee(&lir.nodes, callee) {
                        node.children.get(2).copied()
                    } else {
                        Some(callee)
                    };
                    if let Some(root) = target_root {
                        let mut texts = BTreeSet::new();
                        collect_texts(&lir.nodes, root, &mut texts);
                        for text in texts {
                            if let Some(set) = denotes.get(text) {
                                for target in set {
                                    edges.insert((caller, target));
                                }
                            }
                        }
                    }
                }
            }
            stack.extend(node.children.iter().copied());
        }
    }

    // 6. Reachability + body-context fixpoint over the edge graph.
    let mut successors: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (from, to) in &edges {
        successors.entry(from).or_default().push(to);
    }
    let body_ctx = |name: &str, incoming: &EnvCtx| -> EnvCtx {
        if promotable_owner(name) {
            EnvCtx::Record(name.to_string())
        } else {
            incoming.clone()
        }
    };
    let mut ctx: BTreeMap<&str, EnvCtx> = BTreeMap::new();
    let mut queue: VecDeque<&str> = VecDeque::new();
    for plan in &source_fns {
        if plan.is_entry {
            ctx.insert(plan.name.as_str(), EnvCtx::NoEnv);
            queue.push_back(plan.name.as_str());
        }
    }
    while let Some(from) = queue.pop_front() {
        let Some(incoming) = ctx.get(from).cloned() else {
            continue;
        };
        let out = body_ctx(from, &incoming);
        let Some(next) = successors.get(from).cloned() else {
            continue;
        };
        for to in next {
            let updated = match ctx.get(to) {
                Some(existing) => join(existing, &out),
                None => out.clone(),
            };
            if ctx.get(to) != Some(&updated) {
                ctx.insert(to, updated);
                queue.push_back(to);
            }
        }
    }

    // 7. Verdicts: every REACHABLE edge into an engaged capturer must carry
    //    exactly its owner's record. Unreachable callers impose nothing (their
    //    bodies never run — e.g. an escaped-but-never-invoked closure).
    let mut diagnostics = Vec::new();
    for (from, to) in &edges {
        let Some(owners) = capture_owners.get(to) else {
            continue;
        };
        let Some(incoming) = ctx.get(from) else {
            continue;
        };
        let caller_env = body_ctx(from, incoming);
        for owner in owners {
            if caller_env != EnvCtx::Record((*owner).to_string()) {
                diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "invoking the capturing function '{to}' from '{from}' is unavailable: '{to}' captures binding(s) owned by '{owner}', but the environment record active at this call/registration site cannot be proven to be '{owner}''s — the capture would resolve against the wrong record (dynamic-env vs lexical-capture mismatch); invoke '{to}' from '{owner}' itself or through capture-free helpers"
                    ),
                ));
            }
        }
    }
    diagnostics
}

/// Borrow the `&str` for `needle` out of the subtree at `id` (used to key the
/// alias map, which borrows from `lir`, with a name produced as an owned
/// `String` by the for-of head resolver).
fn find_text<'a>(nodes: &'a [LirNode], id: LirNodeId, needle: &str) -> Option<&'a str> {
    let node = nodes.get(id.0 as usize)?;
    if let Some(text) = node.text.as_deref() {
        if text == needle {
            return Some(text);
        }
    }
    for child in &node.children {
        if let Some(found) = find_text(nodes, *child, needle) {
            return Some(found);
        }
    }
    None
}
