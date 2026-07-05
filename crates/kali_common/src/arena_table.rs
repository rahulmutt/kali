//! Name-keyed arena placement decisions produced by the `kali_mir` escape gate
//! and consumed by codegen.
//!
//! The table is intentionally additive and **fails closed**: every query for a
//! name (or loop ordinal) that was never recorded returns `false`, i.e. "no
//! arena / global allocation". Sending a site to the global heap or vetoing an
//! arena is always sound; the only cost is memory that is not reclaimed. This
//! mirrors the `ReprTable` precedent (empty table == conservative default).

use std::collections::BTreeSet;

/// Arena placement decisions, keyed by function name (and loop ordinal).
///
/// Three disjoint decision sets:
/// - `arena_eligible`: functions whose allocation sites may call `__alloc`
///   (the current arena) instead of `__alloc_global`.
/// - `opens_arena`: functions that should open a function-body arena because
///   they have at least one allocation that dies inside them.
/// - `loop_arena`: `(function, loop_preorder_ordinal)` pairs where the loop
///   body should open a per-iteration arena.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArenaTable {
    arena_eligible: BTreeSet<String>,
    opens_arena: BTreeSet<String>,
    loop_arena: BTreeSet<(String, u32)>,
}

impl ArenaTable {
    /// Mark `func`'s allocation sites as eligible for the current arena.
    pub fn set_arena_eligible(&mut self, func: &str) {
        self.arena_eligible.insert(func.to_string());
    }

    /// Whether `func`'s allocation sites may call `__alloc` (current arena).
    /// Misses fail closed (`false` == use `__alloc_global`).
    pub fn arena_eligible(&self, func: &str) -> bool {
        self.arena_eligible.contains(func)
    }

    /// Mark `func` as opening a function-body arena.
    pub fn set_opens_arena(&mut self, func: &str) {
        self.opens_arena.insert(func.to_string());
    }

    /// Whether `func` opens a function-body arena. Misses fail closed.
    pub fn opens_arena(&self, func: &str) -> bool {
        self.opens_arena.contains(func)
    }

    /// Mark the loop with pre-order `ordinal` in `func` as opening a
    /// per-iteration arena.
    pub fn set_loop_arena(&mut self, func: &str, ordinal: u32) {
        self.loop_arena.insert((func.to_string(), ordinal));
    }

    /// Whether the loop with pre-order `ordinal` in `func` opens a per-iteration
    /// arena. Misses fail closed.
    pub fn loop_arena(&self, func: &str, ordinal: u32) -> bool {
        self.loop_arena.contains(&(func.to_string(), ordinal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_table_fails_closed_on_every_query() {
        let table = ArenaTable::default();
        assert!(!table.arena_eligible("factory"));
        assert!(!table.opens_arena("factory"));
        assert!(!table.loop_arena("factory", 0));
    }

    #[test]
    fn arena_eligible_set_and_get() {
        let mut table = ArenaTable::default();
        table.set_arena_eligible("factory");
        assert!(table.arena_eligible("factory"));
        // A different name still misses (fails closed).
        assert!(!table.arena_eligible("other"));
        // The other decision sets are unaffected.
        assert!(!table.opens_arena("factory"));
        assert!(!table.loop_arena("factory", 0));
    }

    #[test]
    fn opens_arena_set_and_get() {
        let mut table = ArenaTable::default();
        table.set_opens_arena("main");
        assert!(table.opens_arena("main"));
        assert!(!table.opens_arena("other"));
        assert!(!table.arena_eligible("main"));
    }

    #[test]
    fn loop_arena_is_keyed_by_function_and_ordinal() {
        let mut table = ArenaTable::default();
        table.set_loop_arena("f", 0);
        table.set_loop_arena("f", 2);
        assert!(table.loop_arena("f", 0));
        assert!(table.loop_arena("f", 2));
        // A recorded function but unrecorded ordinal fails closed.
        assert!(!table.loop_arena("f", 1));
        // A recorded ordinal under a different function fails closed.
        assert!(!table.loop_arena("g", 0));
    }

    #[test]
    fn set_is_idempotent() {
        let mut table = ArenaTable::default();
        table.set_arena_eligible("f");
        table.set_arena_eligible("f");
        assert!(table.arena_eligible("f"));
    }
}
