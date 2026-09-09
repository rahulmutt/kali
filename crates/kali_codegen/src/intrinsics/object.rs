//! Object literal and Object built-in intrinsic recognition and constant-folding.
use crate::*;
use kali_common::js_number::format_js_number;

/// The property key an EXPRESSION denotes, computed the way JS does
/// (`String(key)`).
///
/// Only one currency exists now: a key-slot node's text is already the
/// property name (`kali_hir`'s `lower_property_name`), so this function is for
/// the PROBE side alone -- the key an expression evaluates to. Its convention
/// is invertible and was never the defect: a quote character means a string
/// literal whose content is the key, an `n` suffix means BigInt digits, and a
/// bare text is a number's spelling to be rendered.
pub(crate) fn canonical_property_key_text(text: &str) -> String {
    // NOT trimmed: whitespace in a key is never padding. `{" a ": 1}`'s key is
    // the three-character name ` a `, and the probe `o[" a "]` must keep it.
    //
    // The length guard is a BYTE length while the delimiter tests below are on
    // CHARS, which is what keeps a one-character multi-byte text such as `é`
    // (two bytes, one char) from ever being read as quoted. Keep that pairing;
    // the `[1..len - 1]` slices are only ever reached once both ends are known
    // to be ASCII quotes, so they cannot split a multi-byte char.
    let long_enough = text.len() >= 2;
    let quoted = long_enough
        && matches!(
            (text.chars().next(), text.chars().last()),
            (Some('"'), Some('"')) | (Some('\''), Some('\'')) | (Some('`'), Some('`'))
        );
    if quoted {
        return text[1..text.len() - 1].to_string();
    }
    // `String(42n)` is "42": exact, and textual, so the digits of a BigInt too
    // large for an `f64` survive.
    if is_bigint_literal_text(text) {
        return text[..text.len() - 1].to_string();
    }
    parse_numeric_literal_value(text)
        .map(format_js_number)
        .unwrap_or_else(|| text.to_string())
}

impl<'a> FunctionEmitter<'a> {
    /// The property name a KEY-SLOT node holds.
    ///
    /// Its text is already `String(key)`, so this reads it. Key-slot nodes are
    /// never resolved as bindings (`{a: 1}`'s key is the name `a`, not the
    /// value of a variable `a`), which is why this does not defer to
    /// `render_static_value`.
    pub(crate) fn static_object_key_text(&self, id: LirNodeId) -> Option<String> {
        let node = self.node(id);
        (node.kind == LirNodeKind::Literal)
            .then(|| node.text.clone())
            .flatten()
    }

    /// The property name a PROBE expression denotes.
    pub(crate) fn static_probe_key_text(&self, id: LirNodeId) -> Option<String> {
        let node = self.node(id);
        if node.kind == LirNodeKind::Literal {
            return Some(canonical_property_key_text(node.text.as_deref()?));
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

    /// The value node an object literal stores under the property name
    /// `field`, or `None` when it has no such own property.
    ///
    /// BOTH sides are property names and neither is un-quoted. `field` reaches
    /// this from a member node's text (`kali_parser`'s
    /// `expression_to_property_name`), a `kali_types` shape field name, or a
    /// normalized static index -- all `String(key)` -- and the stored side is a
    /// key slot's text, which IS `String(key)` since `lower_property_name`.
    ///
    /// **"ALL `String(key)`" IS TRUE OF THE STORED SIDE AND NOT OF THE PROBE,
    /// AND THAT IS REGISTER ENTRY R-59** (§2, Tier 2, filed 2026-09-08).
    /// `expression_to_property_name` reads an index statically only for a
    /// literal, a sequence ending in one, and a folded `+`/`-` unary on one;
    /// for every other shape it FABRICATES a name -- the identifier's own text
    /// for `o[i]`, the literal string `index` for the catch-all. This scan then
    /// finds that fabricated name, and if the receiver happens to declare a
    /// property under it the read returns THAT property's value. Measured at
    /// `35e9ef4ef6` (the tree the binary was built from) against node v26.8.1,
    /// both scopes: over
    /// `const o = {index: 9, i: 7}; let i = 1;`, `o[i]` reads `7` and
    /// `o[i + 0]` reads `9` where node reads `undefined` twice. The un-quoting
    /// symmetry this comment establishes is real; the currency claim held only
    /// for the index shapes that phase actually read.
    ///
    /// **RESOLVED 2026-09-09 at `71b5f42f6c`, and the qualification is
    /// withdrawn.** R-59 is retired (FAIL_CLOSED) by the
    /// computed-member-static-name project. The one-currency claim now holds
    /// for **every** computed access that HAS a name, because a computed access
    /// whose index the parser cannot read no longer has one:
    /// `expression_to_property_name` returns `Option<String>` and declines,
    /// `MemberExpression.property` is `None`, and the node carries its own kind
    /// (`LirNodeKind::ComputedMember`) below HIR, so it never reaches a
    /// name-reading consumer such as this scan -- it is refused at the
    /// checker's gate or at codegen's single computed-member gateway first.
    /// There is no longer a fabricated name for `field` to be, so both sides
    /// are `String(key)` unconditionally, and the paragraph above is kept as
    /// the record of what was true until that commit.
    ///
    /// The one pre-existing exception is ESCAPE SEQUENCES: `{"a\"b": 1}` stores
    /// the undecoded four-character text `a\"b` (the delimiters are stripped,
    /// the escape is not decoded), and a probe written the SAME way -- the
    /// identical source spelling `o['a\"b']` -- arrives undecoded too, so the
    /// two agree only because they are byte-identical text. Neither is the
    /// three-character name node uses, and the agreement does not extend past
    /// this direct comparison: the enumeration lane (`fold_object_enumeration_
    /// call`) re-encodes the same undecoded text with a second escaping pass,
    /// so a key read back out through `Object.keys` diverges again (its
    /// `.length` is 6, not 4). A probe spelled `o['a"b']` (the real, decoded
    /// name) therefore misses. Recorded and pinned, not fixed here; see
    /// docs/superpowers/followups/property-key-trim-site-classification.md,
    /// and **register entry R-57** (§2, Tier 2, filed 2026-09-08 at
    /// `b13c890330`, off `dde0f083c0`), which now owns this divergence and its fix direction.
    /// The fix belongs in `kali_parser`'s `unquote_string_literal`, NOT here:
    /// un-escaping at this comparison would repeat the `trim_matches('"')`
    /// mistake R-56's closure deleted from fourteen sites.
    ///
    /// It used to strip `"` off both sides, which is a guess at the key's type
    /// from its punctuation rather than a property-name comparison, and the
    /// guess invented properties: `const p = {'"a"': 1}; p['a']` read `1`
    /// where node reads `undefined`, in a program whose `Object.hasOwn(p,'a')`
    /// -- which stopped guessing first -- already answered `false`. One
    /// program contradicting itself is R-56's class at a second address.
    pub(crate) fn object_literal_field(&self, node: &LirNode, field: &str) -> Option<LirNodeId> {
        if !self.is_object_literal(node) {
            return None;
        }

        for child in &node.children {
            let property = self.node(*child);
            if property.children.len() != 2 {
                continue;
            }
            let key = self.node(property.children[0]).text.as_deref()?;
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
        // BOTH sides of every comparison below are in the SAME currency: the
        // JavaScript property name. The probe used to arrive as a
        // `render_static_value` string while the stored keys were read as raw
        // HIR text, and that asymmetry folded `Object.hasOwn({1e21: 1}, 1e21)`
        // to a wrong `false` with no diagnostic. It is true by construction
        // now -- a key-slot node's text IS the name, so the stored side only
        // reads it.
        //
        // ONE EXCEPTION, pre-existing and not this lane's: a key whose source
        // spelling contains an ESCAPE SEQUENCE is stored undecoded, because
        // `kali_parser`'s `unquote_string_literal` strips the delimiters
        // without decoding. `{"a\"b": 1}` stores the four-character text
        // `a\"b`, not the three-character name `a"b`. THIS fold's probe and
        // stored key agree ONLY because they are byte-identical source
        // spellings, not because either side holds the real property name --
        // `Object.hasOwn(o, "a\"b")` folds to `true` here, but the SAME
        // object's enumerated key fails a strict-equality probe against that
        // identical literal (`k === "a\"b"` is `false` for `k` read out of
        // `Object.keys(o)`), because the enumeration lane
        // (`fold_object_enumeration_call`) re-encodes the undecoded text with
        // a second escaping pass that this fold never sees. "Self-consistent"
        // describes this one comparison, not the key's behaviour across the
        // object model. See
        // docs/superpowers/followups/property-key-trim-site-classification.md
        // section 6.
        let key = self.static_probe_key_text(key_id)?;
        let resolved = self
            .resolve_literal_aggregate(object_id)
            .unwrap_or(object_id);
        let object = self.node(resolved);
        if self.is_object_literal(object) {
            // This scan and `object_literal_field` now compare the same way --
            // property name against property name, neither side un-quoted.
            // (The un-quoting that helper used to do, and the reason this lane
            // deliberately avoided it, is gone: it was one of the fourteen
            // sites Task 5 deleted, and the comment that used to point at it
            // here has been removed rather than left to rot.)
            //
            // ONE difference remains, and it is why this is still not a call to
            // that helper: `object_literal_field` reads the key with
            // `.as_deref()?`, so a TEXT-LESS key node aborts its whole scan and
            // it reports "no such field". Here a text-less key node makes only
            // that ONE property unmatchable and the scan continues, so a later
            // property really holding the probed name is still found. For a
            // `hasOwn` fold that is the difference between a wrong `false` and
            // the right `true`, so the more conservative scan stays.
            //
            // If that `?` is ever softened to a skip, these two collapse into
            // one and this should become
            // `self.object_literal_field(object, &key).is_some()`.
            return Some(object.children.iter().any(|child| {
                let property = self.node(*child);
                property.children.len() == 2
                    && self
                        .static_object_key_text(property.children[0])
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
        // names, which is the one place a shape's naming could still diverge
        // from the property name -- except that a shape's field names can
        // never be numeric: an object literal with a numeric property name is
        // rejected outright (`E5506` "object literal ... uses a numeric
        // property name", `kali_types/src/repr_infer.rs`), so it never
        // materializes and never interns a shape.
        //
        // MEASURED, with the exact programs, at the commit that wrote this:
        //
        //   function bump(o) { o.a = 9; }
        //   let o = {1: 1, a: 2};
        //   bump(o);
        //   console.log(o.a);
        //
        // fails to compile: `error[E5506]: object literal for
        // Binding("_start", "o") uses a numeric property name, which is
        // unavailable in the current phase`, exit 1. So forcing
        // materialization of a numeric-key literal fails the compile instead
        // of reaching here, and every numeric-key `hasOwn` that DOES compile
        // resolves through the object-literal lane above.
        //
        // The string keys that do intern are compared correctly, including
        // numeric-LOOKING ones:
        //
        //   function bump(o) { o.a = 9; }
        //   let o = {"1": 1, "1e+21": 2, a: 3};
        //   bump(o);
        //   Object.hasOwn(o, "1") / 1 / "1e+21" / 1e21 / "b"
        //
        // answers `true true true true false` at exit 0, which is node's
        // answer byte for byte.
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
            let rendered_key = self.static_probe_key_text(entry_node.children[0])?;
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
