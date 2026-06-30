use super::*;

#[test]
fn effect_reports_normalize_dynamic_reasons_and_analysis_context_axes() {
    let mut context = EffectAnalysisContext::new("deno");
    context.runtime_profiles = vec![
        " wasm-threads ".to_string(),
        "alpha".to_string(),
        "wasm-threads".to_string(),
    ];
    context.compat_features = vec![
        " beta ".to_string(),
        "alpha".to_string(),
        "beta".to_string(),
    ];

    let report = effect_report_from_inference(
        vec!["main.ts".to_string()],
        context,
        EffectInference {
            effects: Vec::new(),
            dynamic_reasons: vec![
                "proxy-traps".to_string(),
                "eval".to_string(),
                "proxy-traps".to_string(),
            ],
        },
    );

    assert_eq!(report.dynamic_reasons, vec!["eval", "proxy-traps"]);
    assert!(report.dynamic_effects);
    assert_eq!(
        report.analysis_context.runtime_profiles,
        vec!["alpha", "wasm-threads"]
    );
    assert_eq!(
        report.analysis_context.compat_features,
        vec!["alpha", "beta"]
    );
}

#[test]
fn effect_reports_trim_and_deduplicate_semantic_axes_before_serialization() {
    let mut context = EffectAnalysisContext::new("deno");
    context.runtime_profiles = vec![
        " wasm-threads ".to_string(),
        "".to_string(),
        "alpha".to_string(),
        "alpha ".to_string(),
    ];
    context.compat_features = vec![
        " eval ".to_string(),
        "beta".to_string(),
        " ".to_string(),
        "eval".to_string(),
    ];

    let report = effect_report_from_inference(
        vec!["main.ts".to_string()],
        context,
        EffectInference {
            effects: Vec::new(),
            dynamic_reasons: Vec::new(),
        },
    );

    assert_eq!(
        report.analysis_context.runtime_profiles,
        vec!["alpha", "wasm-threads"]
    );
    assert_eq!(
        report.analysis_context.compat_features,
        vec!["beta", "eval"]
    );
}

#[test]
fn effect_inference_deduplicates_repeated_roots_before_serialization() {
    let source = write_source_fixture("console.log('hello');");
    let inference = infer_effects_from_roots(
        &[source.clone(), source.clone()],
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert_eq!(
        inference.effects.len(),
        1,
        "unexpected inferred effects: {inference:?}"
    );
    assert_eq!(inference.effects[0].kind, "Console.Write");

    let report = effect_report_from_inference(
        vec![source.display().to_string(), source.display().to_string()],
        EffectAnalysisContext::new("deno"),
        inference,
    );

    assert_eq!(report.entry_points, vec![source.display().to_string()]);
    assert_eq!(report.effects.len(), 1);
    assert_eq!(report.effects[0].kind, "Console.Write");
}

#[test]
fn effect_reports_deduplicate_entry_points_while_preserving_first_seen_order() {
    let report = effect_report_from_inference(
        vec![
            "src/main.ts".to_string(),
            "src/helper.ts".to_string(),
            "src/main.ts".to_string(),
            "src/other.ts".to_string(),
            "src/helper.ts".to_string(),
        ],
        EffectAnalysisContext::new("deno"),
        EffectInference {
            effects: Vec::new(),
            dynamic_reasons: Vec::new(),
        },
    );

    assert_eq!(
        report.entry_points,
        vec!["src/main.ts", "src/helper.ts", "src/other.ts"]
    );
}

#[test]
fn effect_reports_treat_permissions_query_as_effect_free() {
    let source = write_source_fixture(
        r#"Deno.permissions.query({ name: "read" });
Deno.permissions.query({ name: "write" });
Deno.permissions.query({ name: "env" });
Deno.permissions.query({ name: "net" });"#,
    );
    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert!(
        inference.effects.is_empty(),
        "unexpected observed effects: {inference:?}"
    );
    assert!(
        inference.dynamic_reasons.is_empty(),
        "unexpected dynamic reasons: {inference:?}"
    );

    let report = effect_report_from_inference(
        vec![source.display().to_string()],
        EffectAnalysisContext::new("deno"),
        inference,
    );

    assert!(!report.dynamic_effects);
    assert!(report.dynamic_reasons.is_empty());
    assert!(report.effects.is_empty());
}

#[test]
fn effect_reports_treat_computed_permissions_query_as_effect_free() {
    let source = write_source_fixture(
        r#"Deno["permissions"]["query"]({ name: "read" });
Deno.permissions["query"]({ name: "read" });
globalThis["Deno"]["permissions"].query({ name: "read" });
globalThis["Deno"]["permissions"]["query"]({ name: "read" });
Deno["permissions"]["query"]({ name: "write" });
Deno.permissions["query"]({ name: "write" });
globalThis["Deno"]["permissions"].query({ name: "write" });
globalThis["Deno"]["permissions"]["query"]({ name: "write" });
Deno["permissions"]["query"]({ name: "env" });
Deno.permissions["query"]({ name: "env" });
globalThis["Deno"]["permissions"].query({ name: "env" });
globalThis["Deno"]["permissions"]["query"]({ name: "env" });
Deno["permissions"]["query"]({ name: "net" });
Deno.permissions["query"]({ name: "net" });
globalThis["Deno"]["permissions"].query({ name: "net" });
globalThis["Deno"]["permissions"]["query"]({ name: "net" });"#,
    );
    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert!(
        inference.effects.is_empty(),
        "unexpected observed effects: {inference:?}"
    );
    assert_eq!(inference.dynamic_reasons, vec!["computed-host-access"]);

    let report = effect_report_from_inference(
        vec![source.display().to_string()],
        EffectAnalysisContext::new("deno"),
        inference,
    );

    assert!(report.dynamic_effects);
    assert_eq!(report.dynamic_reasons, vec!["computed-host-access"]);
    assert!(report.effects.is_empty());
}

#[test]
fn effect_reports_treat_permissions_query_as_effect_free_in_js_input() {
    let source = write_source_fixture_with_extension(
        r#"Deno.permissions.query({ name: "read" });
Deno.permissions.query({ name: "write" });
Deno.permissions.query({ name: "env" });
Deno.permissions.query({ name: "net" });"#,
        "js",
    );
    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert!(
        inference.effects.is_empty(),
        "unexpected observed effects: {inference:?}"
    );
    assert!(
        inference.dynamic_reasons.is_empty(),
        "unexpected dynamic reasons: {inference:?}"
    );

    let report = effect_report_from_inference(
        vec![source.display().to_string()],
        EffectAnalysisContext::new("deno"),
        inference,
    );

    assert!(!report.dynamic_effects);
    assert!(report.dynamic_reasons.is_empty());
    assert!(report.effects.is_empty());
}

#[test]
fn effect_reports_treat_computed_permissions_query_as_effect_free_in_js_input() {
    let source = write_source_fixture_with_extension(
        r#"Deno["permissions"]["query"]({ name: "read" });
Deno.permissions["query"]({ name: "read" });
globalThis["Deno"]["permissions"].query({ name: "read" });
globalThis["Deno"]["permissions"]["query"]({ name: "read" });
Deno["permissions"]["query"]({ name: "write" });
Deno.permissions["query"]({ name: "write" });
globalThis["Deno"]["permissions"].query({ name: "write" });
globalThis["Deno"]["permissions"]["query"]({ name: "write" });
Deno["permissions"]["query"]({ name: "env" });
Deno.permissions["query"]({ name: "env" });
globalThis["Deno"]["permissions"].query({ name: "env" });
globalThis["Deno"]["permissions"]["query"]({ name: "env" });
Deno["permissions"]["query"]({ name: "net" });
Deno.permissions["query"]({ name: "net" });
globalThis["Deno"]["permissions"].query({ name: "net" });
globalThis["Deno"]["permissions"]["query"]({ name: "net" });"#,
        "js",
    );
    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert!(
        inference.effects.is_empty(),
        "unexpected observed effects: {inference:?}"
    );
    assert_eq!(inference.dynamic_reasons, vec!["computed-host-access"]);

    let report = effect_report_from_inference(
        vec![source.display().to_string()],
        EffectAnalysisContext::new("deno"),
        inference,
    );

    assert!(report.dynamic_effects);
    assert_eq!(report.dynamic_reasons, vec!["computed-host-access"]);
    assert!(report.effects.is_empty());
}

#[test]
fn effect_reports_sort_effect_groups_and_locations_deterministically() {
    let report = effect_report_from_inference(
        vec!["src/main.ts".to_string()],
        EffectAnalysisContext::new("deno"),
        EffectInference {
            effects: vec![
                ObservedEffect {
                    kind: "Network.Fetch".to_string(),
                    location: EffectLocation {
                        file: "b.ts".to_string(),
                        line: 2,
                        column: 3,
                        function: None,
                    },
                    target: None,
                },
                ObservedEffect {
                    kind: "Console.Write".to_string(),
                    location: EffectLocation {
                        file: "a.ts".to_string(),
                        line: 1,
                        column: 10,
                        function: None,
                    },
                    target: None,
                },
                ObservedEffect {
                    kind: "Console.Write".to_string(),
                    location: EffectLocation {
                        file: "a.ts".to_string(),
                        line: 1,
                        column: 2,
                        function: None,
                    },
                    target: None,
                },
                ObservedEffect {
                    kind: "Network.Fetch".to_string(),
                    location: EffectLocation {
                        file: "a.ts".to_string(),
                        line: 3,
                        column: 1,
                        function: None,
                    },
                    target: None,
                },
            ],
            dynamic_reasons: Vec::new(),
        },
    );

    assert_eq!(
        report
            .effects
            .iter()
            .map(|effect| effect.kind.as_str())
            .collect::<Vec<_>>(),
        vec!["Console.Write", "Network.Fetch"]
    );

    assert_eq!(
        report.effects[0]
            .locations
            .iter()
            .map(|location| {
                (
                    location.file.as_str(),
                    location.line,
                    location.column,
                    location.function.as_deref(),
                )
            })
            .collect::<Vec<_>>(),
        vec![("a.ts", 1, 2, None), ("a.ts", 1, 10, None)]
    );

    assert_eq!(
        report.effects[1]
            .locations
            .iter()
            .map(|location| {
                (
                    location.file.as_str(),
                    location.line,
                    location.column,
                    location.function.as_deref(),
                )
            })
            .collect::<Vec<_>>(),
        vec![("a.ts", 3, 1, None), ("b.ts", 2, 3, None)]
    );
}
