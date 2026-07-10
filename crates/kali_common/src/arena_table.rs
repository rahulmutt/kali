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
/// Five disjoint decision sets:
/// - `arena_eligible`: functions whose allocation sites may call `__alloc`
///   (the current arena) instead of `__alloc_global`.
/// - `opens_arena`: functions that should open a function-body arena because
///   they have at least one allocation that dies inside them.
/// - `loop_arena`: `(function, loop_preorder_ordinal)` pairs where the loop
///   body should open a per-iteration arena.
/// - `arena_string_site`: `(function, string_site_preorder_ordinal)` pairs
///   where a string-producing site (`.join()` or `+`) is proven iteration-local
///   and may allocate via the current arena (`__alloc`) instead of the global
///   heap (`__alloc_global`). Misses fail closed.
/// - `string_arena_loop`: `(function, loop_preorder_ordinal)` pairs where the
///   loop should open a per-iteration arena SOLELY because its body contains a
///   granted `arena_string_site` (fasta Spec 7 Task 4f). This channel is
///   emit-only — it drives the SAME open/reset/release wiring `loop_arena`
///   does but changes NO routing decision; it exists because a loop whose only
///   reclaimable allocation is a granted string site (no object/array literal)
///   never trips `loop_arena`'s reaches-alloc rule. Keyed by the identical
///   pre-order loop ordinal stream. Misses fail closed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArenaTable {
    arena_eligible: BTreeSet<String>,
    opens_arena: BTreeSet<String>,
    loop_arena: BTreeSet<(String, u32)>,
    arena_string_site: BTreeSet<(String, u32)>,
    string_arena_loop: BTreeSet<(String, u32)>,
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

    /// Mark the string-producing site with pre-order `site_ordinal` in `func` as
    /// iteration-local (routable to the current arena). Misses fail closed.
    pub fn set_arena_string_site(&mut self, func: &str, site_ordinal: u32) {
        self.arena_string_site
            .insert((func.to_string(), site_ordinal));
    }

    /// Whether the string-producing site with pre-order `site_ordinal` in `func`
    /// is iteration-local. Misses fail closed (`false` == use `__alloc_global`).
    pub fn arena_string_site(&self, func: &str, site_ordinal: u32) -> bool {
        self.arena_string_site
            .contains(&(func.to_string(), site_ordinal))
    }

    /// Mark the loop with pre-order `ordinal` in `func` as opening a
    /// per-iteration arena because its body contains a granted
    /// `arena_string_site` (fasta Spec 7 Task 4f). Emit-only: it never changes
    /// which allocator a site routes to, only whether the loop resets a
    /// per-iteration arena. Misses fail closed.
    pub fn set_string_arena_loop(&mut self, func: &str, ordinal: u32) {
        self.string_arena_loop.insert((func.to_string(), ordinal));
    }

    /// Whether the loop with pre-order `ordinal` in `func` opens a per-iteration
    /// arena on account of a granted string site. Misses fail closed.
    pub fn string_arena_loop(&self, func: &str, ordinal: u32) -> bool {
        self.string_arena_loop
            .contains(&(func.to_string(), ordinal))
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

    #[test]
    fn string_arena_loop_is_keyed_by_function_and_ordinal() {
        let mut table = ArenaTable::default();
        // Fails closed on every query before anything is recorded.
        assert!(!table.string_arena_loop("f", 0));
        table.set_string_arena_loop("f", 0);
        table.set_string_arena_loop("f", 2);
        assert!(table.string_arena_loop("f", 0));
        assert!(table.string_arena_loop("f", 2));
        // A recorded function but unrecorded ordinal fails closed.
        assert!(!table.string_arena_loop("f", 1));
        // A recorded ordinal under a different function fails closed.
        assert!(!table.string_arena_loop("g", 0));
        // The channel is disjoint from `loop_arena` — setting one does not set
        // the other (they OR together only in codegen).
        assert!(!table.loop_arena("f", 0));
    }

    #[test]
    fn arena_string_site_defaults_closed_and_records() {
        let mut t = ArenaTable::default();
        assert!(
            !t.arena_string_site("f", 0),
            "miss must fail closed (global)"
        );
        t.set_arena_string_site("f", 2);
        assert!(t.arena_string_site("f", 2));
        assert!(!t.arena_string_site("f", 1));
        assert!(!t.arena_string_site("g", 2));
    }
}
