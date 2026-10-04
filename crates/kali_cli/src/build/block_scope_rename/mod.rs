//! Block-scope rename (block-scoping spec §3.1–§3.2). See `rename_block_scoped_bindings`.

use kali_ast::Statement;

pub(crate) mod apply;
pub(crate) mod plan;
pub(crate) mod table;
pub(crate) mod walk;

#[cfg(test)]
pub(crate) mod test_support;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenameOutcome {
    pub renamed: usize,
}

/// Give every binding that shadows, or shares its frame chain with, a
/// same-named binding the spelling `<name>{b<N>}` (block-scoping spec §3.2,
/// A-1). Runs before monomorphize so every later stage, all of which key by
/// name, sees one binding per spelling. A program that needs no rename is
/// left untouched.
pub fn rename_block_scoped_bindings(statements: &mut [Statement]) -> RenameOutcome {
    let mut collector = table::Collector::default();
    walk::walk_program(statements, &mut collector);
    let table = collector.finish();
    let plan = plan::plan_renames(&table);
    if plan.is_empty() {
        return RenameOutcome { renamed: 0 };
    }
    let mut renamer = apply::Renamer::new(&table, &plan);
    walk::walk_program(statements, &mut renamer);
    RenameOutcome {
        renamed: plan.len(),
    }
}
