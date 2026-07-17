use crate::*;

/// Length source for [`FunctionEmitter::emit_array_allocation_with_len`]: either
/// a dynamically evaluated size-argument AST node (`new Array(n)`) or a
/// compile-time-known constant (array literals).
enum ArrayLen {
    Dynamic(Option<LirNodeId>),
    Static(usize),
}

impl<'a> FunctionEmitter<'a> {
    /// Emit `id` as a console-import argument: always leaves exactly one i64
    /// (tagged scalar or string handle) on the stack. Float-shaped values are
    /// stringified via the `float_to_string` host import — the i64 value
    /// domain has no float encoding, so passing a raw f64 would emit
    /// type-invalid wasm. Local and param reads emit shape `Unknown`, so the
    /// repr-based `is_float_valued` is consulted as well — the same signal
    /// the float-operand seams use. `is_float_valued` doesn't account for
    /// string concatenation (`"v: " + x` is float-valued on its right operand
    /// but string-shaped overall), so it is gated on `!is_string_valued(id)`
    /// — `emit_binary`'s string-concat path already converts any float
    /// operands internally and returns shape `String`.
    fn emit_console_argument(&mut self, function: &mut Function, id: LirNodeId) {
        if self.object_shape_of_node(id).is_some() {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "printing an object reference is unavailable in the current phase; print its fields instead"
                    .to_string(),
            ));
        }
        let emitted = self.emit_node(function, id, true);
        if !emitted.produced {
            function.instruction(&Instruction::I64Const(0));
            return;
        }
        if matches!(emitted.shape, ValueShape::Float)
            || (!self.is_string_valued(id) && self.is_float_valued(id))
        {
            function.instruction(&Instruction::Call(FLOAT_TO_STRING_IMPORT_INDEX));
        }
    }

    pub(crate) fn emit_call(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        node: &LirNode,
    ) -> EmittedValue {
        let Some(callee) = node.children.first().copied() else {
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        };

        let callee_node = self
            .resolve_bound_member_callable_node(callee)
            .map(|bound| self.node(bound).clone())
            .unwrap_or_else(|| self.node(callee).clone());
        if self.is_kali_test_call(&callee_node) {
            if let Some(callback_index) = self.kali_test_callback_index(node) {
                function.instruction(&Instruction::I32Const(callback_index as i32));
                // Trailing `env_ptr` arg (Stage C C3): the `current_env` active
                // at registration — the registering activation's env record when
                // `Kali.test` runs inside a capturing function, else 0. The host
                // restores it before invoking this callback so its captures
                // resolve (deferred-callback env threading).
                function.instruction(&Instruction::GlobalGet(self.current_env_global()));
                function.instruction(&Instruction::Call(TEST_REGISTER_IMPORT_INDEX));
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }

            // Stage-review IMPORTANT-2: a FUNCTION-VALUE callback that
            // `kali_test_callback_index` cannot resolve to a compiled function
            // — a parameter (`function suite(cb){ Kali.test("a", cb); }`), an
            // alias, or a call result — used to fall through here as a mere
            // WARNING with NO registration: the harness printed `ok 1` with
            // zero tests and exited 0, silently never running the callback
            // (base a57cd09d5 rejected the same capturing programs E5506 at
            // their capture sites). Default-deny those shapes: a bare
            // identifier or a call expression in callback position is a value
            // that WOULD have been the test body — fail closed E5506.
            let callback_is_unregisterable_value = node.children.get(2).is_some_and(|&cb| {
                let cb = self.unwrap_transparent(cb);
                let cb_node = self.node(cb);
                match cb_node.kind {
                    // A call RESULT in callback position (`Kali.test("a", make())`)
                    // is a function value this lane cannot register.
                    LirNodeKind::Call => true,
                    // Post-un-flatten: any `Value` in callback position that
                    // `kali_test_callback_index` did not resolve is a real
                    // value this lane cannot register — deny. A CHILDLESS Value
                    // is a bare identifier that resolved to nothing (a param,
                    // an undefined name); a Value WITH children is a
                    // member/index expression (`obj.m`, `arr[i]`) whose text is
                    // a property/element name that must NOT be resolved to an
                    // unrelated module function (I-4 — that ran the wrong
                    // function and printed a false `ok 1`). Either way it is
                    // unregisterable. (The pre-D3 flattened-arrow
                    // `Value("unknown")` placeholder no longer exists.)
                    LirNodeKind::Value => true,
                    // No-ops by construction: an inline function expression /
                    // declaration (`Instruction`) resolved far above via
                    // `kali_test_callback_index`; the remaining kinds are not
                    // value-producing callback shapes — they keep the
                    // pre-existing warning lane.
                    LirNodeKind::Program
                    | LirNodeKind::Block
                    | LirNodeKind::Instruction
                    | LirNodeKind::Branch
                    | LirNodeKind::Literal
                    | LirNodeKind::Unknown => false,
                }
            });
            if callback_is_unregisterable_value {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "`Kali.test(...)` requires a function callback resolvable to a compiled function at the registration site (a named function or an inline function expression); an unresolvable callback value (parameter, alias, or call result) would register nothing and the test would be silently dropped".to_string(),
                ));
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }

            self.diagnostics.push(Diagnostic::warning(
                e8::IR_UNREADABLE as u32,
                "`Kali.test(...)` requires a function callback lowered as an exported function",
            ));
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(surface) = self.scheduling_surface(&callee_node) {
            if let Some(result) = self.try_emit_scheduling_call(function, node, surface) {
                return result;
            }
        }

        // Stage D event lane: `t.addEventListener(type, cb)` /
        // `t.dispatchEvent(new CustomEvent(type))` where `t` has provable
        // `new EventTarget()` provenance. A member callee named
        // addEventListener/dispatchEvent with a receiver whose local is a
        // recorded event-target handle takes the real registration/dispatch
        // emit arm; anything else (unknown receiver, unstable provenance) falls
        // through to the pre-lane backstop unchanged (out-of-lane behavior
        // preserved this stage — spec §2 out-of-lane).
        if callee_node.text.as_deref() == Some("addEventListener")
            && !callee_node.children.is_empty()
        {
            if let Some(receiver) = self.event_target_receiver(&callee_node) {
                let receiver = receiver.to_string();
                return self.emit_event_listener_add(function, node, &receiver);
            }
            if self.member_receiver_is_free_identifier(&callee_node) {
                return self.deny_captured_event_receiver("addEventListener");
            }
        }
        if callee_node.text.as_deref() == Some("dispatchEvent") && !callee_node.children.is_empty()
        {
            if let Some(receiver) = self.event_target_receiver(&callee_node) {
                // Emit the real dispatch ONLY for a FULLY in-lane argument (an
                // inline `new CustomEvent(<string literal>)`). An out-of-lane
                // argument (a bound event, `new Event(...)`, a CustomEvent with
                // detail) on an in-lane receiver keeps PRE-LANE behavior (spec
                // §2 out-of-lane preservation): fall through to the backstop and
                // build — the web-baseline corpus registers an in-lane listener
                // and then dispatches such shapes, and must stay deployable. The
                // resulting silent-drop is the inventoried out-of-lane residual
                // (Stage P3), NOT a widening of the backstop.
                if node.children.len() == 2
                    && self
                        .event_dispatch_literal(self.node(node.children[1]))
                        .is_some()
                {
                    let receiver = receiver.to_string();
                    return self.emit_event_dispatch(function, node, &receiver);
                }
            } else if self.member_receiver_is_free_identifier(&callee_node) {
                return self.deny_captured_event_receiver("dispatchEvent");
            }
        }
        // Handle-escape discipline (spec §2.4): the add/dispatch arms above
        // consumed the two supported methods on an in-lane EventTarget receiver
        // (an add/dispatch that fell through did so only for an out-of-lane
        // ARGUMENT, which is deliberately PRESERVED on the backstop above — not
        // re-denied here). ANY OTHER method on such a receiver
        // (`removeEventListener`, etc.) has no lowering — fail closed rather than
        // let the backstop silently drop it (a `t.removeEventListener(...)` drop
        // makes a later dispatch diverge from node, which would no longer see the
        // removed listener).
        if !callee_node.children.is_empty()
            && !matches!(
                callee_node.text.as_deref(),
                Some("addEventListener") | Some("dispatchEvent")
            )
            && self.event_target_receiver(&callee_node).is_some()
        {
            let method = callee_node.text.as_deref().unwrap_or("<method>");
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "'{method}' is not supported on an EventTarget in the current phase; only \
                     addEventListener/dispatchEvent have a lowering, and any other method would \
                     be silently dropped"
                ),
            ));
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_kali_write_stdout_bytes_call(&callee_node) {
            let Some(index) = self.stdout_write_bytes_import_index else {
                // Mirror the sibling `Kali.test` gate above: push the diagnostic
                // and return without emitting any value. Emitting an
                // `I64Const(0)` here while claiming `produced: false` would
                // silently unbalance the value stack.
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Kali.writeStdoutBytes is unavailable under this backend".to_string(),
                ));
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };
            // First arg is the byte array; emit its handle (i64) and call the
            // host import. The call produces no value (statement position).
            // The recognizer only checks callee text + object, not arity, so a
            // zero-arg `Kali.writeStdoutBytes()` reaches here — guard the arg
            // fetch and reject it with a diagnostic rather than indexing out of
            // bounds. Mirror the None-index branch above: push nothing, return
            // `produced: false`.
            let Some(arg) = node.children.get(1).copied() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Kali.writeStdoutBytes requires exactly one array argument".to_string(),
                ));
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };
            let _ = self.emit_node(function, arg, true);
            function.instruction(&Instruction::Call(index));
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        // `Promise.resolve(v)` synchronously settles to `v`; a bare
        // `Promise.resolve()` settles to unit (no value). This recognizer makes the
        // await value-passthrough lane (Stage 3 Task 4) yield `v` for
        // `await Promise.resolve(v)` instead of the historical `0`. Not general
        // Promise/microtask semantics — those are Stage 7.
        if self.is_promise_resolve(&callee_node) {
            if let Some(arg) = node.children.get(1).copied() {
                return self.emit_node(function, arg, true);
            }
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        // structuredClone(arg): Stage P2 Lane 2b 3-way DEFAULT-DENY allowlist.
        //   (1) in-envelope object-shaped arg  → request the per-shape deep-clone
        //       synthetic and call it (deep copy; result carries the same shape).
        //   (2) zero-placeholder construct arg (`new Blob(...)`-style) → warn and
        //       keep today's placeholder-0 lowering (corpus builds stay green).
        //   (3) everything else (unproven / unsupported shape, wrong arity) →
        //       fail closed E5506 (never a silent shallow copy).
        if self.is_structured_clone_call(&callee_node) {
            let args: Vec<LirNodeId> = node.children[1..].to_vec();
            if args.len() != 1 {
                return self.deny_e5506(
                    function,
                    "structuredClone expects exactly one argument in the current phase",
                );
            }
            let arg = args[0];
            // Lane 1: in-envelope object (every field a scalar or GrowableArrayI64).
            if let Some(shape) = self.object_shape_of_node(arg) {
                if self.shape_is_clone_envelope(shape) {
                    let clone_name = crate::emit::clone::clone_shape_synthetic_name(shape);
                    // The collection scan (`collect_requested_clone_shapes`)
                    // requests a synthetic for EVERY envelope shape when a
                    // structuredClone call is present, so this resolves. Guard
                    // defensively anyway: a collection/dispatch divergence must
                    // fail closed, never index-panic or miscompile.
                    let Some(&idx) = self.functions.get(&clone_name) else {
                        return self.deny_e5506(
                            function,
                            "structuredClone target shape has no emitted clone synthetic (internal)",
                        );
                    };
                    // Push the source object's base pointer (i64), then deep-clone.
                    self.emit_node(function, arg, true);
                    function.instruction(&Instruction::Call(idx));
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }
                // Object-shaped but OUT of envelope (a String or nested-Object
                // field): the clone body would shallow-share it — fail closed.
                return self.deny_e5506(
                    function,
                    "structuredClone of an object with a non-clonable field is unavailable in the current phase (only number and i64-array fields are supported)",
                );
            }
            // Lane 2: a zero-placeholder construct (`new Blob(...)`) — no real
            // value to clone, but the corpus builds and dispatches these; warn
            // and keep the placeholder-0 lowering (mirrors the Stage D C-1
            // placeholder-construct allowance).
            if crate::lower::declarator_init_is_placeholder_construct(&self.program.nodes, arg) {
                self.diagnostics.push(Diagnostic::warning(
                    e8::UNIMPLEMENTED as u32,
                    "structuredClone of an unsupported construct is a no-op placeholder in the current phase".to_string(),
                ));
                let _ = self.emit_node(function, arg, true);
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }
            // Lane 3: unproven / unsupported argument shape → fail closed.
            return self.deny_e5506(
                function,
                "structuredClone argument of unproven or unsupported shape is unavailable in the current phase",
            );
        }

        let callee_name = callee_node.text.as_deref().unwrap_or_default();
        let resolved = self.functions.get(callee_name).copied();

        if self.is_console_assert(&callee_node) {
            let message_args: Vec<LirNodeId> = node.children.iter().skip(2).copied().collect();
            let Some(condition) = node.children.get(1).copied() else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::Else);
                let (offset, len) = self.strings.intern("Assertion failed");
                function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
                function.instruction(&Instruction::Call(CONSOLE_ERROR_IMPORT_INDEX));
                function.instruction(&Instruction::End);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            let condition_result = self.emit_node(function, condition, true);
            if !condition_result.produced {
                function.instruction(&Instruction::I64Const(0));
            }
            match condition_result.shape {
                ValueShape::Boolean => {
                    function.instruction(&Instruction::I32WrapI64);
                }
                ValueShape::Scalar | ValueShape::Unknown | ValueShape::String => {
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32Eqz);
                }
                ValueShape::Float => {
                    // f64 truthiness: nonzero is truthy. Leaves an i32 for `If`.
                    function.instruction(&Instruction::F64Const(0.0.into()));
                    function.instruction(&Instruction::F64Ne);
                }
            }
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            if !message_args.is_empty() {
                if let Some(rendered) = self.render_console_arguments(&message_args) {
                    let (offset, len) = self.strings.intern(&rendered);
                    let handle = encode_string_handle(offset, len);
                    function.instruction(&Instruction::I64Const(handle));
                    function.instruction(&Instruction::Call(CONSOLE_ERROR_IMPORT_INDEX));
                } else if let Some(first_arg) = message_args.first().copied() {
                    self.emit_console_argument(function, first_arg);
                    function.instruction(&Instruction::Call(CONSOLE_ERROR_IMPORT_INDEX));
                    for arg in message_args.iter().skip(1) {
                        let produced = self.emit_node(function, *arg, true);
                        if produced.produced {
                            function.instruction(&Instruction::Drop);
                        }
                    }
                }
            } else {
                let (offset, len) = self.strings.intern("Assertion failed");
                let handle = encode_string_handle(offset, len);
                function.instruction(&Instruction::I64Const(handle));
                function.instruction(&Instruction::Call(CONSOLE_ERROR_IMPORT_INDEX));
            }
            function.instruction(&Instruction::End);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(import_index) = self.console_import_index(&callee_node) {
            if let Some(rendered) = self.render_console_call(node) {
                let (offset, len) = self.strings.intern(&rendered);
                let handle = encode_string_handle(offset, len);
                function.instruction(&Instruction::I64Const(handle));
                function.instruction(&Instruction::Call(import_index));
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }

            // Stage 4 Task 6 re-review fix: the dynamic lane below emits ONLY
            // the first argument and drops the rest — a pre-existing lane
            // limitation. For a MULTI-argument call whose arguments read a
            // GROWABLE array (a lane new in this stage, so nothing green can
            // depend on it), that would be a brand-new silent divergence
            // (`console.log(o.length, o[0])` printed `2`, node `2 1`): fail
            // closed instead. Non-growable multi-arg calls keep the
            // pre-existing behavior byte-identically (documented follow-up in
            // the stage triage doc).
            if node.children.len() > 2
                && node
                    .children
                    .iter()
                    .skip(1)
                    .any(|arg| self.subtree_mentions_growable(*arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "console output of multiple arguments where one reads a growable array is \
                     unavailable in the current phase (the runtime console lane prints a single \
                     value); log the values separately or join them into one string"
                        .to_string(),
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }

            let mut args = node.children.iter().skip(1);
            if let Some(first_arg) = args.next() {
                self.emit_console_argument(function, *first_arg);
            } else {
                function.instruction(&Instruction::I64Const(0));
            }
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                let produced = self.emit_node(function, *arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
            }
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        // `receiver.fill(v)` on an array receiver: emit a repr-directed init loop
        // that writes `v` into every slot and leaves the array handle on the stack.
        if let Some((receiver, value)) = self.array_fill_call_parts(node) {
            let binding_name = self
                .assignment_target_name(node, receiver)
                .unwrap_or_default();
            return self.emit_array_fill(function, receiver, value, &binding_name);
        }

        // `<recv>.toFixed(<digits>)`: format a float as a fixed-decimal string via the
        // `kali:rt float_to_fixed` host import. The receiver is emitted as an f64
        // (promoting an integer-valued receiver), the digit count is a static integer
        // literal, and the result is a string handle.
        if let Some((receiver, digits)) = self.to_fixed_call_parts(node) {
            return self.emit_to_fixed(function, receiver, digits);
        }

        if self.is_object_freeze_call(node) {
            let mut args = node.children.iter().skip(1);
            let Some(value) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Object.freeze requires at least one argument in the current phase; use an explicit value or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            let emitted = self.emit_node(function, *value, true);
            for arg in args {
                let produced = self.emit_node(function, *arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
            }
            return emitted;
        }

        if let Some(result) = self.resolve_static_global_number_predicate_call(node, &callee_node) {
            function.instruction(&Instruction::I64Const(if result { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(result) = self.resolve_static_parse_int_call(node, &callee_node) {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_parse_float_call(node, &callee_node) {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_string_from_char_code_call(node, &callee_node) {
            let (offset, len) = self.strings.intern(&result);
            function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_array_some_every_call(node, "some") {
            function.instruction(&Instruction::I64Const(if result { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(result) = self.resolve_static_array_some_every_call(node, "every") {
            function.instruction(&Instruction::I64Const(if result { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(result) = self.resolve_static_array_find_call(node, "find") {
            return match result {
                StaticArraySearchResult::Value(value) => self.emit_node(function, value, true),
                StaticArraySearchResult::Index(index) => {
                    function.instruction(&Instruction::I64Const(index));
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    }
                }
            };
        }

        if let Some(result) = self.resolve_static_array_find_call(node, "findIndex") {
            return match result {
                StaticArraySearchResult::Value(value) => self.emit_node(function, value, true),
                StaticArraySearchResult::Index(index) => {
                    function.instruction(&Instruction::I64Const(index));
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    }
                }
            };
        }

        if let Some(result) = self.resolve_static_array_find_call(node, "findLast") {
            return match result {
                StaticArraySearchResult::Value(value) => self.emit_node(function, value, true),
                StaticArraySearchResult::Index(index) => {
                    function.instruction(&Instruction::I64Const(index));
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    }
                }
            };
        }

        if let Some(result) = self.resolve_static_array_find_call(node, "findLastIndex") {
            return match result {
                StaticArraySearchResult::Value(value) => self.emit_node(function, value, true),
                StaticArraySearchResult::Index(index) => {
                    function.instruction(&Instruction::I64Const(index));
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    }
                }
            };
        }

        if let Some(result) = self.resolve_static_array_search_call(node, "includes") {
            function.instruction(&Instruction::I64Const(if result >= 0 { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(result) = self.resolve_static_array_search_call(node, "indexOf") {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_array_search_call(node, "lastIndexOf") {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_string_search_call(node, "includes") {
            function.instruction(&Instruction::I64Const(if result >= 0 { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(result) = self.resolve_static_string_search_call(node, "indexOf") {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_string_search_call(node, "lastIndexOf") {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_string_search_call(node, "startsWith") {
            function.instruction(&Instruction::I64Const(if result >= 0 { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(result) = self.resolve_static_string_search_call(node, "endsWith") {
            function.instruction(&Instruction::I64Const(if result >= 0 { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(result) = self.resolve_static_string_identity_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_slice_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_substring_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some((receiver, start, end)) = self.runtime_substring_call_parts(node) {
            return self.emit_runtime_substring(function, receiver, start, end);
        }

        if let Some(result) = self.resolve_static_string_repeat_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_concat_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_pad_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_at_call(node) {
            let literal = match result {
                StaticStringAtResult::Value(value) => self.alloc_scratch_node(
                    LirNodeKind::Literal,
                    Some(quote_string_literal(&value)),
                    vec![],
                ),
                StaticStringAtResult::OutOfRange => self.alloc_scratch_node(
                    LirNodeKind::Literal,
                    Some("undefined".to_string()),
                    vec![],
                ),
            };
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_char_at_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_char_code_at_call(node) {
            let literal = self.alloc_scratch_node(LirNodeKind::Literal, Some(result), vec![]);
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_code_point_at_call(node) {
            let literal = match result {
                StaticStringAtResult::Value(value) => {
                    self.alloc_scratch_node(LirNodeKind::Literal, Some(value), vec![])
                }
                StaticStringAtResult::OutOfRange => self.alloc_scratch_node(
                    LirNodeKind::Literal,
                    Some("undefined".to_string()),
                    vec![],
                ),
            };
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_trim_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_case_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_normalize_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_replace_call(node, "replace") {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_replace_call(node, "replaceAll") {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(parts) = self.resolve_static_string_split_call(node) {
            let children = parts
                .into_iter()
                .map(|part| {
                    self.alloc_scratch_node(
                        LirNodeKind::Literal,
                        Some(quote_string_literal(&part)),
                        vec![],
                    )
                })
                .collect();
            let literal = self.alloc_scratch_node(LirNodeKind::Value, None, children);
            return self.emit_node(function, literal, true);
        }

        if let Some(method) = self.string_identity_call_method_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "String.prototype.{method} is unavailable with arguments in the current direct-runtime path; use a no-argument static string call or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_repeat_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.repeat is unavailable unless the receiver is a statically-known ASCII string literal and the repeat count is a statically-known integer from 0 through 1024 in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_concat_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.concat is unavailable unless the receiver and all operands are statically-known ASCII string literals in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(method) = self.string_pad_call_method(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "String.prototype.{method} is unavailable unless the receiver is a statically-known ASCII string literal, the target length is a statically-known integer from 0 through 1024, and the optional pad string is statically-known ASCII in the current direct-runtime path; use explicit ASCII literals or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_at_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.at is unavailable unless the receiver is a statically-known ASCII string literal and the optional index is a statically-known integer in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_char_at_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.charAt is unavailable unless the receiver is a statically-known ASCII string literal and the optional index is a statically-known integer in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_char_code_at_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.charCodeAt is unavailable unless the receiver is a statically-known ASCII string literal and the optional index is a statically-known integer in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_code_point_at_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.codePointAt is unavailable unless the receiver is a statically-known ASCII string literal and the optional index is a statically-known integer in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(method) = self.string_case_call_method(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "String.prototype.{method} is unavailable unless the receiver is a statically-known ASCII string literal and no arguments are supplied in the current direct-runtime path; use explicit ASCII literals or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_normalize_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.normalize is unavailable unless the receiver is a statically-known ASCII string literal and the optional normalization form is one of the statically-known strings NFC, NFD, NFKC, or NFKD in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(method) = self.string_replace_call_method_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "String.prototype.{method} is unavailable unless the receiver, search value, and replacement are statically-known ASCII string literals and the replacement contains no substitution markers in the current direct-runtime path; use explicit ASCII literals or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_split_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.split is unavailable unless the receiver is a statically-known ASCII string literal, the optional separator is a statically-known ASCII string literal, and the optional limit is a statically-known integer from 0 through 1024 in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(result) = self.resolve_static_array_at_call(node) {
            match result {
                StaticArrayAtResult::Value(value) => return self.emit_node(function, value, true),
                StaticArrayAtResult::OutOfRange => {
                    let undefined = self.alloc_scratch_node(
                        LirNodeKind::Literal,
                        Some("undefined".to_string()),
                        vec![],
                    );
                    return self.emit_node(function, undefined, true);
                }
            }
        }

        // Growable runtime array `.push` (throw-fallout Stage 4): a
        // bare-identifier receiver recognized by the growable both-sides
        // oracle appends for real (geometric growth) and yields the new
        // length — the miscompile this stage kills was exactly this call
        // falling through to the generic drop-args no-op.
        if let Some((receiver, args)) = self.growable_push_call_parts(node) {
            return self.emit_growable_push_call(function, receiver, &args);
        }

        if let Some(result) = self.resolve_static_array_join_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        // Runtime join over a proven String-element array binding (Spec 3).
        // Placed AFTER the static fold lane so literal receivers keep folding;
        // the recognizer's `array_bindings` check already makes them disjoint.
        if let Some((receiver, separator)) = self.runtime_join_call_parts(node) {
            return self.emit_runtime_join(function, id, receiver, separator);
        }

        // Fail-closed backstop (throw-fallout Stage 2, Lane C review): a `.join`
        // whose receiver is an object-enumeration call
        // (`Object.values(r).join(...)`) that the `kali_optimize` fold DECLINED
        // to fold — because `r` is mutated (a block-nested/computed store) or
        // its shape is otherwise not compile-time-fixed — reaches here with no
        // static-array fold and no runtime-array-binding lowering. The generic
        // fallthrough below would emit a silent `0` (the `.join` twin of the
        // `keys.length`/`keys[0]` miscompile). `kali_types`'
        // `is_static_array_iteration_target` optimistically accepts such a
        // receiver at check time, trusting a fold that the optimizer soundly
        // declines — a hand-mirror break. Reject with the SAME enumeration
        // E5506 the `Object.values(r)[idx]` path takes (call.rs generic
        // fallback). Foldable enumerations are already replaced by array
        // literals before this point, so this only fires on genuinely-unfoldable
        // receivers.
        if self.is_object_enumeration_join_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Object enumeration is only supported where the object has a compile-time-known fixed shape".to_string(),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(result) = self.resolve_static_array_to_string_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if self.is_array_at_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Array.prototype.at is unavailable unless the receiver is a statically-known array literal and the index is a statically-known in-range integer in the current phase; use explicit constants or the later compatibility path".to_string(),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(result) = self.resolve_static_array_reduce_call(node, "reduce") {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_array_reduce_call(node, "reduceRight") {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if self.is_array_is_array_call(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(argument) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Array.isArray requires at least one statically-known argument in the current phase; use an explicit literal or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            if let Some(result) = self.static_array_is_array_result(*argument) {
                for arg in args {
                    let produced = self.emit_node(function, *arg, true);
                    if produced.produced {
                        function.instruction(&Instruction::Drop);
                    }
                }
                function.instruction(&Instruction::I64Const(if result { 1 } else { 0 }));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                };
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Array.isArray is unavailable unless the argument is a statically-known array, object, or primitive literal in the current phase; use explicit literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_object_has_own_call(node, &callee_node) {
            let Some(object_id) = node.children.get(1).copied() else {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };
            let Some(key_id) = node.children.get(2).copied() else {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };
            let Some(key) = self.render_static_value(key_id) else {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            let Some(has_own) = self.static_object_has_own(object_id, &key) else {
                // Placeholder backstop: `Object.hasOwn` folds only when the
                // receiver's own-key set is statically provable (object literal,
                // `Object.fromEntries` of literal entries, or a materialized
                // fixed-shape object — e.g. a quoted-key object since Lane A).
                // An unprovable receiver (`Object.hasOwn(globalThis, "a")`, a
                // dynamic namespace) previously returned `produced: false` while
                // emitting NOTHING: harmless in statement position but a stack
                // underflow in a VALUE position (inside a `!`/`||` condition),
                // which produced an invalid module. Emit a DEFINED placeholder
                // (`0`) so the stack shape is always valid — dropped in
                // statement position, a `false` in value position — instead of
                // either the old stack-corrupting silence or a hard reject that
                // would break the permissive statement-position dynamic-helper
                // probes (throw-fallout Stage 2 checkpoint regression fix).
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                };
            };

            function.instruction(&Instruction::I64Const(if has_own { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if self.is_object_identity_object(&callee_node) && callee_node.text.as_deref() == Some("is")
        {
            let mut args = node.children.iter().skip(1);
            let Some(left) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Object.is requires at least two statically-known primitive literal arguments or the same statically-known reference in the current phase; use explicit constants or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };
            let Some(right) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Object.is requires at least two statically-known primitive literal arguments or the same statically-known reference in the current phase; use explicit constants or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            let left_value = self.resolve_static_object_identity_value(*left);
            let right_value = self.resolve_static_object_identity_value(*right);
            if let (Some(left_value), Some(right_value)) = (left_value, right_value) {
                let same_value = left_value.same_value(&right_value);

                for arg in args {
                    let produced = self.emit_node(function, *arg, true);
                    if produced.produced {
                        function.instruction(&Instruction::Drop);
                    }
                }

                function.instruction(&Instruction::I64Const(if same_value { 1 } else { 0 }));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                };
            }

            if let (Some(left_reference), Some(right_reference)) = (
                self.resolve_static_reference_root_name(*left),
                self.resolve_static_reference_root_name(*right),
            ) {
                if left_reference == right_reference {
                    for arg in args {
                        let produced = self.emit_node(function, *arg, true);
                        if produced.produced {
                            function.instruction(&Instruction::Drop);
                        }
                    }

                    function.instruction(&Instruction::I64Const(1));
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Boolean,
                    };
                }
            }

            if let (Some(left_ref), Some(right_ref)) = (
                self.resolve_literal_aggregate(*left),
                self.resolve_literal_aggregate(*right),
            ) {
                for arg in args {
                    let produced = self.emit_node(function, *arg, true);
                    if produced.produced {
                        function.instruction(&Instruction::Drop);
                    }
                }

                function.instruction(&Instruction::I64Const(if left_ref == right_ref {
                    1
                } else {
                    0
                }));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                };
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Object.is is unavailable unless both arguments are statically-known primitive literals or the same statically-known reference in the current phase; use explicit constants or the later compatibility path".to_string(),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_number_object(&callee_node)
            && matches!(
                callee_node.text.as_deref(),
                Some("isFinite") | Some("isNaN") | Some("isInteger") | Some("isSafeInteger")
            )
        {
            let method = callee_node.text.as_deref().unwrap_or("isFinite");
            let mut args = node.children.iter().skip(1);
            let Some(value) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Number.{method} requires at least one statically-known primitive value in the current phase; use an explicit constant or the later compatibility path"
                    ),
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            let Some(value) = self.resolve_static_object_identity_value(*value) else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Number.{method} is unavailable unless the argument is a statically-known primitive value in the current phase; use an explicit constant or the later compatibility path"
                    ),
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            let supported = match value {
                StaticObjectIdentityValue::Number(number) => match method {
                    "isFinite" => number.is_finite(),
                    "isNaN" => number.is_nan(),
                    "isInteger" => number.is_finite() && number.fract() == 0.0,
                    "isSafeInteger" => {
                        number.is_finite()
                            && number.fract() == 0.0
                            && number.abs() <= 9007199254740991.0
                    }
                    _ => false,
                },
                _ => false,
            };

            for arg in args {
                let produced = self.emit_node(function, *arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
            }

            function.instruction(&Instruction::I64Const(if supported { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(import_index) = self.math_max_import_index(&callee_node) {
            let args: Vec<_> = node.children.iter().skip(1).copied().collect();
            let Some(first_arg) = args.first() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.max requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            if let Some(folded) = self.math_extrema_static_literal_value("max", &args) {
                function.instruction(&Instruction::I64Const(folded));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *first_arg, "max") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            for arg in args.iter().skip(1) {
                if !self.emit_integer_math_arg(function, *arg, "max") {
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                function.instruction(&Instruction::Call(import_index));
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.math_min_import_index(&callee_node) {
            let args: Vec<_> = node.children.iter().skip(1).copied().collect();
            let Some(first_arg) = args.first() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.min requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            if let Some(folded) = self.math_extrema_static_literal_value("min", &args) {
                function.instruction(&Instruction::I64Const(folded));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *first_arg, "min") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            for arg in args.iter().skip(1) {
                if !self.emit_integer_math_arg(function, *arg, "min") {
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                function.instruction(&Instruction::Call(import_index));
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.math_abs_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(first_arg) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.abs requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            if let Some(folded) = self.math_abs_static_literal_value(*first_arg) {
                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *first_arg, "abs") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                if !self.emit_integer_math_arg(function, *arg, "abs") {
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                function.instruction(&Instruction::Drop);
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.math_sign_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(first_arg) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.sign requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            if let Some(folded) = self.math_sign_static_literal_value(*first_arg) {
                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *first_arg, "sign") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                if !self.emit_integer_math_arg(function, *arg, "sign") {
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                function.instruction(&Instruction::Drop);
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.math_imul_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(left) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            };
            let Some(right) = args.next() else {
                let produced = self.emit_node(function, *left, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
                for arg in args {
                    let produced = self.emit_node(function, *arg, true);
                    if produced.produced {
                        function.instruction(&Instruction::Drop);
                    }
                }
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            };

            if let Some(folded) = self.math_imul_static_literal_value(*left, *right) {
                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *left, "imul") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            if !self.emit_integer_math_arg(function, *right, "imul") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                if !self.emit_integer_math_arg(function, *arg, "imul") {
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                function.instruction(&Instruction::Drop);
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.math_round_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(value) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.round requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            if let Some(folded) = self.math_round_like_static_literal_value("round", *value) {
                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *value, "round") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                if !self.emit_integer_math_arg(function, *arg, "round") {
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                function.instruction(&Instruction::Drop);
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.math_clz32_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(value) = args.next() else {
                function.instruction(&Instruction::I64Const(32));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            };

            if let Some(folded) = self.math_clz32_static_literal_value(*value) {
                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *value, "clz32") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                if !self.emit_integer_math_arg(function, *arg, "clz32") {
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                function.instruction(&Instruction::Drop);
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if self.math_pow_import_index(&callee_node).is_some() {
            let operands: Vec<_> = node.children.iter().skip(1).copied().collect();
            return self.emit_exponentiation_expression(function, &operands, "Math.pow");
        }

        if matches!(
            callee_node.text.as_deref(),
            Some("floor") | Some("trunc") | Some("ceil")
        ) && self.is_math_object(&callee_node)
        {
            let method = callee_node.text.as_deref().unwrap_or("floor").to_string();
            let mut args = node.children.iter().skip(1);
            let Some(value) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            };

            if let Some(folded) = self.math_round_like_static_literal_value(method.as_str(), *value)
            {
                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *value, method.as_str()) {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            for arg in args {
                if !self.emit_integer_math_arg(function, *arg, method.as_str()) {
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                function.instruction(&Instruction::Drop);
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(method) = self.math_member_method(&callee_node) {
            if method == "hypot" {
                let args: Vec<_> = node.children.iter().skip(1).copied().collect();
                let Some(root) = self.math_hypot_constant_root(&args) else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "Math.hypot is unavailable unless every argument is a statically-known integer literal whose squared sum is a perfect-square integer literal in the current phase; use explicit constants or the later compatibility path",
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(root));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "atan2" {
                let args: Vec<_> = node.children.iter().skip(1).copied().collect();
                let Some(left) = args.first().copied() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "Math.atan2 is unavailable unless the first argument is a statically-known zero numeric literal and the second argument is a statically-known non-negative numeric literal in the current phase; use explicit constants or the later compatibility path".to_string(),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };
                let Some(right) = args.get(1).copied() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "Math.atan2 is unavailable unless the first argument is a statically-known zero numeric literal and the second argument is a statically-known non-negative numeric literal in the current phase; use explicit constants or the later compatibility path".to_string(),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let Some(folded) = self.math_atan2_zero_slice_value(left, right) else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "Math.atan2 is unavailable unless the first argument is a statically-known zero numeric literal and the second argument is a statically-known non-negative numeric literal in the current phase; use explicit constants or the later compatibility path".to_string(),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args.iter().skip(2).copied() {
                    let _ = self.emit_node(function, arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "exp" || method == "log" || method == "exp2" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let folded = match method {
                    "exp" => self.math_exp_constant_value(*value),
                    "log" => self.math_log_constant_value(*value),
                    "exp2" => self.math_exp2_constant_value(*value),
                    _ => unreachable!(),
                };
                let Some(folded) = folded else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                            if method == "log" {
                                "one"
                            } else if method == "exp2" {
                                "non-negative integer literal within the current integer-fold range"
                            } else {
                                "zero"
                            }
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "expm1" || method == "log1p" || method == "fround" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let folded = match method {
                    "expm1" => self.math_expm1_constant_value(*value),
                    "log1p" => self.math_log1p_constant_value(*value),
                    "fround" => self.math_fround_zero_constant_value(*value),
                    _ => unreachable!(),
                };
                let Some(folded) = folded else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "asinh" || method == "acosh" || method == "atanh" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let folded = self.math_inverse_hyperbolic_constant_value(method, *value);
                let Some(folded) = folded else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                            if method == "acosh" { "one" } else { "zero" }
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "sinh" || method == "cosh" || method == "tanh" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let folded = self.math_hyperbolic_zero_constant_value(method, *value);
                let Some(folded) = folded else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "sin" || method == "cos" || method == "tan" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let folded = self.math_sin_cos_zero_constant_value(method, *value);
                let Some(folded) = folded else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "asin" || method == "acos" || method == "atan" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let folded = self.math_inverse_trig_constant_value(method, *value);
                let Some(folded) = folded else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                            if method == "acos" { "one" } else { "zero" }
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "sin" || method == "cos" || method == "tan" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let folded = self.math_sin_cos_zero_constant_value(method, *value);
                let Some(folded) = folded else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "log2" || method == "log10" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known {} integer literal in the current phase; use an explicit constant or the later compatibility path",
                            if method == "log2" { "positive power-of-two" } else { "positive power-of-ten" }
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let literal_root = if method == "log2" {
                    self.math_log2_constant_exponent(*value)
                } else {
                    self.math_log10_constant_exponent(*value)
                };
                let Some(exponent) = literal_root else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known {} integer literal in the current phase; use an explicit constant or the later compatibility path",
                            if method == "log2" { "positive power-of-two" } else { "positive power-of-ten" }
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(exponent));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "sqrt" || method == "cbrt" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known {} integer literal in the current phase; use an explicit constant or the later compatibility path",
                            if method == "sqrt" { "perfect-square" } else { "perfect-cube" }
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let root = if method == "sqrt" {
                    self.math_sqrt_constant_root(*value)
                } else {
                    self.math_cbrt_constant_root(*value)
                };
                if let Some(root) = root {
                    function.instruction(&Instruction::I64Const(root));
                    for arg in args {
                        let _ = self.emit_node(function, *arg, true);
                        function.instruction(&Instruction::Drop);
                    }
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }

                if method == "sqrt" {
                    // Runtime sqrt: emit the argument as f64, then F64Sqrt.
                    let arg_id = *value;
                    self.emit_node(function, arg_id, true);
                    if !self.is_float_valued(arg_id) {
                        function.instruction(&Instruction::F64ConvertI64S);
                    }
                    function.instruction(&Instruction::F64Sqrt);
                    for arg in args {
                        let _ = self.emit_node(function, *arg, true);
                        function.instruction(&Instruction::Drop);
                    }
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Float,
                    };
                }

                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known {} integer literal in the current phase; use an explicit constant or the later compatibility path",
                        if method == "sqrt" { "perfect-square" } else { "perfect-cube" }
                    ),
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }

            if !matches!(
                method,
                "max"
                    | "min"
                    | "abs"
                    | "sign"
                    | "imul"
                    | "round"
                    | "clz32"
                    | "pow"
                    | "trunc"
                    | "floor"
                    | "tan"
                    | "asin"
                    | "acos"
                    | "atan"
                    | "asinh"
                    | "acosh"
                    | "atanh"
            ) {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable in the current phase; use a supported Math builtin or the later compatibility path"
                    ),
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
        }

        if let Some(import_index) = self.cwd_import_index(&callee_node) {
            let env_buffer_offset = 0i32;
            let env_buffer_capacity = ENV_GET_BUFFER_RESERVED as i32;
            function.instruction(&Instruction::I32Const(0));
            function.instruction(&Instruction::I32Const(0));
            function.instruction(&Instruction::I32Const(env_buffer_offset));
            function.instruction(&Instruction::I32Const(env_buffer_capacity));
            function.instruction(&Instruction::Call(import_index));
            function.instruction(&Instruction::I64ExtendI32U);
            let temp_local = self.locals.len() as u32;
            function.instruction(&Instruction::LocalTee(temp_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(temp_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Const(STRING_HANDLE_TAG as i64));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::End);
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.env_set_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(key_expr) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };
            let Some(value_expr) = args.next() else {
                self.push_placeholder_fallback_diagnostic("call target", "Deno.env.set");
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let Some(key_text) = self.render_static_value(*key_expr) else {
                self.push_placeholder_fallback_diagnostic("call target", "Deno.env.set");
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };
            let Some(value_text) = self.render_static_value(*value_expr) else {
                self.push_placeholder_fallback_diagnostic("call target", "Deno.env.set");
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let (key_offset, key_len) = self.strings.intern(&key_text);
            let (value_offset, value_len) = self.strings.intern(&value_text);
            function.instruction(&Instruction::I32Const(key_offset as i32));
            function.instruction(&Instruction::I32Const(key_len as i32));
            function.instruction(&Instruction::I32Const(value_offset as i32));
            function.instruction(&Instruction::I32Const(value_len as i32));
            function.instruction(&Instruction::Call(import_index));
            function.instruction(&Instruction::Drop);
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(import_index) = self.env_delete_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(key_expr) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let Some(key_text) = self.render_static_value(*key_expr) else {
                self.push_placeholder_fallback_diagnostic("call target", "Deno.env.delete");
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let (key_offset, key_len) = self.strings.intern(&key_text);
            function.instruction(&Instruction::I32Const(key_offset as i32));
            function.instruction(&Instruction::I32Const(key_len as i32));
            function.instruction(&Instruction::I32Const(0));
            function.instruction(&Instruction::I32Const(0));
            function.instruction(&Instruction::Call(import_index));
            function.instruction(&Instruction::Drop);
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(import_index) = self.env_get_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(key_expr) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let Some(key_text) = self.render_static_value(*key_expr) else {
                self.push_placeholder_fallback_diagnostic("call target", "Deno.env.get");
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let (key_offset, key_len) = self.strings.intern(&key_text);
            let env_buffer_offset = 0i32;
            let env_buffer_capacity = ENV_GET_BUFFER_RESERVED as i32;
            function.instruction(&Instruction::I32Const(key_offset as i32));
            function.instruction(&Instruction::I32Const(key_len as i32));
            function.instruction(&Instruction::I32Const(env_buffer_offset));
            function.instruction(&Instruction::I32Const(env_buffer_capacity));
            function.instruction(&Instruction::Call(import_index));
            function.instruction(&Instruction::I64ExtendI32U);
            let temp_local = self.locals.len() as u32;
            function.instruction(&Instruction::LocalTee(temp_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(temp_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Const(STRING_HANDLE_TAG as i64));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::End);
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.env_has_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(key_expr) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let Some(key_text) = self.render_static_value(*key_expr) else {
                self.push_placeholder_fallback_diagnostic("call target", "Deno.env.has");
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let (key_offset, key_len) = self.strings.intern(&key_text);
            function.instruction(&Instruction::I32Const(key_offset as i32));
            function.instruction(&Instruction::I32Const(key_len as i32));
            function.instruction(&Instruction::Call(import_index));
            function.instruction(&Instruction::I64ExtendI32U);
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(import_index) = self.cwd_set_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(path_expr) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let _ = self.emit_node(function, *path_expr, true);
            function.instruction(&Instruction::Call(import_index));
            function.instruction(&Instruction::Drop);
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(import_index) = self.process_exit_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            if let Some(code_expr) = args.next() {
                let _ = self.emit_node(function, *code_expr, true);
            } else {
                function.instruction(&Instruction::I64Const(0));
            }
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            function.instruction(&Instruction::Call(import_index));
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(import_index) = self.performance_now_import_index(&callee_node) {
            // `performance.now()` takes no arguments; any argument is an
            // unsupported shape (symmetric with the kali_types admission arm).
            if node.children.len() > 1 {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "performance.now() does not accept arguments in the current phase".to_string(),
                ));
                function.instruction(&Instruction::F64Const(0.0.into()));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Float,
                };
            }
            function.instruction(&Instruction::Call(import_index));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Float,
            };
        }

        if let Some(import_index) = self.crypto_get_random_values_import_index(&callee_node) {
            // `crypto.getRandomValues(buf)`: the argument is an i64 array handle
            // whose low 32 bits are the linear-memory base (i64 length header at
            // `+0`, element bytes at `+8`). Decode it to `(ptr = base + 8, len =
            // length header)`, fill the buffer IN PLACE via the host, and
            // re-produce the ORIGINAL handle (JS `getRandomValues` returns the
            // same buffer it was given). Mirrors the args_get buffer ptr/len
            // split; symmetric with the kali_types admission arm.
            let mut args = node.children.iter().skip(1);
            let Some(buf_expr) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "crypto.getRandomValues requires a typed-array buffer argument in the current phase"
                        .to_string(),
                ));
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            };
            // General-purpose i64 scratch local (see the `+ 2` extra-locals count
            // in `lower.rs`): holds the buffer handle so it is evaluated once and
            // reused for the ptr, the length load, and the returned value.
            let handle_local = self.locals.len() as u32;
            let produced = self.emit_node(function, *buf_expr, true);
            if !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            }
            function.instruction(&Instruction::LocalSet(handle_local));
            // ptr = (handle low 32 bits) + 8 — skip the i64 length header.
            function.instruction(&Instruction::LocalGet(handle_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Const(8));
            function.instruction(&Instruction::I32Add);
            // len = length header (i64 @ +0 of the base) wrapped to i32.
            function.instruction(&Instruction::LocalGet(handle_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I64Load(MemArg {
                offset: 0,
                align: 3,
                memory_index: 0,
            }));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::Call(import_index));
            // Discard the returned byte count; the fixture wants the buffer back.
            function.instruction(&Instruction::Drop);
            // Any extra arguments are an unsupported shape; evaluate + drop for
            // side effects (the kali_types arm rejects them with a diagnostic).
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            function.instruction(&Instruction::LocalGet(handle_local));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.crypto_random_uuid_import_index(&callee_node) {
            // `crypto.randomUUID()`: allocate a fixed-cap NEVER-reset buffer (so
            // the string handle it backs safely outlives any arena reset), have
            // the host write the UUID's UTF-8 bytes into it, and build a tagged
            // string handle from `(buf, returned byte length)`. Mirrors the argv
            // string-handle construction (`operators.rs`).
            if node.children.len() > 1 {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "crypto.randomUUID() does not accept arguments in the current phase"
                        .to_string(),
                ));
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::String,
                };
            }
            // Canonical UUID is 36 bytes; round up to a comfortable cap. The host
            // returns -1 (via write_guest_bytes) if the UUID overflows `cap`.
            const UUID_BUF_CAP: i32 = 40;
            // Two i64 scratch locals (general-purpose + array-alloc slot): hold
            // the buffer base and the returned byte length. Both are i64-typed,
            // so the i32 alloc result / byte count are extended on store.
            let buf_local = self.locals.len() as u32;
            let len_local = buf_local + 1;
            // buf = __alloc_global(CAP)
            function.instruction(&Instruction::I32Const(UUID_BUF_CAP));
            function.instruction(&Instruction::Call(self.alloc_global_fn_index()));
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(buf_local));
            // len = crypto_random_uuid(buf, CAP)
            function.instruction(&Instruction::LocalGet(buf_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Const(UUID_BUF_CAP));
            function.instruction(&Instruction::Call(import_index));
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(len_local));
            // handle = STRING_HANDLE_TAG | (buf << 32) | len
            function.instruction(&Instruction::I64Const(STRING_HANDLE_TAG as i64));
            function.instruction(&Instruction::LocalGet(buf_local));
            function.instruction(&Instruction::I64Const(32));
            function.instruction(&Instruction::I64Shl);
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalGet(len_local));
            function.instruction(&Instruction::I64Or);
            return EmittedValue {
                produced: true,
                shape: ValueShape::String,
            };
        }

        if let Some(import_index) = self.crypto_subtle_digest_import_index(&callee_node) {
            // `crypto.subtle.digest(algo, bytes)`: `algo` is a string handle (a
            // string literal) and `bytes` is the CONTIGUOUS byte buffer produced
            // by `new TextEncoder().encode(<string>)` — itself a reinterpret of a
            // string handle, so BOTH operands are `STRING_HANDLE_TAG | (buf << 32)
            // | len` handles whose `(buf, len)` name contiguous UTF-8 bytes. Decode
            // each to a `(ptr, len)` i32 pair, allocate a NEVER-reset output buffer
            // (max digest = 64 bytes for SHA-512), have the host write the raw
            // digest bytes and return the length, then build a tagged handle from
            // `(out_buf, returned_len)` so `.byteLength` reads the digest length
            // off the low 32 bits (see the string-valued `.byteLength` arm).
            let mut args = node.children.iter().skip(1);
            let (Some(&algo_expr), Some(&input_expr)) = (args.next(), args.next()) else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "crypto.subtle.digest requires an algorithm name and an input buffer in the current phase"
                        .to_string(),
                ));
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            };
            if args.next().is_some() {
                // Extra arguments are an unsupported shape. The kali_types arm
                // rejects the same arity set symmetrically (both sides reject ONLY
                // on arity — missing / extra args — never on operand string-ness, so
                // a runtime/imported-call input like `encode(describe(count))` falls
                // through here and lowers as a reinterpret, matching the pre-recognizer
                // placeholder that deployed). Once the structural recognizer + arity
                // match, this arm assumes string-handle operands (their `(buf, len)`
                // names contiguous UTF-8 bytes by provenance).
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "crypto.subtle.digest only accepts a string algorithm name and a TextEncoder().encode(<string>) buffer in the current phase"
                        .to_string(),
                ));
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }
            // Two i64 scratch locals (general-purpose + array-alloc slot):
            // `handle_local` holds each string handle in turn (algo, then input,
            // then the returned digest length), `buf_local` holds the output
            // buffer base.
            let handle_local = self.locals.len() as u32;
            let buf_local = handle_local + 1;
            const DIGEST_BUF_CAP: i32 = 64;
            // --- algo (algo_ptr, algo_len) ---
            let produced = self.emit_node(function, algo_expr, true);
            if !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            }
            function.instruction(&Instruction::LocalSet(handle_local));
            self.emit_string_handle_ptr(function, handle_local);
            self.emit_string_handle_len(function, handle_local);
            // --- input (in_ptr, in_len) ---
            let produced = self.emit_node(function, input_expr, true);
            if !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            }
            function.instruction(&Instruction::LocalSet(handle_local));
            self.emit_string_handle_ptr(function, handle_local);
            self.emit_string_handle_len(function, handle_local);
            // --- output buffer (out_ptr, out_cap) ---
            function.instruction(&Instruction::I32Const(DIGEST_BUF_CAP));
            function.instruction(&Instruction::Call(self.alloc_global_fn_index()));
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(buf_local));
            function.instruction(&Instruction::LocalGet(buf_local));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::I32Const(DIGEST_BUF_CAP));
            // len = crypto_subtle_digest(algo_ptr, algo_len, in_ptr, in_len, out_ptr, out_cap)
            function.instruction(&Instruction::Call(import_index));
            function.instruction(&Instruction::I64ExtendI32U);
            function.instruction(&Instruction::LocalSet(handle_local));
            // handle = STRING_HANDLE_TAG | (out_buf << 32) | len
            function.instruction(&Instruction::I64Const(STRING_HANDLE_TAG as i64));
            function.instruction(&Instruction::LocalGet(buf_local));
            function.instruction(&Instruction::I64Const(32));
            function.instruction(&Instruction::I64Shl);
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalGet(handle_local));
            function.instruction(&Instruction::I64Or);
            return EmittedValue {
                produced: true,
                shape: ValueShape::String,
            };
        }

        if self.is_text_encoder_encode(&callee_node) {
            // `new TextEncoder().encode(<string>)`: a thin reinterpret. A kali
            // string is ALREADY a tagged CONTIGUOUS byte-buffer handle
            // (`STRING_HANDLE_TAG | (buf << 32) | len`, `len` UTF-8 bytes at
            // `buf`), so the encoded byte buffer IS the argument's string handle —
            // its `(buf, len)` already names the contiguous UTF-8 bytes and
            // `.byteLength` reads the same low-32 byte count as `.length`. This
            // deliberately does NOT route through `new Uint8Array(n)` (i64-stride
            // elements — not contiguous bytes the digest host can read).
            let mut args = node.children.iter().skip(1);
            let Some(&input_expr) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "TextEncoder().encode requires a string argument in the current phase"
                        .to_string(),
                ));
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::String,
                };
            };
            if args.next().is_some() {
                // Extra arguments are an unsupported shape; the kali_types arm
                // rejects the same arity mismatch symmetrically (both sides reject
                // ONLY on arity, never on operand string-ness — a runtime/imported
                // input falls through and reinterprets, as the pre-recognizer
                // placeholder did), so this passthrough assumes a string-handle operand.
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "TextEncoder().encode only accepts a single string argument in the current phase"
                        .to_string(),
                ));
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::String,
                };
            }
            let produced = self.emit_node(function, input_expr, true);
            if !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::String,
            };
        }

        if self.is_process_kill(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(pid_expr) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    kali_common::process_kill_zero_probe_unavailable_message(),
                ));
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };
            let Some(pid_text) = self.render_static_value(*pid_expr) else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    kali_common::process_kill_zero_probe_unavailable_message(),
                ));
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };
            if parse_number_literal(&pid_text) != Some(0) || args.next().is_some() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    kali_common::process_kill_zero_probe_unavailable_message(),
                ));
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            }
            function.instruction(&Instruction::I64Const(1));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        for (arg_index, arg) in node.children.iter().skip(1).enumerate() {
            // An UNMATERIALIZED array literal (a direct `f([1, 2])` argument,
            // or a fold-lane `const arr = [1, 2]` alias) has no runtime
            // representation: `emit_aggregate_literal` pushes a zero
            // placeholder, so every element read in the callee silently
            // yielded 0 (`g([1, 2]) { return items[0] }` → 0, node says 1).
            // Materialized arrays (`new Array(n)`, `.fill()`, object-element
            // literals) live in locals — NOT the fold-lane `bindings` map that
            // `resolve_literal_aggregate` follows — and pass a real handle,
            // so they are untouched here. REJECT-DON'T-MISCOMPILE.
            if resolved.is_some() {
                // Shape-strict: `new X(1)` and `[X, 1]` are the SAME LIR shape
                // (textless Value), so `is_array_literal` alone would also
                // catch NewExpr nodes (observed: the release pipeline leaves
                // `new Array(3)` unrecognized and this reject misfired on it).
                // Requiring every element to be a Literal keeps the reject on
                // the proven array-literal class (`f([1, 2])`) and leaves
                // call-shaped nodes on their pre-existing lanes.
                let fold_lane_array = self
                    .resolve_literal_aggregate(*arg)
                    .map(|id| self.node(id).clone())
                    .is_some_and(|aggregate| {
                        self.is_array_literal(&aggregate)
                            && !aggregate.children.is_empty()
                            && aggregate.children.iter().all(|&child| {
                                self.node(self.unwrap_transparent(child)).kind
                                    == LirNodeKind::Literal
                            })
                    });
                if fold_lane_array {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "passing an array literal to function '{callee_name}' is unavailable in the current direct-runtime path (the callee would read zero placeholders, not the elements); allocate with `new Array(n)` and assign elements instead"
                        ),
                    ));
                    function.instruction(&Instruction::I64Const(0));
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Unknown,
                    };
                }
            }
            let produced = self.emit_node(function, *arg, true);
            // A function-valued argument (e.g. an arrow, compiled as a
            // standalone function and skipped by `is_function_like` here)
            // produces no stack value; pad with a zero placeholder so the call
            // arity — and the fallback per-argument `Drop` loop — stay valid.
            if !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            }
            // Promote an integer-valued argument to `f64` when the resolved callee
            // declares that parameter as float, so the pushed value matches the
            // callee's f64 param slot. No-op for integer callees (param defaults to
            // I64), keeping integer call sites byte-identical.
            if resolved.is_some()
                && self.repr_table.param(callee_name, arg_index) == kali_common::Repr::F64
                && !self.is_float_valued(*arg)
            {
                function.instruction(&Instruction::F64ConvertI64S);
            }
        }

        if let Some(index) = resolved {
            let shape = if self.repr_table.return_repr(callee_name) == kali_common::Repr::F64 {
                ValueShape::Float
            } else {
                ValueShape::Unknown
            };
            function.instruction(&Instruction::Call(index));
            return EmittedValue {
                produced: true,
                shape,
            };
        }

        if self.has_semver_import() {
            if let Some(rendered) = self.render_semver_intrinsic(callee_name, node) {
                for _ in node.children.iter().skip(1) {
                    function.instruction(&Instruction::Drop);
                }
                if rendered == "0" || rendered == "1" {
                    let value = rendered.parse::<i64>().unwrap_or(0);
                    function.instruction(&Instruction::I64Const(value));
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Boolean,
                    };
                }
                let (offset, len) = self.strings.intern(&rendered);
                function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }
        }

        // Fail-closed backstop (throw-fallout Stage 2 Lane D): an
        // `Object.keys`/`Object.values`/`Object.entries`/`Reflect.ownKeys`
        // call that reaches this generic placeholder fallback means the
        // `kali_optimize` constant fold declined to fold it (unknown-shape
        // operand, or a `__proto__`-keyed literal the fold refuses to
        // enumerate — see the carve-out in `object_fold.rs`). The generic
        // fallback below is a WARNING-level zero placeholder that lets the
        // build exit 0 with a silently wrong value — exactly the
        // `keys.length`/`keys[0]` miscompile this lane exists to close.
        // Reject with a hard error instead of ever emitting the phantom
        // zero placeholder for a call this codebase recognizes as object
        // enumeration.
        if self.is_object_enumeration_call(node).is_some() {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Object enumeration is only supported where the object has a compile-time-known fixed shape".to_string(),
            ));
            for _ in node.children.iter().skip(1) {
                function.instruction(&Instruction::Drop);
            }
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }

        // Stage C fail-closed (triage Concern 2, DEFAULT-DENY per the stage
        // review's IMPORTANT-1): codegen emits no call to `queueMicrotask`/
        // `setTimeout`/`setInterval`/`addEventListener` (Task 6: the host
        // imports exist but no generated module imports them), so the call
        // lowers through this generic zero-placeholder fallback and the
        // scheduled callback is silently dropped. Dropping a CAPTURING callback
        // also drops its captured environment — the exact shape base
        // `a57cd09d5` REJECTED (E5506) at the capture-write/read. The original
        // guard denylisted "provably capturing" arguments and default-ALLOWED
        // everything it could not resolve — live fail-opens: an ALIAS
        // (`let cb2 = cb; setTimeout(cb2, 0)`), a CALL RESULT
        // (`setTimeout(makeCb(), 0)`), and a REASSIGNED binding with stale
        // declarator provenance all silently dropped. Inverted to an
        // allowlist at this single choke point (all four surfaces converge
        // here): E5506 UNLESS every argument is PROVABLY safe — a non-callable
        // literal/number, or a callback whose closure plan resolves through
        // stable provenance to an empty `captured` set. Module-scope and
        // non-capturing callbacks (silently dropped at base too — the
        // bg1-bg3 boundary pins) still resolve and keep running; `Kali.test`
        // — the one codegen-emitted deferred surface that DOES thread
        // `env_ptr` — returned far above and never reaches here.
        if self.is_undrained_scheduling_surface(&callee_node)
            && !self.scheduling_call_args_provably_safe(node)
        {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "a callback passed to '{callee_name}' is unavailable unless it is provably non-capturing: codegen emits no call to this scheduling surface, so the callback — and any captured environment — would be silently dropped; an argument with unresolvable provenance fails closed (event/timer-lowering follow-up)"
                ),
            ));
            for _ in node.children.iter().skip(1) {
                function.instruction(&Instruction::Drop);
            }
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }

        self.push_placeholder_fallback_diagnostic("call target", callee_name);
        for _ in node.children.iter().skip(1) {
            function.instruction(&Instruction::Drop);
        }
        function.instruction(&Instruction::I64Const(0));
        EmittedValue {
            produced: true,
            shape: ValueShape::Unknown,
        }
    }

    /// Stage D scheduling-surface registration emit. Returns `None` only for
    /// surfaces this task has not wired yet (they fall through to the Stage C
    /// default-deny guard below); `Some` is a fully-handled call.
    fn try_emit_scheduling_call(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        surface: SchedulingSurface,
    ) -> Option<EmittedValue> {
        match surface {
            SchedulingSurface::QueueMicrotask => {
                Some(self.emit_queue_microtask_call(function, node))
            }
            SchedulingSurface::SetTimeout | SchedulingSurface::SetInterval => {
                Some(self.emit_timer_set_call(function, node, surface))
            }
            SchedulingSurface::ClearTimeout | SchedulingSurface::ClearInterval => {
                Some(self.emit_timer_clear_call(function, node, surface))
            }
        }
    }

    fn emit_timer_set_call(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        surface: SchedulingSurface,
    ) -> EmittedValue {
        let surface_name = if surface == SchedulingSurface::SetTimeout {
            "setTimeout"
        } else {
            "setInterval"
        };
        let fail_closed = |this: &mut Self, message: String| {
            this.diagnostics
                .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
            EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            }
        };
        // node.children: [callee, callback, optional delay]; anything else —
        // including node's arg-forwarding form setTimeout(fn, 0, x) — fails
        // closed (spec §5).
        if node.children.len() < 2 || node.children.len() > 3 {
            return fail_closed(
                self,
                format!("{surface_name} supports exactly (callback[, delay]) in the current phase; extra arguments have no forwarding lane"),
            );
        }
        // Delay: a NUMERIC LITERAL or absent (spec envelope: provably-numeric
        // only; widening to proven-scalar bindings is precision follow-up
        // work). Clamping below 1 happens host-side (node parity).
        let delay: i32 = match node.children.get(2) {
            None => 0,
            Some(&d) => {
                let d = self.unwrap_transparent(d);
                let d_node = self.node(d);
                let literal = d_node
                    .children
                    .is_empty()
                    .then_some(d_node.text.as_deref())
                    .flatten()
                    .and_then(parse_numeric_literal_value);
                match literal {
                    Some(value) => value.max(0.0).min(i32::MAX as f64) as i32,
                    None => {
                        return fail_closed(
                            self,
                            format!("a {surface_name} delay must be a numeric literal in the current phase (a computed delay has no provably-numeric lowering yet)"),
                        )
                    }
                }
            }
        };
        let import = if surface == SchedulingSurface::SetTimeout {
            self.set_timeout_import_index
        } else {
            self.set_interval_import_index
        };
        let Some(import) = import else {
            return fail_closed(
                self,
                format!("{surface_name} import unavailable (probe/emit desync)"),
            );
        };
        match self.scheduling_callback(node) {
            SchedulingCallback::Resolved(index) => {
                function.instruction(&Instruction::I32Const(index as i32));
                function.instruction(&Instruction::I32Const(delay));
                function.instruction(&Instruction::GlobalGet(self.current_env_global()));
                function.instruction(&Instruction::Call(import));
                // The host returns the i32 timer id; kali scalars are i64.
                function.instruction(&Instruction::I64ExtendI32S);
                EmittedValue { produced: true, shape: ValueShape::Unknown }
            }
            SchedulingCallback::Deny => fail_closed(
                self,
                format!("a {surface_name} callback must resolve through stable provenance to a compiled function; an unresolvable callback would be silently dropped"),
            ),
            SchedulingCallback::DenyUnloweredCapture(class) => {
                self.deny_deferred_unlowered_capture(class)
            }
        }
    }

    fn emit_timer_clear_call(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        surface: SchedulingSurface,
    ) -> EmittedValue {
        let surface_name = if surface == SchedulingSurface::ClearTimeout {
            "clearTimeout"
        } else {
            "clearInterval"
        };
        if node.children.len() != 2 {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!("{surface_name} requires exactly one timer-id argument"),
            ));
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }
        let import = if surface == SchedulingSurface::ClearTimeout {
            self.clear_timeout_import_index
        } else {
            self.clear_interval_import_index
        };
        let Some(import) = import else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!("{surface_name} import unavailable (probe/emit desync)"),
            ));
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        };
        // Any i64 value is SOUND as a cancel id: the host no-ops an unknown/
        // already-fired id (node parity), so no provenance proof is needed.
        let emitted = self.emit_node(function, node.children[1], true);
        if !emitted.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Call(import));
        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }

    fn emit_queue_microtask_call(
        &mut self,
        function: &mut Function,
        node: &LirNode,
    ) -> EmittedValue {
        // node.children: [callee, callback]; extra args fail closed (node
        // ignores them, but silently divergent arity is exactly the class
        // this stage exists to eliminate — precision follow-up).
        if node.children.len() != 2 {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "queueMicrotask requires exactly one callback argument in the current phase"
                    .to_string(),
            ));
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }
        match self.scheduling_callback(node) {
            SchedulingCallback::Resolved(index) => {
                let Some(import) = self.queue_microtask_import_index else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "queueMicrotask import unavailable (probe/emit desync)".to_string(),
                    ));
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };
                function.instruction(&Instruction::I32Const(index as i32));
                // env_ptr: the `current_env` active at the registration site
                // (the C3 test_register pattern); the host restores it before
                // invoking the callback, and env_safety's registration edges
                // prove it is the capture owner's record (or E5506).
                function.instruction(&Instruction::GlobalGet(self.current_env_global()));
                function.instruction(&Instruction::Call(import));
                EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                }
            }
            SchedulingCallback::Deny => {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "a queueMicrotask callback must resolve through stable provenance to a compiled function (an inline function expression, a declarator-recorded function local, or an unshadowed function name); an unresolvable callback would be silently dropped".to_string(),
                ));
                EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                }
            }
            SchedulingCallback::DenyUnloweredCapture(class) => {
                self.deny_deferred_unlowered_capture(class)
            }
        }
    }

    /// True when a member call's receiver (`callee_node.children[0]`) is a bare
    /// identifier NOT bound in the current function (neither a local nor a
    /// module binding) — i.e. a free/captured variable. The construction lane
    /// only ever records an EventTarget handle into a CURRENT-function local, so
    /// a free receiver can never be a provable in-lane handle here; if it IS one
    /// (an event target constructed in an enclosing scope and captured), the
    /// backstop would SILENTLY DROP the registration/dispatch — a proven
    /// divergence from node (the listener never fires). Fail closed instead.
    fn member_receiver_is_free_identifier(&self, callee_node: &LirNode) -> bool {
        let Some(&receiver) = callee_node.children.first() else {
            return false;
        };
        let receiver_node = self.node(receiver);
        if !receiver_node.children.is_empty() {
            return false;
        }
        let Some(name) = receiver_node.text.as_deref() else {
            return false;
        };
        !self.locals.contains_key(name) && !self.module_binding_names.contains(name)
    }

    /// Shared deny for a deferred callback whose env plan carries a captured
    /// binding OUTSIDE the deferred-lane safe class (Task 9 C-1 final,
    /// DEFAULT-DENY). Every registration surface routes its
    /// `SchedulingCallback::DenyUnloweredCapture(class)` here, so the wording is
    /// identical across setTimeout/setInterval/queueMicrotask/addEventListener by
    /// construction.
    fn deny_deferred_unlowered_capture(&mut self, class: &str) -> EmittedValue {
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "a captured {class} binding without closure lowering would read a placeholder \
                 when the deferred callback fires (its owner's frame and arena are gone) — fails \
                 closed; only a by-value scalar capture or a zero-placeholder construct is \
                 restorable in the current phase (closure-lowering follow-up)"
            ),
        ));
        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }

    fn deny_captured_event_receiver(&mut self, surface: &str) -> EmittedValue {
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "a '{surface}' receiver must be a locally-bound EventTarget in the current phase; \
                 a captured or free receiver has no provable provenance and its \
                 registration/dispatch would be silently dropped"
            ),
        ));
        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }

    /// Load a recorded EventTarget handle onto the stack (i64) by reading its
    /// promoted local DIRECTLY — the construction lane stored the handle with a
    /// `LocalSet` into `self.locals[name]`, so the receiver load mirrors it with
    /// a `LocalGet`. This BYPASSES the generic identifier lane on purpose: that
    /// lane's handle-escape choke point denies every bare read of an
    /// `event_target_locals` name (total by construction), and this direct load
    /// is the single allowed consumer. `event_target_receiver` already proved
    /// the name is a recorded handle; a missing local is a provisioning desync,
    /// not a user error — fail closed.
    fn emit_event_target_handle_load(&mut self, function: &mut Function, receiver: &str) -> bool {
        let Some(&index) = self.locals.get(receiver) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!("EventTarget binding `{receiver}` must be declared with a local slot"),
            ));
            return false;
        };
        function.instruction(&Instruction::LocalGet(index));
        true
    }

    fn emit_event_listener_add(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        receiver: &str,
    ) -> EmittedValue {
        let fail_closed = |this: &mut Self, message: String| {
            this.diagnostics
                .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
            EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            }
        };
        // node.children: [callee, name, callback] — exactly 2 args (an
        // options/once/capture third argument has no lowering — spec §2.2).
        if node.children.len() != 3 {
            return fail_closed(
                self,
                "addEventListener supports exactly (type, listener) in the current phase; \
                 options have no lowering"
                    .to_string(),
            );
        }
        // Event type: a string literal only.
        let Some(event_type) = self.string_literal_text(node.children[1]) else {
            return fail_closed(
                self,
                "an addEventListener event type must be a string literal in the current phase"
                    .to_string(),
            );
        };
        // Callback: the scheduling-lane provenance resolver (member layout puts
        // the callback at children[2]); its default-deny tail applies identically.
        let callback_index = match self.scheduling_callback_at(node, 2) {
            SchedulingCallback::Resolved(index) => index,
            SchedulingCallback::Deny => {
                return fail_closed(
                    self,
                    "an addEventListener listener must resolve through stable provenance to a \
                     compiled function; an unresolvable listener would be silently dropped"
                        .to_string(),
                )
            }
            SchedulingCallback::DenyUnloweredCapture(class) => {
                return self.deny_deferred_unlowered_capture(class)
            }
        };
        // Zero-parameter listeners only (no Event-object repr yet — spec §2.2).
        if self.function_param_count_by_index(callback_index) != Some(0) {
            return fail_closed(
                self,
                "an addEventListener listener must declare zero parameters in the current \
                 phase (there is no Event-object lowering yet)"
                    .to_string(),
            );
        }
        let Some(import) = self.event_listener_add_import_index else {
            return fail_closed(
                self,
                "event_listener_add import unavailable (probe/emit desync)".to_string(),
            );
        };
        if !self.emit_event_target_handle_load(function, receiver) {
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }
        let (offset, len) = self.strings.intern(&event_type);
        function.instruction(&Instruction::I32Const(offset as i32));
        function.instruction(&Instruction::I32Const(len as i32));
        function.instruction(&Instruction::I32Const(callback_index as i32));
        function.instruction(&Instruction::GlobalGet(self.current_env_global()));
        function.instruction(&Instruction::Call(import));
        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }

    fn emit_event_dispatch(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        receiver: &str,
    ) -> EmittedValue {
        let fail_closed = |this: &mut Self, message: String| {
            this.diagnostics
                .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
            EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            }
        };
        // node.children: [callee, event] — exactly 1 arg, an INLINE
        // `new CustomEvent(<string literal>)` (spec §2.3). The argument is
        // inspected RAW (never `unwrap_transparent`): the New node is itself a
        // text-less single-child `Value` wrapper that `event_dispatch_literal`
        // descends, and unwrapping would strip it.
        if node.children.len() != 2 {
            return fail_closed(
                self,
                "dispatchEvent takes exactly one event argument".to_string(),
            );
        }
        let Some(event_type) = self.event_dispatch_literal(self.node(node.children[1])) else {
            return fail_closed(
                self,
                "a dispatchEvent argument must be an inline `new CustomEvent(<string literal>)` \
                 in the current phase (bound events, detail, and options have no lowering)"
                    .to_string(),
            );
        };
        let Some(import) = self.event_dispatch_import_index else {
            return fail_closed(
                self,
                "event_dispatch import unavailable (probe/emit desync)".to_string(),
            );
        };
        if !self.emit_event_target_handle_load(function, receiver) {
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }
        let (offset, len) = self.strings.intern(&event_type);
        function.instruction(&Instruction::I32Const(offset as i32));
        function.instruction(&Instruction::I32Const(len as i32));
        function.instruction(&Instruction::Call(import));
        // Host returns i32 1 (true); kali booleans/scalars are i64.
        function.instruction(&Instruction::I64ExtendI32S);
        EmittedValue {
            produced: true,
            shape: ValueShape::Unknown,
        }
    }

    /// True when `callee_node` names a host scheduling surface whose call
    /// codegen does NOT emit: `addEventListener` (a method). This host import
    /// exists (`lower.rs` import table) but no generated module imports it,
    /// so such a call reaches the generic zero-placeholder fallback and its
    /// callback is dropped. `queueMicrotask` was removed from this set in
    /// Stage D task D2 (registration emit arm `emit_queue_microtask_call`);
    /// `setTimeout` / `setInterval` (and their `clearTimeout` / `clearInterval`
    /// counterparts) were removed in Stage D task D2's timer-lane follow-up
    /// (`emit_timer_set_call` / `emit_timer_clear_call`, both dispatched from
    /// `try_emit_scheduling_call` far above) — every bare-unshadowed call to
    /// any of the four now resolves through its own registration/cancel emit
    /// arm and never reaches this guard; a shadowed name takes the normal
    /// user-call lane and is likewise not a scheduling surface. This now
    /// enumerates the exact still-un-emittable scheduling family —
    /// `addEventListener` alone — a closed set, not an open-ended sink
    /// denylist, and is the single recognizer the Concern-2 guard keys on.
    fn is_undrained_scheduling_surface(&self, callee_node: &LirNode) -> bool {
        matches!(callee_node.text.as_deref(), Some("addEventListener"))
    }

    /// DEFAULT-DENY provenance check for the un-emittable scheduling surfaces
    /// (stage-review IMPORTANT-1): true only when EVERY argument of `node` is
    /// PROVABLY safe to drop —
    /// - a literal (string/number/boolean) or a bare numeric constant: not a
    ///   callable, dropping it drops no callback;
    /// - a DIRECT inline function expression (`setTimeout(function(){…})`):
    ///   its node text is the `__kali_fn_N` plan key (`name_anon_functions`
    ///   renamed it in place) — safe iff its plan's `captured` is empty;
    /// - an identifier with STABLE provenance resolving to a non-capturing
    ///   closure plan: `fn_valued_locals` (declarator provenance) first — a
    ///   local binding SHADOWS any same-named module function — then a bare
    ///   module-level function name. A name that is reassigned or re-declared
    ///   anywhere in this function (`unstable_provenance_names`) is NEVER
    ///   resolved: the declarator-time mapping can be stale (the reassignment
    ///   fail-open the stage review's tripwire pinned).
    ///
    /// Everything else — aliases of aliases (`let cb2 = cb`), call results
    /// (`setTimeout(makeCb(), 0)`), member/element reads, params, locals with
    /// non-function declarators — is unresolvable and fails closed (the caller
    /// emits E5506). Module-global captures are NOT env-cell captures
    /// (`derive_env_plans` excludes them), so a module-scope callback has an
    /// empty `captured` and still resolves as safe (the bg1 boundary pin).
    fn scheduling_call_args_provably_safe(&self, node: &LirNode) -> bool {
        let non_capturing_plan = |key: &str| {
            self.env_plans
                .get(key)
                .is_some_and(|plan| plan.captured.is_empty())
        };
        node.children.iter().skip(1).all(|&arg| {
            let arg = self.unwrap_transparent(arg);
            let arg_node = self.node(arg);
            let text = arg_node.text.as_deref().unwrap_or_default();
            match arg_node.kind {
                // Literals can never be a callable — dropping one drops nothing.
                LirNodeKind::Literal => true,
                // Inline function expression / declaration used as a value:
                // its text is its `__kali_fn_N` / declared plan key.
                LirNodeKind::Instruction => non_capturing_plan(text),
                LirNodeKind::Value => {
                    if !arg_node.children.is_empty() {
                        // Member/element/operator expression: unresolvable.
                        return false;
                    }
                    if parse_numeric_literal_value(text).is_some() {
                        return true;
                    }
                    if self.unstable_provenance_names.contains(text) {
                        return false;
                    }
                    if let Some(key) = self.fn_valued_locals.get(text) {
                        return non_capturing_plan(key);
                    }
                    if self.locals.contains_key(text)
                        || self.bindings.contains_key(text)
                        || self.module_binding_names.contains(text)
                    {
                        // A local/const/module binding whose declarator did NOT
                        // record function provenance: value unknown — fail
                        // closed.
                        return false;
                    }
                    if self.env_plans.contains_key(text) {
                        // A bare (unshadowed) function name.
                        return non_capturing_plan(text);
                    }
                    // Post-un-flatten (Stage D Task 7): every arrow is a real
                    // compiled function, so an identifier resolving to NOTHING
                    // in any codegen namespace is a genuinely unresolvable
                    // value — deny. (The `Value("unknown")` flattened
                    // block-arrow argument lane is deleted, so this can no
                    // longer be a placeholder whose callback never existed.)
                    false
                }
                // No-ops by construction: a call argument is an expression —
                // a call RESULT (`makeCb()`), or a malformed/statement-shaped
                // node, has no resolvable callback provenance. Fail closed.
                LirNodeKind::Program
                | LirNodeKind::Block
                | LirNodeKind::Branch
                | LirNodeKind::Call
                | LirNodeKind::Unknown => false,
            }
        })
    }

    /// If `id` (after unwrapping transparent value wrappers) is a `new Array(n)` /
    /// `Array(n)` allocation call, returns `Some(size_arg)` where `size_arg` is the
    /// length argument node (or `None` for the zero-length `new Array()` form).
    pub(crate) fn resolve_array_alloc_call(&self, id: LirNodeId) -> Option<Option<LirNodeId>> {
        let target = self.unwrap_transparent_value_node(id);
        let node = self.node(target);
        if node.kind != LirNodeKind::Call || node.children.len() > 2 {
            return None;
        }
        let callee = node.children.first().copied()?;
        let callee_node = self.node(callee);
        if !self.is_array_like_constructor(callee_node) {
            return None;
        }
        Some(node.children.get(1).copied())
    }

    /// Push the i32 linear-memory pointer of a tagged string handle held in
    /// `handle_local` (an i64 local): `((handle >> 32) & 0x7fff_ffff) as i32`.
    /// Matches the host's `read_guest_string_handle` decode (offset = bits
    /// 32..=62). Shared by the `crypto.subtle.digest` operand decode.
    fn emit_string_handle_ptr(&self, function: &mut Function, handle_local: u32) {
        function.instruction(&Instruction::LocalGet(handle_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(0x7fff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I32WrapI64);
    }

    /// Push the i32 byte length of a tagged string handle held in `handle_local`
    /// (an i64 local): `(handle & 0xffff_ffff) as i32` (the low-32 byte count,
    /// same field `.length` reads). Shared by the `crypto.subtle.digest` operand
    /// decode.
    fn emit_string_handle_len(&self, function: &mut Function, handle_local: u32) {
        function.instruction(&Instruction::LocalGet(handle_local));
        function.instruction(&Instruction::I64Const(0xffff_ffff));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I32WrapI64);
    }

    /// `Array(n)` / `new Array(n)` (bare `Array`), or the `Uint8Array`
    /// typed-array constructor in bare (`new Uint8Array(n)`) or
    /// `globalThis["Uint8Array"]` form (throw-fallout Stage 3 bucket #6). A
    /// `Uint8Array(n)` lowers to the same i64-element linear-memory array as
    /// `Array(n)`: the crypto fixtures only observe `.length` / `.byteLength`
    /// (both `== n`, read from the i64 length header) and hand the buffer to
    /// `crypto.getRandomValues`, which fills it in place via the host; no
    /// per-element byte addressing is exercised.
    fn is_array_like_constructor(&self, callee_node: &LirNode) -> bool {
        match callee_node.text.as_deref() {
            Some("Array") => callee_node.children.is_empty(),
            Some("Uint8Array") => {
                callee_node.children.is_empty()
                    || callee_node
                        .children
                        .first()
                        .is_some_and(|&obj| self.node(obj).text.as_deref() == Some("globalThis"))
            }
            _ => false,
        }
    }

    /// If `id` (after unwrapping transparent value wrappers) is a bare identifier
    /// reference, returns its text. Mirrors the bare-identifier arm shared by
    /// `is_string_valued`/`is_float_valued` (`LirNodeKind::Value` with no children).
    pub(crate) fn bare_identifier_name(&self, id: LirNodeId) -> Option<String> {
        let target = self.unwrap_transparent_value_node(id);
        let node = self.node(target);
        if node.kind == LirNodeKind::Value && node.children.is_empty() {
            node.text.clone()
        } else {
            None
        }
    }

    /// Bump-allocate an array of `size_arg` i64 elements in linear memory, storing the
    /// length at `+0`, advancing the `__heap` global, and leaving the i64 base handle on
    /// the stack. Layout: `[ length:i64 @ +0 ][ elem0 @ +8 ][ elem1 @ +16 ]…`.
    pub(crate) fn emit_array_allocation(
        &mut self,
        function: &mut Function,
        size_arg: Option<LirNodeId>,
    ) -> EmittedValue {
        self.emit_array_allocation_with_len(function, ArrayLen::Dynamic(size_arg))
    }

    /// Bump-allocate an array of statically-known length (array literals), leaving
    /// the i64 base handle on the stack. Same layout as [`Self::emit_array_allocation`].
    pub(crate) fn emit_array_allocation_static(
        &mut self,
        function: &mut Function,
        len: usize,
    ) -> EmittedValue {
        self.emit_array_allocation_with_len(function, ArrayLen::Static(len))
    }

    /// Shared body for [`Self::emit_array_allocation`] and
    /// [`Self::emit_array_allocation_static`]: bump-allocates an array in linear
    /// memory, storing the length at `+0`, advancing the `__heap` global, and
    /// leaving the i64 base handle on the stack. Layout:
    /// `[ length:i64 @ +0 ][ elem0 @ +8 ][ elem1 @ +16 ]…`. The two callers differ
    /// only in how the length value is produced — a dynamically evaluated AST
    /// node (`new Array(n)`) or a compile-time constant (array literals) — which
    /// `ArrayLen` captures so the rest of the emission (length-header store,
    /// `__heap` advance, handle push) stays byte-identical between the two paths.
    fn emit_array_allocation_with_len(
        &mut self,
        function: &mut Function,
        len: ArrayLen,
    ) -> EmittedValue {
        let scratch = self.locals.len() as u32;
        // Second scratch slot (see the `+ 2` extra-locals count in `lower.rs`): holds the
        // evaluated size argument so its AST node is emitted exactly once, then reused for
        // both the length-header store and the `(n+1)*8` byte-count math. Evaluating it
        // once also avoids a double-evaluation of any side effect in the size expression
        // and avoids re-emitting into the `scratch` slot in between the two uses below.
        let size_scratch = scratch + 1;

        // size = evaluated size argument (emitted exactly once) or a constant.
        match len {
            ArrayLen::Dynamic(size_arg) => self.emit_array_length_value(function, size_arg),
            ArrayLen::Static(len) => {
                function.instruction(&Instruction::I64Const(len as i64));
            }
        }
        function.instruction(&Instruction::LocalSet(size_scratch));

        // base = __alloc((length + 1) * 8) — same total-byte-count math the old
        // inline `__heap` bump used (`base + (length + 1) * 8`), just handed to
        // the shared allocator as its argument instead of computed against a
        // pinned-before-size-evaluation `base` local. All existing call sites
        // evaluate a pure-arithmetic size argument (no allocation side effects),
        // so this reordering is behavior-preserving for every current caller.
        function.instruction(&Instruction::LocalGet(size_scratch));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Call(self.alloc_callee_index()));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(scratch));

        // mem[base + 0] = length
        function.instruction(&Instruction::LocalGet(scratch));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(size_scratch));
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));

        // Result: the i64 base handle.
        function.instruction(&Instruction::LocalGet(scratch));
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    fn emit_array_length_value(&mut self, function: &mut Function, size_arg: Option<LirNodeId>) {
        match size_arg {
            Some(arg) => {
                let produced = self.emit_node(function, arg, true);
                if !produced.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
            }
            None => {
                function.instruction(&Instruction::I64Const(0));
            }
        }
    }

    /// If `id` (after unwrapping transparent value wrappers) is a `receiver.fill(v)`
    /// member call whose receiver is an array handle — either a fresh `new Array(n)`
    /// allocation or an existing array binding — returns `(receiver_id, value_id)`.
    /// Non-array receivers (so `.fill` on other objects is left untouched) and the
    /// zero-argument `fill()` form return `None`.
    pub(crate) fn resolve_array_fill_call(&self, id: LirNodeId) -> Option<(LirNodeId, LirNodeId)> {
        let target = self.unwrap_transparent_value_node(id);
        self.array_fill_call_parts(self.node(target))
    }

    /// Node-based counterpart to [`resolve_array_fill_call`], for callers (e.g.
    /// `emit_call`) that already hold the call node.
    pub(crate) fn array_fill_call_parts(&self, node: &LirNode) -> Option<(LirNodeId, LirNodeId)> {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return None;
        }
        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("fill") {
            return None;
        }
        let receiver = callee_node.children.first().copied()?;
        let receiver_is_array = self.resolve_array_alloc_call(receiver).is_some()
            || self
                .assignment_target_name(node, receiver)
                .is_some_and(|name| self.array_bindings.contains(&name));
        if !receiver_is_array {
            return None;
        }
        Some((receiver, node.children[1]))
    }

    /// Recognize a `<recv>.toFixed(<digits>)` member call, returning the receiver and
    /// the digit-count argument node ids. Requires exactly one argument (the digit
    /// count); the receiver is `toFixed`'s member base.
    pub(crate) fn to_fixed_call_parts(&self, node: &LirNode) -> Option<(LirNodeId, LirNodeId)> {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return None;
        }
        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("toFixed") {
            return None;
        }
        let receiver = callee_node.children.first().copied()?;
        Some((receiver, node.children[1]))
    }

    /// Recognizes a RUNTIME `x.substring(a?, b?)` member call: Call node whose
    /// callee is a member node with text "substring" and a string-valued
    /// receiver. Returns (receiver, start_arg, end_arg). Static-foldable
    /// slices are handled by `resolve_static_string_substring_call` FIRST and
    /// never reach this.
    pub(crate) fn runtime_substring_call_parts(
        &self,
        node: &LirNode,
    ) -> Option<(LirNodeId, Option<LirNodeId>, Option<LirNodeId>)> {
        // ASCII-safety of the receiver is NOT checked here: it is enforced
        // upstream by the `kali_types` E5506 gate (byte-offset slicing of a
        // non-ASCII receiver rejects before codegen). This recognizer only gates
        // on `is_string_valued`, so it stays in lockstep with that gate.
        if node.kind != LirNodeKind::Call || !(1..=3).contains(&node.children.len()) {
            return None;
        }
        let callee = self.resolve_transparent_callable_node(node.children[0])?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("substring") {
            return None;
        }
        let receiver = callee_node.children.first().copied()?;
        if !self.is_string_valued(receiver) {
            return None;
        }
        Some((
            receiver,
            node.children.get(1).copied(),
            node.children.get(2).copied(),
        ))
    }

    /// `(receiver, separator)` iff this is a runtime join over a
    /// linear-memory array binding with proven String elements. Literal
    /// receivers are never in `array_bindings`, so the static fold lane
    /// (`resolve_static_array_join_call`) stays disjoint. ASCII-safety of the
    /// elements and separator is enforced upstream by the `kali_types` E5506
    /// gate (byte-count join of a non-ASCII element/separator rejects before
    /// codegen); this recognizer only gates on the array binding's proven
    /// String element axis, staying in lockstep with that gate.
    pub(crate) fn runtime_join_call_parts(
        &self,
        node: &LirNode,
    ) -> Option<(LirNodeId, Option<LirNodeId>)> {
        if node.kind != LirNodeKind::Call || !(1..=2).contains(&node.children.len()) {
            return None;
        }
        let callee = self.resolve_transparent_callable_node(node.children[0])?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("join") {
            return None;
        }
        let receiver = callee_node.children.first().copied()?;
        let receiver_node = self.node(self.unwrap_transparent(receiver));
        let base = receiver_node.text.as_deref()?;
        // Growable receiver (throw-fallout Stage 4 Task 5): a tagged-handle
        // growable array admits for BOTH element reprs — `I64` (rendered via
        // `int_to_string`) and `String`. Checked FIRST and independently of
        // `array_bindings`: the two lanes are layout-distinct, and
        // `emit_runtime_join` re-derives the growable-ness to pick the
        // header-indirected `__join_growable_*` synthetic. Float growables
        // never promote, so no float element repr can reach here.
        if self.is_growable_array(base) {
            return Some((receiver, node.children.get(1).copied()));
        }
        // Growable-array FIELD receiver `o.values.join(...)` (Stage P2 Lane 1
        // Task 5): a `GrowableArrayI64` object field carries the same tagged
        // handle; admit it through the positive `object_field_is_growable_array`
        // proof (allowlist). i64 elements only (Task 3), joined via
        // `__join_growable_i64` — `emit_runtime_join` re-derives that below.
        if self.object_field_is_growable_array(receiver) {
            return Some((receiver, node.children.get(1).copied()));
        }
        if !self.array_bindings.contains(base) {
            return None;
        }
        if self.array_elem_repr(base) != kali_common::Repr::String {
            return None;
        }
        Some((receiver, node.children.get(1).copied()))
    }

    /// `true` iff `node` is a `<recv>.join(sep?)` call whose receiver `<recv>`
    /// is itself an object-enumeration call (`Object.keys/values/entries`,
    /// `Reflect.ownKeys`). Used by the join fail-closed backstop: such a
    /// receiver only survives to codegen when the optimizer declined to fold the
    /// enumeration (mutated / non-fixed-shape object); a `.values.join(...)`
    /// MEMBER receiver is a different shape and does NOT match here.
    fn is_object_enumeration_join_receiver(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call || !(1..=2).contains(&node.children.len()) {
            return false;
        }
        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee) = self.resolve_transparent_callable_node(callee) else {
            return false;
        };
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("join") {
            return false;
        }
        let Some(receiver) = callee_node.children.first().copied() else {
            return false;
        };
        self.is_object_enumeration_call(self.node(receiver))
            .is_some()
    }

    /// Lower `<recv>.toFixed(<digits>)` to `<recv as f64>; I32Const(digits);
    /// Call(float_to_fixed)`. The receiver is promoted to f64 when it is not already
    /// float-valued; the digit count is read from the static integer literal argument
    /// and clamped to the host's supported `0..=100` range. The result is a string
    /// handle (`ValueShape::String`) that prints via the existing string-handle path.
    pub(crate) fn emit_to_fixed(
        &mut self,
        function: &mut Function,
        receiver: LirNodeId,
        digits: LirNodeId,
    ) -> EmittedValue {
        self.emit_node(function, receiver, true);
        if !self.is_float_valued(receiver) {
            function.instruction(&Instruction::F64ConvertI64S);
        }
        let digit_count = self
            .render_static_value(digits)
            .and_then(|rendered| parse_number_literal(&rendered))
            .unwrap_or(0)
            .clamp(0, 100) as i32;
        function.instruction(&Instruction::I32Const(digit_count));
        function.instruction(&Instruction::Call(FLOAT_TO_FIXED_IMPORT_INDEX));
        EmittedValue {
            produced: true,
            shape: ValueShape::String,
        }
    }

    /// Runtime `x.substring(a?, b?)`: push handle + clamped-later bounds, call
    /// the synthetic `__substring`. Defaults: start 0, end i64::MAX (the
    /// helper clamps it to len — the "to end of string" 0/1-arg forms).
    fn emit_runtime_substring(
        &mut self,
        function: &mut Function,
        receiver: LirNodeId,
        start: Option<LirNodeId>,
        end: Option<LirNodeId>,
    ) -> EmittedValue {
        let recv = self.emit_node(function, receiver, true);
        if !recv.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        self.emit_substring_bound(function, start, 0);
        self.emit_substring_bound(function, end, i64::MAX);
        function.instruction(&Instruction::Call(self.substring_fn_index()));
        EmittedValue {
            produced: true,
            shape: ValueShape::String,
        }
    }

    /// Runtime `a.join(sep)`: push the array handle, then the separator handle
    /// (a runtime string, a static separator, or the default ",") and call the
    /// synthetic `__join`. The result is a fresh `__alloc_global` string handle.
    fn emit_runtime_join(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        receiver: LirNodeId,
        separator: Option<LirNodeId>,
    ) -> EmittedValue {
        let base = self.emit_node(function, receiver, true);
        if !base.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        match separator {
            Some(sep) => {
                let emitted = self.emit_node(function, sep, true);
                if !emitted.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
            }
            None => {
                // JS default separator is ",".
                let (offset, len) = self.strings.intern(",");
                function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
            }
        }
        // Growable receiver (Task 5): a tagged-handle growable array joins
        // through the header-indirected `__join_growable_*` synthetic, picked
        // by the binding's element repr (`String` → the string-handle copy
        // body; default `I64` → the `int_to_string`-render body). Both allocate
        // globally (no arena twin this task — a join result must outlive any
        // reset), so the arena routing below is bypassed. The base name is
        // re-derived exactly as `runtime_join_call_parts` recognized it.
        let growable_base = self
            .node(self.unwrap_transparent(receiver))
            .text
            .as_deref()
            .filter(|base| self.is_growable_array(base))
            .map(str::to_string);
        // Field receiver `o.values.join(...)` (Task 5): a `GrowableArrayI64`
        // field is provably i64 (Task 3 conflicts string array fields to
        // E5506), so it always joins via `__join_growable_i64` — no name to key
        // an element-repr lookup on. Checked when the base-name lane found no
        // named growable.
        let field_growable =
            growable_base.is_none() && self.object_field_is_growable_array(receiver);
        let join_index = if let Some(base) = growable_base {
            if self.array_elem_repr(&base) == kali_common::Repr::String {
                self.join_growable_str_fn_index()
            } else {
                self.join_growable_i64_fn_index()
            }
        } else if field_growable {
            self.join_growable_i64_fn_index()
        } else {
            // Per-site arena routing (fasta Spec 7 Task 4c): select the
            // resettable `__join_arena` twin iff the escape gate proved THIS
            // join site's result iteration-local, keyed by the site's pre-order
            // string-site ordinal (`string_site_ordinals`, the codegen mirror
            // of 4b's stream). A miss (site not numbered, or not granted) fails
            // closed to the global `__join`.
            let use_arena = self
                .string_site_ordinals
                .get(&id)
                .is_some_and(|&ord| self.arena_table.arena_string_site(&self.function_name, ord));
            if use_arena {
                self.join_arena_fn_index()
            } else {
                self.join_fn_index()
            }
        };
        function.instruction(&Instruction::Call(join_index));
        EmittedValue {
            produced: true,
            shape: ValueShape::String,
        }
    }

    /// Emits one substring bound as i64, defaulting when absent. Codegen-side
    /// fail-closed backstop behind the types gate: a float- or string-valued
    /// bound gets a diagnostic, never a silent reinterpret.
    fn emit_substring_bound(
        &mut self,
        function: &mut Function,
        arg: Option<LirNodeId>,
        default: i64,
    ) {
        let Some(arg) = arg else {
            function.instruction(&Instruction::I64Const(default));
            return;
        };
        if self.is_float_valued(arg) || self.is_string_valued(arg) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.substring bounds must be integer-typed in the current direct-runtime path".to_string(),
            ));
            function.instruction(&Instruction::I64Const(default));
            return;
        }
        let value = self.emit_node(function, arg, true);
        if !value.produced {
            function.instruction(&Instruction::I64Const(default));
        }
    }

    /// Emit `receiver.fill(value)` as a repr-directed init loop that writes `value`
    /// into every element slot `0..len`, then leaves the array's i64 base handle on
    /// the stack as the expression result (so the call is bindable/chainable).
    ///
    /// Mirrors the `block { loop { <test ⇒ br out>; body; br loop } }` idiom used by
    /// [`Self::emit_loop`]. Uses the two trailing i64 scratch locals reserved in
    /// `lower.rs`: `base_local` holds the array base handle (also the result) and
    /// `counter_local` the loop counter `i`. The length bound is re-read each pass
    /// from the i64 header at `offset: 0`, so no third local is needed.
    ///
    /// `binding_name` selects the element repr (F64 vs I64) and hence the store
    /// width: an f64 array filled with an integer literal stores it as `1.0` via a
    /// `f64.convert_i64_s` promotion and `F64Store`; an i64 array uses `I64Store`.
    pub(crate) fn emit_array_fill(
        &mut self,
        function: &mut Function,
        receiver: LirNodeId,
        value: LirNodeId,
        binding_name: &str,
    ) -> EmittedValue {
        let base_local = self.locals.len() as u32;
        let counter_local = base_local + 1;

        // Materialize the array base handle (i64) into `base_local`. A fresh
        // `new Array(n)` receiver allocates (writing its length header); an existing
        // binding just loads its handle.
        if let Some(size_arg) = self.resolve_array_alloc_call(receiver) {
            let allocated = self.emit_array_allocation(function, size_arg);
            if !allocated.produced {
                function.instruction(&Instruction::I64Const(0));
            }
        } else {
            let base = self.emit_node(function, receiver, true);
            if !base.produced {
                function.instruction(&Instruction::I64Const(0));
            }
        }
        function.instruction(&Instruction::LocalSet(base_local));

        // i = 0
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(counter_local));

        let elem_is_float = self.array_elem_repr(binding_name) == kali_common::Repr::F64;
        let value_is_float = self.is_float_valued(value);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        // Exit test: `i >= len` breaks out of the enclosing `block` (depth 1). The
        // length is the i64 header at `offset: 0` of the base handle.
        function.instruction(&Instruction::LocalGet(counter_local));
        function.instruction(&Instruction::LocalGet(base_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I64Load(MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        }));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::BrIf(1));

        // addr = base + i*8 (the `+8` element header offset is applied via the store
        // immediate, matching the element read/write paths).
        function.instruction(&Instruction::LocalGet(base_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(counter_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(8));
        function.instruction(&Instruction::I32Mul);
        function.instruction(&Instruction::I32Add);

        // Push `value` at the array's element width. An integer literal/expr stored
        // into an f64 array is promoted to f64 first.
        let produced = self.emit_node(function, value, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        if elem_is_float {
            if !produced.produced || !value_is_float {
                function.instruction(&Instruction::F64ConvertI64S);
            }
            function.instruction(&Instruction::F64Store(MemArg {
                offset: 8,
                align: 3,
                memory_index: 0,
            }));
        } else {
            function.instruction(&Instruction::I64Store(MemArg {
                offset: 8,
                align: 3,
                memory_index: 0,
            }));
        }

        // i += 1
        function.instruction(&Instruction::LocalGet(counter_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(counter_local));

        function.instruction(&Instruction::Br(0)); // back to loop top
        function.instruction(&Instruction::End); // end loop
        function.instruction(&Instruction::End); // end block

        // Result: the array's i64 base handle.
        function.instruction(&Instruction::LocalGet(base_local));
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    /// Emit `base[index_text]` as a dynamic linear-memory load. `base_id` is the array
    /// handle expression, `index_text` is the (literal or identifier) index, and
    /// `base_name` is the array binding's name, used to select `F64Load`/`I64Load`
    /// per the array's element repr.
    pub(crate) fn emit_dynamic_array_read(
        &mut self,
        function: &mut Function,
        base_id: LirNodeId,
        index_text: &str,
        base_name: &str,
    ) -> EmittedValue {
        self.emit_array_element_address(function, base_id, index_text);
        let mem_arg = MemArg {
            offset: 8,
            align: 3,
            memory_index: 0,
        };
        match self.array_elem_repr(base_name) {
            kali_common::Repr::F64 => function.instruction(&Instruction::F64Load(mem_arg)),
            // Spec 3 activates the `String` case: a proven string element loads
            // its tagged handle through the same i64 slot the int/object lanes use.
            // A growable-array element (repr inference does not yet produce this
            // for an array element, but the match must stay exhaustive) is
            // likewise a tagged i64 handle into its header, so it loads through
            // the same slot — this arm never coerces to/from a scalar number.
            kali_common::Repr::I64
            | kali_common::Repr::Object(_)
            | kali_common::Repr::String
            | kali_common::Repr::GrowableArrayI64 => {
                function.instruction(&Instruction::I64Load(mem_arg))
            }
        };
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    /// Push the i32 element address `base + index * 8` (the `+8` length header is applied
    /// via the load/store `offset` immediate).
    pub(crate) fn emit_array_element_address(
        &mut self,
        function: &mut Function,
        base_id: LirNodeId,
        index_text: &str,
    ) {
        let index_node =
            self.alloc_scratch_node(LirNodeKind::Value, Some(index_text.to_string()), vec![]);
        self.emit_array_element_address_node(function, base_id, index_node);
    }

    /// Push the array's i64 base handle as an i32 address, with no index term.
    /// Shared by [`emit_array_element_address_node`] (which adds `index * 8` on
    /// top) and the runtime-array `.length` header read, which loads directly at
    /// `offset: 0` from this same base.
    pub(crate) fn emit_array_base_address(&mut self, function: &mut Function, base_id: LirNodeId) {
        let _ = self.emit_node(function, base_id, true);
        function.instruction(&Instruction::I32WrapI64);
    }

    /// Like [`emit_array_element_address`], but the index comes from an existing
    /// node (a computed subscript expression such as `i + 1`) rather than a
    /// stringified literal/identifier.
    pub(crate) fn emit_array_element_address_node(
        &mut self,
        function: &mut Function,
        base_id: LirNodeId,
        index_id: LirNodeId,
    ) {
        self.emit_array_base_address(function, base_id);
        let _ = self.emit_node(function, index_id, true);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(8));
        function.instruction(&Instruction::I32Mul);
        function.instruction(&Instruction::I32Add);
    }

    /// Emit a computed array read `base[index_expr]` sourcing the index from a
    /// node instead of stringified text. `base_name` is the array binding's name,
    /// used to select `F64Load`/`I64Load` per the array's element repr.
    pub(crate) fn emit_dynamic_array_read_node(
        &mut self,
        function: &mut Function,
        base_id: LirNodeId,
        index_id: LirNodeId,
        base_name: &str,
    ) -> EmittedValue {
        self.emit_array_element_address_node(function, base_id, index_id);
        let mem_arg = MemArg {
            offset: 8,
            align: 3,
            memory_index: 0,
        };
        match self.array_elem_repr(base_name) {
            kali_common::Repr::F64 => function.instruction(&Instruction::F64Load(mem_arg)),
            // Spec 3 activates the `String` case: a proven string element loads
            // its tagged handle through the same i64 slot the int/object lanes use.
            // A growable-array element (repr inference does not yet produce this
            // for an array element, but the match must stay exhaustive) is
            // likewise a tagged i64 handle into its header, so it loads through
            // the same slot — this arm never coerces to/from a scalar number.
            kali_common::Repr::I64
            | kali_common::Repr::Object(_)
            | kali_common::Repr::String
            | kali_common::Repr::GrowableArrayI64 => {
                function.instruction(&Instruction::I64Load(mem_arg))
            }
        };
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    pub(crate) fn resolve_static_index_member(
        &self,
        node: &LirNode,
    ) -> Option<StaticIndexMemberResult> {
        // Dot/literal-index access lowers to `[object]` (1 child); computed
        // access `a[<expr>]` lowers to `[object, index]` (2 children). Either
        // shape can still fold when the index is a statically-known integer,
        // recovered from the stringified `text` (populated in both shapes).
        if node.kind != LirNodeKind::Value
            || !(node.children.len() == 1 || node.children.len() == 2)
        {
            return None;
        }

        let index = self.static_member_index(node.text.as_deref()?)?;
        let source = node.children[0];

        if let Some(result) = self.resolve_static_array_concat_element(source, index) {
            return Some(result);
        }

        if let Some(parts) = self.resolve_static_string_split_parts_from_id(source) {
            return Some(
                parts
                    .get(index)
                    .cloned()
                    .map(StaticIndexMemberResult::String)
                    .unwrap_or(StaticIndexMemberResult::Undefined),
            );
        }

        let source = self.resolve_literal_aggregate(source)?;
        let source_node = self.node(source);
        if self.is_array_literal(source_node) {
            return Some(
                source_node
                    .children
                    .get(index)
                    .copied()
                    .map(StaticIndexMemberResult::Node)
                    .unwrap_or(StaticIndexMemberResult::Undefined),
            );
        }

        None
    }

    pub(crate) fn static_member_index(&self, text: &str) -> Option<usize> {
        let index = parse_number_literal(text)?;
        (index >= 0 && text.chars().all(|ch| ch.is_ascii_digit())).then_some(index as usize)
    }

    pub(crate) fn resolve_static_reference_root_name(&self, id: LirNodeId) -> Option<String> {
        let id = self.resolve_bound_node(id);
        let id = self.unwrap_transparent_value_node(id);
        let node = self.node(id);

        if self.is_object_freeze_call(node) {
            return node
                .children
                .get(1)
                .copied()
                .and_then(|child| self.resolve_static_reference_root_name(child));
        }

        if node.kind == LirNodeKind::Value && node.children.len() == 2 {
            match node.text.as_deref() {
                Some("??") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    if left.is_nullish() {
                        return self.resolve_static_reference_root_name(node.children[1]);
                    }
                    return self.resolve_static_reference_root_name(node.children[0]);
                }
                Some("&&") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_static_reference_root_name(node.children[1]);
                        }
                        Some(false) => {
                            return self.resolve_static_reference_root_name(node.children[0]);
                        }
                        None => {
                            let left_root =
                                self.resolve_static_reference_root_name(node.children[0]);
                            let right_root =
                                self.resolve_static_reference_root_name(node.children[1]);
                            if left_root.is_some() && left_root == right_root {
                                return left_root;
                            }
                        }
                    }
                }
                Some("||") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_static_reference_root_name(node.children[0]);
                        }
                        Some(false) => {
                            return self.resolve_static_reference_root_name(node.children[1]);
                        }
                        None => {
                            let left_root =
                                self.resolve_static_reference_root_name(node.children[0]);
                            let right_root =
                                self.resolve_static_reference_root_name(node.children[1]);
                            if left_root.is_some() && left_root == right_root {
                                return left_root;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        match node.kind {
            LirNodeKind::Value if node.children.is_empty() => {
                let text = node.text.as_deref()?;
                if let Some(bound) = self.bindings.get(text).copied() {
                    return self.resolve_static_reference_root_name(bound);
                }
                Some(text.to_string())
            }
            LirNodeKind::Value if node.children.len() == 1 => {
                let object = self.resolve_static_reference_root_name(node.children[0])?;
                let property = node.text.as_deref()?;
                Some(format!("{object}.{property}"))
            }
            _ => None,
        }
    }

    pub(crate) fn unwrap_transparent_value_node(&self, mut id: LirNodeId) -> LirNodeId {
        loop {
            let node = self.node(id);
            if node.kind == LirNodeKind::Value
                && node.children.len() == 1
                && node
                    .text
                    .as_deref()
                    // A text-less/empty 1-child `Value` is a grouping/sequence
                    // wrapper; the `"await"` marker (Stage 3 Task 4) is a
                    // synchronously-settled passthrough — both tunnel to the child
                    // for static-reference resolution (e.g. `await globalThis.Object`).
                    .is_none_or(|text| text.is_empty() || text == "await")
            {
                id = node.children[0];
                continue;
            }

            return id;
        }
    }

    pub(crate) fn is_supported_callable_reference(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Value || node.children.len() != 1 {
            return false;
        }

        match node.text.as_deref() {
            Some("is") => self.is_object_identity_object(node),
            Some("isFinite") | Some("isNaN") | Some("isInteger") | Some("isSafeInteger") => {
                self.is_number_object(node)
            }
            _ => false,
        }
    }

    pub(crate) fn resolve_bound_node(&self, mut id: LirNodeId) -> LirNodeId {
        let mut seen = HashSet::new();

        loop {
            if !seen.insert(id) {
                return id;
            }

            let node = self.node(id);
            if node.kind == LirNodeKind::Value && node.children.is_empty() {
                if let Some(text) = node.text.as_deref() {
                    if let Some(bound) = self.bindings.get(text).copied() {
                        id = bound;
                        continue;
                    }
                }
            }

            return id;
        }
    }

    pub(crate) fn resolve_bound_member_callable_node(&self, id: LirNodeId) -> Option<LirNodeId> {
        let bound = self.resolve_bound_node(id);
        let bound = self.unwrap_transparent_value_node(bound);
        let node = self.node(bound);
        if node.kind == LirNodeKind::Value && node.children.len() == 2 {
            match node.text.as_deref() {
                Some("??") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    if left.is_nullish() {
                        return self.resolve_bound_member_callable_node(node.children[1]);
                    }
                    return self.resolve_bound_member_callable_node(node.children[0]);
                }
                Some("&&") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_bound_member_callable_node(node.children[1])
                        }
                        Some(false) => {
                            return self.resolve_bound_member_callable_node(node.children[0])
                        }
                        None => {
                            let consequent =
                                self.resolve_bound_member_callable_node(node.children[0]);
                            let alternate =
                                self.resolve_bound_member_callable_node(node.children[1]);
                            if consequent.is_some() && consequent == alternate {
                                return consequent;
                            }
                        }
                    }
                }
                Some("||") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_bound_member_callable_node(node.children[0])
                        }
                        Some(false) => {
                            return self.resolve_bound_member_callable_node(node.children[1])
                        }
                        None => {
                            let consequent =
                                self.resolve_bound_member_callable_node(node.children[0]);
                            let alternate =
                                self.resolve_bound_member_callable_node(node.children[1]);
                            if consequent.is_some() && consequent == alternate {
                                return consequent;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if node.kind == LirNodeKind::Value && node.children.len() == 3 {
            let consequent = self.resolve_bound_member_callable_node(node.children[1])?;
            let alternate = self.resolve_bound_member_callable_node(node.children[2])?;
            if self.node(consequent).text.as_deref() == self.node(alternate).text.as_deref() {
                return Some(consequent);
            }
        }
        if node.text.is_some() && !node.children.is_empty() {
            Some(bound)
        } else if self.is_object_freeze_call(node) {
            node.children
                .get(1)
                .copied()
                .and_then(|child| self.resolve_bound_member_callable_node(child))
        } else if node.kind == LirNodeKind::Value && node.children.len() == 1 {
            self.resolve_bound_member_callable_node(node.children[0])
        } else {
            None
        }
    }

    /// Resolve a short-circuit (`??`/`&&`/`||`) or ternary (`?`) callable
    /// SELECTION to the node it statically selects. This rule is mirrored
    /// in `kali_optimize::helpers::Optimizer::callable_selection_member_access_name`
    /// — keep them synchronized.
    pub(crate) fn resolve_transparent_callable_node(&self, id: LirNodeId) -> Option<LirNodeId> {
        let mut id = self.resolve_bound_node(id);
        let mut seen = HashSet::new();

        loop {
            if !seen.insert(id.0) {
                return None;
            }

            let node = self.node(id);
            if node.kind == LirNodeKind::Value
                && node.children.len() == 1
                && node
                    .text
                    .as_deref()
                    .is_none_or(|text| text.is_empty() || text == "await")
            {
                id = node.children[0];
                continue;
            }

            if self.is_object_freeze_call(node) {
                id = node.children.get(1).copied()?;
                continue;
            }

            if node.kind == LirNodeKind::Value && node.children.len() == 2 {
                match node.text.as_deref() {
                    Some("??") => {
                        let left = self.resolve_static_object_identity_value(node.children[0])?;
                        if left.is_nullish() {
                            id = node.children[1];
                            continue;
                        }

                        id = node.children[0];
                        continue;
                    }
                    Some("&&") => {
                        let left = self.resolve_static_object_identity_value(node.children[0])?;
                        match left.truthiness() {
                            Some(true) => {
                                id = node.children[1];
                                continue;
                            }
                            Some(false) => {
                                id = node.children[0];
                                continue;
                            }
                            None => {
                                let consequent =
                                    self.resolve_transparent_callable_node(node.children[0]);
                                let alternate =
                                    self.resolve_transparent_callable_node(node.children[1]);
                                if consequent.is_some() && consequent == alternate {
                                    return consequent;
                                }
                            }
                        }
                    }
                    Some("||") => {
                        let left = self.resolve_static_object_identity_value(node.children[0])?;
                        match left.truthiness() {
                            Some(true) => {
                                id = node.children[0];
                                continue;
                            }
                            Some(false) => {
                                id = node.children[1];
                                continue;
                            }
                            None => {
                                let consequent =
                                    self.resolve_transparent_callable_node(node.children[0]);
                                let alternate =
                                    self.resolve_transparent_callable_node(node.children[1]);
                                if consequent.is_some() && consequent == alternate {
                                    return consequent;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            if node.kind == LirNodeKind::Value && node.children.len() == 3 {
                let consequent = self.resolve_transparent_callable_node(node.children[1])?;
                let alternate = self.resolve_transparent_callable_node(node.children[2])?;
                if self.node(consequent).text.as_deref() == self.node(alternate).text.as_deref() {
                    return Some(consequent);
                }
            }

            if node.text.is_some() {
                return Some(id);
            }

            return None;
        }
    }
}

#[cfg(test)]
#[path = "call_tests.rs"]
mod call_tests;

#[cfg(test)]
#[path = "reclamation_tests.rs"]
mod reclamation_tests;
