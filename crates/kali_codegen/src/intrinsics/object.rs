//! Object literal and Object built-in intrinsic recognition and constant-folding.
use crate::*;
use kali_common::js_number::format_js_number;

/// Which slot a key literal's text was read from.
///
/// The two slots encode the same key with OPPOSITE quoting conventions, so the
/// slot is what says whether a text is a number's spelling or a string's. This
/// is measured, not assumed -- dumping the lowered nodes gives:
///
/// | source | object-literal key slot | same literal as an expression |
/// |---|---|---|
/// | `{a: 1}` / `{"a": 1}` | `a` | `"a"` |
/// | `{5: 1}` | `"5"` | `5` |
/// | `{1e-7: 1}` | `"0.0000001"` | `0.0000001` |
///
/// So in a KEY slot the quotes mean "this was a NUMBER, already stringified by
/// HIR with Rust's `Display`"; in an EXPRESSION slot they mean the opposite,
/// "this was a string". An identifier key and a quoted-string key are already
/// indistinguishable in the key slot, which is fine: JS gives them the same key.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyTextSlot {
    ObjectLiteralKey,
    Expression,
}

/// The property key a literal's text denotes, computed the way JS does
/// (`String(key)`). The single currency both sides of a static key comparison
/// must be in.
///
/// The two sides used to agree by accident: the probe rendered through
/// `render_static_value` and the stored key was read as raw HIR text, and both
/// were the same Rust `Display for f64` expansion. Once the renderer started
/// emitting JS notation they disagreed, and `Object.hasOwn({1e21: 1}, 1e21)`
/// folded to a silent, diagnostic-free `false`.
///
/// The rule is deliberately TYPE-AWARE rather than "renumber anything that
/// parses as a number": the numeric key `1e21` denotes the property `"1e+21"`,
/// while the STRING key `"1000000000000000000000"` denotes itself, and JS keeps
/// those two distinct. Renumbering both would fix one wrong answer by creating
/// another.
/// The value `spelling` denotes, but ONLY when `spelling` is exactly what
/// `kali_hir`'s `lower_property_name` could have written for a numeric property
/// name. Otherwise `None`, meaning "this is a string key, leave it alone".
///
/// That function is
/// `if value == 0.0 { "0" } else { value.to_string() }`, so its image is what
/// this predicate must accept -- no more and no less. Testing with
/// `parse_numeric_literal_value` alone accepts a strictly WIDER language than
/// HIR's writer can produce: `1e21`, `1e-7`, `5e3`, `5E3`, `+5`, `5.`, `.5`,
/// `05`, `-0`, `42n`, `NaN` and `infinity` all parse, and none of them is
/// something that writer can emit. Every spelling in that gap is a string
/// key -- `{'"1e21"': 1}` -- and renumbering it renamed the property, which is
/// how this helper acquired the same defect three times running. A round-trip
/// through the writer closes the whole class instead of one spelling at a time.
///
/// NEGATIVE SPELLINGS ARE REACHABLE, and must be accepted. The plain key
/// grammar has no sign -- `{-5: 1}` really is a syntax error -- but the
/// COMPUTED key path folds one in: `computed_object_property_name`
/// (`kali_parser/src/expression/object.rs:172-191`) matches a unary `+`/`-`
/// over a numeric key and returns `PropertyName::Number(-number)`, which the
/// repo's own parser test pins (`{[-1]: 1}` yields `PropertyName::Number(-1.0)`).
/// So `{[-1]: 1}`, `{[-1.5]: 1}` and `{[-1e999]: 1}` all reach HIR as negative
/// values and are written `-1`, `-1.5` and `-inf`. An earlier version of this
/// predicate rejected every negative spelling on the false premise that the
/// grammar forbids them, which traded a class of false positives for a class of
/// false negatives. Do not re-add that guard.
///
/// ONE reachability guard, because the image is over the values a property name
/// can actually denote: NOT NaN. No numeric literal denotes NaN, and
/// `{NaN: 1}` is an IDENTIFIER key that never reaches this slot quoted -- but
/// `f64::NAN.to_string()` is `"NaN"`, so the round trip alone would accept it.
/// (`{[0/0]: 1}` does not reach here either: only a literal, not an arithmetic
/// expression, folds to a computed property name.)
///
/// The zero case is stated separately rather than left to the round trip
/// because `Display` and HIR disagree there: `(-0.0).to_string()` is `"-0"`,
/// while HIR collapses BOTH zeros to `"0"` (its `*value == 0.0` test is true for
/// `-0.0`, and the parser hands it a signed zero for `{[-0]: 1}`). That branch
/// is therefore what makes `-0` unreachable AS A SPELLING and is the only thing
/// rejecting it -- it needs no help from a sign guard.
fn is_hir_numeric_key_spelling(spelling: &str) -> Option<f64> {
    let value = parse_numeric_literal_value(spelling)?;
    if value.is_nan() {
        return None;
    }
    let written_by_hir = if value == 0.0 {
        spelling == "0"
    } else {
        value.to_string() == spelling
    };
    written_by_hir.then_some(value)
}

pub(crate) fn canonical_property_key_text(text: &str, slot: KeyTextSlot) -> String {
    // NOT trimmed. In the key slot whitespace is never padding -- ` a ` is the
    // three-character property name, and `{" a ": 1}` lowers to exactly that
    // text. Trimming here silently renamed the key to `a`.
    //
    // The length guard is a BYTE length while the delimiter tests are on chars,
    // which is what keeps a one-character multi-byte key such as `{"é": 1}`
    // (two bytes, one char) from ever being mistaken for a quoted text. Keep
    // that pairing; the `[1..len - 1]` slices below are only ever reached once
    // both ends are known to be ASCII quotes.
    let long_enough = text.len() >= 2;

    // The ONE numeric renderer both slots share, so the two sides cannot drift
    // apart on how a number is spelled.
    let renumbered = |spelling: &str| -> Option<String> {
        parse_numeric_literal_value(spelling).map(format_js_number)
    };

    match slot {
        // A DOUBLE quote -- and only a double quote -- is HIR's marker for "this
        // was a NUMBER, already stringified": `lower_property_name`'s `Number`
        // arm is `format!("\"{}\"", ...)`, and its `String` and `Identifier`
        // arms store the name verbatim. Accepting `'` and `` ` `` here as well
        // renumbered string keys whose own content is a quoted number, so
        // `{"'5'": 1}` answered `hasOwn(o, "'5'")` with `false` and
        // `hasOwn(o, 5)` with `true` while member access still found the key --
        // one program contradicting itself in a single output.
        //
        // A quoted-looking text whose inner is NOT a number is likewise a string
        // key whose quote characters are part of the name (`{"\"d\"": 1}`), so
        // it comes back verbatim.
        //
        // THE GATE IS STILL OPEN FOR `"` ITSELF, AND THAT IS A LIVE SILENT
        // MISCOMPILE -- register **R-56** (§2, Tier 2). Narrowing the marker to
        // the double quote fixed `'` and `` ` ``; it cannot fix `"`, because `"`
        // is simultaneously HIR's marker for "this was a number". `{'"5"': 1}`
        // and `{5: 1}` reach this function as the SAME text, so
        // `Object.hasOwn({'"5"': 1}, '"5"')` folds to `false` and
        // `Object.hasOwn({'"5"': 1}, 5)` to `true` -- both wrong, both at exit 0,
        // in a program whose `o['"5"']` still reads `1`. The exact failure this
        // paragraph says the gate exists to prevent, one quote character over.
        // No predicate at THIS level can close it: `lower_property_name`
        // (`crates/kali_hir/src/lowering/object.rs:20`) discards whether the
        // `PropertyName` was `Number` or `String`, so the discriminator must be
        // restored upstream. Do not attempt a textual fix here.
        //
        // "IS a number" here means the INVARIANT, not a spelling: the inner must
        // be exactly what `lower_property_name` could have written
        // (`is_hir_numeric_key_spelling`). Asking `parse_numeric_literal_value`
        // instead accepts a language strictly wider than that function's image,
        // and every spelling in the gap is a STRING key being renumbered.
        KeyTextSlot::ObjectLiteralKey => {
            let double_quoted = long_enough && text.starts_with('"') && text.ends_with('"');
            if !double_quoted {
                return text.to_string();
            }
            is_hir_numeric_key_spelling(&text[1..text.len() - 1])
                .map(format_js_number)
                .unwrap_or_else(|| text.to_string())
        }
        // The convention is inverted here: ANY quote character means a string
        // literal, whose content is the key however it is spelled, and only an
        // UNQUOTED text is a number's spelling.
        KeyTextSlot::Expression => {
            let quoted = long_enough
                && matches!(
                    (text.chars().next(), text.chars().last()),
                    (Some('"'), Some('"')) | (Some('\''), Some('\'')) | (Some('`'), Some('`'))
                );
            if quoted {
                return text[1..text.len() - 1].to_string();
            }
            // `String(42n)` is "42": exact, and textual, so the digits of a
            // BigInt too large for an `f64` survive. Load-bearing on this side
            // only -- a BigInt can reach an expression slot.
            if is_bigint_literal_text(text) {
                return text[..text.len() - 1].to_string();
            }
            renumbered(text).unwrap_or_else(|| text.to_string())
        }
    }
}

impl<'a> FunctionEmitter<'a> {
    /// `canonical_property_key_text` for a key NODE.
    ///
    /// Key-slot nodes are read for their own text and never resolved as
    /// bindings (`{ a: 1 }`'s key is the name `a`, not the value of a variable
    /// `a`), which is why this cannot simply defer to `render_static_value`.
    /// A non-literal PROBE key (a bound identifier, a folding call) still can:
    /// that renderer's numeric output is already the canonical form and its
    /// string output is already the bare content.
    pub(crate) fn static_property_key_text(
        &self,
        id: LirNodeId,
        slot: KeyTextSlot,
    ) -> Option<String> {
        let node = self.node(id);
        if node.kind == LirNodeKind::Literal {
            return Some(canonical_property_key_text(node.text.as_deref()?, slot));
        }
        self.render_static_value(id)
    }

    pub(crate) fn is_object_literal(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Value || node.text.is_some() || node.children.is_empty() {
            return false;
        }

        node.children.iter().all(|child| {
            self.node(*child).children.len() == 2
                && self
                    .node(*child)
                    .text
                    .as_deref()
                    .is_some_and(|kind| matches!(kind, "init" | "get" | "set"))
                && self.node(self.node(*child).children[0]).kind == LirNodeKind::Literal
        })
    }

    pub(crate) fn object_literal_field(&self, node: &LirNode, field: &str) -> Option<LirNodeId> {
        if !self.is_object_literal(node) {
            return None;
        }

        let field = field.trim_matches('"');
        for child in &node.children {
            let property = self.node(*child);
            if property.children.len() != 2 {
                continue;
            }
            let key = self
                .node(property.children[0])
                .text
                .as_deref()
                .map(|value| value.trim_matches('"'))?;
            if key == field {
                return property.children.get(1).copied();
            }
        }

        None
    }

    pub(crate) fn is_math_object(&self, callee_node: &LirNode) -> bool {
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        let Some(object) = self.resolve_transparent_object_root_node(object) else {
            return false;
        };
        matches!(
            self.node(object).text.as_deref(),
            Some("Math")
                | Some("globalThis.Math")
                | Some(r#"globalThis["Math"]"#)
                | Some(r#"globalThis['Math']"#)
        )
    }

    pub(crate) fn is_object_identity_object(&self, callee_node: &LirNode) -> bool {
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        let Some(object) = self.resolve_transparent_object_root_node(object) else {
            return false;
        };
        matches!(
            self.node(object).text.as_deref(),
            Some("Object")
                | Some("globalThis.Object")
                | Some(r#"globalThis["Object"]"#)
                | Some(r#"globalThis['Object']"#)
        )
    }

    pub(crate) fn is_number_object(&self, callee_node: &LirNode) -> bool {
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        let Some(object) = self.resolve_transparent_object_root_node(object) else {
            return false;
        };
        matches!(
            self.node(object).text.as_deref(),
            Some("Number")
                | Some("globalThis.Number")
                | Some(r#"globalThis["Number"]"#)
                | Some(r#"globalThis['Number']"#)
        )
    }

    pub(crate) fn is_object_freeze_call(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let callee_node = self.node(callee);
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };

        matches!(
            callee_node.text.as_deref(),
            Some(text)
                if text == "freeze"
                    || text.ends_with(".freeze")
                    || text.ends_with(r#"["freeze"]"#)
                    || text.ends_with(r#"['freeze']"#)
        ) && matches!(
            self.node(object).text.as_deref(),
            Some("Object")
                | Some("globalThis.Object")
                | Some(r#"globalThis["Object"]"#)
                | Some(r#"globalThis['Object']"#)
        )
    }

    pub(crate) fn resolve_transparent_object_root_node(&self, id: LirNodeId) -> Option<LirNodeId> {
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

            return Some(id);
        }
    }

    pub(crate) fn resolve_static_object_identity_value(
        &self,
        id: LirNodeId,
    ) -> Option<StaticObjectIdentityValue> {
        let node = self.node(id);
        if self.is_object_freeze_call(node) {
            return node
                .children
                .get(1)
                .copied()
                .and_then(|child| self.resolve_static_object_identity_value(child));
        }
        match node.kind {
            LirNodeKind::Literal => match node.text.as_deref() {
                Some("true") => Some(StaticObjectIdentityValue::Boolean(true)),
                Some("false") => Some(StaticObjectIdentityValue::Boolean(false)),
                Some("null") => Some(StaticObjectIdentityValue::Null),
                Some("Infinity") => Some(StaticObjectIdentityValue::Number(f64::INFINITY)),
                Some("NaN") => Some(StaticObjectIdentityValue::Number(f64::NAN)),
                Some("void") => Some(StaticObjectIdentityValue::Undefined),
                Some(text) => text
                    .strip_suffix('n')
                    .and_then(|value| value.parse::<i64>().ok())
                    .map(StaticObjectIdentityValue::BigInt)
                    .or_else(|| {
                        parse_numeric_literal_value(text).map(StaticObjectIdentityValue::Number)
                    })
                    .or_else(|| {
                        Some(StaticObjectIdentityValue::String(
                            strip_string_delimiters(text).to_string(),
                        ))
                    }),
                None => None,
            },
            LirNodeKind::Value if node.children.len() == 2 => match node.text.as_deref() {
                Some("??") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    if left.is_nullish() {
                        self.resolve_static_object_identity_value(node.children[1])
                    } else {
                        Some(left)
                    }
                }
                Some("&&") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => self.resolve_static_object_identity_value(node.children[1]),
                        Some(false) => Some(left),
                        None => {
                            let right =
                                self.resolve_static_object_identity_value(node.children[1])?;
                            if left.same_value(&right) {
                                Some(left)
                            } else {
                                None
                            }
                        }
                    }
                }
                Some("||") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => Some(left),
                        Some(false) => self.resolve_static_object_identity_value(node.children[1]),
                        None => {
                            let right =
                                self.resolve_static_object_identity_value(node.children[1])?;
                            if left.same_value(&right) {
                                Some(left)
                            } else {
                                None
                            }
                        }
                    }
                }
                _ => None,
            },
            LirNodeKind::Value if node.children.is_empty() => {
                let text = node.text.as_deref()?;
                if let Some(bound) = self.bindings.get(text).copied() {
                    return self.resolve_static_object_identity_value(bound);
                }
                match text {
                    "Infinity" => Some(StaticObjectIdentityValue::Number(f64::INFINITY)),
                    "NaN" => Some(StaticObjectIdentityValue::Number(f64::NAN)),
                    _ => parse_numeric_literal_value(text).map(StaticObjectIdentityValue::Number),
                }
            }
            LirNodeKind::Value if node.children.len() == 1 => match node.text.as_deref() {
                // Identity tunnels through a text-less one-child `Value`
                // (transparent grouping/sequence/`new` wrapper AND a
                // single-element array literal `[x]`, which are structurally
                // identical here). That is correct for an identity consumer — it
                // wants the wrapped scalar. The `[x].length` array-vs-string
                // carve-out lives in the `.length` consumer (`render_length`),
                // NOT here: guarding it here also breaks `Object.hasOwn`,
                // number-predicate and spread consumers that legitimately tunnel
                // one-child wrappers (throw-fallout Stage 2). A one-property
                // OBJECT literal's lone child is an `init` node with no scalar
                // identity, so it already resolves to `None`.
                // `"await"` (Stage 3 Task 4) marks a synchronously-settled
                // passthrough wrapper; an identity consumer tunnels through it to
                // the awaited operand exactly like a text-less grouping wrapper
                // (e.g. `Number.isSafeInteger(await alias)`).
                None | Some("") | Some("await") => {
                    self.resolve_static_object_identity_value(node.children[0])
                }
                Some("+") => match self.resolve_static_object_identity_value(node.children[0]) {
                    Some(StaticObjectIdentityValue::BigInt(_)) => None,
                    other => other,
                },
                Some("void") => Some(StaticObjectIdentityValue::Undefined),
                Some("-") => self
                    .resolve_static_object_identity_value(node.children[0])
                    .and_then(|value| match value {
                        StaticObjectIdentityValue::Number(number) => {
                            Some(StaticObjectIdentityValue::Number(if number == 0.0 {
                                -0.0
                            } else {
                                -number
                            }))
                        }
                        StaticObjectIdentityValue::BigInt(value) => {
                            Some(StaticObjectIdentityValue::BigInt(-value))
                        }
                        _ => None,
                    }),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn is_object_has_own_call(&self, node: &LirNode, callee_node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let receiver_text = callee_node
            .children
            .first()
            .and_then(|receiver| self.node(*receiver).text.as_deref())
            .unwrap_or_default();
        match callee_node.text.as_deref() {
            Some(text)
                if text == "hasOwn"
                    || text.ends_with(".hasOwn")
                    || text.ends_with("[\"hasOwn\"]")
                    || text.ends_with("['hasOwn']")
                    || text == "Object.hasOwn"
                    || text == "Object[\"hasOwn\"]"
                    || text == "Object['hasOwn']"
                    || text == "globalThis.Object.hasOwn"
                    || text == "globalThis.Object[\"hasOwn\"]"
                    || text == "globalThis.Object['hasOwn']"
                    || text == r#"globalThis["Object"].hasOwn"#
                    || text == r#"globalThis["Object"]["hasOwn"]"#
                    || text == r#"globalThis["Object"]['hasOwn']"#
                    || text == r#"globalThis['Object'].hasOwn"#
                    || text == r#"globalThis['Object']['hasOwn']"#
                    || text == r#"globalThis['Object']["hasOwn"]"# =>
            {
                true
            }
            Some("call") if receiver_text.contains("hasOwnProperty") => true,
            _ => false,
        }
    }

    pub(crate) fn is_object_from_entries_call(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Call {
            return false;
        }

        let Some(callee) = node.children.first().copied() else {
            return false;
        };
        let callee_node = self.node(callee);
        let _receiver_text = callee_node
            .children
            .first()
            .and_then(|receiver| self.node(*receiver).text.as_deref())
            .unwrap_or_default();
        matches!(
            callee_node.text.as_deref(),
            Some(text)
                if text == "fromEntries"
                    || text.ends_with(".fromEntries")
                    || text.ends_with("[\"fromEntries\"]")
                    || text.ends_with("['fromEntries']")
                    || text == r#"globalThis["Object"]["fromEntries"]"#
                    || text == r#"globalThis["Object"]['fromEntries']"#
                    || text == r#"globalThis['Object']["fromEntries"]"#
                    || text == r#"globalThis['Object']['fromEntries']"#
        )
    }

    pub(crate) fn static_object_has_own(
        &self,
        object_id: LirNodeId,
        key_id: LirNodeId,
    ) -> Option<bool> {
        // BOTH sides of every comparison below go through
        // `static_property_key_text`. The probe used to arrive as a
        // `render_static_value` string while the stored keys were read as raw
        // HIR text, and that asymmetry folded `Object.hasOwn({1e21: 1}, 1e21)`
        // to a wrong `false` with no diagnostic.
        let key = self.static_property_key_text(key_id, KeyTextSlot::Expression)?;
        let resolved = self
            .resolve_literal_aggregate(object_id)
            .unwrap_or(object_id);
        let object = self.node(resolved);
        if self.is_object_literal(object) {
            // Deliberately NOT `object_literal_field`: that helper is fed raw
            // HIR text by every one of its other callers (member-access
            // property names, inferred shape field names), so it is symmetric
            // for them and must stay on the raw-text currency.
            return Some(object.children.iter().any(|child| {
                let property = self.node(*child);
                property.children.len() == 2
                    && self
                        .static_property_key_text(
                            property.children[0],
                            KeyTextSlot::ObjectLiteralKey,
                        )
                        .is_some_and(|stored| stored == key)
            }));
        }

        // An empty aggregate literal (`{}` / `[]`) is a text-less `Value` with
        // no children — `is_object_literal` rejects it (an empty object and an
        // empty array are indistinguishable at this node), but either way it has
        // NO own enumerable keys, so `hasOwn` of any key is provably false.
        if object.kind == LirNodeKind::Value && object.text.is_none() && object.children.is_empty()
        {
            return Some(false);
        }

        if self.is_object_from_entries_call(object) {
            return self.static_object_from_entries_has_key(object, &key);
        }

        // Materialized fixed-shape heap object: since Lane A (throw-fallout
        // Stage 2), a quoted-string-key object literal (`{ a: 1, "b": 2 }`)
        // carries a real interned shape and is allocated as a heap struct, so
        // it is NO LONGER a fold-inlined literal — `resolve_literal_aggregate`
        // stops at the bound identifier, not an object-literal node. Prove
        // `hasOwn` against the shape's field set instead (the shape's field
        // names ARE the object's own enumerable keys). Without this the call
        // falls through to the placeholder backstop at the call site, so a
        // provable `Object.hasOwn` on such an object would emit a `false`
        // placeholder instead of folding to the true answer.
        // This lane compares the canonical probe against RAW interned field
        // names, which is the one place the two currencies could still diverge --
        // except that a shape's field names can never be numeric: an object
        // literal with a numeric property name is rejected outright
        // (`E5506` "object literal ... uses a numeric property name"), so it
        // never materializes and never interns a shape. Measured: every
        // numeric-key `hasOwn` resolves through the object-literal lane above,
        // and forcing materialization (mutating through a function parameter)
        // fails the compile instead of reaching here.
        if let Some(shape) = self.object_shape_of_node(resolved) {
            return Some(self.repr_table.shape_field(shape, &key).is_some());
        }

        None
    }

    pub(crate) fn static_object_from_entries_has_key(
        &self,
        call: &LirNode,
        key: &str,
    ) -> Option<bool> {
        let entries_id = call.children.get(1).copied()?;
        let entries_id = self.resolve_literal_aggregate(entries_id)?;
        let entries_node = self.node(entries_id);
        if !self.is_array_literal(entries_node) {
            return None;
        }

        for entry_id in &entries_node.children {
            let entry_id = self.resolve_literal_aggregate(*entry_id)?;
            let entry_node = self.node(entry_id);
            if !self.is_array_literal(entry_node) || entry_node.children.len() != 2 {
                return None;
            }

            // Same currency as `static_object_has_own`'s probe, which is where
            // `key` comes from -- this lane was already symmetric because both
            // sides rendered, and it stays symmetric by sharing the function.
            let rendered_key =
                self.static_property_key_text(entry_node.children[0], KeyTextSlot::Expression)?;
            if rendered_key == key {
                return Some(true);
            }
        }

        Some(false)
    }

    pub(crate) fn is_object_enumeration_call(
        &self,
        node: &LirNode,
    ) -> Option<ObjectEnumerationMode> {
        let node = if node.kind == LirNodeKind::Value && node.children.len() == 1 {
            self.node(node.children[0])
        } else {
            node
        };

        if node.kind != LirNodeKind::Call {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee = self.resolve_literal_aggregate(callee).unwrap_or(callee);
        let callee_node = self.node(callee);
        let mode = match callee_node.text.as_deref() {
            Some(text)
                if text == "keys"
                    || text.ends_with(".keys")
                    || text.ends_with("[\"keys\"]")
                    || text.ends_with("['keys']")
                    || text == "Object.keys"
                    || text == "Object[\"keys\"]"
                    || text == "Object['keys']"
                    || text == "globalThis.Object.keys"
                    || text == "globalThis.Object[\"keys\"]"
                    || text == "globalThis.Object['keys']"
                    || text == r#"globalThis["Object"].keys"#
                    || text == r#"globalThis["Object"]["keys"]"#
                    || text == r#"globalThis["Object"]['keys']"#
                    || text == r#"globalThis['Object'].keys"#
                    || text == r#"globalThis['Object']['keys']"#
                    || text == r#"globalThis['Object']["keys"]"# =>
            {
                ObjectEnumerationMode::Keys
            }
            Some(text)
                if text == "ownKeys"
                    || text.ends_with(".ownKeys")
                    || text.ends_with("[\"ownKeys\"]")
                    || text.ends_with("['ownKeys']")
                    || text == "Reflect.ownKeys"
                    || text == "Reflect[\"ownKeys\"]"
                    || text == "Reflect['ownKeys']"
                    || text == "globalThis.Reflect.ownKeys"
                    || text == "globalThis.Reflect[\"ownKeys\"]"
                    || text == "globalThis.Reflect['ownKeys']"
                    || text == r#"globalThis["Reflect"].ownKeys"#
                    || text == r#"globalThis["Reflect"]["ownKeys"]"#
                    || text == r#"globalThis["Reflect"]['ownKeys']"#
                    || text == r#"globalThis['Reflect'].ownKeys"#
                    || text == r#"globalThis['Reflect']['ownKeys']"#
                    || text == r#"globalThis['Reflect']["ownKeys"]"# =>
            {
                ObjectEnumerationMode::ReflectOwnKeys
            }
            Some(text)
                if text == "values"
                    || text.ends_with(".values")
                    || text.ends_with("[\"values\"]")
                    || text.ends_with("['values']")
                    || text == "Object.values"
                    || text == "Object[\"values\"]"
                    || text == "Object['values']"
                    || text == "globalThis.Object.values"
                    || text == "globalThis.Object[\"values\"]"
                    || text == "globalThis.Object['values']"
                    || text == r#"globalThis["Object"].values"#
                    || text == r#"globalThis["Object"]["values"]"#
                    || text == r#"globalThis["Object"]['values']"#
                    || text == r#"globalThis['Object'].values"#
                    || text == r#"globalThis['Object']['values']"#
                    || text == r#"globalThis['Object']["values"]"# =>
            {
                ObjectEnumerationMode::Values
            }
            Some(text)
                if text == "entries"
                    || text.ends_with(".entries")
                    || text.ends_with("[\"entries\"]")
                    || text.ends_with("['entries']")
                    || text == "Object.entries"
                    || text == "Object[\"entries\"]"
                    || text == "Object['entries']"
                    || text == "globalThis.Object.entries"
                    || text == "globalThis.Object[\"entries\"]"
                    || text == "globalThis.Object['entries']"
                    || text == r#"globalThis["Object"].entries"#
                    || text == r#"globalThis["Object"]["entries"]"#
                    || text == r#"globalThis["Object"]['entries']"#
                    || text == r#"globalThis['Object'].entries"#
                    || text == r#"globalThis['Object']['entries']"#
                    || text == r#"globalThis['Object']["entries"]"#
                    || text == r#"globalThis["Object"]['entries']"# =>
            {
                ObjectEnumerationMode::Entries
            }
            _ => return None,
        };

        let object = callee_node.children.first().copied()?;
        let object = self.resolve_transparent_object_root_node(object)?;
        let object_text = self.node(object).text.as_deref().unwrap_or_default();
        if object_text.contains("Object") || object_text.contains("Reflect") {
            Some(mode)
        } else {
            None
        }
    }

    pub(crate) fn collect_object_enumeration_iteration_items(
        &mut self,
        node: &LirNode,
        mode: ObjectEnumerationMode,
        items: &mut Vec<LirNodeId>,
    ) -> bool {
        if let Some(string_text) = self.render_static_string_value(node) {
            if matches!(mode, ObjectEnumerationMode::ReflectOwnKeys) {
                return false;
            }
            for (index, value) in string_text.chars().enumerate() {
                let key = self.alloc_scratch_node(
                    LirNodeKind::Literal,
                    Some(format!("{index:?}")),
                    vec![],
                );
                let value = self.alloc_scratch_node(
                    LirNodeKind::Literal,
                    Some(format!("{value:?}")),
                    vec![],
                );
                match mode {
                    ObjectEnumerationMode::Keys | ObjectEnumerationMode::ReflectOwnKeys => {
                        items.push(key)
                    }
                    ObjectEnumerationMode::Values => items.push(value),
                    ObjectEnumerationMode::Entries => {
                        let pair =
                            self.alloc_scratch_node(LirNodeKind::Value, None, vec![key, value]);
                        items.push(pair);
                    }
                }
            }

            return true;
        }

        if self.is_object_literal(node) {
            for child in &node.children {
                let property = self.node(*child);
                if property.children.len() != 2 {
                    return false;
                }

                let key = property.children[0];
                let key_node = self.node(key);
                if key_node.kind != LirNodeKind::Literal || key_node.text.is_none() {
                    return false;
                }

                match mode {
                    ObjectEnumerationMode::Keys | ObjectEnumerationMode::ReflectOwnKeys => {
                        items.push(key)
                    }
                    ObjectEnumerationMode::Values => items.push(property.children[1]),
                    ObjectEnumerationMode::Entries => {
                        let pair = self.alloc_scratch_node(
                            LirNodeKind::Value,
                            None,
                            vec![key, property.children[1]],
                        );
                        items.push(pair);
                    }
                }
            }

            return true;
        }

        if self.is_object_from_entries_call(node) {
            return self.collect_object_from_entries_iteration_items(node, mode, items);
        }

        false
    }

    pub(crate) fn collect_object_from_entries_iteration_items(
        &mut self,
        node: &LirNode,
        mode: ObjectEnumerationMode,
        items: &mut Vec<LirNodeId>,
    ) -> bool {
        let Some(entries_id) = node.children.get(1).copied() else {
            return false;
        };
        let Some(entries_id) = self.resolve_literal_aggregate(entries_id) else {
            return false;
        };
        let entries_node = self.node(entries_id).clone();
        if !self.is_array_literal(&entries_node) {
            return false;
        }

        let mut ordered = Vec::with_capacity(entries_node.children.len());
        for entry_id in &entries_node.children {
            let Some(entry_id) = self.resolve_literal_aggregate(*entry_id) else {
                return false;
            };
            let entry_node = self.node(entry_id).clone();
            if !self.is_array_literal(&entry_node) || entry_node.children.len() != 2 {
                return false;
            }

            let Some(key_text) = self.render_static_value(entry_node.children[0]) else {
                return false;
            };
            let value_id = entry_node.children[1];
            if let Some((_, existing_value)) = ordered
                .iter_mut()
                .find(|(existing_key, _)| existing_key == &key_text)
            {
                *existing_value = value_id;
            } else {
                ordered.push((key_text, value_id));
            }
        }

        for (key_text, value_id) in ordered {
            match mode {
                ObjectEnumerationMode::Keys | ObjectEnumerationMode::ReflectOwnKeys => {
                    items.push(self.alloc_scratch_node(
                        LirNodeKind::Literal,
                        Some(format!("{key_text:?}")),
                        vec![],
                    ))
                }
                ObjectEnumerationMode::Values => items.push(value_id),
                ObjectEnumerationMode::Entries => {
                    let key = self.alloc_scratch_node(
                        LirNodeKind::Literal,
                        Some(format!("{key_text:?}")),
                        vec![],
                    );
                    let pair =
                        self.alloc_scratch_node(LirNodeKind::Value, None, vec![key, value_id]);
                    items.push(pair);
                }
            }
        }

        true
    }
}

#[cfg(test)]
#[path = "object_tests.rs"]
mod object_tests;
