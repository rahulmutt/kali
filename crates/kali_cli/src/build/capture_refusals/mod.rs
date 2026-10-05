//! The `check` mirror of captured-bindings spec §3.1 (A-2.6). It walks the
//! analysed AST with the block-scope rename walk, finds every reference that
//! crosses into an enclosing function, and refuses the shapes codegen's
//! capture lane cannot lower. It computes MIR's structural depth: the number
//! of enclosing functions, up to and including the owner, that own at least
//! one captured binding (`kali_mir::env_plan::env_owning_hops`).
//!
//! Residue (A-2.6): a TaggedVal local codegen does not promote is admitted
//! here, because MIR layouts are not visible before MIR. MIR's non-scalar
//! layouts are mirrored only for a syntactic array, object or function
//! initializer (ruling R7); a non-scalar layout reached any other way (for
//! example `let a = makeArr()`) is admitted here while `run` refuses it.
//!
//! Iteration records (A-2.5, §3.4) are out of this pass: a capture of a
//! binding declared in a loop that may own a per-iteration record, or whose
//! capturer chain was created inside such a loop, is left to block-scoping's
//! own refusal. "May own" over-approximates MIR's rule
//! (`kali_mir::analysis::iteration`): any loop that textually contains a
//! deferred-registration call. Over-approximating only admits more here.

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::{Expression, Statement};
use kali_common::{CaptureRefusal, Repr, ReprTable};
use kali_error::{_error_codes::e5, Diagnostic};

use super::block_scope_rename::walk::{self, BindKind, Hooks, ScopeKind};

#[cfg(test)]
#[path = "capture_refusals_tests.rs"]
mod capture_refusals_tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // `Two` is selected by Tasks 9 and 10.
pub(crate) enum Phase {
    /// Before the parameter rewrite and the widening: parameters, F64 and
    /// boolean consts are refused.
    One,
    /// After Tasks 7-10: F64 and boolean consts are admitted; parameters no
    /// longer occur.
    Two,
}

#[derive(Debug)]
struct Frame {
    /// The plan key; `None` for the module root or an unlabelled function.
    label: Option<String>,
    parent: Option<usize>,
    is_module: bool,
    /// The frame's own (first) scope.
    scope: usize,
    /// The open loops of the parent frame this function was created in.
    created_in_loops: Vec<usize>,
}

#[derive(Debug)]
struct Loop {
    frame: usize,
    /// The loop textually contains a deferred-registration call, so MIR may
    /// give it a per-iteration record.
    registers: bool,
}

#[derive(Debug)]
struct ScopeEntry {
    frame: usize,
    parent: Option<usize>,
    /// The innermost open loop of the same frame when the scope opened.
    innermost_loop: Option<usize>,
    bindings: BTreeMap<String, BindKind>,
}

#[derive(Debug)]
struct Use {
    scope: usize,
    name: String,
}

/// Shared with the captured-parameter rewrite's pass A
/// (`capture_param_rewrite`), which keys frames by the same enter order.
#[derive(Default)]
pub(super) struct Recorder {
    frames: Vec<Frame>,
    scopes: Vec<ScopeEntry>,
    stack: Vec<usize>,
    uses: Vec<Use>,
    loops: Vec<Loop>,
    open_loops: Vec<usize>,
    /// The binding `bind` last reported, as (scope, name).
    last_bind: Option<(usize, String)>,
    /// Bindings whose initializer is an array, object or function literal:
    /// MIR gives them an `Array` / `Struct` / `TaggedVal` / `Closure` layout,
    /// never `Scalar` (`kali_mir::analysis::infer::infer_layout`).
    non_scalar: BTreeSet<(usize, String)>,
    /// Locals whose initializer is a bare parameter of their own function:
    /// Task 7's rewritten `let k = k{p}` and a user-written `let n = k`
    /// (ruling R9 (c')). MIR gives them the parameter's `TaggedVal` layout.
    param_copies: BTreeSet<(usize, String)>,
}

impl Hooks for Recorder {
    fn enter(&mut self, kind: ScopeKind, label: Option<&str>) {
        let parent = self.stack.last().copied();
        let scope = self.scopes.len();
        let parent_frame = parent.map(|p| self.scopes[p].frame);
        let frame = match kind {
            ScopeKind::Module | ScopeKind::Function => {
                let created_in_loops = self.open_loops_of(parent_frame).collect();
                self.frames.push(Frame {
                    label: label.map(str::to_string),
                    parent: parent_frame,
                    is_module: kind == ScopeKind::Module,
                    scope,
                    created_in_loops,
                });
                self.frames.len() - 1
            }
            ScopeKind::Block => parent_frame.unwrap_or(0),
        };
        let innermost_loop = self.open_loops_of(Some(frame)).last();
        let mut bindings = BTreeMap::new();
        if let (ScopeKind::Function, Some(label)) = (kind, label) {
            // MIR binds every function's own name inside its own scope
            // (`walk.rs`, `FunctionDecl` / `FunctionExpr` arms), so a nested
            // reference to it is owned by the function itself.
            bindings.insert(label.to_string(), BindKind::FunctionDecl);
        }
        self.scopes.push(ScopeEntry {
            frame,
            parent,
            innermost_loop,
            bindings,
        });
        self.stack.push(scope);
    }

    fn exit(&mut self) {
        self.stack.pop();
    }

    fn bind(&mut self, name: &mut String, kind: BindKind) {
        let current = *self.stack.last().expect("bind inside a scope");
        // `var` lands in the frame's own scope.
        let target = if kind == BindKind::Var {
            self.frames[self.scopes[current].frame].scope
        } else {
            current
        };
        let bindings = &mut self.scopes[target].bindings;
        if kind == BindKind::Param {
            // A parameter shadows the function's own name.
            bindings.insert(name.clone(), kind);
        } else {
            bindings.entry(name.clone()).or_insert(kind);
        }
        self.last_bind = Some((target, name.clone()));
    }

    fn initializer(&mut self, init: &Expression) {
        let mut init = init;
        while let Expression::ParenthesizedExpression(inner) = init {
            init = &inner.expression;
        }
        let non_scalar = matches!(
            init,
            Expression::ArrayExpression(_)
                | Expression::ObjectExpression(_)
                | Expression::FunctionExpression(_)
                | Expression::ArrowFunctionExpression(_)
        );
        if let (true, Some(binding)) = (non_scalar, self.last_bind.clone()) {
            self.non_scalar.insert(binding);
        }
        // A local initialized from a parameter of its own function: a Task 7
        // rewritten parameter (`let k = k{p}`) or a user-written copy
        // (`let n = k`). Both take the parameter's `TaggedVal` layout.
        if let (Expression::Identifier(source), Some(binding)) = (init, self.last_bind.clone()) {
            let current = *self.stack.last().expect("initializer inside a scope");
            let from_own_param = self.resolve(current, source).is_some_and(|(scope, kind)| {
                kind == BindKind::Param && self.scopes[scope].frame == self.scopes[binding.0].frame
            });
            if from_own_param {
                self.param_copies.insert(binding);
            }
        }
    }

    fn reference(&mut self, name: &mut String) {
        let scope = *self.stack.last().expect("reference inside a scope");
        self.uses.push(Use {
            scope,
            name: name.clone(),
        });
    }

    fn enter_loop(&mut self) {
        let scope = *self.stack.last().expect("loop inside a scope");
        self.loops.push(Loop {
            frame: self.scopes[scope].frame,
            registers: false,
        });
        self.open_loops.push(self.loops.len() - 1);
    }

    fn exit_loop(&mut self) {
        self.open_loops.pop();
    }

    fn call(&mut self, callee: &Expression) {
        let registers = match callee {
            Expression::Identifier(name) => {
                kali_common::is_deferred_registration_callee(name, false)
            }
            Expression::MemberExpression(member) => member
                .property
                .as_deref()
                .is_some_and(|name| kali_common::is_deferred_registration_callee(name, true)),
            _ => false,
        };
        if registers {
            // Every open loop textually contains the call (MIR ruling R8).
            for &id in &self.open_loops {
                self.loops[id].registers = true;
            }
        }
    }
}

impl Recorder {
    fn open_loops_of(&self, frame: Option<usize>) -> impl Iterator<Item = usize> + '_ {
        self.open_loops
            .iter()
            .copied()
            .filter(move |&id| Some(self.loops[id].frame) == frame)
    }

    /// A-2.5: the binding is declared in a loop that may own a per-iteration
    /// record (MIR moves it into that record's plan).
    fn in_iteration_record(&self, scope: usize, kind: BindKind) -> bool {
        kind == BindKind::Lexical
            && self.scopes[scope]
                .innermost_loop
                .is_some_and(|id| self.loops[id].registers)
    }

    /// A-2.5: the walk from `capturer` up to `owner` steps through a loop that
    /// may own a per-iteration record (`through_iteration`).
    fn crosses_iteration(&self, capturer: usize, owner: usize) -> bool {
        let mut current = capturer;
        while current != owner {
            let frame = &self.frames[current];
            if frame
                .created_in_loops
                .iter()
                .any(|&id| self.loops[id].registers)
            {
                return true;
            }
            let Some(parent) = frame.parent else {
                return false;
            };
            current = parent;
        }
        false
    }

    /// The number of Module / Function frames entered, in enter order.
    pub(super) fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// The frame that owns `scope`.
    pub(super) fn frame_of(&self, scope: usize) -> usize {
        self.scopes[scope].frame
    }

    /// Every identifier reference, as (scope, name), in walk order.
    pub(super) fn references(&self) -> impl Iterator<Item = (usize, &str)> + '_ {
        self.uses.iter().map(|u| (u.scope, u.name.as_str()))
    }

    pub(super) fn resolve(&self, from: usize, name: &str) -> Option<(usize, BindKind)> {
        let mut cursor = Some(from);
        while let Some(id) = cursor {
            if let Some(kind) = self.scopes[id].bindings.get(name) {
                return Some((id, *kind));
            }
            cursor = self.scopes[id].parent;
        }
        None
    }
}

struct Capture {
    capturer: usize,
    owner: usize,
    name: String,
    kind: BindKind,
    /// The binding's initializer is an array, object or function literal.
    non_scalar: bool,
    /// The binding is initialized from its function's own parameter.
    param_copy: bool,
}

/// Captured-bindings §3.4 / A-2.6: one E5506 per (capturing function,
/// captured name) whose capture codegen's lane cannot lower, with the text of
/// `kali_common::captured_binding_unavailable_message`.
pub(crate) fn capture_refusals(
    statements: &mut [Statement],
    repr_table: &ReprTable,
    phase: Phase,
) -> Vec<Diagnostic> {
    let mut recorder = Recorder::default();
    walk::walk_program(statements, &mut recorder);

    let mut captures = Vec::new();
    let mut cell_owners = BTreeSet::new();
    for u in &recorder.uses {
        let Some((scope, kind)) = recorder.resolve(u.scope, &u.name) else {
            continue;
        };
        let owner = recorder.scopes[scope].frame;
        let capturer = recorder.scopes[u.scope].frame;
        if owner == capturer || recorder.frames[owner].is_module {
            continue;
        }
        if recorder.in_iteration_record(scope, kind) {
            continue;
        }
        // Function and class names are program-wide downstream, so they are
        // never refused here, but MIR still gives a captured function or
        // class declaration (or a function's own name) a cell, which makes
        // its owner an env owner. A class expression's name is no MIR binding.
        if kind.is_program_wide() {
            if kind != BindKind::ClassExprId {
                cell_owners.insert(owner);
            }
            continue;
        }
        captures.push(Capture {
            capturer,
            owner,
            name: u.name.clone(),
            kind,
            non_scalar: recorder.non_scalar.contains(&(scope, u.name.clone())),
            param_copy: recorder.param_copies.contains(&(scope, u.name.clone())),
        });
    }
    // Owners of a captured binding outside every iteration record, as MIR's
    // `env_owners` after the iteration cells moved out.
    let mut env_owners = cell_owners;
    env_owners.extend(captures.iter().map(|c| c.owner));

    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for c in &captures {
        let (Some(capturer_key), Some(owner_key)) = (
            recorder.frames[c.capturer].label.as_deref(),
            recorder.frames[c.owner].label.as_deref(),
        ) else {
            continue; // A-2.6: no plan key, admitted and recorded in followups
        };
        if recorder.crosses_iteration(c.capturer, c.owner) {
            continue;
        }
        if !seen.insert((c.capturer, c.name.clone())) {
            continue;
        }
        let reason = if structural_depth(&recorder.frames, c.capturer, c.owner, &env_owners) >= 2 {
            Some(CaptureRefusal::Depth)
        } else if phase == Phase::One && c.kind == BindKind::Param {
            Some(CaptureRefusal::Parameter { owner: owner_key })
        } else if (c.param_copy && !param_copy_has_a_cell(repr_table, owner_key, &c.name))
            || !repr_has_a_cell(repr_table, owner_key, &c.name, c.non_scalar, phase)
        {
            Some(CaptureRefusal::ValueType)
        } else {
            None
        };
        if let Some(reason) = reason {
            out.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                kali_common::captured_binding_unavailable_message(capturer_key, &c.name, reason),
            ));
        }
    }
    out
}

/// `env_owning_hops`: count each step into an ancestor that owns a captured
/// binding, from the capturer up to and including the owner.
fn structural_depth(frames: &[Frame], from: usize, to: usize, owners: &BTreeSet<usize>) -> u32 {
    let mut depth = 0;
    let mut current = from;
    while current != to {
        let Some(parent) = frames[current].parent else {
            return 0;
        };
        if owners.contains(&parent) {
            depth += 1;
        }
        current = parent;
    }
    depth
}

/// Ruling R9 (c'): a local copied from a parameter (rewritten or user-written)
/// is a `TaggedVal` cell, which
/// codegen promotes only as a proven-numeric `I64` (A-1 point 3; an Object
/// repr is excluded, spec §1.1). Mirrors `cell_is_promotable`'s tagged branch.
fn param_copy_has_a_cell(table: &ReprTable, owner: &str, name: &str) -> bool {
    // Task 9: F64 with proof
    table.scalar(owner, name) == Repr::I64 && table.binding_is_proven_numeric(owner, name)
}

/// Mirrors `kali_codegen::closure::cell_is_promotable`: a non-scalar cell
/// promotes only as `Object(_)` (or an abort handle), a scalar one as `I64`
/// (and `F64` in phase 2).
fn repr_has_a_cell(
    table: &ReprTable,
    owner: &str,
    name: &str,
    non_scalar: bool,
    phase: Phase,
) -> bool {
    if phase == Phase::One && table.binding_is_boolean_const(owner, name) {
        return false;
    }
    if non_scalar {
        return matches!(
            table.scalar(owner, name),
            Repr::Object(_) | Repr::AbortHandle
        );
    }
    match table.scalar(owner, name) {
        Repr::I64 | Repr::Object(_) | Repr::AbortHandle => true,
        Repr::F64 => phase == Phase::Two,
        _ => false,
    }
}
