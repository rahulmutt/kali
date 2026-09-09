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
