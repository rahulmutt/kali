//! Growable-runtime-arrays spec §3.5: the resolver admits every growable lane
//! codegen lowers (these programs produce no resolver E5506).

use super::*;

fn e5506_of(statements: &[kali_ast::Statement]) -> Vec<String> {
    let mut ctx = TypeContext::new();
    ctx.resolve_statements(statements)
        .diagnostics
        .into_iter()
        .filter(|d| d.code == Some(e5::FEATURE_UNAVAILABLE as u32))
        .map(|d| d.message)
        .collect()
}

fn e5506(source: &str) -> Vec<String> {
    e5506_of(&crate::test_support::parse_statements(source))
}

#[test]
fn for_of_over_string_and_float_growable_arrays_is_admitted() {
    assert_eq!(
        e5506("function main() { const ws = []; ws.push(\"a\"); const fs = []; fs.push(0.5); let n = \"\"; let s = 0; for (const w of ws) n = n + w; for (const f of fs) s = s + f; console.log(n, s); }\nmain();"),
        Vec::<String>::new()
    );
}

#[test]
fn for_of_over_a_growable_call_result_and_a_slice_is_admitted() {
    assert_eq!(
        e5506("function build() { const out = []; out.push(1); return out; }\nfor (const x of build()) console.log(x);\nconst a = build();\nfor (const y of a.slice(1)) console.log(y);"),
        Vec::<String>::new()
    );
}

#[test]
fn nested_for_of_over_growable_arrays_is_admitted() {
    assert_eq!(
        e5506("function main() { const a = []; a.push(1); const b = []; b.push(2); let s = 0; for (const x of a) { for (const y of b) { s += x * y; } } console.log(s); }\nmain();"),
        Vec::<String>::new()
    );
}

#[test]
fn join_slice_and_search_on_growable_values_are_admitted() {
    assert_eq!(
        e5506("function main() { const a = []; for (let i = 0; i < 5; i++) a.push(i); let k = 1; console.log(a.join(\",\"), a.slice(k, -1).join(\"-\"), a.slice(1).slice(1).join(), a.indexOf(3), a.includes(k)); }\nmain();"),
        Vec::<String>::new()
    );
}

#[test]
fn a_growable_literal_is_not_a_literal_array_for_the_mutator_and_store_gates() {
    assert_eq!(
        e5506("function main() { const a = [1, 2]; a.push(3); a[0] = 5; let i = 1; a[i] = 6; console.log(a.pop(), a.length); }\nmain();"),
        Vec::<String>::new()
    );
}

#[test]
fn a_growable_parameter_index_and_length_are_admitted() {
    assert_eq!(
        e5506("function total(a) { let s = 0; for (let i = 0; i < a.length; i++) s += a[i]; return s; }\nfunction main() { const xs = []; xs.push(1); console.log(total(xs)); }\nmain();"),
        Vec::<String>::new()
    );
}

#[test]
fn a_literal_nobody_pushes_pops_or_writes_keeps_its_mutator_refusal() {
    // Unchanged lane (spec A-1): `reverse` is no growable demand, so `a` stays
    // a folded literal and the literal-array-mutators refusal still fires.
    // (`a[0] = 5` on the same literal WOULD make it growable: an index write
    // is a source, spec §3.1.)
    assert!(
        e5506("const a = [1, 2]; a.reverse(); console.log(a[0]);")
            .iter()
            .any(|m| m.contains("on a literal array")),
        "the literal-array mutator refusal must stay"
    );
}

/// Name every top-level `const NAME = () => …` arrow / function expression `__kali_fn_N`, as the
/// CLI's `name_anonymous_functions` pre-pass does (kali_types cannot call it).
fn name_top_level_arrows(statements: &mut [kali_ast::Statement]) {
    let mut n = 0;
    for statement in statements {
        if let kali_ast::Statement::VariableDeclaration(decl) = statement {
            for declarator in &mut decl.declarations {
                // The parser lowers a block-bodied arrow to a FunctionExpression.
                let id = match &mut declarator.init {
                    Some(Expression::ArrowFunctionExpression(arrow)) => &mut arrow.id,
                    Some(Expression::FunctionExpression(function)) => &mut function.id,
                    _ => continue,
                };
                *id = Some(format!("__kali_fn_{n}"));
                n += 1;
            }
        }
    }
}

#[test]
fn a_call_through_a_const_arrow_alias_reaches_the_growable_return() {
    // Controller ruling W2: inference keys `mk()` to the arrow's
    // `__kali_fn_N` (`array_return_callee`), so the resolver must too — the
    // `.length`, `for-of`, `join` and `slice` lanes all admit it.
    let mut statements = crate::test_support::parse_statements(
        "const mk = () => { const o = []; o.push(1); return o; };\nconsole.log(mk().length);\nfor (const x of mk()) console.log(x);\nconsole.log(mk().join(\",\"), mk().slice(1).join());",
    );
    name_top_level_arrows(&mut statements);
    assert_eq!(e5506_of(&statements), Vec::<String>::new());
}
