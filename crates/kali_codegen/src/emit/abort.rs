//! Abort-handle emission (Stage P3): receiver-handle loads under the position
//! allowlist, and the abort-cell store/load idiom. The handle is an i64 pointer
//! to one 8-byte `__alloc_global` cell holding the aborted flag; controller and
//! signal are the same handle.

use crate::*;

/// A recognized member read on a proven abort handle (Task 4). The payload is
/// the LIR node whose emission under the position allowlist
/// (`emit_abort_receiver_handle`) yields the shared i64 cell handle.
pub(crate) enum AbortMemberRead {
    /// `c.signal` — identity: the signal IS the controller handle. Task 5 also
    /// uses this recognition for the `instanceof AbortSignal` left operand.
    Signal(LirNodeId),
    /// `c.aborted`, `s.aborted`, or `c.signal.aborted` — a real load of the
    /// aborted flag from the cell.
    Aborted(LirNodeId),
}

impl<'a> FunctionEmitter<'a> {
    /// Recognize a member read whose receiver is a proven abort handle:
    ///   * `X.signal`  where `X` is a childless identifier with
    ///     `is_abort_handle(X)`                                   → `Signal(X)`
    ///   * `X.aborted` where `X` is a childless identifier with
    ///     `is_abort_handle(X)`, OR `X` is itself a `<ident>.signal`
    ///     member with `is_abort_handle(ident)`                  → `Aborted(X)`
    /// The payload is `node.children[0]` in both cases: emitting it via
    /// `emit_abort_receiver_handle` yields the handle (an identifier read is
    /// the handle directly; a `.signal` member read is identity). Any other
    /// field, or an unproven base, returns `None` (fail-closed by falling
    /// through to the generic member paths, which deny at the identifier
    /// choke point).
    pub(crate) fn abort_member_read_parts(&self, node: LirNodeId) -> Option<AbortMemberRead> {
        let node = self.node(node);
        if node.kind != LirNodeKind::Value || node.children.len() != 1 {
            return None;
        }
        let field = node.text.as_deref().filter(|t| !t.is_empty())?;
        let base_id = node.children[0];
        let base = self.node(base_id);
        let base_is_handle_ident = base.children.is_empty()
            && base
                .text
                .as_deref()
                .is_some_and(|name| self.is_abort_handle(name));
        match field {
            "signal" if base_is_handle_ident => Some(AbortMemberRead::Signal(base_id)),
            "aborted" => {
                let base_is_signal_of_handle = base.kind == LirNodeKind::Value
                    && base.children.len() == 1
                    && base.text.as_deref() == Some("signal")
                    && {
                        let inner = self.node(base.children[0]);
                        inner.children.is_empty()
                            && inner
                                .text
                                .as_deref()
                                .is_some_and(|name| self.is_abort_handle(name))
                    };
                (base_is_handle_ident || base_is_signal_of_handle)
                    .then_some(AbortMemberRead::Aborted(base_id))
            }
            _ => None,
        }
    }

    /// Task 8 round-2 read-position twin of the call-side
    /// `is_module_scope_abort_handle` gate. True when `node` is a member read
    /// `X.<field>` or `X.signal.<field>` whose ultimate receiver `X` is a bare
    /// identifier that is a `_start`-owned abort handle reached from a
    /// non-`_start` emitter (`is_module_scope_abort_handle`). Such reads MUST
    /// deny: `is_abort_handle` (and thus `abort_member_read_parts`) excludes the
    /// `_start` owner by design — the captured env cell is never populated, so
    /// the deferred read is stale — which drops the read into the generic member
    /// fallback that silently yields `0` (`c.signal.aborted` → `0`/`no`). Keyed
    /// structurally (any field), mirroring the call-side choke point's
    /// method-agnostic deny. The not-a-current-fn-local guards live inside
    /// `is_module_scope_abort_handle`, so a genuine same-named local is unaffected.
    pub(crate) fn member_receiver_is_module_abort_handle(&self, node: LirNodeId) -> bool {
        let node = self.node(node);
        if node.kind != LirNodeKind::Value || node.children.len() != 1 {
            return false;
        }
        if node.text.as_deref().filter(|t| !t.is_empty()).is_none() {
            return false;
        }
        let base = self.node(node.children[0]);
        // `X.<field>` — X a bare module-abort-handle identifier.
        if base.children.is_empty() {
            return base
                .text
                .as_deref()
                .is_some_and(|name| self.is_module_scope_abort_handle(name));
        }
        // `X.signal.<field>` — the `.signal` identity hop over a module handle.
        if base.kind == LirNodeKind::Value
            && base.children.len() == 1
            && base.text.as_deref() == Some("signal")
        {
            let inner = self.node(base.children[0]);
            return inner.children.is_empty()
                && inner
                    .text
                    .as_deref()
                    .is_some_and(|name| self.is_module_scope_abort_handle(name));
        }
        false
    }

    /// Task 5 left-operand proof for the `instanceof AbortSignal` allow lane.
    /// True when `left` is a proven abort handle in signal position:
    ///   * a childless identifier `X` with `is_abort_handle(X)`  (the `s` alias
    ///     in `s instanceof AbortSignal`), OR
    ///   * a member node matching `abort_member_read_parts(...) == Signal(_)`
    ///     (the `c.signal instanceof AbortSignal` form).
    /// Matches the RAW `left` node — deliberately NOT `unwrap_transparent`.
    /// Empirically the parser resolves parens at parse time (no wrapper node),
    /// so `(c.signal) instanceof AbortSignal` already arrives as the bare member
    /// and needs no tunneling. Tunneling would be UNSOUND here: a single-element
    /// array literal `[c.signal]` is also a textless one-child `Value`
    /// (structurally identical to a grouping wrapper — see the `unwrap_transparent`
    /// note in operators.rs), so tunneling would fold `[c.signal] instanceof
    /// AbortSignal` to a wrong `true` (a JS array is not an AbortSignal). The raw
    /// match rejects that array node (non-empty children, and `abort_member_read_parts`
    /// returns `None` on a textless node), so it falls through to the runtime
    /// trap — reject, don't miscompile.
    /// SOUNDNESS: the fold emits NO code for the left operand (it is a
    /// compile-time constant `true`). That is sound ONLY because both admitted
    /// shapes are side-effect-free reads — a bare-identifier read and a
    /// `.signal` identity read of an already-bound handle. Do NOT widen this to
    /// any shape with potential effects (a call, an assignment, an index) or the
    /// discarded left operand would drop an observable side effect.
    pub(crate) fn instanceof_left_signal_proof(&self, left: LirNodeId) -> bool {
        let node = self.node(left);
        if node.children.is_empty() {
            if let Some(name) = node.text.as_deref().filter(|t| !t.is_empty()) {
                return self.is_abort_handle(name);
            }
            return false;
        }
        matches!(
            self.abort_member_read_parts(left),
            Some(AbortMemberRead::Signal(_))
        )
    }

    /// The single entry every abort receiver load flows through (mirrors
    /// `emit_growable_receiver_handle`): sets the position-allowlist flag,
    /// emits the receiver, restores the flag. Any abort-handle read NOT coming
    /// through here stays denied E5506 (default-deny).
    pub(crate) fn emit_abort_receiver_handle(
        &mut self,
        function: &mut Function,
        receiver: LirNodeId,
    ) -> EmittedValue {
        let previous = self.admit_abort_handle_read;
        self.admit_abort_handle_read = true;
        let value = self.emit_node(function, receiver, true);
        self.admit_abort_handle_read = previous;
        value
    }

    /// With the handle (i64) already on the stack: store `1` into the cell.
    pub(crate) fn emit_abort_cell_set(&mut self, function: &mut Function) {
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
    }

    /// With the handle (i64) already on the stack: load the aborted flag.
    /// Consumed by Task 4's `.aborted` read.
    pub(crate) fn emit_abort_cell_load(&mut self, function: &mut Function) {
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
    }

    /// Task 6 static-surface recognizer: any call whose callee is a member
    /// access — dot (`AbortSignal.timeout(...)`) OR computed
    /// (`AbortSignal["timeout"](...)`, `AbortSignal[k](...)`) — on the
    /// `AbortSignal` receiver, unshadowed (mirrors
    /// `instanceof_right_is_unshadowed`'s five-namespace guard — a user
    /// binding of `AbortSignal` takes the normal user-call lane, not this
    /// deny). Kali has no lowering for the real JS static surface
    /// (`AbortSignal.timeout`, `.abort`, `.any`); this keys ONLY on the
    /// receiver's identity, never the property/key shape, so it denies the
    /// whole receiver regardless of how the member is spelled — a per-shape
    /// or per-method denylist would leak the sibling shape (review Important
    /// finding: the original dot-only, 1-child-callee check let the
    /// structurally distinct 2-child computed-member callee bypass it
    /// entirely, silently succeeding through the generic warning-only
    /// "undefined call target" fallback).
    ///
    /// Two callee shapes reach here (both empirically verified — see the
    /// `computed_forin_object_access` doc comment for the same split):
    ///   * dot access lowers to a 1-child `Value` node whose `text` is the
    ///     property name, `children[0]` the receiver;
    ///   * computed access (`obj[expr]`) lowers to a 2-child `Value` node
    ///     `[receiver, key]` whose `text` is never a binary-operator token
    ///     (that's how `computed_forin_object_access` and the generic
    ///     2-child dispatch in `control_flow.rs` distinguish it from a
    ///     binary expression, which also lowers to a 2-child `Value`).
    /// Both shapes key the receiver at `children[0]`.
    pub(crate) fn is_abort_signal_static_call(&self, callee_node: &LirNode) -> bool {
        let receiver = match callee_node.children.len() {
            1 if callee_node.text.is_some() => callee_node.children[0],
            2 if !crate::lower::is_binary_operator_text(
                callee_node.text.as_deref().unwrap_or_default(),
            ) =>
            {
                callee_node.children[0]
            }
            _ => return false,
        };
        self.instanceof_right_is_unshadowed(receiver, "AbortSignal")
    }
}
