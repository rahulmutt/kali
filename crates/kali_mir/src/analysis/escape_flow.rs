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

// TEMPORARY: consumers land in Tasks 2-4 of the interprocedural escape-flow
// plan; remove at the Task 3 gate cutover.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

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

    pub(crate) fn global_sites(&self) -> &[GlobalSiteIf] {
        &self.global_sites
    }

    pub(crate) fn outflow_sites(&self) -> &[OutflowIf] {
        &self.outflow_sites
    }

    pub(crate) fn arg_sites(&self) -> &[ArgEscapeIf] {
        &self.arg_sites
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

#[cfg(test)]
#[path = "escape_flow_tests.rs"]
mod escape_flow_tests;
