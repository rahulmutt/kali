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
        "calling `.push()` on a runtime array is unavailable in the current phase: kali has no lowering of it on this array, so kali refuses rather than silently skip the call"
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
fn runtime_array_mutators_are_the_plain_lane_methods_without_a_lowering() {
    assert_eq!(
        RUNTIME_ARRAY_MUTATORS,
        &[
            "push",
            "pop",
            "shift",
            "unshift",
            "splice",
            "reverse",
            "sort",
            "copyWithin"
        ]
    );
}

#[test]
fn literal_array_mutators_are_the_runtime_list_plus_fill() {
    let mut expected: Vec<&str> = RUNTIME_ARRAY_MUTATORS.to_vec();
    expected.push("fill");
    assert_eq!(LITERAL_ARRAY_MUTATORS, expected.as_slice());
}

#[test]
fn literal_array_refusal_messages_are_stable() {
    assert_eq!(
        literal_array_mutator_unavailable_message("pop"),
        "calling `.pop()` on a literal array is unavailable in the current phase: kali folds a literal array to its initial elements, so kali refuses rather than silently skip the call"
    );
    assert_eq!(
        literal_array_length_write_unavailable_message(),
        "assigning to `.length` of a literal array is unavailable in the current phase: kali folds a literal array to its initial elements, so kali refuses rather than silently skip the write"
    );
    assert_eq!(
        literal_array_store_unavailable_message(),
        "mutating a literal array is unavailable in the current direct-runtime path; use new Array(n) for runtime mutation"
    );
    assert_eq!(
        array_mutator_unresolved_receiver_message("sort"),
        "calling `.sort()` is unavailable in the current phase: it mutates an array in place and kali could not prove which array the receiver is, so kali refuses rather than silently skip the call"
    );
}

#[test]
fn unresolved_member_call_message_is_stable() {
    assert_eq!(
        unresolved_member_call_unavailable_message("zork"),
        "calling `.zork()` is unavailable in the current phase: the receiver is a value this program built, and kali has no lowering for a method of that name on it; node would run a method or throw a TypeError, so kali refuses rather than evaluate the call to 0"
    );
}

#[test]
fn object_prototype_names_are_the_eleven_inherited_methods() {
    assert_eq!(
        OBJECT_PROTOTYPE_NAMES,
        &[
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
        ]
    );
}

#[test]
fn class_instance_messages_are_stable() {
    assert_eq!(
        class_construction_unavailable_message("B", CLASS_REASON_EXTENDS),
        "constructing class `B` is unavailable in the current phase: it is in an `extends` chain with another program class and keeps state; kali refuses rather than build an instance whose fields read 0"
    );
    assert_eq!(
        plain_function_construction_unavailable_message("Box"),
        "constructing an object with the plain function `Box` is unavailable in the current phase; use a class"
    );
    assert_eq!(
        class_field_outside_set_message("C", "m"),
        "field `m` of class `C` is assigned outside its declared fields and the constructor's leading `this.m = …` assignments; declare it, or assign it at the start of the constructor"
    );
    assert_eq!(
        class_field_undeclared_read_message("C", "zz"),
        "field `zz` is not declared on class `C`; reading it is unavailable in the current phase"
    );
    assert_eq!(
        class_field_without_initial_value_message("C", "n"),
        "field `n` of class `C` has no initial value; kali cannot hold `undefined` in an instance field"
    );
    assert_eq!(
        class_field_initializer_this_message("C", "n"),
        "the initializer of field `n` of class `C` uses `this` beyond the fields already set; this is unavailable in the current phase"
    );
    assert_eq!(
        constructor_return_unavailable_message(),
        "a constructor that returns a value is unavailable in the current phase"
    );
    assert_eq!(
        class_instance_mixed_message("A", "parameter `x` of `f`"),
        "parameter `x` of `f` may hold an instance of class `A` and other values; this is unavailable in the current phase"
    );
    assert_eq!(
        class_instance_position_message("C", "an argument to a host call"),
        "using an instance of class `C` as an argument to a host call is unavailable in the current phase"
    );
    assert_eq!(
        class_receiver_unresolved_message("get", "C"),
        "could not determine the class of the receiver of `.get()`; method `get` belongs to class `C`, and kali refuses rather than call it without its instance"
    );
    assert_eq!(
        class_method_value_message("C", "get"),
        "taking method `get` of class `C` as a value is unavailable in the current phase"
    );
    assert_eq!(
        class_value_message("C"),
        "using class `C` as a value is unavailable in the current phase; only `new C(…)` is supported"
    );
    assert_eq!(
        class_generated_name_collision_message("__this", "C"),
        "the name `__this` that kali would generate for class `C` is already used by this program; this is unavailable in the current phase"
    );
}
