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
                        self.emit_env_restore(function);
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
        self.emit_env_restore(function);
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
        // A loop opens/resets a per-iteration arena if EITHER channel grants its
        // ordinal: the object/array `loop_arena` channel OR the emit-only
        // `string_arena_loop` channel (fasta Spec 7 Task 4f — a loop whose only
        // reclaimable allocation is a granted string site). OR-ing the two
        // getters keeps this a SINGLE `is_arena_loop` decision, so the
        // open/reset/release wiring below fires exactly once even when both
        // channels grant. `string_arena_loop` changes NO routing decision; it
        // only rebinds g1/g2/g3 for the loop's dynamic extent (see its
        // `ArenaTable` doc comment).
        let is_arena_loop = ordinal.is_some_and(|ord| {
            self.arena_table.loop_arena(&self.function_name, ord)
                || self.arena_table.string_arena_loop(&self.function_name, ord)
        });
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

        // Register for-in-key ALIASES (`last = c`, and transitively `y = last`)
        // over this same shape BEFORE emitting the body (Spec 4a Task 4). A
        // computed read `table[last]` inside the body is emitted BEFORE the
        // `last = c` assignment that grants the alias, so without this
        // pre-registration `computed_forin_object_access` would not recognize
        // `table[last]` as a dynamic for-in-key slot. Aliases reference the loop
        // key (directly or through a chain), so they only occur inside this body;
        // iterate to a fixpoint so a multi-level alias `y = last` is registered
        // too — the codegen mirror of the types-side transitive `= <key>`
        // provenance propagation.
        let mut recognized = std::collections::HashSet::new();
        recognized.insert(key_name.clone());
        loop {
            let before = recognized.len();
            let mut next = recognized.clone();
            crate::lower::for_in_key_aliases_walk(
                &self.program.nodes,
                body_id,
                &recognized,
                &mut next,
            );
            recognized = next;
            if recognized.len() == before {
                break;
            }
        }
        for alias in &recognized {
            if alias != &key_name {
                self.for_in_key_shapes.insert(alias.clone(), shape);
            }
        }

        // Spec 4a Task 5 / fasta Spec 7 Task 4g: the per-shape key handle table
        // is MODULE-CONSTANT DATA. Every slot is a compile-time
        // `encode_string_handle` constant, so the whole table is interned once
        // into the string-pool's data-segment layout (`intern_key_table`,
        // deduped by shape) and referenced by a fixed base offset — zero runtime
        // allocation, O(1) in both N and call count, regardless of loop nesting.
        // (The old code bump-allocated `N*8` bytes here on EVERY for-in
        // execution, which for a for-in nested in a loop leaked once per outer
        // iteration.) The key + every recognized alias map to the base in
        // `for_in_key_handle_tables` so a STRING-VALUE use of any of them
        // (`return c`) materializes the interned field-name handle. Aliases
        // enumerate the same shape (same ordered field names), so one table
        // serves all.
        let names: Vec<String> = self
            .repr_table
            .shape_fields(shape)
            .iter()
            .map(|(name, _)| name.clone())
            .collect();
        let handles: Vec<i64> = names
            .iter()
            .map(|name| {
                let (offset, len) = self.strings.intern(name);
                encode_string_handle(offset, len)
            })
            .collect();
        let table_base = self.strings.intern_key_table(shape, &handles);
        for name in &recognized {
            self.for_in_key_handle_tables
                .insert(name.clone(), table_base);
        }

        // preheader: ord = 0
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(ord_local));

        // block (break target) { loop (continue target) { ... } }. Register the
        // labels so a `break`/`continue` inside the body targets THIS for-in
        // (not an enclosing loop). No loop-arena ordinal is involved — this is
        // label bookkeeping only; for..in still takes no arena.
        let break_index = self.push_control_frame(ControlFlowLabelKind::LoopBreak);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_index = self.push_control_frame(ControlFlowLabelKind::LoopContinue);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.loop_frames.push(LoopFrame {
            break_index,
            continue_index,
        });

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
        self.loop_frames.pop();
        self.pop_control_frame(ControlFlowLabelKind::LoopContinue);
        function.instruction(&Instruction::End); // end block
        self.pop_control_frame(ControlFlowLabelKind::LoopBreak);

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

    /// Stage C prologue: if this function owns a PROMOTABLE env (`lower.rs`
    /// reserved its save local because it has >=1 promotable cell — scalar-i64
    /// or C2 fixed-shape object, per `crate::closure::cell_is_promotable`),
    /// save the incoming `current_env` into that save local, allocate this
    /// activation's record (`parent = incoming`) in the global never-reset
    /// region, and publish it into `current_env`. Mirrors the arena-trio
    /// save/alloc idiom, but with ONE global (`CURRENT_ENV_GLOBAL`) and
    /// `__alloc_global` (the record must outlive any arena reset — the env chain
    /// stays valid after a parent activation returns). A function that captures
    /// outer bindings but owns no promotable env of its own allocates nothing
    /// and leaves `current_env` untouched (it reads through the inherited
    /// record). The record is sized by the plan's FULL cell count so every
    /// promoted cell's `derive_env_plans` offset stays valid; a non-promoted
    /// (heap/non-i64) cell's slot is simply left unused this phase.
    pub(crate) fn emit_function_env_prologue(&mut self, function: &mut Function) {
        if !self.owns_promotable_env() {
            return;
        }
        // The module-capture safety argument (module-scope captures are module
        // globals, never env cells) rests on the module root `_start` / `""`
        // never owning an env. `derive_env_plans` guarantees this (the module
        // function's cells are always empty), and the entry never routes through
        // this prologue anyway (it uses `emit_sequence`), but pin the invariant
        // here at the ownership decision point so a future refactor that let the
        // root own an env trips in debug builds instead of silently rebinding
        // `current_env` at module scope.
        debug_assert!(
            self.function_name != "_start" && !self.function_name.is_empty(),
            "module root ('_start'/module scope) must never own a promotable env"
        );
        let cell_count = self.env_plan.cells.len() as u32;
        let env_global = self.current_env_global();
        let save_local = self.locals[&crate::closure::env_save_local_name()];
        let alloc_global_index = self.alloc_global_fn_index();
        function.instruction(&Instruction::GlobalGet(env_global));
        function.instruction(&Instruction::LocalSet(save_local));
        crate::closure::emit_env_alloc(
            function,
            alloc_global_index,
            cell_count,
            env_global,
            save_local,
        );
    }

    /// Restore `current_env` from this function's env save local. Shared by the
    /// fall-through epilogue and every `return` (mirroring the arena unwind), so
    /// EVERY exit path restores the caller's env — a fresh, distinct record per
    /// activation (no leak across calls or recursion). A no-op unless this
    /// function owns a promotable env.
    pub(crate) fn emit_env_restore(&mut self, function: &mut Function) {
        if !self.owns_promotable_env() {
            return;
        }
        let save_local = self.locals[&crate::closure::env_save_local_name()];
        function.instruction(&Instruction::LocalGet(save_local));
        function.instruction(&Instruction::GlobalSet(self.current_env_global()));
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
        self.emit_function_env_prologue(function);
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
        // Fall-through exit: restore the caller's env (mutually exclusive with
        // every `return`'s inline restore — a `return` exits the wasm frame, so
        // this code is unreachable on that path). Stack-neutral, so a
        // fall-through return value beneath it is preserved.
        self.emit_env_restore(function);
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

                        // Binding provenance for a function-VALUED local: if the
                        // initializer resolves to a named closure (`__kali_fn_N`,
                        // renamed in place by `name_anon_functions`), record
                        // `name -> plan key`. Consulted by the scheduling-surface
                        // guard so an indirectly-passed callback (`setTimeout(cb)`)
                        // is resolved to its closure plan by DECLARATION, not name
                        // guessing. Recorded before any early `continue` below so
                        // the mapping is unconditional; harmless for non-fn inits
                        // (the key is only ever looked up for closure args).
                        if let Some(name) = declarator.text.clone() {
                            let init_node = self.node(self.unwrap_transparent(init));
                            if let Some(fn_key) = init_node.text.as_deref() {
                                if self.env_plans.contains_key(fn_key) {
                                    self.fn_valued_locals.insert(name, fn_key.to_string());
                                }
                            }
                        }

                        // Stage D event lane: `const/let t = new EventTarget()`.
                        // This is the ONE position a handle acquires stable
                        // provenance — emit the host construction call, store the
                        // opaque i64 handle into the binding's promoted local, and
                        // record the name so every later read fails closed at the
                        // identifier choke point (handle-escape discipline). The
                        // init node is inspected RAW (never `unwrap_transparent`):
                        // the New node is itself a text-less single-child `Value`,
                        // so unwrapping would strip it to the bare
                        // `Value("EventTarget")` and defeat the recognizer.
                        {
                            let init_node = self.node(init).clone();
                            if self.is_event_target_new(&init_node) {
                                let Some(name) = declarator.text.clone() else {
                                    // A destructuring/no-name declarator holding a
                                    // construction has no binding to carry
                                    // provenance — fail closed, never a leaked
                                    // handle.
                                    self.diagnostics.push(Diagnostic::error(
                                        e5::FEATURE_UNAVAILABLE as u32,
                                        "a 'new EventTarget()' must be bound by a declarator (const t = new EventTarget()); an unbound handle has no stable provenance".to_string(),
                                    ));
                                    function.instruction(&Instruction::Unreachable);
                                    continue;
                                };
                                let Some(import_index) = self.event_target_new_import_index else {
                                    // The program-wide probe gates the import;
                                    // any in-lane construction reaching emit MUST
                                    // have flipped it. A miss is a probe/emit
                                    // desync, not a user error — fail closed.
                                    self.diagnostics.push(Diagnostic::error(
                                        e5::FEATURE_UNAVAILABLE as u32,
                                        "internal: 'new EventTarget()' reached emit without its host import (probe/emit desync)".to_string(),
                                    ));
                                    function.instruction(&Instruction::Unreachable);
                                    continue;
                                };
                                let Some(index) = self.locals.get(&name).copied() else {
                                    // Locals provisioning promotes every in-lane
                                    // construction to a slot; a miss is a
                                    // provisioning bug — fail closed.
                                    self.diagnostics.push(Diagnostic::error(
                                        e5::FEATURE_UNAVAILABLE as u32,
                                        format!(
                                            "EventTarget binding `{name}` must be declared with a local slot"
                                        ),
                                    ));
                                    function.instruction(&Instruction::Unreachable);
                                    continue;
                                };
                                function.instruction(&Instruction::Call(import_index));
                                function.instruction(&Instruction::LocalSet(index));
                                self.event_target_locals.insert(name);
                                continue;
                            }
                        }

                        // Stage P3 abort lane: `const c = new AbortController()`.
                        // Real lowering — an 8-byte global abort cell whose i64
                        // pointer is the handle (controller and signal share it).
                        // ALL of: a `const` declarator whose init is structurally
                        // `new AbortController()`, whose inferred repr is
                        // `AbortHandle` (inference agreed — rules out a shadow
                        // kali_types saw), AND whose ctor name is unshadowed in
                        // every codegen namespace (defense in depth over the
                        // program-wide inference gate; the `host.rs` five-namespace
                        // shape). Any condition failing falls through to the
                        // unchanged placeholder path (so `let`/mixed-provenance
                        // constructions keep building and their ops fail closed
                        // later).
                        if is_const {
                            if let Some(name) = declarator.text.clone() {
                                if crate::lower::declarator_init_is_abort_controller_new(
                                    &self.program.nodes,
                                    init,
                                ) && self.scalar_repr(&name) == kali_common::Repr::AbortHandle
                                    && !(self.locals.contains_key("AbortController")
                                        || self.bindings.contains_key("AbortController")
                                        || self.module_binding_names.contains("AbortController")
                                        || self.fn_valued_locals.contains_key("AbortController")
                                        || self.functions.contains_key("AbortController"))
                                {
                                    // 8-byte abort cell on the never-reclaimed
                                    // global heap; explicit zero store — do not
                                    // rely on allocator zeroing. Stash the handle
                                    // in the general-purpose scratch local, then
                                    // bind it (plain local or promoted env cell),
                                    // mirroring the C2 object declarator dispatch
                                    // below.
                                    let scratch = self.locals.len() as u32;
                                    function.instruction(&Instruction::I32Const(8));
                                    function.instruction(&Instruction::Call(
                                        self.alloc_global_fn_index(),
                                    ));
                                    function.instruction(&Instruction::I64ExtendI32U);
                                    function.instruction(&Instruction::LocalTee(scratch));
                                    function.instruction(&Instruction::I32WrapI64);
                                    function.instruction(&Instruction::I64Const(0));
                                    function.instruction(&Instruction::I64Store(MemArg {
                                        offset: 0,
                                        align: 3,
                                        memory_index: 0,
                                    }));
                                    // Handle (i64) back on the stack for the bind
                                    // dispatch.
                                    function.instruction(&Instruction::LocalGet(scratch));
                                    if let Some(index) = self.locals.get(&name).copied() {
                                        function.instruction(&Instruction::LocalSet(index));
                                    } else if let Some((depth, offset)) =
                                        self.resolve_capture_access(&name)
                                    {
                                        let env_global = self.current_env_global();
                                        let scratch2 = self.locals.len() as u32;
                                        crate::closure::emit_cell_store(
                                            function, env_global, depth, offset, scratch2,
                                        );
                                    } else {
                                        function.instruction(&Instruction::Drop);
                                    }
                                    self.abort_handle_locals.insert(name);
                                    continue;
                                }
                            }
                        }

                        // Stage P4 URL/URLSearchParams lane:
                        // `const u = new URL(<string-literal>)` and
                        // `const q = new URLSearchParams(<string-literal>)`. A
                        // structural `new URL(...)` / `new URLSearchParams(...)`
                        // with the UNSHADOWED builtin ctor is intercepted here:
                        // an admittable single-string-literal arg that parses is
                        // materialized (URL → 48-byte arena struct of interned
                        // component handles + embedded USP; USP → growable
                        // pair-store) and its handle bound; ANY non-admittable
                        // shape (non-literal `new URL(s)` / multi-arg
                        // `new URL(rel, base)` / non-parseable literal) is denied
                        // E5506 instead of silently lowering to the `0`
                        // placeholder — the raw handle representation must never
                        // escape as a bare `0` (reject-don't-miscompile). A
                        // user-shadowed ctor falls through to the normal call lane.
                        if is_const {
                            if let Some(name) = declarator.text.clone() {
                                if crate::lower::declarator_init_is_url_ctor(
                                    &self.program.nodes,
                                    init,
                                    "URL",
                                ) && self.url_ctor_unshadowed("URL")
                                {
                                    let admitted = if self.scalar_repr(&name)
                                        == kali_common::Repr::Url
                                    {
                                        crate::lower::new_ctor_string_literal_arg(
                                            &self.program.nodes,
                                            init,
                                            "URL",
                                        )
                                        .and_then(|t| {
                                            crate::lower::parse_url_literal(
                                                crate::strip_string_delimiters(&t),
                                            )
                                        })
                                    } else {
                                        None
                                    };
                                    if let Some(components) = admitted {
                                        self.emit_url_construction(function, &components);
                                        if let Some(index) = self.locals.get(&name).copied() {
                                            function.instruction(&Instruction::LocalSet(index));
                                        } else if let Some((depth, offset)) =
                                            self.resolve_capture_access(&name)
                                        {
                                            let env_global = self.current_env_global();
                                            let scratch2 = self.locals.len() as u32;
                                            crate::closure::emit_cell_store(
                                                function, env_global, depth, offset, scratch2,
                                            );
                                        } else {
                                            function.instruction(&Instruction::Drop);
                                        }
                                        self.url_locals.insert(name);
                                        continue;
                                    }
                                    self.deny_e5506(
                                        function,
                                        "a URL can only be constructed from a single \
                                         string-literal argument that parses as an absolute URL \
                                         in the current phase (fail-closed)",
                                    );
                                    continue;
                                }
                                if crate::lower::declarator_init_is_url_ctor(
                                    &self.program.nodes,
                                    init,
                                    "URLSearchParams",
                                ) && self.url_ctor_unshadowed("URLSearchParams")
                                {
                                    let admitted = if self.scalar_repr(&name)
                                        == kali_common::Repr::UrlSearchParams
                                    {
                                        crate::lower::new_ctor_string_literal_arg(
                                            &self.program.nodes,
                                            init,
                                            "URLSearchParams",
                                        )
                                        .map(|t| {
                                            crate::lower::parse_query_literal(
                                                crate::strip_string_delimiters(&t),
                                            )
                                        })
                                    } else {
                                        None
                                    };
                                    if let Some(pairs) = admitted {
                                        self.emit_usp_store_from_pairs(function, &pairs);
                                        if let Some(index) = self.locals.get(&name).copied() {
                                            function.instruction(&Instruction::LocalSet(index));
                                        } else if let Some((depth, offset)) =
                                            self.resolve_capture_access(&name)
                                        {
                                            let env_global = self.current_env_global();
                                            let scratch2 = self.locals.len() as u32;
                                            crate::closure::emit_cell_store(
                                                function, env_global, depth, offset, scratch2,
                                            );
                                        } else {
                                            function.instruction(&Instruction::Drop);
                                        }
                                        self.usp_locals.insert(name);
                                        continue;
                                    }
                                    self.deny_e5506(
                                        function,
                                        "a URLSearchParams can only be constructed from a single \
                                         string-literal argument in the current phase \
                                         (fail-closed)",
                                    );
                                    continue;
                                }
                            }
                        }

                        // Stage P3 Task 4: `const s = c.signal` alias. The signal
                        // shares the controller's handle cell (identity), so
                        // binding `s` to the receiver handle makes `s.aborted`
                        // read — and `s` deny as a raw value — through the SAME
                        // provenance as the controller. Gated on inference agreeing
                        // (`scalar_repr == AbortHandle`) AND a structurally proven
                        // `<ident>.signal` init over an abort handle; the receiver
                        // flows through the sole admitted read
                        // (`emit_abort_receiver_handle`), then binds with the same
                        // local/promoted-cell discipline as the controller arm.
                        if is_const {
                            if let Some(name) = declarator.text.clone() {
                                let init_node = self.node(init);
                                let init_is_signal_alias = init_node.kind == LirNodeKind::Value
                                    && init_node.children.len() == 1
                                    && init_node.text.as_deref() == Some("signal")
                                    && {
                                        let base = self.node(init_node.children[0]);
                                        base.children.is_empty()
                                            && base
                                                .text
                                                .as_deref()
                                                .is_some_and(|n| self.is_abort_handle(n))
                                    };
                                if init_is_signal_alias
                                    && self.scalar_repr(&name) == kali_common::Repr::AbortHandle
                                {
                                    let base_id = self.node(init).children[0];
                                    let handle = self.emit_abort_receiver_handle(function, base_id);
                                    if !handle.produced {
                                        function.instruction(&Instruction::I64Const(0));
                                    }
                                    if let Some(index) = self.locals.get(&name).copied() {
                                        function.instruction(&Instruction::LocalSet(index));
                                    } else if let Some((depth, offset)) =
                                        self.resolve_capture_access(&name)
                                    {
                                        let env_global = self.current_env_global();
                                        let scratch2 = self.locals.len() as u32;
                                        crate::closure::emit_cell_store(
                                            function, env_global, depth, offset, scratch2,
                                        );
                                    } else {
                                        function.instruction(&Instruction::Drop);
                                    }
                                    self.abort_handle_locals.insert(name);
                                    continue;
                                }
                            }
                        }

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
                                    } else if let Some((depth, offset)) =
                                        self.resolve_capture_access(&name)
                                    {
                                        // Stage C C2: a captured object binding was
                                        // promoted out of its local into the owner's
                                        // env cell (`lower.rs`, same owner-keyed
                                        // predicate). Store the freshly-allocated
                                        // base pointer into the env record so the
                                        // capturer reads a live pointer, not a dropped
                                        // one. This is a DECLARATION of the binding in
                                        // its owner, so it always resolves to the
                                        // owner's own cell (`depth` 0); `depth` is
                                        // threaded for uniformity with the other cell
                                        // access sites.
                                        let env_global = self.current_env_global();
                                        let scratch = self.locals.len() as u32;
                                        crate::closure::emit_cell_store(
                                            function, env_global, depth, offset, scratch,
                                        );
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

                        // Growable runtime array declarator (throw-fallout
                        // Stage 4): `const/let x = []` / `[seed…]` promoted
                        // by the types-side growable gate lowers to a real
                        // header+data allocation with the tagged handle in
                        // the binding's local — never the aggregate no-op
                        // fold lane. Deliberately BEFORE the object-array
                        // branch below: the two lanes are disjoint by the
                        // promotion gate (i64 elements only), and the
                        // growable oracle must win for its bindings.
                        if let Some(name) = declarator.text.clone() {
                            if self.is_growable_array(&name) {
                                let aggregate = self
                                    .resolve_literal_aggregate(init)
                                    .map(|id| self.node(id).clone())
                                    .filter(|node| self.is_array_literal(node));
                                let (Some(aggregate), Some(index)) =
                                    (aggregate, self.locals.get(&name).copied())
                                else {
                                    // Promotion admitted exactly this shape;
                                    // anything else here is a gate/provisioning
                                    // bug — fail closed, never a silent no-op.
                                    self.diagnostics.push(Diagnostic::error(
                                        e5::FEATURE_UNAVAILABLE as u32,
                                        format!(
                                            "growable array `{name}` must be declared with an array-literal initializer and a local slot"
                                        ),
                                    ));
                                    function.instruction(&Instruction::Unreachable);
                                    continue;
                                };
                                let seed_len = aggregate.children.len();
                                let cap = seed_len.max(crate::emit::growable::GROWABLE_INITIAL_CAP);
                                let allocated = self.emit_growable_alloc(function, seed_len, cap);
                                if !allocated.produced {
                                    function.instruction(&Instruction::I64Const(0));
                                }
                                function.instruction(&Instruction::LocalSet(index));
                                // Seed elements: *(data_ptr + i*8) = seed_i.
                                // The promotion gate admits only scalar-shaped
                                // (never float/string/object) seeds.
                                for (i, child) in aggregate.children.iter().copied().enumerate() {
                                    function.instruction(&Instruction::LocalGet(index));
                                    function.instruction(&Instruction::I64Const(
                                        !(crate::ARRAY_HANDLE_TAG) as i64,
                                    ));
                                    function.instruction(&Instruction::I64And);
                                    function.instruction(&Instruction::I32WrapI64);
                                    function.instruction(&Instruction::I64Load(MemArg {
                                        offset: 16,
                                        align: 3,
                                        memory_index: 0,
                                    }));
                                    function.instruction(&Instruction::I32WrapI64);
                                    let produced = self.emit_node(function, child, true);
                                    if !produced.produced {
                                        function.instruction(&Instruction::I64Const(0));
                                    }
                                    function.instruction(&Instruction::I64Store(MemArg {
                                        offset: (i * 8) as u64,
                                        align: 3,
                                        memory_index: 0,
                                    }));
                                }
                                continue;
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
                                && self.is_null_or_undefined_expr(init)
                            {
                                if let Some(index) = self.locals.get(&name).copied() {
                                    function.instruction(&Instruction::I64Const(-1));
                                    function.instruction(&Instruction::LocalSet(index));
                                    continue;
                                }
                            }
                        }

                        // Module-scope mutable scalar promoted to a global: its
                        // MODULE declarator (in `_start`) stores its init through
                        // `GlobalSet`, not a local slot. Gated on the name NOT
                        // being a local of THIS function: a same-named `var`/`let`
                        // inside another function is a distinct local (it is in
                        // that function's `locals`), so it must fall through to
                        // the normal local store below and NOT clobber the module
                        // global. In `_start` the promoted name is filtered out of
                        // its locals, so this fires only for the real module
                        // declarator. A no-initializer `var g;` skips here (the
                        // `children.len() < 2` guard above `continue`d); the global
                        // is zero-initialized in the `GlobalSection`.
                        if let Some(name) = declarator.text.clone() {
                            if let (false, Some(&(global_index, repr))) = (
                                self.locals.contains_key(&name),
                                self.module_global_slots.get(&name),
                            ) {
                                let is_f64 = repr == kali_common::Repr::F64;
                                let produced = self.emit_node(function, init, true);
                                if !produced.produced {
                                    if is_f64 {
                                        function.instruction(&Instruction::F64Const(0.0.into()));
                                    } else {
                                        function.instruction(&Instruction::I64Const(0));
                                    }
                                } else if is_f64 && !self.is_float_valued(init) {
                                    function.instruction(&Instruction::F64ConvertI64S);
                                }
                                function.instruction(&Instruction::GlobalSet(global_index));
                                continue;
                            }
                        }

                        // Stage C: a captured scalar promoted to an env cell has
                        // no WASM local slot — its initializer is stored into the
                        // owner's env cell (depth 0), not a `LocalSet`.
                        // `try_emit_captured_decl` returns `Some` iff `name` is an
                        // own cell (handled, or rejected E5506 for a non-i64/heap
                        // cell), so a promoted binding never falls through to the
                        // generic store below (which would `Drop` its value).
                        if let Some(name) = declarator.text.clone() {
                            if self.try_emit_captured_decl(function, &name, init).is_some() {
                                continue;
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
                            // A `const` on the fold lane has no slot, so its
                            // reads re-emit the recorded init node. One
                            // promoted by the STABILITY allowlist gets BOTH: the
                            // slot carries the runtime value (bound exactly once
                            // here), and the `bindings` entry carries the
                            // compile-time denotation the alias/intrinsic
                            // analyses need. Reads are unaffected — the
                            // identifier path consults `locals` before
                            // `bindings`.
                            //
                            // A HANDLE-promoted `const` is excluded on purpose:
                            // its lane keys on a provenance set, and a
                            // denotation entry would re-resolve the name to its
                            // initializer.
                            if is_const
                                && (!self.locals.contains_key(&name)
                                    || self.allowlist_promoted_consts.contains(&name))
                            {
                                self.bindings.insert(name.clone(), init);
                            }
                            if let Some(index) = self.locals.get(&name).copied() {
                                // `let`/`var`, or a promoted `const` — store
                                // eagerly at the declaration site.
                                function.instruction(&Instruction::LocalSet(index));
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
            LirNodeKind::Value => self.emit_value(function, id, &node, want_value),
            LirNodeKind::Call => self.emit_call(function, id, &node),
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
                Some("throw") => self.emit_throw(function, &node),
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
        id: LirNodeId,
        node: &LirNode,
        want_value: bool,
    ) -> EmittedValue {
        if node.text.is_none() {
            // Stage D event lane: a `new EventTarget()` reaching the generic
            // value path is OUTSIDE a declarator-init (a bare expression
            // statement, an assignment RHS, or a call argument) — the
            // declarator construction lane intercepts and `continue`s before
            // any init reaches here. Such a construction has no binding to carry
            // its opaque handle's provenance, so fail closed rather than emit an
            // untracked handle (or the drop-and-push-0 aggregate placeholder,
            // which would silently discard the constructor).
            if self.is_event_target_new(node) {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "a 'new EventTarget()' must be bound by a declarator (const t = new EventTarget()); an unbound handle has no stable provenance".to_string(),
                ));
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            // `new TextEncoder().encode(<string>)` (throw-fallout Stage 3 bucket #6
            // part 2): the parser hoists the `new` to wrap the whole member-call
            // chain, so this arrives as a text-less 1-child wrapper (the `new`)
            // around the `.encode` `Call`. The generic text-less aggregate path
            // below DROPS its operand and pushes `0` (the "unsupported `new`
            // returns an empty object" placeholder) — which would discard the
            // encoded byte buffer and store `0` into the binding, so the digest
            // input reads empty. Instead, pass through to the encode call (whose
            // emit arm reinterprets the string handle to a contiguous byte
            // buffer). Mirrors the `await` marker passthrough below. Scoped
            // strictly to a child whose callee is `is_text_encoder_encode`, so a
            // genuine `new SomeClass()` (or any other text-less aggregate) keeps
            // the drop-and-push-0 fallback.
            if node.children.len() == 1 {
                let child = node.children[0];
                let child_node = self.node(child).clone();
                if child_node.kind == LirNodeKind::Call {
                    if let Some(callee) = child_node.children.first().copied() {
                        let callee_node = self.node(callee).clone();
                        if self.is_text_encoder_encode(&callee_node) {
                            return self.emit_node(function, child, want_value);
                        }
                    }
                }
            }
            return self.emit_aggregate_literal(function, node, want_value);
        }

        // Ternary `test ? a : b` — marker text "?" set by the HIR lowering.
        if node.text.as_deref() == Some("?") && node.children.len() == 3 {
            return self.emit_conditional(function, node, want_value);
        }

        // `await <operand>` — marker text "await" set by the HIR lowering
        // (throw-fallout Stage 3 Task 4). Kali has no microtask machinery and no
        // genuinely-pending promise in the current phase, so every operand settles
        // synchronously: the await's value IS the operand's value. Pass it through
        // by emitting the child and KEEPING its produced value (the historical
        // text-less aggregate path dropped it and pushed `0`, so `await
        // Promise.resolve(7)` yielded 0). This is fully transparent — it never
        // masks the operand's own failure mode, since emitting the child still hits
        // whatever reject/trap that child would hit on its own.
        if node.text.as_deref() == Some("await") && node.children.len() == 1 {
            return self.emit_node(function, node.children[0], want_value);
        }

        // Bare value-position `process.kill` (uncalled member reference, e.g.
        // `!process.kill`): Node exposes `process.kill` as a function, which is
        // truthy. Kali has no first-class function values, so the historical
        // path lowered this member read to a `0` placeholder — making the
        // supported liveness-probe guards `!process.kill` throw. Emit a truthy
        // sentinel scoped EXACTLY to the `process.kill` receiver shapes the call
        // arm recognizes (`is_process_kill`), so no other member read is
        // affected. (Known latent divergence: `console.log(process.kill)` would
        // print `1`, not `[Function: kill]`; no fixture reads a bare
        // `process.kill` outside truthiness position.)
        if self.is_process_kill(node) {
            function.instruction(&Instruction::I64Const(1));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
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
                    // Stage D event-lane handle-escape choke point (spec §2.4):
                    // a binding holding an opaque `new EventTarget()` handle may
                    // ONLY be consumed as the receiver of
                    // addEventListener/dispatchEvent — and Task 4's emit arms
                    // read that receiver directly (a local/global access), never
                    // through this generic identifier lane. Every OTHER read
                    // (`console.log(t)`, `t` as an argument, arithmetic on `t`,
                    // a `return t`) reaches HERE and fails closed: the raw i64
                    // handle must never escape as an observable value. Allowlist
                    // safe positions at the single read site, don't denylist
                    // sinks (the Spec-4a lesson) — the deny is total by
                    // construction because the only allowed consumer bypasses
                    // this lane entirely.
                    if self.event_target_locals.contains(text) {
                        self.diagnostics.push(Diagnostic::error(
                            e5::FEATURE_UNAVAILABLE as u32,
                            format!(
                                "'{text}' holds an EventTarget handle, which may only be used as the \
                                 receiver of addEventListener/dispatchEvent in the current phase; any \
                                 other use would leak the internal handle representation"
                            ),
                        ));
                        return EmittedValue {
                            produced: false,
                            shape: ValueShape::Unknown,
                        };
                    }
                    // Stage P3 abort-handle escape choke point (spec §3): a bare
                    // read of an AbortController/AbortSignal handle is E5506
                    // unless an allowlisted consumer set `admit_abort_handle_read`
                    // while emitting its receiver (`emit_abort_receiver_handle`).
                    // The raw i64 pointer must never escape as an observable
                    // value (`console.log(c)`, `c` as an argument, arithmetic,
                    // `return c`). Allowlist the safe position at the single read
                    // site, don't denylist sinks — the deny is total by
                    // construction because every admitted consumer sets the flag.
                    if self.is_abort_handle(text) && !self.admit_abort_handle_read {
                        return self.deny_e5506(
                            function,
                            "an AbortController/AbortSignal handle cannot be read in this \
                             position: kali admits it only as an `abort()`/`signal`/`aborted` \
                             receiver or a `const s = c.signal` alias (fail-closed)",
                        );
                    }
                    // Task 8 round-2 read-position twin of the call-side gate: a
                    // BARE read (`console.log(c)`, `c` as an argument) of a
                    // `_start`-owned abort handle reached from a non-`_start`
                    // emitter. `is_abort_handle` excludes the `_start` owner (its
                    // captured env cell is never populated — the declarator
                    // intercept bound it as a plain `_start` LOCAL), so absent
                    // this gate the read falls through to the generic
                    // module-binding / placeholder lanes and silently yields 0
                    // (the raw-print REGRESSION `console.log(c)` → `0`). Mirror
                    // the method-call `is_module_scope_abort_handle` deny.
                    if self.is_module_scope_abort_handle(text) {
                        return self.deny_e5506(
                            function,
                            "an AbortController/AbortSignal handle declared at module scope \
                             (`_start`) cannot cross the module/function boundary as a value in \
                             the current phase; reading it from inside a function/closure fails \
                             closed (fail-closed)",
                        );
                    }
                    // Stage P4 URL/URLSearchParams escape choke point (spec §3):
                    // a bare read of a URL/USP handle is E5506 unless an
                    // allowlisted consumer set `admit_url_handle_read` while
                    // emitting its receiver (`emit_url_receiver_handle`). The raw
                    // i64 handle (a URL struct pointer or a tagged USP pair-store)
                    // must never escape as an observable value
                    // (`console.log(u)`, `u` as an argument, `return u`).
                    // Allowlist the safe position at the single read site, don't
                    // denylist sinks — the deny is total by construction because
                    // every admitted consumer sets the flag.
                    if (self.is_url(text) || self.is_url_search_params(text))
                        && !self.admit_url_handle_read
                    {
                        return self.deny_e5506(
                            function,
                            "a URL/URLSearchParams handle cannot be read in this position: kali \
                             admits it only as a recognized method/component receiver \
                             (fail-closed)",
                        );
                    }
                    // Read-position twin of the module-scope abort gate: a bare
                    // read of a `_start`-owned URL/USP handle from a non-`_start`
                    // emitter (arena lifetime unproven across frames, never
                    // captured into the callee env). Deny fail-closed.
                    if self.is_module_scope_url_handle(text) {
                        return self.deny_e5506(
                            function,
                            "a URL/URLSearchParams handle declared at module scope (`_start`) \
                             cannot cross the module/function boundary as a value in the current \
                             phase; reading it from inside a function/closure fails closed \
                             (fail-closed)",
                        );
                    }
                    // Spec 4a Task 5: a for-in key (or alias) emitted in a
                    // STRING-VALUE context materializes its interned field-name
                    // handle from this loop's key handle table at `base + ord*8`,
                    // instead of yielding the raw ordinal local. PROVENANCE is
                    // structural (`for_in_key_handle_tables` membership, the
                    // Task-4 threaded signal — not re-derived from repr); the
                    // STRING context is the key's scalar repr being `String`
                    // (lifted by `repr_infer` only where the key flows into a
                    // string sink). The ORDINAL uses — computed index
                    // (`table[c]`, via `emit_forin_key_ordinal`), `if`-truthiness
                    // (`emit_branch`), and the alias ordinal-copy (`last = c`) —
                    // never reach here as strings, so this arm is the sole
                    // string-materialization site (the dual-role disambiguation).
                    if let Some(&table_base) = self.for_in_key_handle_tables.get(text) {
                        if self.scalar_repr(text) == kali_common::Repr::String {
                            if let Some(&ord_local) = self.locals.get(text) {
                                // addr = table_base(const) + ord*8, load the i64
                                // handle. `table_base` is now a compile-time
                                // data-segment offset (fasta Spec 7 Task 4g), not
                                // a runtime local holding a bump-allocated base.
                                function.instruction(&Instruction::I32Const(table_base as i32));
                                function.instruction(&Instruction::LocalGet(ord_local));
                                function.instruction(&Instruction::I32WrapI64);
                                function.instruction(&Instruction::I32Const(8));
                                function.instruction(&Instruction::I32Mul);
                                function.instruction(&Instruction::I32Add);
                                function.instruction(&Instruction::I64Load(MemArg {
                                    offset: 0,
                                    align: 3,
                                    memory_index: 0,
                                }));
                                return EmittedValue {
                                    produced: true,
                                    shape: ValueShape::String,
                                };
                            }
                        }
                    }
                    // Module-scope mutable scalar promoted to a persistent
                    // global (see `collect_module_scalar_globals`): read it with
                    // `GlobalGet` from a function OR module scope. Gated on the
                    // name NOT being a local/param FIRST — a param or local
                    // `var`/`let` that shadows a same-named module global wins
                    // (JS lexical scoping), so this must yield to the local
                    // lookup below. (In `_start` a promoted name is never a
                    // local — it is filtered out of `_start`'s locals — so this
                    // still fires there.) Also wins ahead of the E5506
                    // module-binding gate, which only handled the inline-const case.
                    if !self.locals.contains_key(text) {
                        if let Some(&(global_index, repr)) = self.module_global_slots.get(text) {
                            function.instruction(&Instruction::GlobalGet(global_index));
                            return EmittedValue {
                                produced: true,
                                shape: if repr == kali_common::Repr::F64 {
                                    ValueShape::Float
                                } else {
                                    ValueShape::Scalar
                                },
                            };
                        }
                    }

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

                    // Stage C: a bare identifier resolving to neither a local,
                    // module global, nor module binding may be a captured scalar
                    // promoted to an env cell (own cell, or a single-level
                    // synchronous outer capture). MUST precede the zero
                    // placeholder — an in-plan name that this returns for is
                    // either a real cell load or an E5506 reject, never a silent
                    // zero.
                    if let Some(value) = self.try_emit_captured_read(function, text) {
                        return value;
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
                    // Growable-array FIELD `.length` (`o.values.length`, Stage
                    // P2 Lane 1 Task 5): the field slot holds the tagged handle
                    // (Task 3), so `emit_growable_length` emits the field read
                    // then reads `hdr.len` — same lane as the named case, keyed
                    // on the positive `object_field_is_growable_array` proof.
                    // A scalar field (`o.count.length`) or a nested chain proves
                    // false and keeps its existing route.
                    if self.object_field_is_growable_array(base_id) {
                        return self.emit_growable_length(function, base_id);
                    }
                    // Stage P4 Task 4: `q.getAll('k').length`. The base is a CALL
                    // producing a fresh tagged growable handle — neither a named
                    // growable binding nor a growable field — so route it through
                    // `emit_growable_length`, whose base emit leaves the tagged
                    // handle before the `hdr.len` decode.
                    if self.is_usp_getall_call(base_id) {
                        return self.emit_growable_length(function, base_id);
                    }
                    if let Some(base_name) = self.assignment_target_name(node, base_id) {
                        // Growable runtime array `.length` (throw-fallout
                        // Stage 4): decode the tagged handle, read `hdr.len`.
                        // Must win before the plain-array lane — the two
                        // layouts differ (tagged header vs inline base).
                        if self.is_growable_array(&base_name) {
                            return self.emit_growable_length(function, base_id);
                        }
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

                // Typed-array `.byteLength` (throw-fallout Stage 3 bucket #6): for
                // the `Uint8Array` representation (an i64-element linear-memory
                // array; see `is_array_like_constructor`) `byteLength` equals the
                // element count, so it reads the same i64 length header at `+0` of
                // the base handle as `.length`. Only fires for a known array
                // binding; other receivers fall through to their existing paths.
                if node.text.as_deref() == Some("byteLength") {
                    let base_id = node.children[0];
                    // String-backed byte buffer (throw-fallout Stage 3 bucket #6
                    // part 2): `new TextEncoder().encode(<string>)` and
                    // `crypto.subtle.digest(...)` both produce tagged string
                    // handles (`STRING_HANDLE_TAG | (buf << 32) | len`) whose low
                    // 32 bits are the byte count. `byteLength` reads it the same
                    // way `.length` does for a string handle (mirrors the string
                    // `.length` arm above). MUST win before the array
                    // interpretation below — these bindings resolve `Repr::String`,
                    // not an array handle. Excludes a static-string receiver (none
                    // arises here, but symmetric with the `.length` guard).
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

                // Growable runtime array element read `x[i]` (throw-fallout
                // Stage 4): literal/identifier index in `text`. Must win
                // before the plain-array recognizer below (disjoint oracles;
                // a growable base is never in `array_bindings`) and before
                // the generic unary fallback that would silently mis-emit.
                if self.growable_array_read_base(node).is_some()
                    || self.growable_field_read_base(node)
                {
                    let index_text = node.text.as_deref().unwrap_or_default().to_string();
                    let index_node =
                        self.alloc_scratch_node(LirNodeKind::Value, Some(index_text), vec![]);
                    return self.emit_growable_index_read(function, node.children[0], index_node);
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

                // Stage P3 Task 4: member reads on a proven abort handle.
                // `.aborted` is a real load of the shared cell (Boolean shape);
                // `.signal` is identity, admitted ONLY under an enclosing admitted
                // consumer (`admit_abort_handle_read`, set by the declarator alias
                // arm or a future `instanceof` left operand) — otherwise E5506, so
                // an AbortSignal never escapes as a value. Recognized BEFORE
                // `emit_unary`'s growable-field gate below so the two allowlists
                // stay independent. Any OTHER field on a proven handle is NOT
                // recognized here (`abort_member_read_parts` returns `None`) and
                // falls through to the generic member fallback, whose receiver
                // emit hits the identifier choke point and denies E5506
                // (default-deny — closes the t3-m2 silent-`0` hole).
                // Task 8 round-2 read-position twin: a member read
                // (`c.signal`, `c.signal.aborted`, or any field) whose ultimate
                // receiver is a `_start`-owned abort handle reached from a
                // non-`_start` emitter. `abort_member_read_parts` returns `None`
                // for it (`is_abort_handle` excludes the `_start` owner), so
                // without this gate the read falls through to the generic member
                // fallback and silently yields `0`. Deny fail-closed, mirroring
                // the call-side `is_module_scope_abort_handle` gate.
                if self.member_receiver_is_module_abort_handle(id) {
                    return self.deny_e5506(
                        function,
                        "an AbortController/AbortSignal handle declared at module scope \
                         (`_start`) cannot be read from inside a function/closure in the \
                         current phase; its `.signal`/`.aborted` cross the module/function \
                         boundary and fail closed (fail-closed)",
                    );
                }
                if let Some(part) = self.abort_member_read_parts(id) {
                    match part {
                        crate::emit::abort::AbortMemberRead::Aborted(receiver) => {
                            let handle = self.emit_abort_receiver_handle(function, receiver);
                            if !handle.produced {
                                function.instruction(&Instruction::I64Const(0));
                            }
                            self.emit_abort_cell_load(function);
                            return EmittedValue {
                                produced: true,
                                shape: ValueShape::Boolean,
                            };
                        }
                        crate::emit::abort::AbortMemberRead::Signal(receiver) => {
                            if self.admit_abort_handle_read {
                                return self.emit_abort_receiver_handle(function, receiver);
                            }
                            return self.deny_e5506(
                                function,
                                "an AbortSignal cannot escape as a value: admitted \
                                 positions are `.aborted`, `instanceof AbortSignal`, and \
                                 `const s = c.signal` (fail-closed)",
                            );
                        }
                    }
                }

                // Stage P4: URL component member reads. `u.href`/`.origin`/
                // `.pathname`/`.search`/`.hash` are pure loads of an interned
                // string handle from the parsed arena struct; the receiver flows
                // through the sole admitted read (`emit_url_receiver_handle`).
                // `u.searchParams` is RECOGNIZED here (Task 4's
                // `u.searchParams.get(...)` composition consults this recognizer
                // via the method path in `call.rs`) but as a BARE VALUE read it
                // fails closed: loading slot 5 would leak the raw USP handle
                // integer into value sinks (`console.log(u.searchParams)`,
                // `u.searchParams + 'z'`) — a plausible-value misrender (node
                // prints `URLSearchParams { … }`). Admitted only as a method
                // receiver; the alias bind (`const sp = u.searchParams`) lands in
                // Task 4. Any OTHER field on a proven URL, or a member on a
                // non-URL base, is NOT recognized (`url_member_read_parts` returns
                // `None`) and falls through — its receiver emit hits the
                // identifier choke point and denies (default-deny).
                if let Some((base_id, member)) = self.url_member_read_parts(id) {
                    if matches!(member, crate::emit::url::UrlMember::SearchParams) {
                        return self.deny_e5506(
                            function,
                            "reading `.searchParams` as a bare value is not supported; it is \
                             admitted only as a method receiver such as `u.searchParams.get(...)` \
                             (fail-closed)",
                        );
                    }
                    let handle = self.emit_url_receiver_handle(function, base_id);
                    if !handle.produced {
                        function.instruction(&Instruction::I64Const(0));
                    }
                    let slot = FunctionEmitter::url_member_slot(&member);
                    self.emit_url_slot_load(function, slot);
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
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
                    return self.emit_binary(function, id, node);
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

                // Growable runtime array computed read `x[<expr>]`
                // (throw-fallout Stage 4) — the 2-child twin of the 1-child
                // growable arm above; same ordering rationale.
                if self.growable_array_read_base(node).is_some()
                    || self.growable_field_read_base(node)
                {
                    return self.emit_growable_index_read(
                        function,
                        node.children[0],
                        node.children[1],
                    );
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

        if (node.text.is_none() || node.text.as_deref() == Some("await"))
            && node.children.len() == 1
        {
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

        // Spec 4a Task 4/5: a for-in-key alias condition (`if (last)`) is ALWAYS
        // the raw ordinal truthiness (`>= 0`), never a string. Read its ordinal
        // local DIRECTLY, bypassing `emit_value`'s string-materialization arm —
        // otherwise a key that is ALSO string-used elsewhere (scalar repr lifted
        // to `String`) would materialize a handle here and truthiness-test it.
        // (For a key never string-used this is byte-identical to `emit_node`,
        // which resolves the same `LocalGet` ordinal.) `reject_string_condition`
        // is skipped for the same reason: the condition is an ordinal, not a
        // string. Structural recognition via `for_in_key_aliases`.
        let is_alias_cond = self.is_for_in_key_alias_condition(cond);
        let condition = if is_alias_cond {
            let ord_local = self
                .bare_identifier_name(cond)
                .and_then(|name| self.locals.get(&name).copied());
            if let Some(ord_local) = ord_local {
                function.instruction(&Instruction::LocalGet(ord_local));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            } else {
                self.emit_node(function, cond, true)
            }
        } else {
            self.reject_string_condition(cond);
            self.emit_node(function, cond, true)
        };
        if !condition.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        // Spec 4a Task 4 null-sentinel truthiness: a for-in-key alias condition
        // (`if (last)`) holds either a real key ordinal (`>= 0`) or the null
        // sentinel `-1`. Truthy iff a real key, i.e. `value >= 0` — NOT the
        // default `!= 0`, which would treat the first-field ordinal `0` as
        // falsy.
        if is_alias_cond {
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
