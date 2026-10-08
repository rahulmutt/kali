//! Growable-runtime-arrays spec §3.1-§3.2: the growable property, solved.
//!
//! Every array VALUE a program names is a [`GrowNode`]: a binding or
//! parameter, a function's return value, or a temporary (a `slice` result, a
//! `?:`/`||`/`&&` merge, a literal written directly as an argument or
//! `return`, an allocation written directly). [`GrowFacts::edges`] join two
//! nodes that can hold the same array at run time: an alias, an argument and
//! its parameter, a `return` and the call, a `slice` and its receiver. Edges
//! are undirected, so the solve is a union-find and a COMPONENT is the unit
//! every decision is made on.
//!
//! A component is growable when it holds a literal-initialized binding AND a
//! length/element mutation (`push`, `pop`, an index write) somewhere in it
//! (spec A-2). A growable component must not also hold an allocation
//! (`new Array(n)`, §3.2), a literal written directly as an argument or
//! `return` (A-5), or a non-array value (A-7); each is a [`GrowConflict`].
//! A component that is not growable decides nothing: every node in it keeps
//! the lane it has today (A-1).
//!
//! The walk that fills [`GrowFacts`] is `super::facts`; the checks that turn
//! uses into refusals are `super::positions`.

use std::collections::{BTreeMap, BTreeSet};

/// The synthetic module-scope function, as `repr_infer` keys it.
pub(crate) const TOP_LEVEL: &str = "_start";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum GrowNode {
    /// `(func, name)`: a binding or parameter, keyed like `ReprTable`.
    Binding(String, String),
    /// The value function `func` returns.
    Return(String),
    /// A temporary, numbered by the fact walk; its kind and site are in
    /// [`GrowFacts::temp_kinds`].
    Temp(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TempKind {
    Slice,
    Merge,
    LiteralExpression,
    /// An array literal assigned to a binding (`a = []`, minor 6).
    LiteralAssignment,
    Allocation,
}

/// The position one occurrence of an array value sits in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UseKind {
    /// Declarator init, alias right-hand side, argument to a declared
    /// function, `return` argument: the edge carries it.
    Flow,
    Push,
    Pop,
    IndexRead,
    IndexWrite,
    LengthRead,
    LengthWrite,
    ForOf,
    Join,
    Slice,
    Search {
        method: String,
        from_index: bool,
    },
    /// A whole array handed to `console.log` and friends.
    Console,
    /// Any other method call; the text is the quoted operation, e.g.
    /// "`.reverse()`".
    Method(String),
    /// Any other position (operand, property read, object or array element,
    /// argument to an unknown callee, …).
    Plain,
    /// Named from a nested function, closure or class body.
    Captured,
    /// A module-scope binding named inside a function.
    ModuleRead,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Use {
    pub(crate) node: GrowNode,
    /// The function whose body holds the occurrence.
    pub(crate) site: String,
    pub(crate) kind: UseKind,
}

/// One argument of a call to a declared function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CallFact {
    pub(crate) site: String,
    pub(crate) callee: String,
    pub(crate) index: usize,
    pub(crate) node: GrowNode,
}

/// One `for-of` over an array value, with what its body does (nested
/// functions excluded: a capture is refused on its own).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LoopFacts {
    pub(crate) iterable: GrowNode,
    pub(crate) site: String,
    /// Receivers of every `push`/`pop` in the body.
    pub(crate) mutations: Vec<GrowNode>,
    /// Every declared-function call argument in the body.
    pub(crate) calls: Vec<CallFact>,
    /// Module-scope loops only (Task 11b): the names the loop declares, its
    /// variables and every binding in its body (nested blocks and loops
    /// included).
    pub(crate) declared: BTreeSet<String>,
    /// Module-scope loops only (Task 11b): every module binding a closure
    /// or nested function inside the body names, by name.
    pub(crate) captured: BTreeSet<String>,
}

/// Final review C1 (spec A-36): where an operand of a growable-array
/// operation that is not a stored element sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum OperandPosition {
    /// `a[i]`, read or written.
    Index,
    /// The first argument of `a.slice(…)`.
    SliceStart,
    /// The second argument of `a.slice(…)`.
    SliceEnd,
    /// The first argument of `a.indexOf(…)` or `a.includes(…)` (spec A-14).
    SearchValue,
}

impl OperandPosition {
    /// What the refusal calls the operand.
    pub(crate) fn text(self) -> &'static str {
        match self {
            OperandPosition::Index => "index",
            OperandPosition::SliceStart => "`slice` start",
            OperandPosition::SliceEnd => "`slice` end",
            OperandPosition::SearchValue => "search value",
        }
    }

    /// A search value may be a string (on a string array: a number or string
    /// of the wrong kind is the element solve's mixed-elements conflict); an
    /// index or a bound must be a number.
    pub(crate) fn admits_strings(self) -> bool {
        self == OperandPosition::SearchValue
    }
}

#[derive(Debug, Default)]
pub(crate) struct GrowFacts {
    pub(crate) edges: Vec<(GrowNode, GrowNode)>,
    /// Bindings declared with an array literal.
    pub(crate) literal_origins: BTreeSet<GrowNode>,
    /// Bindings declared with an allocation, and allocation temporaries.
    pub(crate) plain_origins: BTreeSet<GrowNode>,
    /// Receivers of `push`, `pop` and index writes.
    pub(crate) demands: BTreeSet<GrowNode>,
    pub(crate) non_array_writes: BTreeSet<GrowNode>,
    /// `Temp(n)` → (kind, function whose body holds it).
    pub(crate) temp_kinds: BTreeMap<usize, (TempKind, String)>,
    pub(crate) uses: Vec<Use>,
    pub(crate) calls: Vec<CallFact>,
    pub(crate) loops: Vec<LoopFacts>,
    /// Every value stored into an array (a `push` argument, an index
    /// write, a literal seed), with its number-or-string proof (M2).
    pub(crate) element_values: Vec<(GrowNode, super::elem_proof::ElemProof)>,
    /// Every index, `slice` bound and search value applied to an array, with
    /// its proof (final review C1, spec A-36): a number, or for a search
    /// value a number or a string.
    pub(crate) operands: Vec<(GrowNode, OperandPosition, super::elem_proof::ElemProof)>,
    /// The function-key stack (outermost first) at every class, JSX, `with`,
    /// enum or module-syntax site the walk cannot see through.
    pub(crate) opaque_sites: Vec<Vec<String>>,
    /// Next `Temp` number.
    pub(crate) temps: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GrowConflict {
    /// A growable component that also holds an allocation; the node is the
    /// meeting point (a parameter or alias where possible).
    MixedLayout(GrowNode),
    NonArrayWrite(GrowNode),
    /// The function whose body holds the literal expression.
    LiteralExpression(String),
    /// The function whose body assigns an array literal to a binding.
    LiteralAssignment(String),
}

#[derive(Debug, Default)]
pub(crate) struct GrowSolution {
    component: BTreeMap<GrowNode, usize>,
    members: Vec<Vec<GrowNode>>,
    growable: BTreeSet<usize>,
    pub(crate) conflicts: Vec<GrowConflict>,
}

impl GrowSolution {
    pub(crate) fn component_of(&self, node: &GrowNode) -> Option<usize> {
        self.component.get(node).copied()
    }

    pub(crate) fn is_growable(&self, node: &GrowNode) -> bool {
        self.component_of(node)
            .is_some_and(|component| self.growable.contains(&component))
    }

    pub(crate) fn is_growable_binding(&self, func: &str, name: &str) -> bool {
        self.is_growable(&GrowNode::Binding(func.to_string(), name.to_string()))
    }

    pub(crate) fn same_component(&self, a: &GrowNode, b: &GrowNode) -> bool {
        matches!((self.component_of(a), self.component_of(b)), (Some(x), Some(y)) if x == y)
    }

    /// Every node of every growable component, component by component, each
    /// in `GrowNode` order.
    pub(crate) fn growable_members(&self) -> impl Iterator<Item = &GrowNode> {
        self.growable
            .iter()
            .flat_map(|&component| self.members[component].iter())
    }

    /// Every function whose return value is in a growable component.
    pub(crate) fn growable_returning(&self) -> impl Iterator<Item = &String> {
        self.growable_members().filter_map(|node| match node {
            GrowNode::Return(func) => Some(func),
            _ => None,
        })
    }

    /// The nodes of component `component`.
    pub(crate) fn members_of_component(&self, component: usize) -> &[GrowNode] {
        self.members.get(component).map_or(&[], Vec::as_slice)
    }

    /// The nodes of `node`'s component, or nothing for an unknown node.
    pub(crate) fn members_of(&self, node: &GrowNode) -> &[GrowNode] {
        self.component_of(node)
            .map_or(&[], |component| self.members[component].as_slice())
    }

    /// Spec §3.3, A-16: the binding is the only node of its component and is
    /// not at module scope.
    pub(crate) fn is_local_only(&self, func: &str, name: &str) -> bool {
        func != TOP_LEVEL
            && self
                .members_of(&GrowNode::Binding(func.to_string(), name.to_string()))
                .len()
                == 1
    }
}

fn find(parent: &mut [usize], x: usize) -> usize {
    let mut root = x;
    while parent[root] != root {
        root = parent[root];
    }
    let mut cursor = x;
    while parent[cursor] != root {
        let next = parent[cursor];
        parent[cursor] = root;
        cursor = next;
    }
    root
}

pub(crate) fn solve(facts: &GrowFacts) -> GrowSolution {
    // 1. Every node any fact names, in `GrowNode` order.
    let mut all: BTreeSet<GrowNode> = BTreeSet::new();
    for (a, b) in &facts.edges {
        all.insert(a.clone());
        all.insert(b.clone());
    }
    all.extend(facts.literal_origins.iter().cloned());
    all.extend(facts.plain_origins.iter().cloned());
    all.extend(facts.demands.iter().cloned());
    all.extend(facts.non_array_writes.iter().cloned());
    all.extend(facts.temp_kinds.keys().map(|&n| GrowNode::Temp(n)));
    let nodes: Vec<GrowNode> = all.into_iter().collect();
    let position: BTreeMap<&GrowNode, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node, i))
        .collect();

    // 2. Union along every edge; the smaller index is always the root, so a
    //    component's root is its first member.
    let mut parent: Vec<usize> = (0..nodes.len()).collect();
    for (a, b) in &facts.edges {
        let (ra, rb) = (
            find(&mut parent, position[a]),
            find(&mut parent, position[b]),
        );
        if ra != rb {
            let (low, high) = if ra < rb { (ra, rb) } else { (rb, ra) };
            parent[high] = low;
        }
    }

    // 3. Dense component ids in order of first member.
    let mut dense: BTreeMap<usize, usize> = BTreeMap::new();
    let mut members: Vec<Vec<GrowNode>> = Vec::new();
    let mut component: BTreeMap<GrowNode, usize> = BTreeMap::new();
    for (i, node) in nodes.iter().enumerate() {
        let root = find(&mut parent, i);
        let next = dense.len();
        let id = *dense.entry(root).or_insert(next);
        if id == members.len() {
            members.push(Vec::new());
        }
        members[id].push(node.clone());
        component.insert(node.clone(), id);
    }

    // 4. Growable: a literal origin and a demand in the same component.
    let growable: BTreeSet<usize> = members
        .iter()
        .enumerate()
        .filter(|(_, group)| {
            group.iter().any(|n| facts.literal_origins.contains(n))
                && group.iter().any(|n| facts.demands.contains(n))
        })
        .map(|(id, _)| id)
        .collect();

    // 5. Conflicts, growable components only.
    let mut conflicts = Vec::new();
    for &id in &growable {
        let group = &members[id];
        if group.iter().any(|n| facts.plain_origins.contains(n)) {
            conflicts.push(GrowConflict::MixedLayout(meeting_point(group, facts)));
        }
        for node in group {
            if let GrowNode::Temp(n) = node {
                match facts.temp_kinds.get(n) {
                    Some((TempKind::LiteralExpression, site)) => {
                        conflicts.push(GrowConflict::LiteralExpression(site.clone()));
                    }
                    Some((TempKind::LiteralAssignment, site)) => {
                        conflicts.push(GrowConflict::LiteralAssignment(site.clone()));
                    }
                    _ => {}
                }
            }
        }
        for node in group {
            if facts.non_array_writes.contains(node) {
                conflicts.push(GrowConflict::NonArrayWrite(node.clone()));
            }
        }
    }

    GrowSolution {
        component,
        members,
        growable,
        conflicts,
    }
}

/// Where two layouts meet: the first binding that is neither origin (a
/// parameter or alias), else the first binding, else the first return, else
/// the first node.
fn meeting_point(group: &[GrowNode], facts: &GrowFacts) -> GrowNode {
    let is_binding = |n: &&GrowNode| matches!(n, GrowNode::Binding(..));
    group
        .iter()
        .filter(is_binding)
        .find(|n| !facts.literal_origins.contains(*n) && !facts.plain_origins.contains(*n))
        .or_else(|| group.iter().find(is_binding))
        .or_else(|| group.iter().find(|n| matches!(n, GrowNode::Return(_))))
        .unwrap_or(&group[0])
        .clone()
}

#[cfg(test)]
#[path = "flow_tests.rs"]
mod flow_tests;
