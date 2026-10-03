use crate::emit::computed_member::computed_member_tests::{assert_e5506, diagnostics_for};
use crate::test_support::parse_and_lower_lir;
use crate::{lower_lir_to_wasm, CodegenCtx, TargetConfig};

const LIT: &str = "on a literal array is unavailable in the current phase";
const LIT_LEN: &str = "assigning to `.length` of a literal array is unavailable";
const MUT: &str = "on a runtime array is unavailable in the current phase";

#[test]
fn codegen_refuses_each_mutator_on_a_literal_binding_and_a_nameless_literal() {
    for call in [
        "a.push(4)",
        "a.pop()",
        "a.shift()",
        "a.unshift(0)",
        "a.splice(0, 1)",
        "a.reverse()",
        "a.sort()",
        "a.fill(9)",
        "a.copyWithin(0, 2)",
        "(a).pop()",
        "(a.pop)()",
    ] {
        for source in [
            format!("const a = [1,2,3]; {call}; console.log(a[0]);"),
            format!("function main(){{ const a = [1,2,3]; {call}; console.log(a[0]); }} main();"),
        ] {
            assert_e5506(&diagnostics_for(&source), LIT, &source);
        }
    }
    assert_e5506(
        &diagnostics_for("console.log([1,2].push(3));"),
        LIT,
        "nameless",
    );
}

#[test]
fn codegen_refuses_a_length_write_on_a_literal_binding() {
    let source = "const a = [1,2,3]; a.length = 1; console.log(a[0]);";
    assert_e5506(&diagnostics_for(source), LIT_LEN, source);
}

#[test]
fn codegen_refuses_the_reorderers_on_a_plain_runtime_array() {
    // `a.push?.(1)` is not asserted here: HIR lowers an optional chain
    // transparently, and the parser drops an optional call's arguments, so at
    // LIR it is byte-identical to the plain read `a.push`. The type layer
    // (`resolve_optional_chain`) is the only gate that can see it.
    for call in [
        "a.reverse()",
        "a.sort()",
        "a.copyWithin(0, 2)",
        "(a.sort)()",
    ] {
        let source =
            format!("function main(){{ const a = new Array(3).fill(4); {call}; }} main();");
        assert_e5506(&diagnostics_for(&source), MUT, &source);
    }
}

#[test]
fn codegen_leaves_growable_push_and_plain_fill_alone() {
    // A growable binding is the type layer's promotion (repr table); codegen
    // alone sees only the literal, so seed the promotion the driver would.
    let growable =
        "function main(){ const a = [1,2,3]; a.push(4); console.log(a.length); } main();";
    let program = parse_and_lower_lir(growable);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.repr_table.set_growable_array_binding("main", "a");
    let growable_diagnostics = lower_lir_to_wasm(&mut ctx, &program).diagnostics;
    let plain =
        "function main(){ const a = new Array(3).fill(4); a.fill(5); console.log(a[1]); } main();";
    for (source, diagnostics) in [
        (growable, growable_diagnostics),
        (plain, diagnostics_for(plain)),
    ] {
        let errors: Vec<_> = diagnostics.into_iter().filter(|d| d.is_error()).collect();
        assert!(errors.is_empty(), "{source}: {errors:?}");
    }
}
