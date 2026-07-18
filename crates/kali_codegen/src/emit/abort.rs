//! Abort-handle emission (Stage P3): receiver-handle loads under the position
//! allowlist, and the abort-cell store/load idiom. The handle is an i64 pointer
//! to one 8-byte `__alloc_global` cell holding the aborted flag; controller and
//! signal are the same handle.

use crate::*;

impl<'a> FunctionEmitter<'a> {
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
    /// Consumed by Task 4's `.aborted` read; unused this task but part of the
    /// abort-cell idiom produced here.
    #[allow(dead_code)]
    pub(crate) fn emit_abort_cell_load(&mut self, function: &mut Function) {
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
    }
}
