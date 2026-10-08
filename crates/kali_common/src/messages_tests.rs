use crate::*;

#[test]
fn captured_binding_message_names_capturer_binding_and_reason() {
    assert_eq!(
        captured_binding_unavailable_message("g", "s", CaptureRefusal::ValueType),
        "a closure `g` that captures `s` is unavailable in the current phase: its value type has no closure cell"
    );
    assert_eq!(
        captured_binding_unavailable_message("h", "a", CaptureRefusal::Depth),
        "a closure `h` that captures `a` is unavailable in the current phase: `a` is two or more closures away"
    );
    assert_eq!(
        captured_binding_unavailable_message("__kali_fn_0", "k", CaptureRefusal::Parameter { owner: "f" }),
        "a closure `__kali_fn_0` that captures `k` is unavailable in the current phase: `k` is a parameter of `f`"
    );
}

#[test]
fn captured_binding_message_shows_written_names() {
    assert_eq!(
        captured_binding_unavailable_message("g{b2}", "k{p}", CaptureRefusal::Parameter { owner: "f{b1}" }),
        "a closure `g` that captures `k` is unavailable in the current phase: `k` is a parameter of `f`"
    );
}

#[test]
fn captured_parameter_eval_message_names_the_rewrite_not_shadowing() {
    let message = captured_parameter_eval_refused_message();
    assert_eq!(
        message,
        "a closure that captures a parameter is unavailable with `--compat eval` in the current phase: kali gives the captured parameter a renamed local so a closure can share it, and `eval` code could name the parameter"
    );
    assert!(!message.contains("shadow"));
    assert_ne!(message, block_scope_eval_refused_message());
}

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
fn unresolved_member_read_message_names_the_property() {
    assert_eq!(
        unresolved_member_read_unavailable_message("a"),
        "reading `.a` is unavailable in the current phase: the receiver is a value this program built, and kali has no lowering for that read; node would read a property or `undefined`, so kali refuses rather than read 0"
    );
}

#[test]
fn unresolved_member_read_message_is_neutral_for_spread_and_sequence_text() {
    // LIR spells a spread `...a` as text "spread" and a comma expression as
    // text "" (spec A-1); neither is a property read.
    let neutral = "this expression is unavailable in the current phase: the receiver is a value this program built, and kali has no lowering for that read; kali refuses rather than evaluate it to 0";
    assert_eq!(
        unresolved_member_read_unavailable_message("spread"),
        neutral
    );
    assert_eq!(unresolved_member_read_unavailable_message(""), neutral);
}

#[test]
fn unresolved_store_message_names_the_target() {
    assert_eq!(
        unresolved_store_unavailable_message("x"),
        "assigning to `x` is unavailable in the current phase: the receiver is a value this program built, and kali has no lowering for that store; node would store the value or throw a TypeError, so kali refuses rather than drop the store"
    );
    assert!(unresolved_store_unavailable_message(".a").starts_with("assigning to `.a` is"));
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
        class_construction_unavailable_message("P", CLASS_REASON_FIELD_VALUE),
        "constructing class `P` is unavailable in the current phase: it has a field that may hold a value other than a number or boolean; kali refuses rather than build an instance whose fields read 0"
    );
    assert_eq!(
        class_construction_unavailable_message("C", CLASS_REASON_ENCLOSING_LOCAL),
        "constructing class `C` is unavailable in the current phase: it reads a variable of an enclosing function; kali refuses rather than build an instance whose fields read 0"
    );
    assert_eq!(
        class_construction_unavailable_message("C", CLASS_REASON_INITIALIZER_SCOPE),
        "constructing class `C` is unavailable in the current phase: it has a field initializer that names a parameter or variable of its constructor; kali refuses rather than build an instance whose fields read 0"
    );
    assert_eq!(
        class_construction_unavailable_message("C", CLASS_REASON_FIELD_METHOD),
        "constructing class `C` is unavailable in the current phase: it has a field and a method with the same name; kali refuses rather than build an instance whose fields read 0"
    );
    assert_eq!(
        class_construction_unavailable_message("S", CLASS_REASON_SAME_EXPRESSION),
        "constructing class `S` is unavailable in the current phase: it is outside the class-instances slice and its new result is used in the same expression; kali refuses rather than build an instance whose fields read 0"
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
        class_instance_position_message("C", CLASS_POSITION_TYPEOF_FIELD),
        "using an instance of class `C` as the object of a field read under `typeof` is unavailable in the current phase"
    );
    assert_eq!(
        class_instance_position_message("C", CLASS_POSITION_TYPE_ASSERTION),
        "using an instance of class `C` as an operand of a type assertion is unavailable in the current phase"
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

#[test]
fn default_parameter_messages_name_the_function_and_parameter() {
    assert_eq!(
        default_param_non_declaration_message(),
        "default parameters are only available on function declarations in the current phase"
    );
    assert_eq!(
        default_param_exported_message("f"),
        "a function with default parameters cannot be exported in the current phase: `f`"
    );
    assert_eq!(
        default_param_not_literal_message("f", "b"),
        "a default parameter value must be a number, string, boolean, null or BigInt literal in the current phase: `b` in `f`"
    );
    assert_eq!(
        default_param_composite_message("f", "o"),
        "an object or array default parameter is unavailable in the current phase: kali cannot pass an object or array literal directly as a call argument; `o` in `f`"
    );
    assert_eq!(
        default_param_async_or_generator_message("f"),
        "default parameters are only available on function declarations in the current phase: `f` is a generator or `async` function"
    );
    assert_eq!(
        default_param_value_use_message("f"),
        "a function with default parameters can only be called directly by name in the current phase; `f` is used as a value here"
    );
    assert_eq!(
        default_param_rebound_name_message("f"),
        "a function with default parameters must have a name no other binding in the program uses in the current phase: `f` is also bound elsewhere"
    );
    assert_eq!(
        default_param_spread_call_message("f"),
        "a function with default parameters cannot be called with a spread argument in the current phase: `f(...)`"
    );
    assert_eq!(
        default_param_omitted_argument_message("f", "b"),
        "`f(…)` omits an argument for `b`, which has no default"
    );
    assert_eq!(
        default_param_eval_refused_message(),
        "default parameters are unavailable under --compat eval"
    );
    assert_eq!(
        default_param_out_of_scope_use_message("f"),
        "a function with default parameters declared inside a function or block can only be called inside that scope in the current phase: `f` is used outside it"
    );
}

#[test]
fn growable_runtime_array_messages_name_the_array_and_say_why() {
    assert_eq!(growable_scope_phrase("_start"), "at module scope");
    assert_eq!(growable_scope_phrase("main"), "in `main`");
    assert_eq!(growable_binding_subject("xs", "main"), "`xs` in `main`");
    assert_eq!(
        growable_binding_subject("xs", "_start"),
        "`xs` at module scope"
    );
    assert_eq!(
        growable_return_subject("build"),
        "the array `build` returns"
    );
    assert_eq!(
        growable_call_result_source("build"),
        "the array `build(…)` returns"
    );
    assert_eq!(growable_slice_result_source(), "a `slice()` result");
    assert_eq!(
        growable_mixed_layout_message("`a` in `total`"),
        "`a` in `total` would hold both a growable array (built with `push`, `pop` or an index write) and a fixed-length `new Array(n)` array; mixing the two array layouts is unavailable in the current phase"
    );
    assert_eq!(
        growable_unsupported_element_message("`o` in `main`"),
        "`o` in `main` is a growable array with an element that is an object, an array, a function, a boolean, `null` or `undefined`; a growable array holds only numbers or only strings in the current phase"
    );
    assert_eq!(
        growable_module_read_message("out", "size"),
        "function `size` uses the module-level growable array `out`; a function can reach a module-level growable array only through a parameter in the current phase"
    );
    assert_eq!(
        growable_capture_message("`o` in `main`"),
        "the growable array `o` in `main` is captured by a closure, nested function or class body; capturing a growable array is unavailable in the current phase"
    );
    assert_eq!(
        growable_for_of_mutation_message("`a` in `main`"),
        "`push` or `pop` on the growable array `a` in `main`, directly, through an alias or through a function it is passed to, inside a `for-of` loop over that same array is unavailable in the current phase: kali fixes the iteration count when the loop starts"
    );
    assert_eq!(
        growable_loop_capture_message("x"),
        "the closure or nested function at module scope captures `x`, which a `for-of` loop over a growable array declares; capturing a loop binding at module scope is unavailable in the current phase"
    );
    assert_eq!(
        growable_from_index_message("indexOf"),
        "`indexOf` with a `fromIndex` argument on a growable array is unavailable in the current phase"
    );
    assert_eq!(
        growable_length_write_message("`a` in `main`"),
        "assigning to `.length` of the growable array `a` in `main` is unavailable in the current phase"
    );
    assert_eq!(
        growable_unsupported_operation_message("`.reverse()`", "`a` in `main`"),
        "`.reverse()` on the growable array `a` in `main` is unavailable in the current phase; a growable array supports `push`, `pop`, `indexOf`, `includes`, `slice`, `join`, `.length`, index reads and writes, and `for-of`"
    );
    assert_eq!(
        runtime_array_print_unavailable_message(),
        "printing a whole runtime array is unavailable in the current phase: kali would print its handle; print its elements instead"
    );
    assert_eq!(
        growable_plain_use_message("`a` in `main`"),
        "the growable array `a` in `main` is used as a plain value here; a growable array can only be bound, passed to a function, returned, iterated with `for-of`, or used through `.length`, an index or a supported method in the current phase"
    );
    assert_eq!(
        growable_literal_expression_message("f"),
        "an array literal written directly as a call argument or `return` value in `f` would be a growable array; bind it to a `const` first in the current phase"
    );
    assert_eq!(
        growable_literal_assignment_message("main"),
        "assigning an array literal to a binding that holds a growable array in `main` is unavailable in the current phase; declare a new `const` for the new array instead"
    );
    assert_eq!(
        growable_unproven_operand_message("index", "`a` in `main`", false),
        "the index of the growable array `a` in `main` is not proven to be a number; a growable array's index and `slice` bounds must be numbers, and its search value a number or a string, in the current phase"
    );
    assert_eq!(
        growable_unproven_operand_message("search value", "`a` at module scope", true),
        "the search value of the growable array `a` at module scope is not proven to be a number or a string; a growable array's index and `slice` bounds must be numbers, and its search value a number or a string, in the current phase"
    );
    assert_eq!(
        growable_temporary_use_message("the array `make(…)` returns"),
        "indexing, `push` or `pop` directly on the array `make(…)` returns is unavailable in the current phase; bind it to a `const` first"
    );
    assert_eq!(
        growable_non_array_write_message("`a` in `f`"),
        "`a` in `f` holds a growable array and is also given a value that is not an array (another value, `undefined`, or a missing argument or `return`); this is unavailable in the current phase"
    );
    assert_eq!(
        growable_pop_empty_message(),
        "kali: pop on empty array: node returns undefined here; kali refuses rather than return a value that is not there"
    );
}
