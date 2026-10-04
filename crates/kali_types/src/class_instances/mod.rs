//! Class instances (spec docs/superpowers/specs/2026-10-04-class-instances-design.md):
//! rewrites each in-slice program class to an object-literal factory and
//! `__this`-taking functions, and refuses every instance it cannot prove.

// Later tasks (4-7) consume these; until then they are only exercised by tests.
#[allow(dead_code)]
pub(crate) mod classes;
#[allow(dead_code)]
pub(crate) mod scopes;
#[allow(dead_code)]
pub(crate) mod walk;

#[cfg(test)]
#[path = "classes_tests.rs"]
mod classes_tests;
#[cfg(test)]
#[path = "walk_tests.rs"]
mod walk_tests;
#[cfg(test)]
#[path = "scopes_tests.rs"]
mod scopes_tests;
