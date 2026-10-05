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
pub(crate) enum Phase {
    /// Before the parameter rewrite: a captured parameter is refused.
    One,
    /// After the parameter rewrite (Task 7), so parameters no longer occur.
    /// F64 stays refused in both phases (ruling R14); Task 10 admits boolean
    /// consts here.
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
    /// The target of an assignment statement whose value codegen discards,
    /// with an operator an F64 cell lowers from a capturer (ruling R14).
    f64_write: bool,
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
    /// Task 7 rewritten-parameter locals: `let k = k{p}`, whose initializer
    /// is the bare identifier spelled `<name>{p}` (rulings R9 (c'), R13).
    rewritten_params: BTreeSet<(usize, String)>,
    /// The identifier target of the assignment statement being walked, when
    /// its operator is one ruling R14 lowers on a captured F64 cell; the next
    /// reference (the left-hand side, walked first) is that write.
    pending_f64_write: Option<String>,
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
        // A Task 7 rewritten-parameter local: `let k = k{p}`, whose
        // initializer is the bare identifier spelled `<name>{p}` (ruling R13
        // keys on the spelling alone; a user-written `let n = p` stays
        // admitted, the A-2.6 residue).
        if let (Expression::Identifier(source), Some(binding)) = (init, self.last_bind.clone()) {
            if *source == format!("{}{{p}}", binding.1) {
                self.rewritten_params.insert(binding);
            }
        }
    }

    fn reference(&mut self, name: &mut String) {
        let scope = *self.stack.last().expect("reference inside a scope");
        let f64_write = self.pending_f64_write.take().as_ref() == Some(name);
        self.uses.push(Use {
            scope,
            name: name.clone(),
            f64_write,
        });
    }

    fn assignment_statement(&mut self, assign: &kali_ast::AssignmentExpression) {
        use kali_ast::AssignmentOperator as Op;
        let lowered = matches!(
            assign.operator,
            Op::Assign | Op::AddAssign | Op::SubtractAssign | Op::MultiplyAssign | Op::DivideAssign
        );
        self.pending_f64_write = match &assign.left {
            Expression::Identifier(target) if lowered => Some(target.clone()),
            _ => None,
        };
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
    /// The binding is a Task 7 rewritten-parameter local (`let k = k{p}`).
    rewritten_param: bool,
    /// This reference is a discarded `=` / `+=` / `-=` / `*=` / `/=` target.
    f64_write: bool,
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
            rewritten_param: recorder.rewritten_params.contains(&(scope, u.name.clone())),
            f64_write: u.f64_write,
        });
    }
    // Owners of a captured binding outside every iteration record, as MIR's
    // `env_owners` after the iteration cells moved out.
    let mut env_owners = cell_owners;
    env_owners.extend(captures.iter().map(|c| c.owner));

    // Ruling R14: a capturer that reads a captured binding anywhere other
    // than as a discarded write target. Codegen refuses such a read of an F64
    // cell; the writes alone lower.
    let reads: BTreeSet<(usize, &str)> = captures
        .iter()
        .filter(|c| !c.f64_write)
        .map(|c| (c.capturer, c.name.as_str()))
        .collect();

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
        } else if (c.rewritten_param && !rewritten_param_has_a_cell(repr_table, owner_key, &c.name))
            || !repr_has_a_cell(
                repr_table,
                owner_key,
                &c.name,
                c.non_scalar,
                !reads.contains(&(c.capturer, c.name.as_str())),
            )
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

/// Rulings R9 (c') / R13: a rewritten-parameter local is a `TaggedVal` cell.
/// Codegen promotes it as a proven-numeric `I64` (A-1 point 3) or, on either
/// side of the scalar bit, as an abort handle (`cell_is_promotable`'s first
/// arm). An `Object` repr is refused here although C2 promotes it: spec §1.1
/// does not widen Object parameters, and `run` refuses the member access no
/// lane resolves (ruling R13).
fn rewritten_param_has_a_cell(table: &ReprTable, owner: &str, name: &str) -> bool {
    match table.scalar(owner, name) {
        Repr::AbortHandle => true,
        // Ruling R14: an F64 capture is refused by `repr_has_a_cell`.
        Repr::I64 => table.binding_is_proven_numeric(owner, name),
        _ => false,
    }
}

/// Mirrors `kali_codegen::closure::cell_is_promotable`: a non-scalar cell
/// promotes only as `Object(_)` (or an abort handle), a scalar one as `I64`
/// or `F64`. Ruling R14 (A-4): codegen lowers only a capturer's discarded
/// `=` / `+=` / `-=` / `*=` / `/=` on an F64 cell and refuses every other read,
/// so an F64 capture is admitted only when `writes_only` (the capturer never
/// reads it otherwise). An assignment statement is the one discarded shape
/// recognized here; codegen may lower more, so `check` stays the stricter.
fn repr_has_a_cell(
    table: &ReprTable,
    owner: &str,
    name: &str,
    non_scalar: bool,
    writes_only: bool,
) -> bool {
    // Task 10: a captured boolean `const` is refused in both phases until its
    // capture read carries `ValueShape::Boolean` (A-2.1).
    if table.binding_is_boolean_const(owner, name) {
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
        Repr::F64 => writes_only,
        _ => false,
    }
}
