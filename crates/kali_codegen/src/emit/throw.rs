//! `throw` lowering: print a node-shaped message when the argument is
//! provable, then trap. kali has no exception machinery (try/catch stays
//! rejected), and an uncaught JS throw's observable behavior is exactly
//! "message on stderr, abort nonzero" — which is what unreachable + the
//! CLI's E4000 trap envelope produce.

use crate::*;

impl<'a> FunctionEmitter<'a> {
    /// Node-shaped message for a throw argument, when statically provable:
    /// `throw new Error("m")` → `Uncaught Error: m`; `throw "m"` → `Uncaught m`;
    /// anything else → None (caller prints the generic message).
    fn throw_message_text(&self, arg: LirNodeId) -> Option<String> {
        let arg = self.unwrap_transparent(arg);
        let node = self.node(arg);
        // `throw "m"`: string literal.
        if node.kind == LirNodeKind::Literal {
            let text = node.text.as_deref()?;
            let unquoted = text.trim_matches(|c| c == '"' || c == '\'');
            if unquoted.len() != text.len() {
                return Some(format!("Uncaught {unquoted}"));
            }
            return None;
        }
        // `throw new Error("m")`: a `NewExpr`-shaped node (LIR: `Value` with
        // no text, first child the callee) whose callee text is "Error" and
        // whose sole argument is a string literal. The recognizer stays
        // fail-closed (None) on any unexpected shape.
        let callee_is_error = node
            .children
            .first()
            .map(|&c| self.node(self.unwrap_transparent(c)).text.as_deref() == Some("Error"))
            .unwrap_or(false);
        if !callee_is_error {
            return None;
        }
        for &child in &node.children {
            let child_node = self.node(self.unwrap_transparent(child));
            if child_node.kind == LirNodeKind::Literal {
                if let Some(text) = child_node.text.as_deref() {
                    let unquoted = text.trim_matches(|c| c == '"' || c == '\'');
                    if unquoted.len() != text.len() {
                        return Some(format!("Uncaught Error: {unquoted}"));
                    }
                }
            }
        }
        None
    }

    pub(crate) fn emit_throw(&mut self, function: &mut Function, node: &LirNode) -> EmittedValue {
        let message = node
            .children
            .first()
            .and_then(|&arg| self.throw_message_text(arg))
            .unwrap_or_else(|| "Uncaught exception".to_string());
        // Print via console.error's host import: intern the message and push
        // its encoded string handle, then call the import directly — the
        // same shape `console.assert`'s synthesized-message fallback uses
        // (single i64 handle arg, no extra arity/handshake instructions).
        let (offset, len) = self.strings.intern(&message);
        function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
        function.instruction(&Instruction::Call(crate::CONSOLE_ERROR_IMPORT_INDEX));
        function.instruction(&Instruction::Unreachable);
        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }
}
