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
}
