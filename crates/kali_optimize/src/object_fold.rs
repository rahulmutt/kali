use crate::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct BindingEnv {
    pub(crate) bindings: BTreeMap<String, LirNodeId>,
}

impl Optimizer {
    pub(crate) fn fold_object_has_own_call(
        &self,
        program: &mut LirProgram,
        snapshot: &LirNode,
        callee_node: &LirNode,
        bindings: &BindingEnv,
    ) -> Option<LirNodeId> {
        let callee_name = self.normalized_member_access_name(program, callee_node)?;
        if !matches!(
            callee_name.as_str(),
            "Object.hasOwn"
                | "globalThis.Object.hasOwn"
                | "Object.prototype.hasOwnProperty.call"
                | "globalThis.Object.prototype.hasOwnProperty.call"
                | "Object.hasOwnProperty.call"
                | "globalThis[\"Object\"].prototype.hasOwnProperty[\"call\"]"
                | "globalThis.Object.hasOwnProperty.call"
        ) {
            return None;
        }

        let object_id =
            self.resolve_constant_binding(program, *snapshot.children.get(1)?, bindings)?;
        if !self.is_object_literal(program, object_id) {
            return None;
        }

        let key_id =
            self.resolve_constant_binding(program, *snapshot.children.get(2)?, bindings)?;
        let key = self.constant_property_key(program, key_id)?;
        let has_own = self
            .object_literal_field(program, object_id, &key)
            .is_some();
        Some(self.clone_boolean_literal(program, has_own))
    }

    pub(crate) fn fold_object_enumeration_call(
        &self,
        program: &mut LirProgram,
        snapshot: &LirNode,
        callee_node: &LirNode,
        bindings: &BindingEnv,
    ) -> Option<LirNodeId> {
        let callee_name = self.normalized_member_access_name(program, callee_node)?;
        let string_mode = match callee_name.as_str() {
            "Object.keys" | "globalThis.Object.keys" => Some("keys"),
            "Object.values" | "globalThis.Object.values" => Some("values"),
            "Object.entries" | "globalThis.Object.entries" => Some("entries"),
            _ => None,
        };
        let is_reflect_own_keys = matches!(
            callee_name.as_str(),
            "Reflect.ownKeys" | "globalThis.Reflect.ownKeys"
        );
        if string_mode.is_none() && !is_reflect_own_keys {
            return None;
        }

        let object_id =
            self.resolve_constant_binding(program, *snapshot.children.get(1)?, bindings)?;
        if let Some(ConstantValue::String(string_text)) = literal_value(program, object_id) {
            if let Some(mode) = string_mode {
                let mut elements = Vec::with_capacity(string_text.chars().count());
                match mode {
                    "keys" => {
                        for (index, _) in string_text.chars().enumerate() {
                            elements.push(
                                self.clone_string_literal(
                                    program,
                                    format!("{:?}", index.to_string()),
                                ),
                            );
                        }
                    }
                    "values" => {
                        for value in string_text.chars() {
                            elements.push(
                                self.clone_string_literal(
                                    program,
                                    format!("{:?}", value.to_string()),
                                ),
                            );
                        }
                    }
                    "entries" => {
                        for (index, value) in string_text.chars().enumerate() {
                            let key_id = self
                                .clone_string_literal(program, format!("{:?}", index.to_string()));
                            let value_id = self
                                .clone_string_literal(program, format!("{:?}", value.to_string()));
                            elements.push(self.push_array_literal(program, vec![key_id, value_id]));
                        }
                    }
                    _ => unreachable!(),
                }
                return Some(self.push_array_literal(program, elements));
            }
        }
        if !self.is_object_literal(program, object_id) {
            return None;
        }

        let properties = self.ordered_object_literal_properties(program, object_id)?;
        // Honest fail-closed residue (throw-fallout Stage 2 Lane D, carried
        // over from Lane A's repr-shape carve-out): `__proto__` (identifier
        // OR quoted-string form) is JS's PROTOTYPE SETTER, not an own
        // property key — `Object.keys({ "__proto__": 1, "a": 2 })` is
        // `["a"]` in node, never `["__proto__", "a"]`. The enumeration fold
        // reads LIR property text directly (it never consults repr shapes),
        // so it must replicate the carve-out here at its own key-admission
        // point: never fold an enumeration over an object literal carrying a
        // `__proto__` key. Leave the call unfolded so it falls through to
        // the reject/backstop lane instead of ever emitting the phantom key.
        if properties
            .iter()
            .any(|(key, _)| key.trim_matches('"') == "__proto__")
        {
            return None;
        }
        match callee_name.as_str() {
            "Object.keys" | "globalThis.Object.keys" => {
                let mut elements = Vec::with_capacity(properties.len());
                for (key, _) in properties {
                    elements.push(
                        self.clone_string_literal(program, format!("{:?}", key.trim_matches('"'))),
                    );
                }
                Some(self.push_array_literal(program, elements))
            }
            "Reflect.ownKeys" | "globalThis.Reflect.ownKeys" => {
                let mut elements = Vec::with_capacity(properties.len());
                for (key, _) in properties {
                    elements.push(
                        self.clone_string_literal(program, format!("{:?}", key.trim_matches('"'))),
                    );
                }
                Some(self.push_array_literal(program, elements))
            }
            "Object.values" | "globalThis.Object.values" => {
                let mut elements = Vec::with_capacity(properties.len());
                for (_, value) in properties {
                    elements.push(self.clone_subtree_with_substitution(
                        program,
                        value,
                        &BTreeMap::new(),
                        &mut HashMap::new(),
                    ));
                }
                Some(self.push_array_literal(program, elements))
            }
            "Object.entries" | "globalThis.Object.entries" => {
                let mut elements = Vec::with_capacity(properties.len());
                for (key, value) in properties {
                    let key_id =
                        self.clone_string_literal(program, format!("{:?}", key.trim_matches('"')));
                    let value_id = self.clone_subtree_with_substitution(
                        program,
                        value,
                        &BTreeMap::new(),
                        &mut HashMap::new(),
                    );
                    let pair = self.push_array_literal(program, vec![key_id, value_id]);
                    elements.push(pair);
                }
                Some(self.push_array_literal(program, elements))
            }
            _ => None,
        }
    }

    pub(crate) fn fold_object_from_entries_call(
        &self,
        program: &mut LirProgram,
        snapshot: &LirNode,
        callee_node: &LirNode,
        bindings: &BindingEnv,
    ) -> Option<LirNodeId> {
        let callee_name = self.normalized_member_access_name(program, callee_node)?;
        if !matches!(
            callee_name.as_str(),
            "Object.fromEntries" | "globalThis.Object.fromEntries"
        ) {
            return None;
        }

        let entries_id =
            self.resolve_constant_binding(program, *snapshot.children.get(1)?, bindings)?;
        if !self.is_array_literal(program, entries_id) {
            return None;
        }

        let entries_node = program.nodes.get(entries_id.0 as usize)?;
        let mut properties: Vec<(String, usize, LirNodeId)> = Vec::new();
        let mut key_positions: HashMap<String, usize> = HashMap::new();
        for (entry_index, entry_id) in entries_node.children.iter().copied().enumerate() {
            let entry_id = self.resolve_constant_binding(program, entry_id, bindings)?;
            if !self.is_array_literal(program, entry_id) {
                return None;
            }

            let entry_node = program.nodes.get(entry_id.0 as usize)?;
            if entry_node.children.len() != 2 {
                return None;
            }

            let key_id =
                self.resolve_constant_binding(program, entry_node.children[0], bindings)?;
            let key = self.constant_property_key(program, key_id)?;
            let value_id =
                self.resolve_constant_binding(program, entry_node.children[1], bindings)?;

            if let Some(position) = key_positions.get(&key).copied() {
                properties[position].2 = value_id;
                continue;
            }

            key_positions.insert(key.clone(), properties.len());
            properties.push((key, entry_index, value_id));
        }

        let object_properties = properties
            .into_iter()
            .map(|(key, _, value)| (key, value))
            .collect::<Vec<_>>();
        Some(self.push_object_literal(program, object_properties))
    }

    pub(crate) fn fold_object_enumeration_calls(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        bindings: &BindingEnv,
    ) {
        let snapshot = program.nodes[id.0 as usize].clone();
        for child in snapshot.children.iter().copied() {
            self.fold_object_enumeration_calls(program, child, bindings);
        }

        if snapshot.kind != LirNodeKind::Call {
            return;
        }

        let Some(callee_id) = snapshot.children.first().copied() else {
            return;
        };
        let Some(callee_node) = program.nodes.get(callee_id.0 as usize).cloned() else {
            return;
        };
        if let Some(folded) =
            self.fold_object_enumeration_call(program, &snapshot, &callee_node, bindings)
        {
            program.nodes[id.0 as usize] = program.nodes[folded.0 as usize].clone();
        }
    }

    pub(crate) fn ordered_object_literal_properties(
        &self,
        program: &LirProgram,
        id: LirNodeId,
    ) -> Option<Vec<(String, LirNodeId)>> {
        if !self.is_object_literal(program, id) {
            return None;
        }

        let node = program.nodes.get(id.0 as usize)?;
        let mut properties = Vec::new();
        for (source_index, property) in node.children.iter().copied().enumerate() {
            let property_node = program.nodes.get(property.0 as usize)?;
            if property_node.children.len() != 2 {
                continue;
            }
            let key_node = program.nodes.get(property_node.children[0].0 as usize)?;
            let key = key_node.text.as_deref()?.to_string();
            properties.push((key, source_index, property_node.children[1]));
        }

        properties.sort_by(|(left_key, left_index, _), (right_key, right_index, _)| {
            match (
                Self::object_property_order_key(left_key),
                Self::object_property_order_key(right_key),
            ) {
                (Some(left_order), Some(right_order)) => left_order
                    .cmp(&right_order)
                    .then_with(|| left_index.cmp(right_index)),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => left_index.cmp(right_index),
            }
        });

        Some(
            properties
                .into_iter()
                .map(|(key, _, value)| (key, value))
                .collect(),
        )
    }

    pub(crate) fn resolve_constant_binding(
        &self,
        program: &LirProgram,
        mut id: LirNodeId,
        bindings: &BindingEnv,
    ) -> Option<LirNodeId> {
        let mut seen = BTreeSet::new();
        loop {
            if !seen.insert(id.0) {
                return None;
            }

            let node = program.nodes.get(id.0 as usize)?;
            // Guard (throw-fallout Stage 2 Lane D): a bona fide object
            // literal with exactly ONE property is ALSO a `Value` node with
            // one child and no text (its lone `init`-tagged property node) —
            // structurally indistinguishable, at this generic check, from a
            // transparent grouping/chain wrapper around a single inner
            // expression. Without this guard the loop below tunnels straight
            // through the one-property object literal into its property
            // node, so `Object.keys({ "b": 1 })` (and any single-property
            // enumeration/hasOwn target) resolves to the WRONG node instead
            // of the object literal — a silent miscompile, not merely a
            // missed fold. Never unwrap past a node that is itself a valid
            // object literal.
            if node.kind == LirNodeKind::Value
                && node.children.len() == 1
                && node.text.as_deref().is_none_or(|text| text.is_empty())
                && !self.is_object_literal(program, id)
            {
                id = node.children[0];
                continue;
            }

            if self.is_object_freeze_call(program, node) {
                id = node.children[1];
                continue;
            }

            if node.kind == LirNodeKind::Value
                && node.children.is_empty()
                && node.text.as_deref().is_some()
            {
                let name = node.text.as_deref()?;
                if let Some(bound) = bindings.bindings.get(name).copied() {
                    id = bound;
                    continue;
                }
            }

            return Some(id);
        }
    }

    pub(crate) fn is_object_freeze_call(&self, program: &LirProgram, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call || node.children.len() < 2 {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let Some(callee_node) = program.nodes.get(callee.0 as usize) else {
            return false;
        };
        matches!(
            self.normalized_member_access_name(program, callee_node)
                .as_deref(),
            Some("Object.freeze") | Some("globalThis.Object.freeze")
        )
    }

    pub(crate) fn collect_constant_bindings(
        &self,
        program: &LirProgram,
        id: LirNodeId,
    ) -> BindingEnv {
        let mut env = BindingEnv::default();
        self.collect_constant_bindings_into(program, id, &mut env);
        env
    }

    pub(crate) fn collect_constant_bindings_into(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        env: &mut BindingEnv,
    ) {
        let snapshot = program.nodes[id.0 as usize].clone();
        if let Some((name, init)) = self.extract_const_binding(program, id) {
            let resolved = self
                .resolve_constant_binding(program, init, env)
                .unwrap_or(init);
            if self.is_specializable_binding(program, resolved) {
                env.bindings.insert(name, resolved);
            }
        }

        for child in snapshot.children {
            self.collect_constant_bindings_into(program, child, env);
        }
    }

    /// Names that are the base of any member store (`x.k = v`) or member
    /// delete (`delete x.k`) anywhere in the program. Name-based and
    /// shadowing-blind BY DESIGN: a shadowed name over-approximates to
    /// "mutated", which only ever DISABLES folding (fail-closed direction).
    pub(crate) fn collect_mutated_binding_names(&self, program: &LirProgram) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        for node in &program.nodes {
            let is_store = node.kind == LirNodeKind::Value
                && node.text.as_deref() == Some("=")
                && node.children.len() == 2;
            let is_delete = node.kind == LirNodeKind::Value
                && node.text.as_deref() == Some("delete")
                && node.children.len() == 1;
            if !is_store && !is_delete {
                continue;
            }
            let member = node.children[0];
            // Dot member (`x.k`) — the timeline-lane base+key form.
            if let Some((base, _key)) = self.dot_member_base_and_key(program, member) {
                names.insert(base);
                continue;
            }
            // Computed member (`x[expr]`, 2-child member node) is invisible to
            // `dot_member_base_and_key` but STILL mutates its base. Walk the
            // member expression's first-child spine conservatively: if any bare
            // identifier bottoms out under it, mark that identifier mutated.
            // Over-marking is safe (it only ever DISABLES folding); and because
            // `as_timeline_mutation`/`member_mutation_base_node` stay dot-only,
            // the computed store's base occurrence goes uncredited → the binding
            // is ineligible → its enumerations fail closed instead of folding
            // the stale pre-store shape. (ONLY the mutated-name scan changes;
            // the timeline application lane must not admit computed stores.)
            if let Some(base) = self.member_chain_base_identifier(program, member) {
                names.insert(base);
            }
        }
        names
    }

    /// Conservatively walk a member expression's first-child (base) spine and
    /// return the bare identifier it bottoms out at, if any. Handles both dot
    /// (`x.k`, 1-child) and computed (`x[expr]`, 2-child) member nodes, and
    /// nested chains (`x.a[b].c`). Returns None if the spine does not reach a
    /// bare identifier. Used ONLY by the mutated-name scan (over-approximation),
    /// never by the timeline application lane.
    fn member_chain_base_identifier(
        &self,
        program: &LirProgram,
        member: LirNodeId,
    ) -> Option<String> {
        let mut id = member;
        let mut seen = BTreeSet::new();
        loop {
            if !seen.insert(id.0) {
                return None;
            }
            let node = program.nodes.get(id.0 as usize)?;
            if node.kind != LirNodeKind::Value {
                return None;
            }
            // Bare identifier: a Value leaf carrying text.
            if node.children.is_empty() {
                return node.text.as_deref().map(|text| text.to_string());
            }
            // Member-like node (dot = 1 child, computed = 2 children): descend
            // the base (first child) spine. Anything else is not a member chain.
            if node.children.len() == 1 || node.children.len() == 2 {
                id = node.children[0];
                continue;
            }
            return None;
        }
    }

    /// Strip from `env` (a) every mutated name and (b) any binding whose
    /// resolved node id equals a mutated binding's literal id — i.e. an alias
    /// (`const s = r`) of a mutated object literal, which would otherwise fold
    /// against the stale pre-mutation snapshot. Shared by the ordered pass and
    /// the release inline path so BOTH fail closed on stale aliases identically
    /// (do not hand-roll a second scan). `mutated_ids` is computed before the
    /// retain so mutated names are still present to seed it.
    pub(crate) fn strip_mutated_bindings(&self, env: &mut BindingEnv, mutated: &BTreeSet<String>) {
        let mutated_ids: BTreeSet<u32> = env
            .bindings
            .iter()
            .filter(|(name, _)| mutated.contains(*name))
            .map(|(_, id)| id.0)
            .collect();
        env.bindings
            .retain(|name, id| !mutated.contains(name) && !mutated_ids.contains(&id.0));
    }

    /// `x.k` dot-member: node text = key, exactly one child = bare
    /// identifier base. Computed access (`x[expr]`, 2 children) returns
    /// None — outside the timeline lane.
    fn dot_member_base_and_key(
        &self,
        program: &LirProgram,
        id: LirNodeId,
    ) -> Option<(String, String)> {
        let node = program.nodes.get(id.0 as usize)?;
        if node.kind != LirNodeKind::Value || node.children.len() != 1 {
            return None;
        }
        let key = node.text.as_deref()?.to_string();
        let base_node = program.nodes.get(node.children[0].0 as usize)?;
        if base_node.kind != LirNodeKind::Value || !base_node.children.is_empty() {
            return None;
        }
        Some((base_node.text.as_deref()?.to_string(), key))
    }

    /// Order-aware enumeration folding (throw-fallout Stage 2, Lane C).
    ///
    /// Non-mutated bindings keep the exact old flat (order-blind,
    /// forward-reference-friendly) behavior. Mutated bindings are eligible
    /// for the static shape timeline iff EVERY occurrence of the name sits at
    /// a permitted straight-line top-level site (its own const declarator, a
    /// `delete x.k` / `x.k = v` base, or an enumeration-call object argument).
    /// Eligible mutated bindings fold against a per-program-point snapshot;
    /// ineligible mutated bindings — and any binding that aliases a mutated
    /// binding's initial literal — are excluded from the fold env entirely
    /// (killing every stale fold). Their deletes are left in place for
    /// codegen's default-deny arm; consumed deletes are erased to empty
    /// blocks so they never reach codegen.
    pub(crate) fn fold_object_enumeration_calls_ordered(&self, program: &mut LirProgram) {
        let mutated = self.collect_mutated_binding_names(program);
        if mutated.is_empty() {
            // No member stores/deletes at all: exact old flat behavior.
            let env = self.collect_constant_bindings(program, program.root);
            self.fold_object_enumeration_calls(program, program.root, &env);
            return;
        }

        let eligible = self.timeline_eligible_bindings(program, &mutated);

        // Global flat env. Non-mutated bindings keep their order-blind
        // folding. Drop (a) every mutated name and (b) any binding that
        // resolves to the SAME node id as a mutated binding — i.e. an alias
        // of a mutated object literal, which would otherwise fold against the
        // stale pre-mutation snapshot. Eligible mutated names are re-seeded
        // at their declaration point below.
        let mut env = self.collect_constant_bindings(program, program.root);
        self.strip_mutated_bindings(&mut env, &mutated);

        let root_children = program.nodes[program.root.0 as usize].children.clone();
        for stmt in root_children {
            // 1. const decl of an eligible mutated binding → seed its snapshot
            //    from the literal (the timeline starts here).
            if let Some((name, init)) = self.extract_const_binding(program, stmt) {
                if eligible.contains(&name) {
                    let resolved = self
                        .resolve_constant_binding(program, init, &env)
                        .unwrap_or(init);
                    if self.is_specializable_binding(program, resolved) {
                        env.bindings.insert(name, resolved);
                    } else {
                        // Non-literal init for a mutated binding: cannot track.
                        env.bindings.remove(&name);
                    }
                }
            }

            // 2. timeline mutation → advance the binding's snapshot literal.
            if let Some((kind, name, key, value)) = self.as_timeline_mutation(program, stmt) {
                if eligible.contains(&name) {
                    let applied = env.bindings.get(&name).copied().and_then(|current| {
                        self.apply_timeline_mutation(program, current, kind, &key, value)
                    });
                    match applied {
                        Some(next) => {
                            env.bindings.insert(name.clone(), next);
                            if kind == TimelineMutation::Delete {
                                self.erase_statement(program, stmt);
                                continue; // erased: nothing left to fold in it
                            }
                        }
                        None => {
                            // Could not maintain the snapshot: drop the binding
                            // so no later enumeration folds against a stale
                            // shape (fail-closed).
                            env.bindings.remove(&name);
                        }
                    }
                }
            }

            // 3. fold enumeration calls inside this statement against the env
            //    as of THIS program point.
            self.fold_object_enumeration_calls(program, stmt, &env);
        }
    }

    /// Unwrap statement wrappers (`Value`, empty text, exactly one child) —
    /// the same deref rule `resolve_constant_binding` uses, including its
    /// refusal to tunnel through a genuine single-property object literal.
    fn unwrap_statement_wrapper(&self, program: &LirProgram, mut id: LirNodeId) -> LirNodeId {
        loop {
            let Some(node) = program.nodes.get(id.0 as usize) else {
                break;
            };
            if node.kind == LirNodeKind::Value
                && node.children.len() == 1
                && node.text.as_deref().is_none_or(|text| text.is_empty())
                && !self.is_object_literal(program, id)
            {
                id = node.children[0];
            } else {
                break;
            }
        }
        id
    }

    /// Classify a top-level statement as a timeline mutation:
    /// `delete x.k` → `(Delete, base, key, None)`,
    /// `x.k = v` → `(Store, base, key, Some(value))`.
    fn as_timeline_mutation(
        &self,
        program: &LirProgram,
        stmt: LirNodeId,
    ) -> Option<(TimelineMutation, String, String, Option<LirNodeId>)> {
        let inner = self.unwrap_statement_wrapper(program, stmt);
        let node = program.nodes.get(inner.0 as usize)?;
        if node.kind != LirNodeKind::Value {
            return None;
        }
        match node.text.as_deref() {
            Some("delete") if node.children.len() == 1 => {
                let (base, key) = self.dot_member_base_and_key(program, node.children[0])?;
                Some((TimelineMutation::Delete, base, key, None))
            }
            Some("=") if node.children.len() == 2 => {
                let (base, key) = self.dot_member_base_and_key(program, node.children[0])?;
                Some((TimelineMutation::Store, base, key, Some(node.children[1])))
            }
            _ => None,
        }
    }

    /// Apply a timeline mutation to `current` (an object literal snapshot),
    /// returning a freshly-built literal in SOURCE (insertion) order. Delete
    /// removes the key; Store updates it in place when present or appends at
    /// the END (reinsertion order restarts). The enumeration fold does the ES
    /// sort at fold time, exactly as it does for source literals. Never
    /// applies a `__proto__` mutation (fail-closed; the caller keeps such a
    /// binding out of the eligible set anyway).
    fn apply_timeline_mutation(
        &self,
        program: &mut LirProgram,
        current: LirNodeId,
        kind: TimelineMutation,
        key: &str,
        value: Option<LirNodeId>,
    ) -> Option<LirNodeId> {
        let normalized_key = key.trim_matches('"');
        if normalized_key == "__proto__" {
            return None;
        }
        let mut properties = self.source_order_object_properties(program, current)?;
        match kind {
            TimelineMutation::Delete => {
                properties
                    .retain(|(existing_key, _)| existing_key.trim_matches('"') != normalized_key);
            }
            TimelineMutation::Store => {
                let value = value?;
                if let Some(slot) = properties
                    .iter_mut()
                    .find(|(existing_key, _)| existing_key.trim_matches('"') == normalized_key)
                {
                    slot.1 = value;
                } else {
                    properties.push((key.to_string(), value));
                }
            }
        }
        Some(self.push_object_literal(program, properties))
    }

    /// Object-literal properties in SOURCE order (no ES sort) — insertion
    /// order preserved so timeline appends land last.
    fn source_order_object_properties(
        &self,
        program: &LirProgram,
        id: LirNodeId,
    ) -> Option<Vec<(String, LirNodeId)>> {
        if !self.is_object_literal(program, id) {
            return None;
        }
        let node = program.nodes.get(id.0 as usize)?;
        let mut properties = Vec::new();
        for property in node.children.iter().copied() {
            let property_node = program.nodes.get(property.0 as usize)?;
            if property_node.children.len() != 2 {
                continue;
            }
            let key_node = program.nodes.get(property_node.children[0].0 as usize)?;
            let key = key_node.text.as_deref()?.to_string();
            properties.push((key, property_node.children[1]));
        }
        Some(properties)
    }

    /// Erase a consumed statement to an empty `Block` so codegen never sees
    /// the delete (Task 6's default-deny arm enforces the invariant).
    fn erase_statement(&self, program: &mut LirProgram, stmt: LirNodeId) {
        program.nodes[stmt.0 as usize] = LirNode {
            kind: LirNodeKind::Block,
            text: None,
            children: vec![],
            function_flavor: None,
        };
    }

    /// The mutated bindings that qualify for the static shape timeline:
    /// every occurrence of the name sits at a permitted straight-line
    /// top-level site, and no timeline mutation touches `__proto__`.
    pub(crate) fn timeline_eligible_bindings(
        &self,
        program: &LirProgram,
        mutated: &BTreeSet<String>,
    ) -> BTreeSet<String> {
        if mutated.is_empty() {
            return BTreeSet::new();
        }

        // Total occurrences of each mutated name anywhere in the program.
        let mut total: BTreeMap<String, usize> = BTreeMap::new();
        for node in &program.nodes {
            if node.kind == LirNodeKind::Value && node.children.is_empty() {
                if let Some(text) = node.text.as_deref() {
                    if mutated.contains(text) {
                        *total.entry(text.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }

        // Permitted occurrence node ids + names disqualified by a __proto__
        // mutation. The walk stops at Branch nodes and function definitions,
        // so any occurrence buried there stays unpermitted and drives its
        // binding ineligible (the count cross-check below is what makes this
        // nesting-blindness safe).
        let mut permitted: BTreeSet<u32> = BTreeSet::new();
        let mut disqualified: BTreeSet<String> = BTreeSet::new();
        let root_children = program.nodes[program.root.0 as usize].children.clone();
        for stmt in root_children {
            self.collect_permitted_occurrences(
                program,
                stmt,
                mutated,
                &mut permitted,
                &mut disqualified,
            );
        }

        let mut permitted_count: BTreeMap<String, usize> = BTreeMap::new();
        for id in &permitted {
            if let Some(node) = program.nodes.get(*id as usize) {
                if let Some(text) = node.text.as_deref() {
                    *permitted_count.entry(text.to_string()).or_insert(0) += 1;
                }
            }
        }

        mutated
            .iter()
            .filter(|name| !disqualified.contains(*name))
            .filter(|name| {
                total.get(*name).copied().unwrap_or(0)
                    == permitted_count.get(*name).copied().unwrap_or(0)
            })
            .cloned()
            .collect()
    }

    fn collect_permitted_occurrences(
        &self,
        program: &LirProgram,
        id: LirNodeId,
        mutated: &BTreeSet<String>,
        permitted: &mut BTreeSet<u32>,
        disqualified: &mut BTreeSet<String>,
    ) {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return;
        };
        // Non-straight-line regions: do NOT descend. Any mutated-name
        // occurrence inside stays unpermitted, forcing the binding ineligible.
        // `Block` is a nested lexical scope (a bare `{ ... }`) — a store buried
        // there is applied by no direct-root-child timeline step, so crediting
        // its base would leave the binding eligible yet folded against the
        // stale pre-block shape. The GLOBAL occurrence counter in
        // `timeline_eligible_bindings` still counts inside blocks (it scans all
        // nodes flatly), so stopping here leaves the block occurrence uncredited
        // → count mismatch → ineligible. Count everywhere, credit only
        // straight-line: that asymmetry is the safety mechanism.
        if node.kind == LirNodeKind::Branch || node.kind == LirNodeKind::Block {
            return;
        }
        if self.function_summary(program, id).is_some() {
            return;
        }

        // (a) const declarator's own name child.
        if node.kind == LirNodeKind::Instruction && node.children.len() >= 2 {
            if let Some(name) = node.text.as_deref() {
                let name_node = node.children[0];
                if mutated.contains(name)
                    && program.nodes.get(name_node.0 as usize).is_some_and(|n| {
                        n.kind == LirNodeKind::Value
                            && n.children.is_empty()
                            && n.text.as_deref() == Some(name)
                    })
                {
                    permitted.insert(name_node.0);
                }
            }
        }

        // (b) timeline-mutation base position (delete x.k / x.k = v).
        if let Some((base_id, key)) = self.member_mutation_base_node(program, id) {
            if let Some(base_node) = program.nodes.get(base_id.0 as usize) {
                if let Some(base_name) = base_node.text.as_deref() {
                    if mutated.contains(base_name) {
                        permitted.insert(base_id.0);
                        if key.trim_matches('"') == "__proto__" {
                            disqualified.insert(base_name.to_string());
                        }
                    }
                }
            }
        }

        // (c) enumeration-call object argument.
        if node.kind == LirNodeKind::Call && node.children.len() == 2 {
            let callee = node.children[0];
            if self.is_enumeration_callee(program, callee) {
                let arg = node.children[1];
                if program.nodes.get(arg.0 as usize).is_some_and(|n| {
                    n.kind == LirNodeKind::Value
                        && n.children.is_empty()
                        && n.text.as_deref().is_some_and(|t| mutated.contains(t))
                }) {
                    permitted.insert(arg.0);
                }
            }
        }

        let children = node.children.clone();
        for child in children {
            self.collect_permitted_occurrences(program, child, mutated, permitted, disqualified);
        }
    }

    /// The base identifier node id + key of a raw `delete x.k` / `x.k = v`
    /// node (not statement-unwrapped — used by the occurrence walk).
    fn member_mutation_base_node(
        &self,
        program: &LirProgram,
        id: LirNodeId,
    ) -> Option<(LirNodeId, String)> {
        let node = program.nodes.get(id.0 as usize)?;
        if node.kind != LirNodeKind::Value {
            return None;
        }
        let member = match node.text.as_deref() {
            Some("delete") if node.children.len() == 1 => node.children[0],
            Some("=") if node.children.len() == 2 => node.children[0],
            _ => return None,
        };
        let member_node = program.nodes.get(member.0 as usize)?;
        if member_node.kind != LirNodeKind::Value || member_node.children.len() != 1 {
            return None;
        }
        let key = member_node.text.as_deref()?.to_string();
        let base_id = member_node.children[0];
        let base_node = program.nodes.get(base_id.0 as usize)?;
        if base_node.kind != LirNodeKind::Value
            || !base_node.children.is_empty()
            || base_node.text.is_none()
        {
            return None;
        }
        Some((base_id, key))
    }

    fn is_enumeration_callee(&self, program: &LirProgram, callee: LirNodeId) -> bool {
        let Some(node) = program.nodes.get(callee.0 as usize) else {
            return false;
        };
        let Some(name) = self.normalized_member_access_name(program, node) else {
            return false;
        };
        matches!(
            name.as_str(),
            "Object.keys"
                | "globalThis.Object.keys"
                | "Object.values"
                | "globalThis.Object.values"
                | "Object.entries"
                | "globalThis.Object.entries"
                | "Reflect.ownKeys"
                | "globalThis.Reflect.ownKeys"
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelineMutation {
    Delete,
    Store,
}

#[cfg(test)]
#[path = "object_fold_tests.rs"]
mod object_fold_tests;
