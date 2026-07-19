//! Type-directed `===` / `!==` / `==` / `!=` and `??` decisions.
//!
//! kali stores every value in an untyped i64 slot: `0`, `false`, `null` and
//! `undefined` all lower to the bit pattern `0`, and `true` lowers to `1`. The
//! generic `i64.eq` lowering in `operators.rs` therefore reported `0 === null`,
//! `0 === false`, `null === undefined` and `1 === true` as `true` — wrong
//! CONTROL FLOW, since every `if (x === null)` guard fired when `x` was `0`.
//! The same raw-bit-pattern test (`i64.eqz`) made `??` treat `0` and `false`
//! as nullish.
//!
//! The closure is a compile-time TYPE classification of both operands
//! ([`EqClass`]). The classification is deliberately partial — the repr axes
//! carry no boolean/null/undefined dimension, so a plain `Repr::I64` binding
//! is genuinely unclassifiable and stays [`None`]. The decision table then
//! either folds the comparison to a constant, keeps the existing runtime
//! lowering where the bit-pattern test is PROVABLY the right test, or fails
//! closed with `E5506`. It never emits a boolean it cannot justify.
//!
//! Scope discipline: [`equality_decision`] returns [`EqDecision::Runtime`]
//! (leave the pre-existing lowering completely untouched) unless at least one
//! operand is proven `null`, `undefined` or boolean. Number-vs-number,
//! string-vs-string and object-vs-object comparisons are therefore byte-identical
//! to the pre-fix compiler.
//!
//! ## Documented residual inventory (unfixed on purpose; each is pinned honestly
//! rather than silently absorbed)
//!
//! 1. An `UntypedObjectField` operand proves nothing (see [`EqClass::is_unproven`])
//!    and keeps the pre-existing lowering even when the OTHER operand is a
//!    proven `null`/`undefined`/boolean, because the field slot may itself hold
//!    a pointer, a number or a boolean.
//! 2. An unprovable operand against a proven **boolean** keeps the pre-existing
//!    lowering (see the comment in [`FunctionEmitter::equality_decision`]):
//!    kali cannot prove "this returns a boolean" without a `Repr::Boolean` axis,
//!    and the corpus cost of failing it closed is 33 pinned programs.
//! 3. An unprovable operand against a proven **number** (including a NUMBER
//!    LITERAL) never even reaches the decision table: `Number` does not "arm
//!    the gate" (see [`EqClass::arms_the_gate`]), so the pair falls through to
//!    the pre-existing bit-pattern `i64.eq`, which is unsound whenever the
//!    unprovable side is at runtime `false`/`null`/`undefined` (all bit `0`).
//!    `function f(b){return b;} f(false) === 0` is `true` under kali, `false`
//!    under node. Same architectural blocker as residual 2. Pinned by
//!    `unprovable_operand_against_number_literal_is_a_known_residual`
//!    in `crates/kali_cli/tests/soundness_strict_equality.rs`.
use crate::*;

/// The JS type class of an operand, when compile-time provable.
///
/// Absence (`None` from [`FunctionEmitter::static_equality_class`]) means "kali
/// cannot prove what kind of value this slot holds", which for a comparison
/// against a `null`/`undefined`/boolean operand is a fail-closed condition, not
/// a licence to guess.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EqClass {
    /// A JS `number` (integer or float repr).
    Number,
    /// A JS `bigint` literal (`5n`). Distinct from `Number` because
    /// `5n === 5` is `false` in JS.
    BigInt,
    Boolean,
    String,
    Null,
    Undefined,
    /// A fixed-shape object-reference slot. A live reference is a NONZERO
    /// bump-allocated heap address and `null` lowers to `0`, so `i64.eq`
    /// against `null` is the CORRECT test — but the slot cannot be
    /// distinguished from `undefined`, which also lowers to `0`.
    ObjectOrNull,
    /// A `base.field` read whose shape-table repr is the untyped `I64`
    /// default. This is the ONE slot kind that may legitimately hold a raw
    /// object POINTER: a nested-object field interns as a plain `I64` pointer
    /// with no nested-object tracking (see `repr_infer`'s object-pointer field
    /// note), which is exactly the CLBG binary-trees `{ left, right }` shape
    /// whose `t.left === null` guard is correct today. It may equally hold a
    /// plain number or a boolean, so nothing can be PROVEN about it — the
    /// class exists only to keep those sites on their pre-existing lowering
    /// instead of failing closed. See the residual note on
    /// [`FunctionEmitter::equality_decision`].
    UntypedObjectField,
    /// A `Deno.env.get(...)` result: a string handle when the variable is set,
    /// and the `0` bit pattern (kali's `undefined`) when it is not. So
    /// `=== undefined` IS exact on it, `=== null` is not, and it is never a
    /// boolean.
    EnvGetResult,
}

/// What the type classification says the comparison site should emit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EqDecision {
    /// Fold to this constant boolean. Operand side effects are still emitted
    /// (and dropped) by the caller.
    Const(bool),
    /// The pre-existing bit-pattern lowering is provably correct here; leave
    /// it alone.
    Runtime,
    /// kali cannot justify any boolean at this site. Reject with `E5506`.
    FailClosed,
}

impl EqClass {
    /// True for the classes whose repr collides with a number's: these are the
    /// operands that make a raw `i64.eq` unsound, and the ONLY ones that arm
    /// the decision table (see the scope discipline note above).
    ///
    /// RESIDUAL 3 (unpinned prior to soundness-batch1-pra wave 0, now pinned by
    /// `unprovable_operand_against_number_literal_is_a_known_residual`
    /// in `soundness_strict_equality.rs`): `Number` is deliberately absent from
    /// this list, which means a proven `Number` operand — INCLUDING a literal
    /// like `0` — never arms the gate. Paired with an unprovable (`None`)
    /// operand, `equality_decision`'s `if !armed { return Runtime }` check
    /// fires immediately, so the pair never reaches the asymmetric
    /// one-side-classified branch that fails closed for `null`/`undefined` and
    /// knowingly keeps the runtime lowering for `boolean` (residual 2, below).
    /// The old unsound bit-pattern `i64.eq` runs unchecked. Concretely:
    /// `function f(b) { return b; } f(false) === 0` prints `true` (kali) vs
    /// `false` (node) — `f(false)`'s parameter is unprovable, `0` is a proven
    /// `Number`, neither side arms the gate, so the comparison never enters the
    /// type-directed table at all. This is the same wrong-CONTROL-FLOW shape as
    /// residual 1's null/undefined case, but reached by a different route (the
    /// gate is never armed, vs. armed-but-unproven-sibling), so it is tracked
    /// separately. NOT fixed here: the real fix needs the same `Repr::Boolean`
    /// axis residual 2 is blocked on, so that a `Number`-vs-unprovable pair
    /// could distinguish "unprovable but provably not boolean" from "unprovable
    /// and possibly boolean" instead of an all-or-nothing gate membership test.
    fn arms_the_gate(self) -> bool {
        matches!(self, EqClass::Null | EqClass::Undefined | EqClass::Boolean)
    }

    /// True when the class carries no proof at all — the decision table must
    /// leave these sites on their pre-existing lowering.
    fn is_unproven(self) -> bool {
        matches!(self, EqClass::UntypedObjectField)
    }

    fn is_nullish(self) -> bool {
        matches!(self, EqClass::Null | EqClass::Undefined)
    }

    /// `??` view: this class is ALWAYS nullish.
    pub(crate) fn is_nullish_class(self) -> bool {
        self.is_nullish()
    }

    /// `??` view: this class is NEVER nullish. `ObjectOrNull` is deliberately
    /// excluded — that slot may hold `null` — and keeps the runtime zero test,
    /// which is exact for it.
    pub(crate) fn is_never_nullish(self) -> bool {
        matches!(
            self,
            EqClass::Number | EqClass::BigInt | EqClass::Boolean | EqClass::String
        )
    }
}

/// Strict (`===`) decision for two PROVEN classes, in operand order-independent
/// form. `left`/`right` are interchangeable; every arm below is symmetric.
fn strict_decision(left: EqClass, right: EqClass) -> EqDecision {
    use EqClass::*;
    match (left, right) {
        // Same nullish class: `null === null`, `undefined === undefined`.
        (Null, Null) | (Undefined, Undefined) => EqDecision::Const(true),
        // Cross nullish: `null === undefined` is `false` in JS (this is
        // precisely the pair the shared `0` bit pattern collapsed).
        (Null, Undefined) | (Undefined, Null) => EqDecision::Const(false),
        // A `null` operand against an object-reference slot is the ONE case
        // where the bit-pattern test is exact: live pointer != 0 == null.
        // This is the binary-trees `t.left === null` shape.
        (Null, ObjectOrNull) | (ObjectOrNull, Null) => EqDecision::Runtime,
        // ... but `undefined` against that same slot is NOT decidable: a slot
        // holding `null` is bit-identical to `undefined`, so `i64.eq` would
        // answer `true` where node answers `false`.
        (Undefined, ObjectOrNull) | (ObjectOrNull, Undefined) => EqDecision::FailClosed,
        // `Deno.env.get(k) === undefined` is exact: unset reads back as the
        // `0` bit pattern, a set value as a nonzero string handle.
        (Undefined, EnvGetResult) | (EnvGetResult, Undefined) => EqDecision::Runtime,
        // ... but `=== null` on it is not: an unset variable would answer
        // `true` where node answers `false` (`undefined === null`).
        (Null, EnvGetResult) | (EnvGetResult, Null) => EqDecision::FailClosed,
        // A string-or-undefined is never a boolean.
        (Boolean, EnvGetResult) | (EnvGetResult, Boolean) => EqDecision::Const(false),
        // An untyped object FIELD proves nothing (it may hold a pointer, a
        // number or a boolean). Keep the pre-existing lowering rather than
        // reject: this is the documented residual, not a proof.
        (a, b) if a.is_unproven() || b.is_unproven() => EqDecision::Runtime,
        // Both booleans live in the 0/1 integer repr, so `i64.eq` is exact.
        (Boolean, Boolean) => EqDecision::Runtime,
        // Any remaining pair reaching here has at least one nullish/boolean
        // operand meeting a DIFFERENT proven class, and differing types are
        // never strictly equal.
        _ => EqDecision::Const(false),
    }
}

/// Loose (`==`) decision for two PROVEN classes. Differs from strict in that
/// `null == undefined` is `true` and a boolean is coerced with `ToNumber`
/// (which is exactly kali's 0/1 boolean repr, so the runtime compare stays
/// exact against numbers and bigints).
fn loose_decision(left: EqClass, right: EqClass) -> EqDecision {
    use EqClass::*;
    match (left, right) {
        // `null == null`, `null == undefined`, `undefined == undefined`.
        (a, b) if a.is_nullish() && b.is_nullish() => EqDecision::Const(true),
        (a, EnvGetResult) | (EnvGetResult, a) if a.is_nullish() => EqDecision::Runtime,
        (Boolean, EnvGetResult) | (EnvGetResult, Boolean) => EqDecision::FailClosed,
        // A nullish operand is loosely equal ONLY to the other nullish values,
        // so against a proven number/bigint/string/boolean it is `false`.
        (a, b) if a.is_unproven() || b.is_unproven() => EqDecision::Runtime,
        (a, ObjectOrNull) | (ObjectOrNull, a) if a.is_nullish() => EqDecision::Runtime,
        (a, _) | (_, a) if a.is_nullish() => EqDecision::Const(false),
        // `false == 0`, `true == 1`, `true == 1n`: ToNumber(boolean) is 0/1,
        // which IS the boolean repr, so the existing compare is exact.
        (Boolean, Boolean) | (Boolean, Number) | (Number, Boolean) => EqDecision::Runtime,
        (Boolean, BigInt) | (BigInt, Boolean) => EqDecision::Runtime,
        // `true == "1"` needs ToNumber on a runtime string, and
        // `true == someObject` needs ToPrimitive. Neither exists here.
        _ => EqDecision::FailClosed,
    }
}

impl FunctionEmitter<'_> {
    /// Compile-time JS type class of the value `id` produces, when provable.
    ///
    /// Every arm must be a PROOF, not a guess. In particular a bare
    /// `Repr::I64` binding is deliberately NOT classified as `Number`: the I64
    /// slot is also how kali stores booleans, `null`, `undefined`, interned
    /// key ordinals and abort handles, so classifying it would re-open exactly
    /// the conflation this module closes. Same caution as `typeof_static_text`.
    pub(crate) fn static_equality_class(&self, id: LirNodeId) -> Option<EqClass> {
        let id = self.unwrap_transparent(id);

        // Repr-backed proofs first — these hold for runtime values, not just
        // literals, and are the reason `s === null` (proven string) folds
        // instead of failing closed.
        if self.object_shape_of_node(id).is_some() {
            return Some(EqClass::ObjectOrNull);
        }
        if let Some(class) = self.object_field_equality_class(id) {
            return Some(class);
        }
        if self.is_bigint_literal_valued(id) {
            return Some(EqClass::BigInt);
        }
        if self.is_float_valued(id) {
            return Some(EqClass::Number);
        }
        if self.is_string_valued(id) {
            return Some(EqClass::String);
        }
        if self.is_env_get_string_call(id) {
            return Some(EqClass::EnvGetResult);
        }

        // Syntactic proofs. Resolve through the const-fold binding chain the
        // same way `typeof_static_text` does, so `const zero = 0; zero === null`
        // classifies `zero` rather than giving up on the identifier.
        let resolved = self.resolve_literal_aggregate(id).unwrap_or(id);
        let resolved = self.unwrap_transparent(resolved);
        let node = self.node(resolved);

        // Unary forms with a type-determined result. `void x` is ALWAYS
        // `undefined` and `!x` is ALWAYS a boolean, regardless of operand.
        // `-x` / `~x` are always numbers (a string operand under them is
        // already rejected in `emit_unary`).
        if node.children.len() == 1 {
            match node.text.as_deref() {
                Some("void") => return Some(EqClass::Undefined),
                Some("!") => return Some(EqClass::Boolean),
                Some("-") | Some("~") => {
                    // `-5n` stays a bigint; the bigint probe above already
                    // resolved that, so anything left here is a number.
                    return Some(EqClass::Number);
                }
                Some("typeof") => return Some(EqClass::String),
                // `delete o.k` always evaluates to a boolean.
                Some("delete") => return Some(EqClass::Boolean),
                _ => {}
            }
        }

        // A relational/equality operator always produces a boolean, whatever
        // its operands were.
        if node.children.len() == 2
            && matches!(
                node.text.as_deref(),
                Some("<" | "<=" | ">" | ">=" | "==" | "!=" | "===" | "!==" | "in" | "instanceof")
            )
        {
            return Some(EqClass::Boolean);
        }

        // Statically-folded CALL results (`Object.is(a, b)`, the `Number.is*`
        // predicates, `arr.at(oob)`, ...). The renderer is textual, so a call
        // that statically folds to the STRING "true" would render
        // indistinguishably — but every static-string-producing call is
        // already classified `String` by `is_string_valued` above and has
        // returned, so only genuinely boolean/nullish folds reach here.
        if node.kind == LirNodeKind::Call {
            match self.render_static_value(resolved).as_deref() {
                Some("true") | Some("false") => return Some(EqClass::Boolean),
                Some("undefined") => return Some(EqClass::Undefined),
                Some("null") => return Some(EqClass::Null),
                _ => {}
            }
        }

        // Bare global identifiers that lower as childless `Value` nodes.
        if node.kind == LirNodeKind::Value && node.children.is_empty() {
            return match node.text.as_deref() {
                Some("undefined") => Some(EqClass::Undefined),
                Some("NaN") | Some("Infinity") => Some(EqClass::Number),
                _ => None,
            };
        }

        if node.kind != LirNodeKind::Literal {
            return None;
        }
        let text = node.text.as_deref()?;
        let unquoted = text.trim_matches(|c| c == '"' || c == '\'');
        if unquoted.len() != text.len() {
            return Some(EqClass::String);
        }
        match text {
            "undefined" => Some(EqClass::Undefined),
            "null" => Some(EqClass::Null),
            "true" | "false" => Some(EqClass::Boolean),
            _ => crate::intrinsics::parse_numeric_literal_value(text).map(|_| EqClass::Number),
        }
    }

    /// Class of a `base.field` read, taken from the shape table. Mirrors the
    /// field arm of `is_float_valued`. An `I64` field is NOT classified (the
    /// slot may hold a boolean, a `null`, an ordinal or a handle).
    fn object_field_equality_class(&self, id: LirNodeId) -> Option<EqClass> {
        let node = self.node(id);
        if node.kind != LirNodeKind::Value || node.children.len() != 1 {
            return None;
        }
        let field = node.text.as_deref().filter(|text| !text.is_empty())?;
        let shape = self.object_shape_of_node(node.children[0])?;
        match self.repr_table.shape_field(shape, field)?.1 {
            kali_common::Repr::Object(_) => Some(EqClass::ObjectOrNull),
            kali_common::Repr::F64 => Some(EqClass::Number),
            kali_common::Repr::String => Some(EqClass::String),
            // The untyped `I64` default: a number, a boolean, a `null`, or a
            // raw nested-object pointer. Unprovable, but recognized so the
            // comparison keeps its pre-existing lowering.
            kali_common::Repr::I64 => Some(EqClass::UntypedObjectField),
            _ => None,
        }
    }

    /// The type-directed decision for an equality site, or [`EqDecision::Runtime`]
    /// when this gate does not apply (see the scope discipline note above).
    pub(crate) fn equality_decision(
        &self,
        op: &str,
        left: LirNodeId,
        right: LirNodeId,
    ) -> EqDecision {
        let strict = matches!(op, "===" | "!==");
        if !strict && !matches!(op, "==" | "!=") {
            return EqDecision::Runtime;
        }
        let left_class = self.static_equality_class(left);
        let right_class = self.static_equality_class(right);
        // Only a proven `null` / `undefined` / boolean operand arms the gate.
        // Everything else keeps the pre-existing lowering byte-for-byte.
        let armed = left_class.is_some_and(EqClass::arms_the_gate)
            || right_class.is_some_and(EqClass::arms_the_gate);
        if !armed {
            return EqDecision::Runtime;
        }
        let (Some(left_class), Some(right_class)) = (left_class, right_class) else {
            // Exactly one side is classified. What to do with the other,
            // unclassifiable, side is where this fix draws its line:
            //
            // * against a proven `null`/`undefined` the raw `i64.eq` is wrong
            //   for EVERY value the unknown slot can hold except a heap
            //   pointer (a number `0`, a `false`, an unset optional all read
            //   back as `true`). That is the briefed wrong-control-flow class
            //   — `if (x === null)` firing when `x` is `0` — so it fails
            //   closed.
            //
            // * against a proven BOOLEAN the raw `i64.eq` is CORRECT whenever
            //   the unknown slot is itself boolean-producing, because kali's
            //   boolean repr is exactly the integers 0 and 1. The corpus is
            //   dominated by that shape (`Object.is(a, b) !== true`,
            //   `delete o.k !== true`, `same(a, b) !== true` — 33 pinned
            //   programs), and kali cannot prove "returns a boolean" for a
            //   user function without a `Repr::Boolean` axis, which is
            //   separate approved scope. Rejecting them would trade a narrow
            //   residual for a broad regression, so these keep the
            //   pre-existing lowering. RESIDUAL: `f() === true` still answers
            //   `true` when `f` returns the NUMBER `1`.
            let known = left_class.or(right_class);
            return if known.is_some_and(EqClass::is_nullish) {
                EqDecision::FailClosed
            } else {
                EqDecision::Runtime
            };
        };
        let decision = if strict {
            strict_decision(left_class, right_class)
        } else {
            loose_decision(left_class, right_class)
        };
        match decision {
            // `!==` / `!=` negate a folded constant; the runtime and
            // fail-closed outcomes are operator-independent.
            EqDecision::Const(value) if matches!(op, "!==" | "!=") => EqDecision::Const(!value),
            other => other,
        }
    }

    /// Restates the emitted value of the branch `??` statically selected with
    /// the shape its proven class implies.
    ///
    /// `emit_node` reports `ValueShape::Scalar` for an interned string-handle
    /// literal, so propagating its `EmittedValue` verbatim would make
    /// `"x" + ("" ?? "y")` render the raw tagged handle as an integer. The
    /// class is a proof about the value, so it is the better shape source; a
    /// class that carries no shape information leaves the emitted value alone.
    pub(crate) fn selected_nullish_operand(
        &self,
        selected: LirNodeId,
        emitted: EmittedValue,
    ) -> EmittedValue {
        // `produced` is normalized to `true` unconditionally: both call sites
        // push an `i64.const 0` placeholder when the branch emitted nothing,
        // so a value IS on the stack either way and reporting otherwise would
        // desync the consumer's stack accounting.
        let shape = match self.static_equality_class(selected) {
            Some(EqClass::String) => ValueShape::String,
            Some(EqClass::Boolean) => ValueShape::Boolean,
            _ => emitted.shape,
        };
        EmittedValue {
            produced: true,
            shape,
        }
    }

    /// Emits `id` purely for its side effects and drops the result, so a folded
    /// comparison still runs what JS would have run.
    ///
    /// A literal or a bare identifier read has no side effect AND must not be
    /// re-emitted (re-emitting a const-fold alias would re-run its
    /// initializer), so it is skipped — the same carve-out `typeof`'s static
    /// lane uses.
    pub(crate) fn emit_operand_for_effects(&mut self, function: &mut Function, id: LirNodeId) {
        let operand = self.unwrap_transparent(id);
        let node = self.node(operand).clone();
        let effect_free = node.kind == LirNodeKind::Literal
            || (node.kind == LirNodeKind::Value && node.children.is_empty());
        if effect_free {
            return;
        }
        if self.emit_node(function, id, true).produced {
            function.instruction(&Instruction::Drop);
        }
    }

    /// Emits a folded boolean constant after running both operands' effects.
    pub(crate) fn emit_folded_equality(
        &mut self,
        function: &mut Function,
        left: LirNodeId,
        right: LirNodeId,
        value: bool,
    ) -> EmittedValue {
        self.emit_operand_for_effects(function, left);
        self.emit_operand_for_effects(function, right);
        function.instruction(&Instruction::I64Const(if value { 1 } else { 0 }));
        EmittedValue {
            produced: true,
            shape: ValueShape::Boolean,
        }
    }

    /// Rejects an equality site whose operand types kali cannot reconcile.
    pub(crate) fn reject_unprovable_equality(
        &mut self,
        function: &mut Function,
        op: &str,
    ) -> EmittedValue {
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "operator '{op}' cannot be decided here: kali stores `null`, `undefined`, `false` and `0` in the same untyped i64 slot, and the type of at least one operand is not provable at this site, so no boolean can be emitted without risking wrong control flow; compare against a value whose type is provable (a literal, a proven string/float/object operand) or use the later compatibility path"
            ),
        ));
        function.instruction(&Instruction::I64Const(0));
        EmittedValue {
            produced: true,
            shape: ValueShape::Boolean,
        }
    }
}
