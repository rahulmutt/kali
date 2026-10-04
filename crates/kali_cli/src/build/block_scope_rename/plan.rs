//! Pass B, step 1: decide which bindings get a new spelling.

use std::collections::BTreeMap;

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

    let mut chosen: Vec<(u32, ScopeId, &str)> = Vec::new();
    for &(s, name, b) in &all {
        if s == 0 {
            continue;
        }
        let chain = frame_chain(table, s);
        let mine = rank(s, b.ordinal);
        let variable_rival = all.iter().any(|&(o, n, ob)| {
            n == name
                && o != s
                && chain.contains(&table.scopes[o].frame)
                && rank(o, ob.ordinal) < mine
        });
        let program_rival = b.kind.is_program_wide()
            && all.iter().any(|&(o, n, ob)| {
                n == name
                    && o != s
                    && ob.kind.is_program_wide()
                    && (o != 0, ob.ordinal) < (true, b.ordinal)
            });
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
