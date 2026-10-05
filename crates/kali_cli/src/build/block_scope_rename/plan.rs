//! Pass B, step 1: decide which bindings get a new spelling.

use std::collections::{BTreeMap, HashMap};

use super::table::{Binding, ScopeId, ScopeTable};

/// `(scope, original name)` to the new spelling.
pub(crate) type RenamePlan = BTreeMap<(ScopeId, String), String>;

/// Decide which bindings are renamed (block-scoping spec §3.2, A-1).
///
/// A binding `b = (s, name)` with `s != 0` (not module scope) is renamed if
/// either rule below says so. Module-scope bindings are never renamed.
///
/// - **Variable rule.** `chain(s)` is `s`'s frame and every enclosing frame.
///   The rivals are every other binding of `name` whose scope's frame is in
///   `chain(s)`, excluding a binding in scope `s` itself. `b` is renamed if
///   any rival ranks lower, where rank is `(frame_level, depth, ordinal)` and
///   lower wins. Enclosing frames have a lower `frame_level`, so they always
///   win.
/// - **Program-wide rule** (only when `b.kind.is_program_wide()`). The rivals
///   are every other program-wide binding of `name` anywhere. `b` is renamed
///   if any rival ranks lower, where rank is `(scope != 0, ordinal)`: a
///   top-level one first, then the earliest.
///
/// Renamed bindings, sorted by `ordinal`, get `format!("{name}{{b{n}}}")`
/// with `n` = 0, 1, 2, ….
pub(crate) fn plan_renames(table: &ScopeTable) -> RenamePlan {
    let all: Vec<(ScopeId, &str, &Binding)> = table
        .scopes
        .iter()
        .enumerate()
        .flat_map(|(id, scope)| {
            scope
                .bindings
                .iter()
                .map(move |(name, binding)| (id, name.as_str(), binding))
        })
        .collect();

    let rank = |scope: ScopeId, ordinal: u32| {
        let s = &table.scopes[scope];
        (s.frame_level, s.depth, ordinal)
    };

    // A scope binds a name at most once, so "a rival in another scope ranks
    // lower" only needs the two lowest-ranked bindings of each key: if the
    // lowest is `b` itself, the second lowest is the best rival.
    let mut by_frame: HashMap<(ScopeId, &str), Lowest2<VariableRank>> = HashMap::new();
    let mut program_wide: HashMap<&str, Lowest2<(bool, u32)>> = HashMap::new();
    for &(s, name, b) in &all {
        by_frame
            .entry((table.scopes[s].frame, name))
            .or_default()
            .offer(rank(s, b.ordinal), s);
        if b.kind.is_program_wide() {
            program_wide
                .entry(name)
                .or_default()
                .offer((s != 0, b.ordinal), s);
        }
    }

    let mut chains: HashMap<ScopeId, Vec<ScopeId>> = HashMap::new();
    let mut chosen: Vec<(u32, ScopeId, &str)> = Vec::new();
    for &(s, name, b) in &all {
        if s == 0 {
            continue;
        }
        let frame = table.scopes[s].frame;
        let chain = chains.entry(frame).or_insert_with(|| frame_chain(table, s));
        let mine = rank(s, b.ordinal);
        let variable_rival = chain.iter().any(|f| {
            by_frame
                .get(&(*f, name))
                .is_some_and(|lowest| lowest.has_rival_below(s, &mine))
        });
        let program_rival = b.kind.is_program_wide()
            && program_wide
                .get(name)
                .is_some_and(|lowest| lowest.has_rival_below(s, &(true, b.ordinal)));
        if variable_rival || program_rival {
            chosen.push((b.ordinal, s, name));
        }
    }
    chosen.sort();
    chosen
        .into_iter()
        .enumerate()
        .map(|(n, (_, s, name))| ((s, name.to_string()), format!("{name}{{b{n}}}")))
        .collect()
}

/// The variable rule's rank: `(frame_level, depth, ordinal)`, lower wins.
type VariableRank = (u32, u32, u32);

/// The two lowest `(rank, scope)` entries offered, lowest first.
#[derive(Debug)]
struct Lowest2<R> {
    entries: Vec<(R, ScopeId)>,
}

impl<R> Default for Lowest2<R> {
    fn default() -> Self {
        Self {
            entries: Vec::with_capacity(2),
        }
    }
}

impl<R: Ord + Copy> Lowest2<R> {
    fn offer(&mut self, rank: R, scope: ScopeId) {
        let at = self
            .entries
            .iter()
            .position(|entry| (rank, scope) < *entry)
            .unwrap_or(self.entries.len());
        if at < 2 {
            self.entries.insert(at, (rank, scope));
            self.entries.truncate(2);
        }
    }

    /// Whether an entry from a scope other than `scope` ranks below `bound`.
    fn has_rival_below(&self, scope: ScopeId, bound: &R) -> bool {
        self.entries
            .iter()
            .find(|(_, other)| *other != scope)
            .is_some_and(|(rank, _)| rank < bound)
    }
}

/// The frame of `s` and the frames of every enclosing scope.
fn frame_chain(table: &ScopeTable, s: ScopeId) -> Vec<ScopeId> {
    let mut chain = Vec::new();
    let mut frame = Some(table.scopes[s].frame);
    while let Some(f) = frame {
        chain.push(f);
        frame = table.scopes[f].parent.map(|p| table.scopes[p].frame);
    }
    chain
}

#[cfg(test)]
#[path = "plan_tests.rs"]
mod plan_tests;
