use super::*;

#[test]
fn runtime_entrypoint_rejects_async_class_expressions_in_js_input() {
    assert_runtime_entrypoint_rejects_async_class_expression_in_input(
        "js",
        "const Example = class NamedExample { async main() { return 1; } };\nnew Example().main();\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_async_default_export_class_expressions_in_ts_input() {
    assert_runtime_entrypoint_rejects_async_class_expression_in_input(
        "ts",
        "export default (class NamedExample { async main() { return 1; } });\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_async_class_expressions_in_jsx_input() {
    assert_runtime_entrypoint_rejects_async_class_expression_in_input(
        "jsx",
        "const Example = class NamedExample { async main() { return 1; } };\nnew Example().main();\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_async_class_expressions_in_tsx_input() {
    assert_runtime_entrypoint_rejects_async_class_expression_in_input(
        "tsx",
        "const Example = class NamedExample { async main() { return 1; } };\nnew Example().main();\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_async_generator_default_export_class_expressions_in_jsx_input() {
    assert_runtime_entrypoint_rejects_generator_class_expression_in_input(
        "jsx",
        "export default (class NamedExample { async *main() { yield 1; } });\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_async_generator_default_export_class_expressions_in_tsx_input() {
    assert_runtime_entrypoint_rejects_generator_class_expression_in_input(
        "tsx",
        "export default (class NamedExample { async *main() { yield 1; } });\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_mixed_generator_class_expressions_in_js_input() {
    assert_runtime_entrypoint_rejects_mixed_generator_class_expression_in_input(
        "js",
        "const Example = class NamedExample { *main() { yield 1; } async *other() { yield* main(); } };\nnew Example();\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_mixed_generator_class_expressions_in_ts_jsx_tsx_input() {
    for extension in ["ts", "jsx", "tsx"] {
        assert_runtime_entrypoint_rejects_mixed_generator_class_expression_in_input(
            extension,
            "const Example = class NamedExample { *main() { yield 1; } async *other() { yield* main(); } };\nnew Example();\n",
        );
    }
}

#[test]
fn runtime_entrypoint_rejects_sequence_wrapped_generator_class_expressions_in_js_input() {
    assert_runtime_entrypoint_rejects_generator_class_expression_in_input(
        "js",
        "const Example = ((0, class NamedExample { *main() { yield 1; } }));\nnew Example();\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_sequence_wrapped_async_generator_class_expressions_in_js_input() {
    assert_runtime_entrypoint_rejects_generator_class_expression_in_input(
        "js",
        "const Example = ((0, class NamedExample { async *main() { yield 1; } }));\nnew Example();\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_generator_class_expressions_wrapped_in_type_assertions_in_ts_input() {
    assert_runtime_entrypoint_rejects_generator_class_expression_in_input(
        "ts",
        "const Example = ((class NamedExample { *main() { yield 1; } })) as unknown;\nExample;\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_generator_class_expressions_wrapped_in_satisfies_in_ts_input() {
    assert_runtime_entrypoint_rejects_generator_class_expression_in_input(
        "ts",
        "const Example = ((class NamedExample { *main() { yield 1; } })) satisfies unknown;\nExample;\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_async_generator_default_export_class_expressions_wrapped_in_as_in_ts_input(
) {
    assert_runtime_entrypoint_rejects_generator_class_expression_in_input(
        "ts",
        "export default ((class NamedExample { async *main() { yield 1; } })) as unknown;\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_async_generator_default_export_class_expressions_wrapped_in_satisfies_in_ts_input(
) {
    assert_runtime_entrypoint_rejects_generator_class_expression_in_input(
        "ts",
        "export default ((class NamedExample { async *main() { yield 1; } })) satisfies unknown;\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_generator_class_expressions_in_js_input() {
    assert_runtime_entrypoint_rejects_generator_class_expression_in_input(
        "js",
        "const Example = class NamedExample { *main() { yield 1; } };\nnew Example();\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_generator_class_expressions_in_ts_input() {
    assert_runtime_entrypoint_rejects_generator_class_expression_in_input(
        "ts",
        "const Example = class NamedExample { *main() { yield 1; } };\nnew Example();\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_generator_class_expressions_in_jsx_input() {
    assert_runtime_entrypoint_rejects_generator_class_expression_in_input(
        "jsx",
        "const Example = class NamedExample { *main() { yield 1; } };\nnew Example();\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_generator_class_expressions_in_tsx_input() {
    assert_runtime_entrypoint_rejects_generator_class_expression_in_input(
        "tsx",
        "const Example = class NamedExample { *main() { yield 1; } };\nnew Example();\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_async_generator_default_export_class_expressions_in_js_input() {
    assert_runtime_entrypoint_rejects_generator_class_expression_in_input(
        "js",
        "export default (class NamedExample { async *main() { yield 1; } });\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_async_generator_default_export_class_expressions_in_ts_input() {
    assert_runtime_entrypoint_rejects_generator_class_expression_in_input(
        "ts",
        "export default (class NamedExample { async *main() { yield 1; } });\n",
    );
}

#[test]
fn runtime_entrypoint_rejects_anonymous_default_export_generator_function_declarations_in_supported_input_matrix(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_runtime_entrypoint_rejects_generator_function_declaration_in_input(
            extension,
            "export default function*() { yield* []; }\n",
        );
    }
}

#[test]
fn runtime_entrypoint_rejects_anonymous_default_export_async_generator_function_declarations_with_yield_delegation_in_supported_input_matrix(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_runtime_entrypoint_rejects_generator_function_declaration_in_input(
            extension,
            "export default async function*() { yield* []; }\n",
        );
    }
}

#[test]
fn runtime_entrypoint_rejects_generator_function_expressions_in_supported_input_matrix() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_runtime_entrypoint_rejects_generator_function_expression_in_input(
            extension,
            "const generatorExpr = function* generatorExpr() { yield 1; };\ngeneratorExpr;\n",
        );
    }
}

#[test]
fn runtime_entrypoint_rejects_async_generator_function_expressions_in_supported_input_matrix() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_runtime_entrypoint_rejects_generator_function_expression_in_input(
            extension,
            "const asyncGeneratorExpr = async function* asyncGeneratorExpr() { yield 1; };\nasyncGeneratorExpr;\n",
        );
    }
}

#[test]
fn runtime_entrypoint_rejects_generator_function_declarations_in_supported_input_matrix() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_runtime_entrypoint_rejects_generator_function_declaration_in_input(
            extension,
            "function* main() { yield 1; }\nmain();\n",
        );
    }
}

#[test]
fn runtime_entrypoint_rejects_async_generator_function_declarations_in_supported_input_matrix() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_runtime_entrypoint_rejects_generator_function_declaration_in_input(
            extension,
            "async function* main() { yield 1; }\nmain();\n",
        );
    }
}
