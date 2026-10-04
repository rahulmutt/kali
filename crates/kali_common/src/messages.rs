/// Canonical feature-unavailable wording for the supported async class-method lowering slice.
pub const fn async_class_method_lowering_unavailable_message() -> &'static str {
    "async class method lowering is unavailable in the direct runtime path; use a plain method or the later compatibility path"
}

/// Canonical feature-unavailable wording for the supported generator class-method lowering slice.
pub const fn generator_class_method_lowering_unavailable_message(is_async: bool) -> &'static str {
    if is_async {
        "async-generator class method lowering is unavailable in the direct runtime path; use a plain or async method, or the later compatibility path"
    } else {
        "generator class method lowering is unavailable in the direct runtime path; use a plain or async method, or the later compatibility path"
    }
}

/// Canonical feature-unavailable wording for generator-class-method yield-delegation slices.
pub const fn generator_class_method_yield_lowering_unavailable_message(
    is_async: bool,
    is_delegate: bool,
) -> &'static str {
    match (is_async, is_delegate) {
        (true, true) => {
            "async-generator class method lowering is unavailable in the direct runtime path for yield* delegation; use a plain or async method, or the later compatibility path"
        }
        (true, false) => generator_class_method_lowering_unavailable_message(true),
        (false, true) => {
            "generator class method lowering is unavailable in the direct runtime path for yield* delegation; use a plain or async method, or the later compatibility path"
        }
        (false, false) => generator_class_method_lowering_unavailable_message(false),
    }
}

/// Canonical feature-unavailable wording for mixed generator/async-generator class-method lowering slices.
pub const fn generator_class_method_lowering_unavailable_message_for_flavors(
    has_generator: bool,
    has_async_generator: bool,
) -> &'static str {
    match (has_generator, has_async_generator) {
        (true, true) => "generator and async-generator class method lowering is unavailable in the direct runtime path; use a plain or async method, or the later compatibility path",
        (true, false) => generator_class_method_lowering_unavailable_message(false),
        (false, true) => generator_class_method_lowering_unavailable_message(true),
        (false, false) => generator_class_method_lowering_unavailable_message(false),
    }
}

/// Canonical feature-unavailable wording for mixed generator/async-generator class-method yield-delegation slices.
pub const fn generator_class_method_yield_lowering_unavailable_message_for_flavors(
    has_generator: bool,
    has_async_generator: bool,
    is_delegate: bool,
) -> &'static str {
    match (has_generator, has_async_generator, is_delegate) {
        (true, true, true) => {
            "generator and async-generator class method lowering is unavailable in the direct runtime path for yield* delegation; use a plain or async method, or the later compatibility path"
        }
        (true, true, false) => {
            generator_class_method_lowering_unavailable_message_for_flavors(true, true)
        }
        (true, false, true) => generator_class_method_yield_lowering_unavailable_message(false, true),
        (true, false, false) => generator_class_method_lowering_unavailable_message(false),
        (false, true, true) => generator_class_method_yield_lowering_unavailable_message(true, true),
        (false, true, false) => generator_class_method_lowering_unavailable_message(true),
        (false, false, true) => generator_class_method_lowering_unavailable_message(false),
        (false, false, false) => generator_class_method_lowering_unavailable_message(false),
    }
}

/// Canonical feature-unavailable wording for the supported generator-function lowering slice.
pub const fn generator_function_lowering_unavailable_message(is_async: bool) -> &'static str {
    if is_async {
        "async-generator function lowering is unavailable in the current phase; use a synchronous function or the later compatibility path"
    } else {
        "generator function lowering is unavailable in the current phase; use a synchronous function or the later compatibility path"
    }
}

/// Canonical feature-unavailable wording for yield-delegation slices.
pub const fn generator_function_yield_lowering_unavailable_message(
    is_async: bool,
    is_delegate: bool,
) -> &'static str {
    match (is_async, is_delegate) {
        (true, true) => "async-generator function lowering is unavailable in the current phase for yield* delegation; use a synchronous function or the later compatibility path",
        (true, false) => generator_function_lowering_unavailable_message(true),
        (false, true) => "generator function lowering is unavailable in the current phase for yield* delegation; use a synchronous function or the later compatibility path",
        (false, false) => generator_function_lowering_unavailable_message(false),
    }
}

/// Canonical feature-unavailable wording for mixed generator/async-generator function lowering slices.
pub const fn generator_function_lowering_unavailable_message_for_flavors(
    has_generator: bool,
    has_async_generator: bool,
) -> &'static str {
    match (has_generator, has_async_generator) {
        (true, true) => "generator and async-generator function lowering is unavailable in the current phase; use a synchronous function or the later compatibility path",
        (true, false) => generator_function_lowering_unavailable_message(false),
        (false, true) => generator_function_lowering_unavailable_message(true),
        (false, false) => generator_function_lowering_unavailable_message(false),
    }
}

/// Canonical feature-unavailable wording for a computed member access whose
/// index neither the parser nor the `const` fold can name, and which no
/// runtime lane admits. Used verbatim by `kali_types` (so `kali check`
/// refuses) and `kali_codegen` (so `kali build`/`run` refuse the same way).
/// Spec: docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md §4.4.
pub const fn computed_member_access_unavailable_message() -> &'static str {
    "computed member access `o[k]` is unavailable in the current phase unless the index is a literal or a compile-time-constant `const` binding, or the receiver is a runtime array or a `for..in` key over the same object"
}

/// Canonical feature-unavailable wording for indexing a statically-known
/// string, in both the literal (`s[1]`) and the folded (`const k = 1; s[k]`)
/// spelling. The literal spelling was a silent `0` before; there is no
/// character fold here on purpose (spec §4.4, "String receivers").
pub const fn string_index_access_unavailable_message() -> &'static str {
    "indexing a string `s[i]` is unavailable in the current phase; use `charAt`/`at` on a statically-known ASCII string or the later compatibility path"
}

#[cfg(test)]
#[path = "messages_tests.rs"]
mod messages_tests;

/// Refusal reasons for an array-shaped return that is not admitted
/// (docs/superpowers/specs/2026-10-02-array-return-design.md §3.1).
pub const ARRAY_RETURN_MIXED: &str = "it mixes array and non-array returns";
pub const ARRAY_RETURN_ELEMENT: &str = "an element is not an integer";
pub const ARRAY_RETURN_GROWABLE: &str = "it returns a growable array";
pub const ARRAY_RETURN_FORM: &str =
    "only a `function` declaration with a unique name can return an array";
pub const ARRAY_RETURN_LET_LITERAL: &str =
    "it returns a `let`/`var` binding of an array literal, which can be reassigned";
pub const ARRAY_RETURN_CONST_COMPUTED: &str = "it returns a `const` binding of an array literal with computed elements, which kali would re-evaluate at the return";

pub fn array_return_refused_message(func: &str, reason: &str) -> String {
    format!("returning an array from `{func}` is unavailable in the current phase: {reason}")
}

/// [`array_return_refused_message`] for an anonymous function that has no
/// source name: one called immediately (anon-array-return spec §3.3).
pub fn array_return_refused_message_anonymous(reason: &str) -> String {
    format!(
        "returning an array from an immediately-invoked function is unavailable in the current phase: {reason}"
    )
}

/// Methods with no lowering on a plain `[len][elem…]` runtime array: the
/// length changers (the array has a fixed length) and the in-place reorderers.
/// Each refuses on one (array-bounds spec §3.2, literal-array-mutators spec §3.1).
/// `fill` is absent: the plain lane lowers it.
pub const RUNTIME_ARRAY_MUTATORS: &[&str] = &[
    "push",
    "pop",
    "shift",
    "unshift",
    "splice",
    "reverse",
    "sort",
    "copyWithin",
];

/// In-place mutators refused on a literal array, which kali folds to its
/// initial elements (literal-array-mutators spec §3.1). The runtime list plus
/// `fill`, which the literal lane does not lower either.
pub const LITERAL_ARRAY_MUTATORS: &[&str] = &[
    "push",
    "pop",
    "shift",
    "unshift",
    "splice",
    "reverse",
    "sort",
    "copyWithin",
    "fill",
];

/// Canonical wording for a negative integer-literal index on a plain runtime array.
pub const fn runtime_array_negative_index_unavailable_message() -> &'static str {
    "a negative index on a runtime array is unavailable in the current phase: node reads `undefined` there and kali has no `undefined` value, so kali refuses rather than read the array's length header"
}

/// Canonical wording for a [`RUNTIME_ARRAY_MUTATORS`] call on a plain runtime array.
pub fn runtime_array_mutator_unavailable_message(method: &str) -> String {
    format!(
        "calling `.{method}()` on a runtime array is unavailable in the current phase: kali has no lowering of it on this array, so kali refuses rather than silently skip the call"
    )
}

/// Canonical wording for a [`LITERAL_ARRAY_MUTATORS`] call on a literal array.
pub fn literal_array_mutator_unavailable_message(method: &str) -> String {
    format!(
        "calling `.{method}()` on a literal array is unavailable in the current phase: kali folds a literal array to its initial elements, so kali refuses rather than silently skip the call"
    )
}

/// Canonical wording for an assignment to a literal array's `.length`.
pub const fn literal_array_length_write_unavailable_message() -> &'static str {
    "assigning to `.length` of a literal array is unavailable in the current phase: kali folds a literal array to its initial elements, so kali refuses rather than silently skip the write"
}

/// Canonical wording for an element store into a literal array (`a[0] = 7`).
pub const fn literal_array_store_unavailable_message() -> &'static str {
    "mutating a literal array is unavailable in the current direct-runtime path; use new Array(n) for runtime mutation"
}

/// The `run` backstop's wording: a [`LITERAL_ARRAY_MUTATORS`] name reached the
/// placeholder fallback, so kali cannot tell which array, if any, it mutates.
pub fn array_mutator_unresolved_receiver_message(method: &str) -> String {
    format!(
        "calling `.{method}()` is unavailable in the current phase: it mutates an array in place and kali could not prove which array the receiver is, so kali refuses rather than silently skip the call"
    )
}

/// The methods every object inherits from `Object.prototype`. The `check`
/// mirror of the unresolved-member-call gate never calls one of these
/// missing (unresolved-member-call spec §3.3).
pub const OBJECT_PROTOTYPE_NAMES: &[&str] = &[
    "constructor",
    "hasOwnProperty",
    "isPrototypeOf",
    "propertyIsEnumerable",
    "toLocaleString",
    "toString",
    "valueOf",
    "__defineGetter__",
    "__defineSetter__",
    "__lookupGetter__",
    "__lookupSetter__",
];

/// Canonical wording for a member call on a program-owned receiver that kali
/// cannot lower (unresolved-member-call spec §3.1). Shared by `check` and `run`.
pub fn unresolved_member_call_unavailable_message(method: &str) -> String {
    format!(
        "calling `.{method}()` is unavailable in the current phase: the receiver is a value this program built, and kali has no lowering for a method of that name on it; node would run a method or throw a TypeError, so kali refuses rather than evaluate the call to 0"
    )
}

/// Why a stateful out-of-slice class refuses at `new` (class-instances spec
/// A-1, A-8, A-10). The text completes "it …".
pub const CLASS_REASON_EXTENDS: &str =
    "is in an `extends` chain with another program class and keeps state";
pub const CLASS_REASON_ACCESSOR: &str = "has a getter or setter";
pub const CLASS_REASON_STATIC: &str = "has a static member and keeps state";
pub const CLASS_REASON_PRIVATE: &str = "has a #private member";
pub const CLASS_REASON_COMPUTED: &str = "has a computed member name and keeps state";
pub const CLASS_REASON_EXPRESSION: &str = "is a class expression and keeps state";
pub const CLASS_REASON_EXPORTED: &str = "is exported and keeps state";
pub const CLASS_REASON_AMBIGUOUS: &str = "is declared more than once and keeps state";
pub const CLASS_REASON_UNLOWERED: &str = "reached code generation without being lowered";
/// Ruling R-23: a rewritten class's field holds one 8-byte number slot.
pub const CLASS_REASON_FIELD_VALUE: &str =
    "has a field that may hold a value other than a number or boolean";

/// `new C(…)` of a program class kali does not lower to an object
/// (class-instances spec §3.4, A-1).
pub fn class_construction_unavailable_message(class: &str, reason: &str) -> String {
    format!(
        "constructing class `{class}` is unavailable in the current phase: it {reason}; kali refuses rather than build an instance whose fields read 0"
    )
}

/// `new f(…)` of a program `function` (class-instances spec §3.4).
pub fn plain_function_construction_unavailable_message(name: &str) -> String {
    format!(
        "constructing an object with the plain function `{name}` is unavailable in the current phase; use a class"
    )
}

pub fn class_field_outside_set_message(class: &str, field: &str) -> String {
    format!(
        "field `{field}` of class `{class}` is assigned outside its declared fields and the constructor's leading `this.{field} = …` assignments; declare it, or assign it at the start of the constructor"
    )
}

pub fn class_field_undeclared_read_message(class: &str, field: &str) -> String {
    format!(
        "field `{field}` is not declared on class `{class}`; reading it is unavailable in the current phase"
    )
}

pub fn class_field_without_initial_value_message(class: &str, field: &str) -> String {
    format!(
        "field `{field}` of class `{class}` has no initial value; kali cannot hold `undefined` in an instance field"
    )
}

pub fn class_field_initializer_this_message(class: &str, field: &str) -> String {
    format!(
        "the initializer of field `{field}` of class `{class}` uses `this` beyond the fields already set; this is unavailable in the current phase"
    )
}

pub const fn constructor_return_unavailable_message() -> &'static str {
    "a constructor that returns a value is unavailable in the current phase"
}

pub fn class_instance_mixed_message(class: &str, place: &str) -> String {
    format!(
        "{place} may hold an instance of class `{class}` and other values; this is unavailable in the current phase"
    )
}

/// `typeof o.f` on an instance field (ruling R-24): the field slot is read as a number.
pub const CLASS_POSITION_TYPEOF_FIELD: &str = "the object of a field read under `typeof`";
/// `o as T` / `o satisfies T` / `<T>o` on an instance (ruling R-28).
pub const CLASS_POSITION_TYPE_ASSERTION: &str = "an operand of a type assertion";

pub fn class_instance_position_message(class: &str, position: &str) -> String {
    format!(
        "using an instance of class `{class}` as {position} is unavailable in the current phase"
    )
}

pub fn class_receiver_unresolved_message(method: &str, class: &str) -> String {
    format!(
        "could not determine the class of the receiver of `.{method}()`; method `{method}` belongs to class `{class}`, and kali refuses rather than call it without its instance"
    )
}

pub fn class_method_value_message(class: &str, method: &str) -> String {
    format!(
        "taking method `{method}` of class `{class}` as a value is unavailable in the current phase"
    )
}

pub fn class_value_message(class: &str) -> String {
    format!(
        "using class `{class}` as a value is unavailable in the current phase; only `new {class}(…)` is supported"
    )
}

pub fn class_generated_name_collision_message(name: &str, class: &str) -> String {
    format!(
        "the name `{name}` that kali would generate for class `{class}` is already used by this program; this is unavailable in the current phase"
    )
}

/// Canonical wording for an assignment to a plain runtime array's `.length`.
pub const fn runtime_array_length_write_unavailable_message() -> &'static str {
    "assigning to `.length` of a runtime array is unavailable in the current phase: the array has a fixed length, so kali refuses rather than store into an element"
}

/// What `__array_elem_addr` prints on stderr before it traps (array-bounds spec §3.1).
/// It names kali, not a JavaScript error, because node raises nothing here.
pub const fn runtime_array_index_out_of_bounds_message() -> &'static str {
    "kali: array index out of bounds: node reads undefined here (or grows the array on a write); kali refuses rather than read or write past the allocation"
}
