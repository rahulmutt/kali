use crate::*;

#[test]
fn test_async_class_method_lowering_unavailable_message_is_stable() {
    assert_eq!(
        async_class_method_lowering_unavailable_message(),
        "async class method lowering is unavailable in the direct runtime path; use a plain method or the later compatibility path"
    );
}

#[test]
fn test_generator_class_method_lowering_unavailable_message_lists_async_and_sync_variants() {
    assert_eq!(
        generator_class_method_lowering_unavailable_message(false),
        "generator class method lowering is unavailable in the direct runtime path; use a plain or async method, or the later compatibility path"
    );
    assert_eq!(
        generator_class_method_lowering_unavailable_message(true),
        "async-generator class method lowering is unavailable in the direct runtime path; use a plain or async method, or the later compatibility path"
    );
}

#[test]
fn test_generator_class_method_lowering_unavailable_message_for_flavors_is_stable() {
    const BOTH: &str = generator_class_method_lowering_unavailable_message_for_flavors(true, true);

    assert_eq!(
        BOTH,
        "generator and async-generator class method lowering is unavailable in the direct runtime path; use a plain or async method, or the later compatibility path"
    );
    assert_eq!(
        generator_class_method_lowering_unavailable_message_for_flavors(true, false),
        generator_class_method_lowering_unavailable_message(false)
    );
    assert_eq!(
        generator_class_method_lowering_unavailable_message_for_flavors(false, true),
        generator_class_method_lowering_unavailable_message(true)
    );
    assert_eq!(
        generator_class_method_lowering_unavailable_message_for_flavors(false, false),
        generator_class_method_lowering_unavailable_message(false)
    );
}

#[test]
fn test_generator_class_method_yield_lowering_unavailable_message_for_flavors_is_stable() {
    const BOTH: &str =
        generator_class_method_yield_lowering_unavailable_message_for_flavors(true, true, true);

    assert_eq!(
        BOTH,
        "generator and async-generator class method lowering is unavailable in the direct runtime path for yield* delegation; use a plain or async method, or the later compatibility path"
    );
    assert_eq!(
        generator_class_method_yield_lowering_unavailable_message_for_flavors(true, false, true),
        generator_class_method_yield_lowering_unavailable_message(false, true)
    );
    assert_eq!(
        generator_class_method_yield_lowering_unavailable_message_for_flavors(false, true, true),
        generator_class_method_yield_lowering_unavailable_message(true, true)
    );
    assert_eq!(
        generator_class_method_yield_lowering_unavailable_message_for_flavors(true, false, false),
        generator_class_method_lowering_unavailable_message(false)
    );
    assert_eq!(
        generator_class_method_yield_lowering_unavailable_message_for_flavors(false, true, false),
        generator_class_method_lowering_unavailable_message(true)
    );
    assert_eq!(
        generator_class_method_yield_lowering_unavailable_message_for_flavors(false, false, true),
        generator_class_method_lowering_unavailable_message(false)
    );
    assert_eq!(
        generator_class_method_yield_lowering_unavailable_message_for_flavors(false, false, false),
        generator_class_method_lowering_unavailable_message(false)
    );
}

#[test]
fn test_generator_function_lowering_unavailable_message_lists_async_and_sync_variants() {
    assert_eq!(
        generator_function_lowering_unavailable_message(false),
        "generator function lowering is unavailable in the current phase; use a synchronous function or the later compatibility path"
    );
    assert_eq!(
        generator_function_lowering_unavailable_message(true),
        "async-generator function lowering is unavailable in the current phase; use a synchronous function or the later compatibility path"
    );
}

#[test]
fn test_generator_function_lowering_unavailable_message_for_yield_delegation_is_stable() {
    assert_eq!(
        generator_function_yield_lowering_unavailable_message(false, true),
        "generator function lowering is unavailable in the current phase for yield* delegation; use a synchronous function or the later compatibility path"
    );
    assert_eq!(
        generator_function_yield_lowering_unavailable_message(true, true),
        "async-generator function lowering is unavailable in the current phase for yield* delegation; use a synchronous function or the later compatibility path"
    );
    assert_eq!(
        generator_function_yield_lowering_unavailable_message(false, false),
        generator_function_lowering_unavailable_message(false)
    );
    assert_eq!(
        generator_function_yield_lowering_unavailable_message(true, false),
        generator_function_lowering_unavailable_message(true)
    );
}

#[test]
fn test_generator_function_lowering_unavailable_message_for_flavors_is_stable() {
    const BOTH: &str = generator_function_lowering_unavailable_message_for_flavors(true, true);

    assert_eq!(
        BOTH,
        "generator and async-generator function lowering is unavailable in the current phase; use a synchronous function or the later compatibility path"
    );
    assert_eq!(
        generator_function_lowering_unavailable_message_for_flavors(true, false),
        generator_function_lowering_unavailable_message(false)
    );
    assert_eq!(
        generator_function_lowering_unavailable_message_for_flavors(false, true),
        generator_function_lowering_unavailable_message(true)
    );
    assert_eq!(
        generator_function_lowering_unavailable_message_for_flavors(false, false),
        generator_function_lowering_unavailable_message(false)
    );
}

#[test]
fn runtime_array_refusal_messages_are_stable() {
    assert_eq!(
        runtime_array_negative_index_unavailable_message(),
        "a negative index on a runtime array is unavailable in the current phase: node reads `undefined` there and kali has no `undefined` value, so kali refuses rather than read the array's length header"
    );
    assert_eq!(
        runtime_array_mutator_unavailable_message("push"),
        "calling `.push()` on a runtime array is unavailable in the current phase: the array has a fixed length, so kali refuses rather than silently skip the call"
    );
    assert_eq!(
        runtime_array_length_write_unavailable_message(),
        "assigning to `.length` of a runtime array is unavailable in the current phase: the array has a fixed length, so kali refuses rather than store into an element"
    );
    assert_eq!(
        runtime_array_index_out_of_bounds_message(),
        "kali: array index out of bounds: node reads undefined here (or grows the array on a write); kali refuses rather than read or write past the allocation"
    );
}

#[test]
fn runtime_array_mutators_are_the_five_fixed_length_breakers() {
    assert_eq!(RUNTIME_ARRAY_MUTATORS, &["push", "pop", "shift", "unshift", "splice"]);
}
