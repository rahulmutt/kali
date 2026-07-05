//! Interprocedural escape-flow analysis: the shared, walk-order-independent
//! answer to "may this value be heap?" and "is this value retained beyond a
//! dynamic extent?".
//!
//! The ownership walk RECORDS tri-state value judgments ([`ValueClass`]),
//! plain-ident/param/return flow edges, and deferred veto sites into a
//! [`FlowCollector`] instead of resolving heap-ness against possibly-stale
//! layouts mid-walk. [`solve`] then runs one monotone worklist fixpoint:
//! may-heap propagates FORWARD along value flow; escape taint propagates
//! BACKWARD (if `b = a` and `b`'s contents escape, `a`'s do too). Bits only
//! flip false→true, so termination is structural and the solution cannot
//! depend on walk order — this is what closes the hoisted-function launder.
//!
//! Two consumers: `ArenaCollector::into_facts` resolves the deferred sites
//! (gate vetoes), and `apply_escape_verdicts` feeds `MirBinding.escapes`
//! (fixing the engine's `sink = p` param-escape blindness).
//!
//! Every unknown fails closed: unresolved identifiers and unknown callees
//! classify as heap; unknown callees taint their arguments; params seed
//! may-heap unconditionally (a function may be called from contexts the
//! graph cannot see); name-collided functions poison their param summaries.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use kali_hir::{HirNodeId, HirNodeKind};

use crate::{
    LayoutDescriptor, MirBindingKind, MirFunction, MirFunctionKind, OwnershipAnalyzer,
    OwnershipClass,
};

/// A dataflow node. Bindings are keyed (owner function label, name) — the
/// same name-keyed granularity as the rest of the gate, with the same
/// collision conservatism (see [`FlowCollector::poison_function`]).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum FlowNode {
    Binding {
        owner: String,
        name: String,
    },
    /// Positional param slot: call-site arg edges land here even when the
    /// callee's declaration has not been walked yet (hoisting).
    Param {
        function: String,
        index: usize,
    },
    Return {
        function: String,
    },
}

/// Tri-state RHS judgment. `Heap(embeds)` carries the identifier sources
/// embedded in the value (literal property values, member-read bases, ...)
/// so structures keep the identity of their contents for taint purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ValueClass {
    Scalar,
    Heap(BTreeSet<FlowNode>),
    DependsOn(BTreeSet<FlowNode>),
}

impl ValueClass {
    pub(crate) fn heap() -> Self {
        ValueClass::Heap(BTreeSet::new())
    }

    pub(crate) fn is_scalar(&self) -> bool {
        matches!(self, ValueClass::Scalar)
    }

    pub(crate) fn take_nodes(self) -> BTreeSet<FlowNode> {
        match self {
            ValueClass::Scalar => BTreeSet::new(),
            ValueClass::Heap(nodes) | ValueClass::DependsOn(nodes) => nodes,
        }
    }

    /// Join for branch-producing positions (ternary, logical, sequence):
    /// Heap absorbs (a possibly-heap branch makes the whole expression
    /// possibly-heap), node sets union.
    pub(crate) fn join(self, other: ValueClass) -> ValueClass {
        match (self, other) {
            (ValueClass::Scalar, x) | (x, ValueClass::Scalar) => x,
            (ValueClass::Heap(a), ValueClass::Heap(b))
            | (ValueClass::Heap(a), ValueClass::DependsOn(b))
            | (ValueClass::DependsOn(b), ValueClass::Heap(a)) => {
                ValueClass::Heap(a.into_iter().chain(b).collect())
            }
            (ValueClass::DependsOn(a), ValueClass::DependsOn(b)) => {
                ValueClass::DependsOn(a.into_iter().chain(b).collect())
            }
        }
    }
}

/// Deferred "this function has a Global-fate site if the stored value is
/// may-heap" record.
#[derive(Debug, Clone)]
pub(crate) struct GlobalSiteIf {
    pub(crate) function: String,
    pub(crate) class: ValueClass,
}

/// Deferred "this loop has outflow if the value is may-heap" record.
#[derive(Debug, Clone)]
pub(crate) struct OutflowIf {
    pub(crate) function: String,
    pub(crate) ordinal: u32,
    pub(crate) class: ValueClass,
}

/// Deferred call-argument record: fires (global site + outflow for the open
/// loops) iff the callee's param summary says the slot escapes AND the
/// argument is may-heap.
#[derive(Debug, Clone)]
pub(crate) struct ArgEscapeIf {
    pub(crate) function: String,
    pub(crate) callee: String,
    pub(crate) index: usize,
    pub(crate) class: ValueClass,
    pub(crate) loop_ordinals: Vec<u32>,
}

/// Deferred "this function has a Returned-fate site if the returned value is
/// may-heap" record. Generalizes round-1's shape-specific bare-literal /
/// name-bound-literal detection: `class` is the same joined [`ValueClass`]
/// already computed for every `return` statement (covers literals, plain
/// idents, ternaries, and call-results transitively via `DependsOn(Return {
/// callee })` edges), so a `return factory()` whose callee's own return
/// resolves may-heap through the fixpoint resolves true here too.
#[derive(Debug, Clone)]
pub(crate) struct ReturnedSiteIf {
    pub(crate) function: String,
    pub(crate) class: ValueClass,
}

/// Everything the walk records for the fixpoint. All containers are ordered
/// (determinism).
#[derive(Debug, Default)]
pub(crate) struct FlowCollector {
    /// Value-flow edges: the value of `from` may become the value of `to`.
    edges: BTreeSet<(FlowNode, FlowNode)>,
    /// Nodes assigned a definitely-heap value.
    heap_seeds: BTreeSet<FlowNode>,
    /// Nodes whose contents are retained beyond a dynamic extent.
    taint_seeds: BTreeSet<FlowNode>,
    /// Every registered param slot (seeded may-heap: fail closed).
    param_nodes: BTreeSet<FlowNode>,
    /// Name-collided functions: param summaries are worst-cased.
    poisoned_functions: BTreeSet<String>,
    global_sites: Vec<GlobalSiteIf>,
    outflow_sites: Vec<OutflowIf>,
    arg_sites: Vec<ArgEscapeIf>,
    returned_sites: Vec<ReturnedSiteIf>,
}

impl FlowCollector {
    pub(crate) fn note_edge(&mut self, from: FlowNode, to: FlowNode) {
        self.edges.insert((from, to));
    }

    /// Record that `class`'s value flows into `target`.
    pub(crate) fn note_value_into(&mut self, target: FlowNode, class: &ValueClass) {
        match class {
            ValueClass::Scalar => {}
            ValueClass::Heap(nodes) => {
                self.heap_seeds.insert(target.clone());
                for node in nodes {
                    self.note_edge(node.clone(), target.clone());
                }
            }
            ValueClass::DependsOn(nodes) => {
                for node in nodes {
                    self.note_edge(node.clone(), target.clone());
                }
            }
        }
    }

    /// The value's sources escape directly (stored into an object field, or
    /// handed to an unknown callee): taint them without a target node.
    pub(crate) fn note_taint_class(&mut self, class: &ValueClass) {
        match class {
            ValueClass::Scalar => {}
            ValueClass::Heap(nodes) | ValueClass::DependsOn(nodes) => {
                self.taint_seeds.extend(nodes.iter().cloned());
            }
        }
    }

    pub(crate) fn note_taint_node(&mut self, node: FlowNode) {
        self.taint_seeds.insert(node);
    }

    /// Register param slot `index`/`name` of `function`, linking the
    /// positional node call sites target to the named binding the body uses.
    pub(crate) fn note_param(&mut self, function: &str, index: usize, name: &str) {
        let param = FlowNode::Param {
            function: function.to_string(),
            index,
        };
        self.param_nodes.insert(param.clone());
        self.note_edge(
            param,
            FlowNode::Binding {
                owner: function.to_string(),
                name: name.to_string(),
            },
        );
    }

    pub(crate) fn poison_function(&mut self, name: &str) {
        self.poisoned_functions.insert(name.to_string());
    }

    pub(crate) fn push_global_site(&mut self, function: &str, class: ValueClass) {
        if class.is_scalar() {
            return;
        }
        self.global_sites.push(GlobalSiteIf {
            function: function.to_string(),
            class,
        });
    }

    pub(crate) fn push_outflow(&mut self, function: &str, ordinal: u32, class: ValueClass) {
        if class.is_scalar() {
            return;
        }
        self.outflow_sites.push(OutflowIf {
            function: function.to_string(),
            ordinal,
            class,
        });
    }

    pub(crate) fn push_arg_site(
        &mut self,
        function: &str,
        callee: &str,
        index: usize,
        class: ValueClass,
        loop_ordinals: Vec<u32>,
    ) {
        if class.is_scalar() {
            return;
        }
        self.arg_sites.push(ArgEscapeIf {
            function: function.to_string(),
            callee: callee.to_string(),
            index,
            class,
            loop_ordinals,
        });
    }

    /// Register a deferred returned-fate site for `function`: resolved true
    /// (vetoes `opens_arena`) iff `class` turns out may-heap under the
    /// fixpoint. A genuinely `Scalar` return (e.g. `return 1 + n;`) is
    /// dropped here just like the other deferred sites — the function stays
    /// eligible to open its own arena for that return.
    pub(crate) fn push_returned_site(&mut self, function: &str, class: ValueClass) {
        if class.is_scalar() {
            return;
        }
        self.returned_sites.push(ReturnedSiteIf {
            function: function.to_string(),
            class,
        });
    }

    pub(crate) fn global_sites(&self) -> &[GlobalSiteIf] {
        &self.global_sites
    }

    pub(crate) fn outflow_sites(&self) -> &[OutflowIf] {
        &self.outflow_sites
    }

    pub(crate) fn arg_sites(&self) -> &[ArgEscapeIf] {
        &self.arg_sites
    }

    pub(crate) fn returned_sites(&self) -> &[ReturnedSiteIf] {
        &self.returned_sites
    }
}

/// Fixpoint results. Both sets are complete (post-closure) when returned.
#[derive(Debug, Default)]
pub(crate) struct FlowSolution {
    may_heap: BTreeSet<FlowNode>,
    tainted: BTreeSet<FlowNode>,
    poisoned_functions: BTreeSet<String>,
}

impl FlowSolution {
    pub(crate) fn class_may_heap(&self, class: &ValueClass) -> bool {
        match class {
            ValueClass::Scalar => false,
            ValueClass::Heap(_) => true,
            ValueClass::DependsOn(nodes) => nodes.iter().any(|node| self.may_heap.contains(node)),
        }
    }

    pub(crate) fn param_escapes(&self, function: &str, index: usize) -> bool {
        self.poisoned_functions.contains(function)
            || self.tainted.contains(&FlowNode::Param {
                function: function.to_string(),
                index,
            })
    }

    pub(crate) fn binding_escapes(&self, owner: &str, name: &str) -> bool {
        self.tainted.contains(&FlowNode::Binding {
            owner: owner.to_string(),
            name: name.to_string(),
        })
    }
}

/// One monotone worklist fixpoint: may-heap forward from
/// `heap_seeds ∪ param_nodes`, escape taint backward from `taint_seeds`.
/// Bits only flip false→true over a finite node set, so this terminates in
/// O(edges) enqueues per direction regardless of cycles.
pub(crate) fn solve(flow: &FlowCollector) -> FlowSolution {
    let mut forward: BTreeMap<&FlowNode, Vec<&FlowNode>> = BTreeMap::new();
    let mut backward: BTreeMap<&FlowNode, Vec<&FlowNode>> = BTreeMap::new();
    for (from, to) in &flow.edges {
        forward.entry(from).or_default().push(to);
        backward.entry(to).or_default().push(from);
    }

    let mut may_heap: BTreeSet<FlowNode> = flow.heap_seeds.clone();
    may_heap.extend(flow.param_nodes.iter().cloned());
    let mut queue: VecDeque<FlowNode> = may_heap.iter().cloned().collect();
    while let Some(node) = queue.pop_front() {
        if let Some(successors) = forward.get(&node) {
            for succ in successors {
                if may_heap.insert((*succ).clone()) {
                    queue.push_back((*succ).clone());
                }
            }
        }
    }

    let mut tainted: BTreeSet<FlowNode> = flow.taint_seeds.clone();
    let mut queue: VecDeque<FlowNode> = tainted.iter().cloned().collect();
    while let Some(node) = queue.pop_front() {
        if let Some(predecessors) = backward.get(&node) {
            for pred in predecessors {
                if tainted.insert((*pred).clone()) {
                    queue.push_back((*pred).clone());
                }
            }
        }
    }

    FlowSolution {
        may_heap,
        tainted,
        poisoned_functions: flow.poisoned_functions.clone(),
    }
}

// ---------------------------------------------------------------------------
// Tri-state value classification (walk-time, pure — records nothing).
// ---------------------------------------------------------------------------

/// Heap-typed layouts, fail-closed: `TaggedVal` AND `Scalar("unknown")` (the
/// layout of `null`/`undefined`) count as heap. Moved here from arena_gate
/// when the gate's own judgment was retired.
pub(crate) fn is_heap_layout(layout: &LayoutDescriptor) -> bool {
    match layout {
        LayoutDescriptor::Scalar(name) => name == "unknown",
        _ => true,
    }
}

impl<'a> OwnershipAnalyzer<'a> {
    /// Classify an expression's value without resolving binding heap-ness:
    /// identifiers become `DependsOn` nodes for the fixpoint. Operator-aware:
    /// `sum + itemCheck(tree)` is `Scalar` no matter what `tree` is — this is
    /// what keeps the binary-trees loop grant alive. Every unknown shape
    /// fails closed to `Heap`.
    pub(crate) fn classify_value(&self, node_id: HirNodeId) -> ValueClass {
        let node = &self.nodes[node_id.0 as usize];
        match node.kind {
            HirNodeKind::ObjectExpr | HirNodeKind::ArrayExpr => {
                // A structure carries the identity of everything embedded in
                // it: if the structure escapes, its contents escape.
                let mut embeds = BTreeSet::new();
                for child in &node.children {
                    embeds.extend(self.classify_value(*child).take_nodes());
                }
                ValueClass::Heap(embeds)
            }
            HirNodeKind::ObjectProperty => match node.children.get(1) {
                Some(value) => self.classify_value(*value),
                None => ValueClass::heap(),
            },
            HirNodeKind::Literal => {
                if is_heap_layout(&self.infer_layout(node_id)) {
                    ValueClass::heap()
                } else {
                    ValueClass::Scalar
                }
            }
            // A ternary produces whichever branch runs (children[1..]).
            HirNodeKind::ConditionalExpr => {
                if node.children.len() < 2 {
                    return ValueClass::heap();
                }
                node.children
                    .iter()
                    .skip(1)
                    .fold(ValueClass::Scalar, |acc, child| {
                        acc.join(self.classify_value(*child))
                    })
            }
            // Logical ops return an operand; a sequence returns its last
            // expression — join everything (fail closed on malformed).
            HirNodeKind::LogicalExpr | HirNodeKind::SequenceExpr => {
                if node.children.is_empty() {
                    return ValueClass::heap();
                }
                node.children.iter().fold(ValueClass::Scalar, |acc, child| {
                    acc.join(self.classify_value(*child))
                })
            }
            HirNodeKind::BinaryExpr => match node.text.as_deref() {
                // Genuinely scalar-producing operators. String `+` concat is
                // scalar too in v1: runtime strings are global-arena host
                // values and never dangle across a reset.
                Some(
                    "+" | "-" | "*" | "/" | "%" | "**" | "&" | "|" | "^" | "<<" | ">>" | ">>>"
                    | "==" | "!=" | "===" | "!==" | "<" | "<=" | ">" | ">=",
                ) => ValueClass::Scalar,
                // `&&`/`||`/`??` parse as BinaryExpr and return an operand.
                Some("&&" | "||" | "??") => {
                    if node.children.is_empty() {
                        return ValueClass::heap();
                    }
                    node.children.iter().fold(ValueClass::Scalar, |acc, child| {
                        acc.join(self.classify_value(*child))
                    })
                }
                _ => self.classify_children_as_heap(node_id),
            },
            HirNodeKind::UnaryExpr => {
                if matches!(node.text.as_deref(), Some("!" | "-" | "+" | "~" | "typeof")) {
                    ValueClass::Scalar
                } else {
                    self.classify_children_as_heap(node_id)
                }
            }
            HirNodeKind::Ident => {
                let name = node.text.as_deref().unwrap_or_default();
                match self.resolve_binding(name) {
                    Some((scope_index, _)) => {
                        let owner = self
                            .scope_stack
                            .get(scope_index)
                            .map(|scope| scope.label.clone())
                            .unwrap_or_else(|| "<module>".to_string());
                        ValueClass::DependsOn(
                            std::iter::once(FlowNode::Binding {
                                owner,
                                name: name.to_string(),
                            })
                            .collect(),
                        )
                    }
                    None => ValueClass::heap(),
                }
            }
            HirNodeKind::CallExpr => {
                let target = node
                    .children
                    .first()
                    .map(|id| &self.nodes[id.0 as usize])
                    .filter(|callee| callee.kind == HirNodeKind::Ident)
                    .and_then(|callee| callee.text.as_deref())
                    .and_then(|name| self.resolve_function_target(name));
                match target {
                    Some(function) => ValueClass::DependsOn(
                        std::iter::once(FlowNode::Return { function }).collect(),
                    ),
                    None => ValueClass::heap(),
                }
            }
            // A member read shares its base's structure: if the read value
            // escapes, the base's contents escape with it. EXCEPT when the
            // base's inferred layout gives POSITIVE PROOF the accessed field
            // is itself scalar (a resolvable `Struct` layout with a matching
            // non-heap field) — a genuinely scalar field read (e.g. `o.v`
            // where `o = { v: 1 }`) does not share `o`'s heap identity at
            // all, and must not itself taint/veto as though the whole
            // struct escaped (this is what keeps a returned scalar field of
            // a ScopeLocal struct from wrongly vetoing `opens_arena` — see
            // arena_gate's `opens_arena_still_true_for_all_scope_local_function`).
            // Anything unresolved (a parameter's opaque `TaggedVal` layout,
            // for instance) keeps the fail-closed Heap classification below,
            // which is what keeps `member_read_carries_base_identity_for_taint`
            // sound (a param's `p.left` still taints `p`).
            HirNodeKind::MemberExpr | HirNodeKind::OptionalChain | HirNodeKind::ChainExpr => {
                match node.children.first() {
                    Some(base) => {
                        if let (Some(field), LayoutDescriptor::Struct { fields }) =
                            (node.text.as_deref(), self.infer_layout(*base))
                        {
                            if let Some((_, field_layout)) =
                                fields.iter().find(|(name, _)| name == field)
                            {
                                if !is_heap_layout(field_layout) {
                                    return ValueClass::Scalar;
                                }
                            }
                        }
                        ValueClass::Heap(self.classify_value(*base).take_nodes())
                    }
                    None => ValueClass::heap(),
                }
            }
            // Everything else (templates, new-expressions, unknown nodes):
            // heap, carrying any identifier sources found underneath.
            _ => self.classify_children_as_heap(node_id),
        }
    }

    fn classify_children_as_heap(&self, node_id: HirNodeId) -> ValueClass {
        let node = &self.nodes[node_id.0 as usize];
        let mut embeds = BTreeSet::new();
        for child in &node.children {
            embeds.extend(self.classify_value(*child).take_nodes());
        }
        ValueClass::Heap(embeds)
    }
}

// ---------------------------------------------------------------------------
// Engine consumer: fold fixpoint escape verdicts into finalized bindings.
// ---------------------------------------------------------------------------

/// The fourth disjunct of the engine's escape judgment: a binding whose value
/// the fixpoint proves stored beyond a dynamic extent escapes, even when the
/// walk saw only plain-ident dataflow (`sink = p`, alias chains, hoisted
/// helpers). Only ever flips verdicts toward escaping. Module-scope bindings
/// are storage roots, not escapees — skipped.
///
/// The `captured_by.is_empty()` check below is DELIBERATELY not the same
/// predicate `finalise_binding` uses: that function keys `SharedHeap` on
/// `capture_escapes` (some capturing function itself escapes), not on
/// `captured_by` being non-empty. This post-pass only ever runs on bindings
/// `finalise_binding` already left at `escapes == false` — which means
/// `capture_escapes` was known false for them — so using the coarser
/// `!captured_by.is_empty()` here can only pick `SharedHeap` where
/// `finalise_binding` would have picked `OwnedHeap` (never the reverse). That
/// is strictly more conservative (multi-owner-safe representation for a
/// value that may in fact be uniquely owned), never less, so it stays sound
/// with the fail-closed invariant even though it diverges from
/// `finalise_binding`'s exact mapping.
pub(crate) fn apply_escape_verdicts(functions: &mut [MirFunction], solution: &FlowSolution) {
    for function in functions.iter_mut() {
        if function.kind == MirFunctionKind::Module {
            continue;
        }
        let Some(owner) = function.name.clone() else {
            continue;
        };
        for binding in &mut function.bindings {
            if binding.escapes {
                continue;
            }
            if !solution.binding_escapes(&owner, &binding.name) {
                continue;
            }
            binding.escapes = true;
            match binding.kind {
                MirBindingKind::Local | MirBindingKind::Function => {
                    binding.ownership = if binding.captured_by.is_empty() {
                        OwnershipClass::OwnedHeap
                    } else {
                        OwnershipClass::SharedHeap
                    };
                }
                // finalise_binding's returned/escaped arm keeps Parameter and
                // Import ownership untouched.
                MirBindingKind::Parameter | MirBindingKind::Import => {}
            }
        }
    }
}

#[cfg(test)]
#[path = "escape_flow_tests.rs"]
mod escape_flow_tests;
