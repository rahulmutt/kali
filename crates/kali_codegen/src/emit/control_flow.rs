use crate::*;

impl<'a> FunctionEmitter<'a> {
    pub(crate) fn emit_break_or_continue(
        &mut self,
        function: &mut Function,
        is_continue: bool,
        node: &LirNode,
    ) -> EmittedValue {
        let Some(loop_frame) = self.loop_frames.last().copied() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "break and continue are unavailable outside the supported static loop lowering path; use a supported loop form or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        };

        let control_text = node.text.as_deref().unwrap_or_default();
        let (keyword, label) = if let Some(label) = control_text.strip_prefix("break:") {
            ("break", label)
        } else if let Some(label) = control_text.strip_prefix("continue:") {
            ("continue", label)
        } else if control_text == "break" {
            ("break", "")
        } else if control_text == "continue" {
            ("continue", "")
        } else {
            (control_text, "")
        };

        if !label.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "{keyword} labels are unavailable in the current phase; use an unlabeled {keyword} inside the supported static loop lowering path or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        let target_index = if is_continue {
            loop_frame.continue_index
        } else {
            loop_frame.break_index
        };
        let depth = self.control_frame_depth(target_index);

        // No inline arena release here — this is intentional, not a gap.
        // Labels are rejected above, so an unlabeled `break`/`continue`
        // always targets the innermost open loop, and a `break`'s `Br` lands
        // exactly where `emit_loop` already emits its own unconditional
        // normal-exit release (`arena_frames.pop()` + `emit_arena_release`,
        // right after that loop's closing `End`s — a wasm branch to a
        // `block`'s label lands immediately after its `End`, the same target
        // as normal fallthrough). An earlier version of this function ALSO
        // emitted an inline release here, reasoning that it "only executes on
        // the break path" — true, but irrelevant: the break path then falls
        // straight into `emit_loop`'s unconditional release too, so the same
        // `ArenaFrame` got released twice. For a loop nested inside an
        // already-allocating enclosing arena, the second `__arena_reset` ran
        // against the *enclosing* arena's now-current (restored) page list,
        // splicing its still-live pages onto the free list — a corrupted
        // free list / use-after-free. Releasing exactly once here — via the
        // fallthrough into `emit_loop`'s own close — is correct for both the
        // normal-exit path and the break path, and `continue` never releases
        // anything (it re-enters the same iteration, never leaving the loop).
        function.instruction(&Instruction::Br(depth));
        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }

    /// `Call(__arena_reset)` followed by restoring the saved current-arena
    /// trio (`g1`/`g2`/`g3`) from `frame`'s three saved locals. Emit-only: the
    /// caller decides whether/when to pop `arena_frames`.
    pub(crate) fn emit_arena_release(&mut self, function: &mut Function, frame: &ArenaFrame) {
        function.instruction(&Instruction::Call(self.arena_reset_fn_index()));
        function.instruction(&Instruction::LocalGet(frame.saved_page_local));
        function.instruction(&Instruction::GlobalSet(1));
        function.instruction(&Instruction::LocalGet(frame.saved_cursor_local));
        function.instruction(&Instruction::GlobalSet(2));
        function.instruction(&Instruction::LocalGet(frame.saved_limit_local));
        function.instruction(&Instruction::GlobalSet(3));
    }

    /// Releases every currently-open loop arena, newest→oldest, emit-only (no
    /// pop — `arena_frames` is only ever popped by the owning loop's own
    /// normal-exit release in `emit_loop`). Shared by both `Instruction::Return`
    /// emission sites in `emit_return` below: a `return` unwinds the wasm
    /// call frame directly, bypassing every enclosing `block`/`loop`
    /// construct (unlike `break`, it never lands at the loop's own
    /// post-`End` release code), so every live arena must be released inline
    /// here or its pages would never be recycled.
    fn emit_arena_unwind_for_return(&mut self, function: &mut Function) {
        let frames: Vec<ArenaFrame> = self.arena_frames.iter().rev().copied().collect();
        for frame in &frames {
            self.emit_arena_release(function, frame);
        }
    }

    pub(crate) fn emit_return(&mut self, function: &mut Function, node: &LirNode) -> EmittedValue {
        if let Some(arg) = node.children.first().copied() {
            // A function whose return repr is Object(shape) returning an
            // object literal materializes it (factory functions). Only the
            // direct return argument routes here — other literals in the
            // body keep their own lanes.
            if let kali_common::Repr::Object(shape) =
                self.repr_table.return_repr(&self.function_name)
            {
                if let Some(aggregate_id) = self.resolve_literal_aggregate(arg) {
                    let aggregate = self.node(aggregate_id).clone();
                    if self.is_object_literal(&aggregate) {
                        let produced = self.emit_object_allocation(function, &aggregate, shape);
                        if !produced.produced {
                            function.instruction(&Instruction::I64Const(0));
                        }
                        self.emit_arena_unwind_for_return(function);
                        function.instruction(&Instruction::Return);
                        return EmittedValue {
                            produced: false,
                            shape: ValueShape::Unknown,
                        };
                    }
                }
            }
            let produced = self.emit_node(function, arg, true);
            if !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            }
        } else {
            function.instruction(&Instruction::I64Const(0));
        }
        self.emit_arena_unwind_for_return(function);
        function.instruction(&Instruction::Return);
        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }

    /// Lowers `while` / `do-while` / `for` to a real wasm `block { loop { ... } }`
    /// with a back-edge, so the body can run more than once and `break`/`continue`
    /// resolve via the loop-frame stack.
    ///
    /// Known limitation: in a `for` loop, `update` is emitted at the tail of the
    /// body (since wasm's `loop` has no native init/update clause), so a
    /// `continue` re-enters the loop *without* running `update`. `while` /
    /// `do-while` re-test correctly on `continue` since they have no `update`.
    pub(crate) fn emit_loop(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        node: &LirNode,
    ) -> EmittedValue {
        let kind = node.text.as_deref().unwrap_or_default();

        // The escape gate keys `loop_arena` by this loop's pre-order ordinal
        // (see `crate::lower::loop_preorder_ordinals`'s doc comment for why
        // this lookup and the locals `collect_function_locals` reserved for
        // it can never diverge). A miss (no ordinal recorded, or the gate
        // never granted this ordinal) fails closed: `is_arena_loop = false`,
        // i.e. no arena — plain `__alloc`/`__alloc_global` routing per
        // `alloc_callee_index`, exactly like before this task.
        let ordinal = self.loop_ordinals.get(&id).copied();
        let is_arena_loop =
            ordinal.is_some_and(|ord| self.arena_table.loop_arena(&self.function_name, ord));
        let arena_save_locals = ordinal.filter(|_| is_arena_loop).map(|ord| {
            let (page_name, cursor_name, limit_name) = crate::lower::arena_save_local_names(ord);
            (
                self.locals[&page_name],
                self.locals[&cursor_name],
                self.locals[&limit_name],
            )
        });

        // Resolve clauses by loop kind.
        let (init, test, update, body) = match kind {
            "while" => (
                None,
                node.children.first().copied(),
                None,
                node.children.get(1).copied(),
            ),
            "do-while" => (
                None,
                node.children.get(1).copied(),
                None,
                node.children.first().copied(),
            ),
            _ /* "for" */ => {
                // [init?, test?, update?, body] — body is always last; classify by count.
                let n = node.children.len();
                let body = node.children.last().copied();
                let (init, test, update) = match n {
                    1 => (None, None, None),
                    2 => (None, node.children.first().copied(), None),
                    3 => (
                        node.children.first().copied(),
                        node.children.get(1).copied(),
                        None,
                    ),
                    _ => (
                        node.children.first().copied(),
                        node.children.get(1).copied(),
                        node.children.get(2).copied(),
                    ),
                };
                (init, test, update, body)
            }
        };

        // for-init runs once, before the loop.
        if let Some(init) = init {
            let produced = self.emit_node(function, init, false);
            if produced.produced {
                function.instruction(&Instruction::Drop);
            }
        }

        // Open: save the current-arena trio (`g1`/`g2`/`g3`) into this loop's
        // three reserved locals, then zero it — so allocations from here
        // until the loop closes start a fresh, empty arena instead of
        // continuing to bump whatever arena was active before this loop
        // (the enclosing function's, or an enclosing loop's, borrowed for the
        // duration exactly like a callee borrows and restores a register).
        if let Some((saved_page, saved_cursor, saved_limit)) = arena_save_locals {
            function.instruction(&Instruction::GlobalGet(1));
            function.instruction(&Instruction::LocalSet(saved_page));
            function.instruction(&Instruction::GlobalGet(2));
            function.instruction(&Instruction::LocalSet(saved_cursor));
            function.instruction(&Instruction::GlobalGet(3));
            function.instruction(&Instruction::LocalSet(saved_limit));
            function.instruction(&Instruction::I32Const(0));
            function.instruction(&Instruction::GlobalSet(1));
            function.instruction(&Instruction::I32Const(0));
            function.instruction(&Instruction::GlobalSet(2));
            function.instruction(&Instruction::I32Const(0));
            function.instruction(&Instruction::GlobalSet(3));
        }

        let break_index = self.push_control_frame(ControlFlowLabelKind::LoopBreak);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_index = self.push_control_frame(ControlFlowLabelKind::LoopContinue);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        let loop_frame_index = self.loop_frames.len();
        self.loop_frames.push(LoopFrame {
            break_index,
            continue_index,
        });

        // Per-iteration reset: recycle the PREVIOUS iteration's pages onto
        // the free list and zero the trio again, at the very top of every
        // iteration (before the test/body) — including the first, which is a
        // no-op recycle against the still-empty arena `Open` just installed.
        // Placing it here (rather than at the bottom, before the back-edge)
        // makes `continue` correct with zero extra unwinding: every loop
        // re-entry, however it was reached, passes through this reset, and
        // the outflow veto in the escape gate already guarantees nothing
        // live spans an iteration boundary.
        if let Some((saved_page, saved_cursor, saved_limit)) = arena_save_locals {
            function.instruction(&Instruction::Call(self.arena_reset_fn_index()));
            self.arena_frames.push(ArenaFrame {
                saved_page_local: saved_page,
                saved_cursor_local: saved_cursor,
                saved_limit_local: saved_limit,
                loop_frame_index: Some(loop_frame_index),
            });
        }

        let emit_body_and_update = |emitter: &mut Self, function: &mut Function| {
            if let Some(body) = body {
                let produced = emitter.emit_node(function, body, false);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
            }
            if let Some(update) = update {
                let produced = emitter.emit_node(function, update, false);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
            }
        };

        if kind == "do-while" {
            // body first, then test at the bottom.
            emit_body_and_update(self, function);
            if let Some(test) = test {
                self.reject_string_condition(test);
                let cond = self.emit_node(function, test, true);
                if !cond.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
            } else {
                function.instruction(&Instruction::I64Const(1));
            }
            function.instruction(&Instruction::I64Eqz); // 1 if falsy
            function.instruction(&Instruction::I32Eqz); // invert: 1 if truthy
            function.instruction(&Instruction::BrIf(0)); // continue if truthy
        } else {
            // test at the top; exit (break) when falsy.
            if let Some(test) = test {
                self.reject_string_condition(test);
                let cond = self.emit_node(function, test, true);
                if !cond.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
                function.instruction(&Instruction::I64Eqz); // 1 if falsy
                function.instruction(&Instruction::BrIf(1)); // break out of `block` when falsy
            }
            emit_body_and_update(self, function);
            function.instruction(&Instruction::Br(0)); // back to loop top
        }

        function.instruction(&Instruction::End); // end loop
        self.loop_frames.pop();
        self.pop_control_frame(ControlFlowLabelKind::LoopContinue);
        function.instruction(&Instruction::End); // end block
        self.pop_control_frame(ControlFlowLabelKind::LoopBreak);

        // Release: unconditional fallthrough code reached both by the
        // natural falsy-test exit AND by any `break` inside this loop (both
        // land here — a wasm branch to a `block`'s label lands immediately
        // after its `End`, same as normal fallthrough). Recycle whatever the
        // last iteration left in the current arena, then restore the trio
        // this loop's `Open` step saved, so the enclosing scope's allocations
        // resume exactly where they left off. This is the ONLY place that
        // permanently retires this loop's `ArenaFrame`.
        if arena_save_locals.is_some() {
            if let Some(frame) = self.arena_frames.pop() {
                self.emit_arena_release(function, &frame);
            }
        }

        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }

    /// Bound name of a `for..in` key (`for-in` node's `children[0]`, i.e.
    /// `left`): either a declaration form (`var`/`let`/`const` `Instruction`
    /// wrapping one declarator whose own `text` is the name — the same shape
    /// `collect_function_locals_from_node` recognizes when reserving the
    /// key's local, so this lookup and that reservation can never disagree
    /// about the name), or a plain already-bound identifier (`for (c in obj)`
    /// with `c` declared elsewhere), whose own `text` already IS the name.
    fn for_in_key_name(&self, left_id: LirNodeId) -> String {
        let left = self.node(left_id);
        if left.kind == LirNodeKind::Instruction
            && matches!(left.text.as_deref(), Some("let" | "var" | "const"))
        {
            if let Some(&declarator_id) = left.children.first() {
                if let Some(name) = self.node(declarator_id).text.clone() {
                    return name;
                }
            }
        }
        left.text.clone().unwrap_or_default()
    }

    /// Lower `for (KEY in OBJ)` over a compile-time-known fixed-shape object.
    /// `children` = `[left(key binding), right(object), body]`. The key is
    /// bound to an ordinal `0..N-1` (`N` = `OBJ`'s shape field count); it is
    /// not yet usable as an index or a string (Tasks 3/5). No arena: this
    /// loop allocates nothing per iteration.
    ///
    /// Critically, this loop must NEVER be numbered by
    /// `crate::lower::loop_preorder_ordinals` / looked up in `ArenaTable` —
    /// see that function's doc comment and `kali_mir::analysis::walk`'s
    /// `ForInStmt` arm: assigning this loop a real loop-arena ordinal would
    /// desync every REAL loop lexically following it in the same function.
    /// The dedicated counter local this function uses instead comes from the
    /// wholly separate `crate::lower::for_in_preorder_ordinals` bookkeeping.
    pub(crate) fn emit_for_in(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        node: &LirNode,
    ) -> EmittedValue {
        let left_id = node.children[0];
        let right_id = node.children[1];
        let body_id = node.children[2];

        // N from the object's shape. Fail closed if the object has no known shape.
        let shape = match self.object_shape_of_node(right_id) {
            Some(s) => s,
            None => {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "for..in is only supported over an object with a compile-time-known shape",
                ));
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
        };
        let n = self.repr_table.shape_fields(shape).len() as i64;

        // Resolve the key variable's local slot — the same slot an
        // identifier read of the key resolves to via `self.locals` (see
        // `emit_value`'s 0-child `Value` arm).
        let key_name = self.for_in_key_name(left_id);
        let Some(key_local) = self.locals.get(&key_name).copied() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!("for..in key binding '{key_name}' has no reserved local"),
            ));
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        };
        // Dedicated ordinal-counter local — see `for_in_preorder_ordinals`.
        let ordinal = self.for_in_ordinals[&id];
        let ord_local = self.locals[&crate::lower::for_in_ord_local_name(ordinal)];

        // Record the key → shape provenance BEFORE emitting the body, so a
        // computed access `table[c]` inside the body recognizes `c` as this
        // loop's ordinal over `shape` (Spec 4a Task 3). Mirror of
        // `kali_types`'s `register_for_in_key`; the codegen recognizer
        // (`computed_forin_object_access`) is the structural twin of the
        // types gate that the emitter relies on for correctness.
        self.for_in_key_shapes.insert(key_name.clone(), shape);

        // Register for-in-key ALIASES (`last = c`) over this same shape BEFORE
        // emitting the body (Spec 4a Task 4). A computed read `table[last]`
        // inside the body is emitted BEFORE the `last = c` assignment that
        // grants the alias, so without this pre-registration
        // `computed_forin_object_access` would not recognize `table[last]` as a
        // dynamic for-in-key slot. `last = c` aliases reference the loop key, so
        // they only occur inside this body; mirror of the types-side `last = c`
        // provenance propagation.
        let mut key_only = std::collections::HashSet::new();
        key_only.insert(key_name.clone());
        let mut aliases = std::collections::HashSet::new();
        crate::lower::for_in_key_aliases_walk(
            &self.program.nodes,
            body_id,
            &key_only,
            &mut aliases,
        );
        for alias in aliases {
            self.for_in_key_shapes.insert(alias, shape);
        }

        // preheader: ord = 0
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(ord_local));

        // block (break target) { loop (continue target) { ... } }
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));

        // break when ord >= N
        function.instruction(&Instruction::LocalGet(ord_local));
        function.instruction(&Instruction::I64Const(n));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::BrIf(1)); // -> break out of block

        // key = ord
        function.instruction(&Instruction::LocalGet(ord_local));
        function.instruction(&Instruction::LocalSet(key_local));

        // body
        let produced = self.emit_node(function, body_id, false);
        if produced.produced {
            function.instruction(&Instruction::Drop);
        }

        // ord = ord + 1
        function.instruction(&Instruction::LocalGet(ord_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(ord_local));

        function.instruction(&Instruction::Br(0)); // back to loop top
        function.instruction(&Instruction::End); // end loop
        function.instruction(&Instruction::End); // end block

        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }

    /// Task 7 prologue: if `ArenaTable::opens_arena` grants this function a
    /// function-body arena, save the current-arena trio (`g1`/`g2`/`g3`) into
    /// this function's three reserved locals (`arena_save_local_names_for_function`,
    /// provisioned by `collect_function_locals`) and zero it — the same
    /// "Open" shape `emit_loop` uses for a loop arena, just once per call
    /// instead of once per loop entry. Pushes an `ArenaFrame { loop_frame_index:
    /// None, .. }` as the BOTTOM of `arena_frames` (called before any loop in
    /// the body can push its own frame on top), so `emit_return`'s all-frames
    /// unwind (`emit_arena_unwind_for_return`, which walks `arena_frames`
    /// newest→oldest) releases this frame too on every explicit `return` —
    /// including one nested inside a loop, where it must run OUTSIDE/AFTER
    /// that loop's own frame release, oldest-released-last, which "newest
    /// first" already guarantees since this frame is the oldest (pushed
    /// first, bottom of the stack). A miss (`opens_arena` false) is a no-op:
    /// no frame is pushed, so `alloc_callee_index`'s existing
    /// `arena_eligible`/`__alloc_global` routing and `emit_return`'s unwind
    /// are both completely unaffected — behavior-identical to before this task.
    pub(crate) fn emit_function_arena_prologue(&mut self, function: &mut Function) {
        if !self.arena_table.opens_arena(&self.function_name) {
            return;
        }
        let (page_name, cursor_name, limit_name) =
            crate::lower::arena_save_local_names_for_function();
        let saved_page_local = self.locals[&page_name];
        let saved_cursor_local = self.locals[&cursor_name];
        let saved_limit_local = self.locals[&limit_name];
        function.instruction(&Instruction::GlobalGet(1));
        function.instruction(&Instruction::LocalSet(saved_page_local));
        function.instruction(&Instruction::GlobalGet(2));
        function.instruction(&Instruction::LocalSet(saved_cursor_local));
        function.instruction(&Instruction::GlobalGet(3));
        function.instruction(&Instruction::LocalSet(saved_limit_local));
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::GlobalSet(1));
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::GlobalSet(2));
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::GlobalSet(3));
        self.arena_frames.push(ArenaFrame {
            saved_page_local,
            saved_cursor_local,
            saved_limit_local,
            loop_frame_index: None,
        });
    }

    /// Task 7 fall-through epilogue: the counterpart to
    /// `emit_function_arena_prologue`, emitted after the function body (right
    /// before the trailing wasm `End`, i.e. only on the path where the
    /// function's own control flow runs off the end of its body rather than
    /// hitting an explicit `return`). Pops the function-level `ArenaFrame`
    /// pushed by the prologue (a no-op if the prologue never pushed one,
    /// i.e. `opens_arena` is false) and releases it exactly once here.
    ///
    /// This can never double-release with `emit_return`'s unwind: a `return`
    /// always emits `Instruction::Return`, which exits the wasm function
    /// immediately, so any code emitted after it (including this epilogue,
    /// which only runs after `emit_function_body`'s call to `emit_node` for
    /// the WHOLE body returns) is unreachable on that path — the two release
    /// sites are mutually exclusive per invocation, not merely per lexical
    /// position. `emit_return`'s unwind is emit-only (never pops
    /// `arena_frames`), so on the fall-through path this frame is still on
    /// top of the stack here with its original saved-locals intact, letting
    /// this epilogue restore from the SAME locals the prologue wrote,
    /// regardless of how many `return`s executed inside nested branches that
    /// were NOT taken at runtime.
    pub(crate) fn emit_function_arena_epilogue(&mut self, function: &mut Function) {
        if !self.arena_table.opens_arena(&self.function_name) {
            return;
        }
        let Some(frame) = self.arena_frames.pop() else {
            return;
        };
        debug_assert_eq!(
            frame.loop_frame_index, None,
            "function epilogue must pop exactly the function-level arena frame \
             (bottom of `arena_frames`, pushed by emit_function_arena_prologue); \
             a non-None loop_frame_index here means a loop frame was left \
             unpopped, which is an emit_loop bug, not a Task-7 one"
        );
        self.emit_arena_release(function, &frame);
    }

    pub(crate) fn emit_function_body(
        &mut self,
        function: &mut Function,
        body: LirNodeId,
        returns_value: bool,
        coverage_id: Option<u32>,
    ) {
        self.emit_coverage_hit(function, coverage_id);
        self.emit_function_arena_prologue(function);
        let produced = self.emit_node(function, body, returns_value);
        if returns_value && !produced.produced {
            // Fallthrough value must match the function's declared result type: an
            // f64-returning function needs an f64 zero here (this trailing value
            // still has to type-check even when the body always returns early).
            if self.repr_table.return_repr(&self.function_name) == kali_common::Repr::F64 {
                function.instruction(&Instruction::F64Const(0.0.into()));
            } else {
                function.instruction(&Instruction::I64Const(0));
            }
        } else if !returns_value && produced.produced {
            function.instruction(&Instruction::Drop);
        }
        self.emit_function_arena_epilogue(function);
    }

    pub(crate) fn emit_sequence(
        &mut self,
        function: &mut Function,
        children: &[LirNodeId],
        want_value: bool,
    ) -> EmittedValue {
        let mut final_value = EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        };
        for (idx, child) in children.iter().enumerate() {
            let child_want_value = want_value && idx + 1 == children.len();
            let child_result = self.emit_node(function, *child, child_want_value);
            if child_result.produced && !child_want_value {
                function.instruction(&Instruction::Drop);
            }
            if child_want_value {
                final_value = child_result;
            }
        }

        if want_value {
            final_value
        } else {
            EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            }
        }
    }

    pub(crate) fn emit_node(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        want_value: bool,
    ) -> EmittedValue {
        let node = self.node(id).clone();
        match node.kind {
            LirNodeKind::Program | LirNodeKind::Block => {
                self.emit_sequence(function, &node.children, want_value)
            }
            LirNodeKind::Instruction => {
                if matches!(node.text.as_deref(), Some("const" | "let" | "var")) {
                    let is_const = node.text.as_deref() == Some("const");
                    for declarator_id in &node.children {
                        let declarator = self.node(*declarator_id).clone();
                        if declarator.children.len() < 2 {
                            continue;
                        }
                        let init = declarator.children[1];

                        // Materialized object-literal binding: `const p = {…}`
                        // whose inferred repr is Object(shape) — allocate the
                        // fixed-layout struct and bind the base pointer.
                        // Unmaterialized literals keep the fold lane below.
                        if let Some(name) = declarator.text.clone() {
                            if let kali_common::Repr::Object(shape) = self.scalar_repr(&name) {
                                let aggregate = self
                                    .resolve_literal_aggregate(init)
                                    .map(|id| self.node(id).clone())
                                    .filter(|node| self.is_object_literal(node));
                                if let Some(aggregate) = aggregate {
                                    let allocated =
                                        self.emit_object_allocation(function, &aggregate, shape);
                                    if !allocated.produced {
                                        function.instruction(&Instruction::I64Const(0));
                                    }
                                    if let Some(index) = self.locals.get(&name).copied() {
                                        function.instruction(&Instruction::LocalSet(index));
                                    } else {
                                        function.instruction(&Instruction::Drop);
                                    }
                                    continue;
                                }
                                // A shaped binding aliasing an existing object
                                // (identifier / element / call): the generic
                                // emission below yields the i64 pointer.
                            }
                        }

                        // Array literal of object references:
                        // `const bodies = [ … ]` with element repr
                        // Object(shape) — allocate the array, then
                        // materialize/store each element pointer.
                        if let Some(name) = declarator.text.clone() {
                            if let kali_common::Repr::Object(elem_shape) =
                                self.array_elem_repr(&name)
                            {
                                let aggregate = self
                                    .resolve_literal_aggregate(init)
                                    .map(|id| self.node(id).clone())
                                    .filter(|node| self.is_array_literal(node));
                                if let (Some(aggregate), Some(index)) =
                                    (aggregate, self.locals.get(&name).copied())
                                {
                                    let allocated = self.emit_array_allocation_static(
                                        function,
                                        aggregate.children.len(),
                                    );
                                    if !allocated.produced {
                                        function.instruction(&Instruction::I64Const(0));
                                    }
                                    function.instruction(&Instruction::LocalSet(index));
                                    self.array_bindings.insert(name.clone());
                                    for (i, child) in aggregate.children.iter().copied().enumerate()
                                    {
                                        function.instruction(&Instruction::LocalGet(index));
                                        function.instruction(&Instruction::I32WrapI64);
                                        let child_node = self.node(child).clone();
                                        let produced = if self.is_object_literal(&child_node) {
                                            self.emit_object_allocation(
                                                function,
                                                &child_node,
                                                elem_shape,
                                            )
                                        } else {
                                            // Factory call / identifier: already
                                            // an i64 pointer.
                                            self.emit_node(function, child, true)
                                        };
                                        if !produced.produced {
                                            function.instruction(&Instruction::I64Const(0));
                                        }
                                        function.instruction(&Instruction::I64Store(MemArg {
                                            offset: (8 + i * 8) as u64,
                                            align: 3,
                                            memory_index: 0,
                                        }));
                                    }
                                    continue;
                                }
                            }
                        }

                        // `new Array(n)` allocations need a stable handle held in a
                        // local slot regardless of `const`/`let`, so the binding can be
                        // read and written through linear memory.
                        if let Some(size_arg) = self.resolve_array_alloc_call(init) {
                            let allocated = self.emit_array_allocation(function, size_arg);
                            if !allocated.produced {
                                function.instruction(&Instruction::I64Const(0));
                            }
                            if let Some(name) = declarator.text.clone() {
                                if let Some(index) = self.locals.get(&name).copied() {
                                    function.instruction(&Instruction::LocalSet(index));
                                    self.array_bindings.insert(name);
                                    continue;
                                }
                            }
                            function.instruction(&Instruction::Drop);
                            continue;
                        }

                        // `const u = new Array(n).fill(v)` (or `u = a.fill(v)`):
                        // register `u` as an array binding — so its element repr and
                        // subsequent reads resolve — and run the repr-directed fill
                        // loop at initialization, binding `u` to the filled array's
                        // base handle.
                        if let Some((receiver, value)) = self.resolve_array_fill_call(init) {
                            if let Some(name) = declarator.text.clone() {
                                if let Some(index) = self.locals.get(&name).copied() {
                                    self.array_bindings.insert(name.clone());
                                    let filled =
                                        self.emit_array_fill(function, receiver, value, &name);
                                    if !filled.produced {
                                        function.instruction(&Instruction::I64Const(0));
                                    }
                                    function.instruction(&Instruction::LocalSet(index));
                                    continue;
                                }
                            }
                        }

                        // Spec 4a Task 4 null-sentinel: `var last = null` where
                        // `last` carries for-in-key provenance stores the
                        // sentinel `-1` (an out-of-range ordinal), NOT `0` — `0`
                        // is a valid first-field ordinal and would collide with
                        // the key. `if (last)` then lowers to `last >= 0` (see
                        // `emit_branch`), so the sentinel reads false. Recognized
                        // structurally via the precomputed `for_in_key_aliases`
                        // set (the loop that grants `last` its provenance is
                        // emitted AFTER this init, so a runtime signal is unusable
                        // here).
                        if let Some(name) = declarator.text.clone() {
                            if self.for_in_key_aliases.contains(&name)
                                && self.is_null_or_undefined_literal(init)
                            {
                                if let Some(index) = self.locals.get(&name).copied() {
                                    function.instruction(&Instruction::I64Const(-1));
                                    function.instruction(&Instruction::LocalSet(index));
                                    continue;
                                }
                            }
                        }

                        let init_result = self.emit_node(function, init, true);
                        // A named scalar local whose chosen repr is F64 must receive an
                        // f64 on the stack; promote an integer-valued init before the store.
                        let f64_local = match declarator.text.as_deref() {
                            Some(name) => {
                                self.locals.contains_key(name)
                                    && self.scalar_repr(name) == kali_common::Repr::F64
                            }
                            None => false,
                        };
                        if !init_result.produced {
                            if f64_local {
                                function.instruction(&Instruction::F64Const(0.0.into()));
                            } else {
                                function.instruction(&Instruction::I64Const(0));
                            }
                        } else if f64_local && !self.is_float_valued(init) {
                            function.instruction(&Instruction::F64ConvertI64S);
                        }
                        if let Some(name) = declarator.text.clone() {
                            if let Some(index) = self.locals.get(&name).copied() {
                                // `let`/`var`, or a `const` promoted to a local slot
                                // (array allocation / array read) — store eagerly.
                                function.instruction(&Instruction::LocalSet(index));
                            } else if is_const {
                                self.bindings.insert(name, declarator.children[1]);
                                function.instruction(&Instruction::Drop);
                            } else {
                                function.instruction(&Instruction::Drop);
                            }
                        } else {
                            function.instruction(&Instruction::Drop);
                        }
                    }
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                if is_function_like(&self.program.nodes, id) {
                    // Function declarations are emitted separately from the body scan.
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                self.emit_sequence(function, &node.children, false)
            }
            LirNodeKind::Literal => emit_literal(function, node.text.as_deref(), self.strings),
            LirNodeKind::Value => self.emit_value(function, &node, want_value),
            LirNodeKind::Call => self.emit_call(function, &node),
            LirNodeKind::Branch => match node.text.as_deref() {
                Some(text) if text.starts_with("break") => {
                    self.emit_break_or_continue(function, false, &node)
                }
                Some(text) if text.starts_with("continue") => {
                    self.emit_break_or_continue(function, true, &node)
                }
                Some("for-of") | Some("for-await-of") => {
                    self.emit_for_of_array_iteration(function, &node)
                }
                Some("return") => self.emit_return(function, &node),
                Some("while") | Some("do-while") | Some("for") => {
                    self.emit_loop(function, id, &node)
                }
                Some("for-in") => self.emit_for_in(function, id, &node),
                _ => self.emit_branch(function, &node, want_value),
            },
            LirNodeKind::Unknown => {
                function.instruction(&Instruction::Unreachable);
                EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                }
            }
        }
    }

    /// Base binding name iff `node` is a dynamic array-element read this
    /// emitter routes to `emit_dynamic_array_read{_node}`. SINGLE source of
    /// truth shared by the two dispatch sites in `emit_value` and by the string
    /// oracles (`is_string_valued`, `is_runtime_concat_string`), so the emitter
    /// and the oracles can never drift about which nodes are element reads.
    ///
    /// Mirrors each dispatch site's guard exactly:
    /// - 1-child `[object]` (dot/literal-index): a non-empty index `text` over
    ///   an array binding — EXCLUDING `.length`, which the dedicated length lane
    ///   handles before the dynamic-read dispatch is ever reached (so a `.length`
    ///   node with an array base never routes here, and the oracles must not
    ///   mistake it for an element read).
    /// - 2-child `[object, index]` (computed): not a binary operator, not a
    ///   static-index fold, over an array binding.
    pub(crate) fn dynamic_array_read_base(&self, node: &LirNode) -> Option<String> {
        match node.children.len() {
            1 => {
                let index_text = node.text.as_deref()?;
                if index_text.is_empty() || index_text == "length" {
                    return None;
                }
                let base_name = self.assignment_target_name(node, node.children[0])?;
                self.array_bindings
                    .contains(&base_name)
                    .then_some(base_name)
            }
            2 => {
                if is_binary_operator_text(node.text.as_deref().unwrap_or_default()) {
                    return None;
                }
                if self.resolve_static_index_member(node).is_some() {
                    return None;
                }
                let base_name = self.assignment_target_name(node, node.children[0])?;
                self.array_bindings
                    .contains(&base_name)
                    .then_some(base_name)
            }
            _ => None,
        }
    }

    pub(crate) fn emit_value(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        want_value: bool,
    ) -> EmittedValue {
        if node.text.is_none() {
            return self.emit_aggregate_literal(function, node, want_value);
        }

        // Ternary `test ? a : b` — marker text "?" set by the HIR lowering.
        if node.text.as_deref() == Some("?") && node.children.len() == 3 {
            return self.emit_conditional(function, node, want_value);
        }

        if self.is_supported_callable_reference(node) {
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }

        match node.children.len() {
            0 => {
                if let Some(text) = node.text.as_deref() {
                    if let Some(index) = self.locals.get(text).copied() {
                        function.instruction(&Instruction::LocalGet(index));
                        return EmittedValue {
                            produced: true,
                            shape: ValueShape::Unknown,
                        };
                    }

                    if let Some(bound) = self.bindings.get(text).copied() {
                        return self.emit_node(function, bound, want_value);
                    }

                    // Module-scope binding read from inside a function: inline
                    // a compile-time-pure `const` initializer (all emitters
                    // share one LirProgram node space), and gate every other
                    // module binding — the old path lowered these through a
                    // silent zero placeholder (a wrong answer, not an error).
                    if self.function_name != "_start" {
                        if let Some(&init) = self.module_const_inits.get(text) {
                            if self.is_pure_module_const_init(init, 0) {
                                return self.emit_node(function, init, want_value);
                            }
                        }
                        if self.module_binding_names.contains(text) {
                            self.diagnostics.push(Diagnostic::error(
                                e5::FEATURE_UNAVAILABLE as u32,
                                format!(
                                    "reading module binding '{text}' from a function is only available for compile-time-constant `const` initializers in the current phase"
                                ),
                            ));
                            function.instruction(&Instruction::I64Const(0));
                            return EmittedValue {
                                produced: true,
                                shape: ValueShape::Unknown,
                            };
                        }
                    }

                    if let Some(constant) = parse_number_literal(text) {
                        function.instruction(&Instruction::I64Const(constant));
                        return EmittedValue {
                            produced: true,
                            shape: ValueShape::Scalar,
                        };
                    }

                    match text {
                        "true" => {
                            function.instruction(&Instruction::I64Const(1));
                            return EmittedValue {
                                produced: true,
                                shape: ValueShape::Boolean,
                            };
                        }
                        "false" | "null" | "undefined" => {
                            function.instruction(&Instruction::I64Const(0));
                            return EmittedValue {
                                produced: true,
                                shape: ValueShape::Boolean,
                            };
                        }
                        "Set" | "Map" => {
                            function.instruction(&Instruction::I64Const(0));
                            return EmittedValue {
                                produced: true,
                                shape: ValueShape::Unknown,
                            };
                        }
                        _ => {}
                    }

                    self.push_placeholder_fallback_diagnostic("identifier", text);
                    function.instruction(&Instruction::I64Const(0));
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Unknown,
                    }
                } else {
                    function.instruction(&Instruction::I64Const(0));
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Unknown,
                    }
                }
            }
            1 => {
                if let Some(result) = self.resolve_static_index_member(node) {
                    return self.emit_static_index_member_result(function, result);
                }

                // Runtime array `.length`: this shares the same one-child `Value`
                // shape (`text` = property name, single `object` child) as a
                // dot-access member read. Since bracket-index reads always carry
                // the index as an explicit second child (see the `2 =>` arm), a
                // one-child node's non-empty `text` here is a property name, not
                // an index — so this must be checked (and win) ahead of the
                // generic "dynamic array element read" fallback below, which
                // would otherwise misinterpret "length" as an index text. The
                // length header is always an i64 at `offset: 0` of the same base
                // handle used for element reads, for both i64 and f64 arrays.
                if node.text.as_deref() == Some("length") {
                    let base_id = node.children[0];
                    // Runtime string length: low 32 bits of the tagged handle
                    // (byte count == JS code-unit count for ASCII-provable
                    // strings; `kali_types`'s `reject_unprovable_string_length`
                    // gate rejects everything else). MUST win before the array
                    // interpretation below — repr_infer registers ANY `.length`
                    // receiver as an array binding, and the array lane would
                    // read garbage memory through a tagged handle. Excludes a
                    // receiver resolvable as a static string (`.substring`-free
                    // literal/const fold): that stays on the `emit_unary` fold
                    // lane below, which counts UTF-16 units and is correct for
                    // non-ASCII literals too, whereas the handle byte count is
                    // not.
                    if self.is_string_valued(base_id)
                        && self.resolve_static_object_identity_value(base_id).is_none()
                    {
                        let base = self.emit_node(function, base_id, true);
                        if !base.produced {
                            function.instruction(&Instruction::I64Const(0));
                        }
                        function.instruction(&Instruction::I64Const(0xFFFF_FFFF));
                        function.instruction(&Instruction::I64And);
                        return EmittedValue {
                            produced: true,
                            shape: ValueShape::Scalar,
                        };
                    }
                    if let Some(base_name) = self.assignment_target_name(node, base_id) {
                        if self.array_bindings.contains(&base_name) {
                            self.emit_array_base_address(function, base_id);
                            function.instruction(&Instruction::I64Load(MemArg {
                                offset: 0,
                                align: 3,
                                memory_index: 0,
                            }));
                            return EmittedValue {
                                produced: true,
                                shape: ValueShape::Scalar,
                            };
                        }
                    }
                }

                // Dynamic array element read: `a[i]` where `a` is a linear-memory
                // array. Recognizer shared with the string oracles via
                // `dynamic_array_read_base` (same guard: non-empty, non-`length`
                // index text over an array binding).
                if let Some(base_name) = self.dynamic_array_read_base(node) {
                    let index_text = node.text.as_deref().unwrap_or_default();
                    return self.emit_dynamic_array_read(
                        function,
                        node.children[0],
                        index_text,
                        &base_name,
                    );
                }

                if node.text.as_deref().unwrap_or_default().is_empty() {
                    self.emit_node(function, node.children[0], want_value)
                } else {
                    self.emit_unary(function, node)
                }
            }
            2 => {
                // A computed member access `a[<expr>]` also lowers to a two-child
                // `Value` node (`[object, index]`). A binary expression always
                // carries an operator `text`, while a computed index never
                // stringifies to a bare operator, so `text` cleanly separates the
                // two shapes.
                if is_binary_operator_text(node.text.as_deref().unwrap_or_default()) {
                    return self.emit_binary(function, node);
                }

                // Computed for-in-key read `obj[c]` over a uniform-repr fixed
                // shape (Spec 4a Task 3): a dynamic headerless field slot at
                // `base + c*8`, offset 0. Must precede the static-index and
                // array lanes below — its base is an object (never an array
                // binding) and its index is the loop ordinal, not a literal.
                if let Some((base, index, elem)) = self.computed_forin_object_access(node) {
                    return self.emit_object_field_read_dynamic(function, base, index, elem);
                }

                if let Some(result) = self.resolve_static_index_member(node) {
                    return self.emit_static_index_member_result(function, result);
                }

                // Dynamic linear-memory read `a[<expr>]` when the base is an array
                // binding; otherwise fall back to member handling (e.g. host
                // member chains such as `globalThis["process"]["pid"]`), matching
                // the single-child member path. Recognizer shared with the string
                // oracles via `dynamic_array_read_base` (the binary-operator and
                // static-index-fold cases above have already returned, so the
                // helper's re-checks of them are no-ops here).
                if let Some(base_name) = self.dynamic_array_read_base(node) {
                    return self.emit_dynamic_array_read_node(
                        function,
                        node.children[0],
                        node.children[1],
                        &base_name,
                    );
                }

                self.emit_unary(function, node)
            }
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "array/object aggregate lowering is unavailable in the current phase; use a supported literal shape or the later compatibility path".to_string(),
                ));
                function.instruction(&Instruction::Unreachable);
                EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                }
            }
        }
    }

    pub(crate) fn emit_static_index_member_result(
        &mut self,
        function: &mut Function,
        result: StaticIndexMemberResult,
    ) -> EmittedValue {
        match result {
            StaticIndexMemberResult::Node(value) => self.emit_node(function, value, true),
            StaticIndexMemberResult::String(value) => {
                let (offset, len) = self.strings.intern(&value);
                function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            StaticIndexMemberResult::Undefined => {
                function.instruction(&Instruction::I64Const(0));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
        }
    }

    pub(crate) fn for_of_binding_name(&self, node: &LirNode) -> Option<String> {
        let left = node.children.first().copied()?;
        self.for_of_binding_name_from_node(left)
    }

    pub(crate) fn for_of_binding_name_from_node(&self, id: LirNodeId) -> Option<String> {
        let node = self.node(id);
        if node.children.is_empty() {
            return node.text.clone();
        }

        if matches!(node.text.as_deref(), Some("const" | "let" | "var")) {
            let declarator = node.children.first().copied()?;
            return self.node(declarator).text.clone();
        }

        if node.text.as_deref().is_some_and(|text| text.is_empty()) && !node.children.is_empty() {
            return self
                .for_of_binding_name_from_node(*node.children.last().expect("wrapper child"));
        }

        if node.text.is_none() && node.children.len() == 1 {
            return self.for_of_binding_name_from_node(node.children[0]);
        }

        None
    }

    /// True when `cond` is a bare identifier carrying for-in-key provenance
    /// (`for_in_key_aliases`) — a "key-or-null" whose truthiness is `value >= 0`
    /// (Spec 4a Task 4), not the default `!= 0`.
    pub(crate) fn is_for_in_key_alias_condition(&self, cond: LirNodeId) -> bool {
        self.bare_identifier_name(cond)
            .is_some_and(|name| self.for_in_key_aliases.contains(&name))
    }

    pub(crate) fn emit_branch(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        want_value: bool,
    ) -> EmittedValue {
        let Some(cond) = node.children.first().copied() else {
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        };

        let then_branch = node.children.get(1).copied();
        let else_branch = node.children.get(2).copied();

        self.reject_string_condition(cond);
        let condition = self.emit_node(function, cond, true);
        if !condition.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        // Spec 4a Task 4 null-sentinel truthiness: a for-in-key alias condition
        // (`if (last)`) holds either a real key ordinal (`>= 0`) or the null
        // sentinel `-1`. Truthy iff a real key, i.e. `value >= 0` — NOT the
        // default `!= 0`, which would treat the first-field ordinal `0` as
        // falsy. Recognized structurally via `for_in_key_aliases`.
        if self.is_for_in_key_alias_condition(cond) {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64GeS);
        } else {
            match condition.shape {
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
        }
        let if_index = self.push_control_frame(ControlFlowLabelKind::If);
        function.instruction(&Instruction::If(if want_value {
            BlockType::Result(ValType::I64)
        } else {
            BlockType::Empty
        }));

        if let Some(then_branch) = then_branch {
            let produced = self.emit_node(function, then_branch, want_value);
            if want_value && !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            } else if !want_value && produced.produced {
                function.instruction(&Instruction::Drop);
            }
        } else if want_value {
            function.instruction(&Instruction::I64Const(0));
        }

        if let Some(else_branch) = else_branch {
            function.instruction(&Instruction::Else);
            let produced = self.emit_node(function, else_branch, want_value);
            if want_value && !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            } else if !want_value && produced.produced {
                function.instruction(&Instruction::Drop);
            }
        } else if want_value {
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(0));
        }

        function.instruction(&Instruction::End);
        self.pop_control_frame(ControlFlowLabelKind::If);
        debug_assert!(self.control_frames.get(if_index).is_none());
        EmittedValue {
            produced: want_value,
            shape: ValueShape::Unknown,
        }
    }

    /// `test ? consequent : alternate`: value-producing if/else. Only the
    /// taken arm evaluates (JS semantics — never `select`). Result block type
    /// is repr-directed: f64 when either arm is float-valued (the other arm
    /// promotes), i64 otherwise (ints, booleans, string handles).
    pub(crate) fn emit_conditional(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        want_value: bool,
    ) -> EmittedValue {
        let cond = node.children[0];
        let cons = node.children[1];
        let alt = node.children[2];

        self.reject_string_condition(cond);

        let float_result = want_value && (self.is_float_valued(cons) || self.is_float_valued(alt));
        let string_result =
            want_value && (self.is_string_valued(cons) || self.is_string_valued(alt));
        if float_result && string_result {
            // A float result block would reinterpret a string handle as f64.
            self.diagnostics.push(Diagnostic::error(
                e3::TYPE_MISMATCH as u32,
                "a conditional expression mixing string and float branches is unavailable in the current direct-runtime path".to_string(),
            ));
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        let condition = self.emit_node(function, cond, true);
        if !condition.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        match condition.shape {
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

        let if_index = self.push_control_frame(ControlFlowLabelKind::If);
        function.instruction(&Instruction::If(if want_value {
            BlockType::Result(if float_result {
                ValType::F64
            } else {
                ValType::I64
            })
        } else {
            BlockType::Empty
        }));
        self.emit_conditional_arm(function, cons, want_value, float_result);
        function.instruction(&Instruction::Else);
        self.emit_conditional_arm(function, alt, want_value, float_result);
        function.instruction(&Instruction::End);
        self.pop_control_frame(ControlFlowLabelKind::If);
        debug_assert!(self.control_frames.get(if_index).is_none());

        EmittedValue {
            produced: want_value,
            shape: if !want_value {
                ValueShape::Unknown
            } else if float_result {
                ValueShape::Float
            } else if string_result {
                ValueShape::String
            } else {
                ValueShape::Unknown
            },
        }
    }

    fn emit_conditional_arm(
        &mut self,
        function: &mut Function,
        arm: LirNodeId,
        want_value: bool,
        float_result: bool,
    ) {
        if want_value && float_result {
            // Emits the arm and inserts F64ConvertI64S when it isn't already
            // float — the same promotion `+` uses for mixed operands.
            self.emit_float_operand(function, arm, true);
            return;
        }
        let produced = self.emit_node(function, arm, want_value);
        if want_value && !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        } else if !want_value && produced.produced {
            function.instruction(&Instruction::Drop);
        }
    }
}

#[cfg(test)]
#[path = "control_flow_tests.rs"]
mod control_flow_tests;
