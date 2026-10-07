use super::growable_array_candidates;
use kali_ast::Statement;

/// Parse `src` and return the body of the FIRST function declaration.
fn func_body(src: &str) -> (Vec<String>, Vec<Statement>) {
    let parsed = crate::test_support::parse_statements(src);
    for stmt in parsed {
        if let Statement::FunctionDeclaration(decl) = stmt {
            return (decl.params.clone(), decl.body.body.clone());
        }
    }
    panic!("source has no function declaration");
}

fn candidates(src: &str) -> Vec<String> {
    let (params, body) = func_body(src);
    let (set, _, _) = growable_array_candidates(&params, &body);
    set.into_iter().collect()
}

/// The Task 6 fail-closed reject set: growable-shape `.push` receivers that
/// could not promote (an occurrence outside the safe-position allowlist, or a
/// malformed `.push` call).
fn rejects(src: &str) -> Vec<String> {
    let (params, body) = func_body(src);
    let (_, _, rejects) = growable_array_candidates(&params, &body);
    rejects.into_keys().collect()
}

/// Reject kinds, for asserting the diagnostic routing (position vs push).
fn reject_kinds(src: &str) -> Vec<(String, super::GrowableRejectKind)> {
    let (params, body) = func_body(src);
    let (_, _, rejects) = growable_array_candidates(&params, &body);
    rejects.into_iter().collect()
}

#[test]
fn reject_kind_routes_position_vs_malformed_push() {
    use super::GrowableRejectKind::{UnsafePosition, UnsupportedPush};
    // Escape → position kind.
    assert_eq!(
        reject_kinds("function m() { const o = []; o.push(1); return o; }"),
        vec![("o".to_string(), UnsafePosition)]
    );
    // Object-literal argument → push kind (no position applies).
    assert_eq!(
        reject_kinds("function m() { const o = []; o.push({a: 1}); }"),
        vec![("o".to_string(), UnsupportedPush)]
    );
    // Wrong arity → push kind.
    assert_eq!(
        reject_kinds("function m() { const o = []; o.push(1, 2); }"),
        vec![("o".to_string(), UnsupportedPush)]
    );
    // Malformed push AND an unsafe position → position kind wins.
    assert_eq!(
        reject_kinds("function m() { const o = []; o.push(1, 2); return o; }"),
        vec![("o".to_string(), UnsafePosition)]
    );
}

#[test]
fn escaping_via_return_is_a_reject_not_a_candidate() {
    let src = "function make() { const o = []; o.push(1); return o; }";
    assert!(candidates(src).is_empty());
    assert_eq!(rejects(src), vec!["o".to_string()]);
}

#[test]
fn alias_binding_is_a_reject() {
    let src = "function m() { const o = []; o.push(1); const p = o; }";
    assert_eq!(rejects(src), vec!["o".to_string()]);
}

#[test]
fn computed_push_call_is_a_reject_without_any_clean_push() {
    // No clean `.push` occurrence exists, so `o` is never a candidate — the
    // push-receiver-mention scan still catches `o["push"](..)`.
    let src = "function m() { const o = []; o[\"push\"](1); o[\"push\"](2); }";
    assert!(candidates(src).is_empty());
    assert_eq!(rejects(src), vec!["o".to_string()]);
}

#[test]
fn optional_chain_push_call_is_a_reject() {
    let src = "function m() { const o = []; o?.push(1); o?.push(2); }";
    assert_eq!(rejects(src), vec!["o".to_string()]);
}

#[test]
fn closure_capture_push_is_a_reject() {
    let src = "function m() { const o = []; o.push(1); const f = () => o.push(2); }";
    assert_eq!(rejects(src), vec!["o".to_string()]);
}

#[test]
fn pop_mutator_is_a_reject() {
    let src = "function m() { const o = []; o.push(1); o.pop(); }";
    assert_eq!(rejects(src), vec!["o".to_string()]);
}

#[test]
fn wrong_arity_push_is_a_reject() {
    let src = "function m() { const o = []; o.push(1, 2); }";
    assert_eq!(rejects(src), vec!["o".to_string()]);
}

#[test]
fn a_promoted_candidate_is_never_a_reject() {
    // A binding used only in safe positions promotes and must NOT be rejected.
    let src = "function m() { const o = []; o.push(1); console.log(o.length); }";
    assert_eq!(candidates(src), vec!["o".to_string()]);
    assert!(rejects(src).is_empty());
}

#[test]
fn a_non_growable_shape_push_receiver_is_not_a_reject() {
    // A param (not a `const [] ` declaration) is not growable-shape, so a
    // `.push` on it stays on the pre-existing lane byte-identically (protects
    // the ~29 existing `.push` test files whose receivers are params/plain
    // arrays, never `const o = []`).
    let src = "function m(o) { o.push(1); return o; }";
    assert!(rejects(src).is_empty());
    assert!(candidates(src).is_empty());
}

#[test]
fn push_length_index_only_binding_is_a_candidate() {
    let names = candidates(
        "function main() { const o = []; o.push(1); o.push(2); \
         console.log(o.length); console.log(o[0]); \
         let s = 0; for (let i = 0; i < o.length; i++) { s += o[i]; } }",
    );
    assert_eq!(names, vec!["o".to_string()]);
}

#[test]
fn seeded_literal_and_let_declarations_are_candidates() {
    let names = candidates("function main() { let o = [1, 2]; o.push(3); }");
    assert_eq!(names, vec!["o".to_string()]);
}

#[test]
fn for_of_rhs_and_join_receiver_are_safe_positions() {
    let names = candidates(
        "function main() { const o = []; o.push(1); \
         for (const x of o) { console.log(x); } console.log(o.join(\",\")); }",
    );
    assert_eq!(names, vec!["o".to_string()]);
}

#[test]
fn no_push_means_no_candidate() {
    let names = candidates("function main() { const o = []; console.log(o.length); }");
    assert!(names.is_empty());
}

#[test]
fn escaping_positions_disqualify() {
    // Bare read (call argument).
    assert!(candidates("function main() { const o = []; o.push(1); f(o); }").is_empty());
    // Returned.
    assert!(candidates("function main() { const o = []; o.push(1); return o; }").is_empty());
    // Logged directly.
    assert!(candidates("function main() { const o = []; o.push(1); console.log(o); }").is_empty());
    // Stored into another array.
    assert!(candidates("function main() { const o = []; o.push(1); const p = [o]; }").is_empty());
    // Aliased.
    assert!(candidates("function main() { const o = []; o.push(1); const q = o; }").is_empty());
}

#[test]
fn mutating_positions_disqualify() {
    // Reassignment.
    assert!(candidates("function main() { let o = []; o.push(1); o = []; }").is_empty());
    // Index write.
    assert!(candidates("function main() { const o = []; o.push(1); o[0] = 2; }").is_empty());
    // Non-push method.
    assert!(candidates("function main() { const o = []; o.push(1); o.pop(); }").is_empty());
    // Multi-arg push.
    assert!(candidates("function main() { const o = []; o.push(1, 2); }").is_empty());
    // Self-push.
    assert!(candidates("function main() { const o = []; o.push(o); }").is_empty());
}

#[test]
fn closure_capture_and_shadowing_disqualify() {
    // Capture inside an arrow.
    assert!(
        candidates("function main() { const o = []; o.push(1); [1].map((v) => o.length); }")
            .is_empty()
    );
    // Same name declared twice (block shadowing breaks the name-based scan).
    assert!(candidates(
        "function main() { const o = []; o.push(1); { const o = [2]; console.log(o.length); } }"
    )
    .is_empty());
    // Param of the same name.
    assert!(candidates("function main(o) { const o = []; o.push(1); }").is_empty());
}

#[test]
fn non_scalar_shapes_disqualify() {
    // Object seed.
    assert!(candidates("function main() { const o = [{ x: 1 }]; o.push(1); }").is_empty());
    // Identifier seed (could be an object/array handle).
    assert!(candidates("function main() { const p = 1; const o = [p]; o.push(1); }").is_empty());
    // Call-result push arg.
    assert!(candidates("function main() { const o = []; o.push(f()); }").is_empty());
    // `var` declaration (hoisting makes pre-declaration pushes reachable).
    assert!(candidates("function main() { var o = []; o.push(1); }").is_empty());
}

#[test]
fn push_sites_report_identifier_arguments() {
    let (params, body) = func_body(
        "function main() { const o = []; for (const item of [1, 2]) { o.push(item); } \
         console.log(o.length); }",
    );
    let (set, pushes, _) = growable_array_candidates(&params, &body);
    assert!(set.contains("o"));
    assert_eq!(pushes.len(), 1);
    assert_eq!(pushes[0].name, "o");
    assert_eq!(pushes[0].arg_identifier.as_deref(), Some("item"));
}

#[test]
fn independent_bindings_are_judged_independently() {
    let names =
        candidates("function main() { const a = []; a.push(1); const b = []; b.push(2); f(b); }");
    assert_eq!(names, vec!["a".to_string()]);
}
