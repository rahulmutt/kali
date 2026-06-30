use super::*;

#[test]
fn effect_analysis_tracks_phase_three_deno_host_capabilities() {
    let source = write_source_fixture(
        r#"
Deno.env.set('KALI_CORPUS_FLAG', 'set');
Deno.env.delete('KALI_CORPUS_FLAG');
new Deno.Command('sh').spawn();
Deno.connect('127.0.0.1', 1);
Deno.listen('127.0.0.1', 0);
Deno.serve('127.0.0.1', 0);
Deno.open('/workspace/input.txt');
Deno.create('/workspace/output.txt');
Deno.mkdir('/workspace/newdir');
Deno.remove('/workspace/old.txt');
Deno.rename('/workspace/from.txt', '/workspace/to.txt');
Deno.lstat('/workspace/input.txt');
"#,
    );

    let inference = infer_effects_from_roots(&[source], EffectAnalysisContext::new("deno"))
        .expect("infer effects");

    let kinds: Vec<_> = inference
        .effects
        .iter()
        .map(|effect| effect.kind.as_str())
        .collect();
    for kind in [
        "FileSystem.Read",
        "FileSystem.Write",
        "Network.Connect",
        "Network.Listen",
        "Process.EnvWrite",
        "Process.Spawn",
    ] {
        assert!(
            kinds.contains(&kind),
            "missing effect kind {kind:?}: {kinds:?}"
        );
    }

    let diagnostics = compare_effects_to_policy(&inference.effects, &valid_policy());
    assert!(
        diagnostics.len() >= 4,
        "expected policy diagnostics for the phase-three capability slice, got {diagnostics:?}"
    );
    assert!(
        diagnostics.iter().any(|diag| diag.code == Some(9007)),
        "expected an E9007 policy mismatch diagnostic: {diagnostics:?}"
    );
    assert!(inference.dynamic_reasons.is_empty());
}

#[test]
fn effect_analysis_marks_computed_bracketed_deno_command_constructors_as_dynamic_in_js_input() {
    let source = write_source_fixture_with_extension(
        r#"
new globalThis["Deno"]["Command"]('sh').spawn();
new globalThis["Deno"].Command('sh').spawn();
new Deno["Command"]('sh').spawn();
"#,
        "js",
    );

    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert_eq!(inference.dynamic_reasons, vec!["computed-host-access"]);
    assert!(inference
        .effects
        .iter()
        .any(|effect| effect.kind == "Process.Spawn"));

    let diagnostics = compare_effects_to_policy(&inference.effects, &valid_policy());
    assert!(
        diagnostics.iter().any(|diag| diag.code == Some(9007)),
        "expected an E9007 policy mismatch diagnostic: {diagnostics:?}"
    );
}

#[test]
fn effect_analysis_marks_computed_deno_host_access_as_dynamic() {
    let source = write_source_fixture(
        r#"
globalThis["Deno"]["env"]["set"]('KALI_CORPUS_FLAG', 'set');
globalThis["Deno"]["env"]["delete"]('KALI_CORPUS_FLAG');
"#,
    );

    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert_eq!(inference.dynamic_reasons, vec!["computed-host-access"]);
    assert!(inference
        .effects
        .iter()
        .any(|effect| effect.kind == "Process.EnvWrite"));
}

#[test]
fn effect_analysis_tracks_node_process_env_assignment_in_js_input() {
    let source = write_source_fixture_with_extension(
        r#"
process.env = {};
process["env"] = {};
globalThis.process.env = {};
globalThis.process["env"] = {};
globalThis["process"].env = {};
globalThis["process"]["env"] = {};
"#,
        "js",
    );

    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("node"),
    )
    .expect("infer effects");

    assert_eq!(inference.dynamic_reasons, vec!["computed-host-access"]);
    assert!(inference
        .effects
        .iter()
        .any(|effect| effect.kind == "Process.EnvWrite"));
}

#[test]
fn effect_analysis_tracks_direct_deno_network_calls_in_js_input() {
    let source = write_source_fixture_with_extension(
        r#"
Deno.connect('127.0.0.1', 1);
Deno.listen('127.0.0.1', 0);
Deno.serve('127.0.0.1', 0);
"#,
        "js",
    );

    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert!(inference.dynamic_reasons.is_empty());
    assert!(inference
        .effects
        .iter()
        .any(|effect| effect.kind == "Network.Connect"));
    assert!(inference
        .effects
        .iter()
        .any(|effect| effect.kind == "Network.Listen"));

    let diagnostics = compare_effects_to_policy(&inference.effects, &valid_policy());
    assert!(
        diagnostics.len() >= 2,
        "expected policy diagnostics for the direct network capability slice, got {diagnostics:?}"
    );
    assert!(
        diagnostics.iter().any(|diag| diag.code == Some(9007)),
        "expected an E9007 policy mismatch diagnostic: {diagnostics:?}"
    );
}

#[test]
fn effect_analysis_marks_computed_bracketed_deno_network_calls_as_dynamic_in_js_input() {
    let source = write_source_fixture_with_extension(
        r#"
globalThis["Deno"]["connect"]('127.0.0.1', 1);
globalThis["Deno"]["listen"]('127.0.0.1', 0);
globalThis["Deno"]["serve"]('127.0.0.1', 0);
"#,
        "js",
    );

    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert_eq!(inference.dynamic_reasons, vec!["computed-host-access"]);
    assert!(inference
        .effects
        .iter()
        .any(|effect| effect.kind == "Network.Connect"));
    assert!(inference
        .effects
        .iter()
        .any(|effect| effect.kind == "Network.Listen"));

    let diagnostics = compare_effects_to_policy(&inference.effects, &valid_policy());
    assert!(
        diagnostics.len() >= 2,
        "expected policy diagnostics for the computed network capability slice, got {diagnostics:?}"
    );
    assert!(
        diagnostics.iter().any(|diag| diag.code == Some(9007)),
        "expected an E9007 policy mismatch diagnostic: {diagnostics:?}"
    );
}

#[test]
fn effect_analysis_marks_computed_bracketed_deno_env_read_as_dynamic() {
    let source = write_source_fixture_with_extension(
        r#"
Deno["env"]["get"]("KALI_CORPUS_FLAG");
globalThis["Deno"]["env"]["get"]("KALI_CORPUS_FLAG");
"#,
        "js",
    );

    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert_eq!(inference.dynamic_reasons, vec!["computed-host-access"]);
    assert!(inference
        .effects
        .iter()
        .any(|effect| effect.kind == "Process.EnvRead"));
}

#[test]
fn effect_analysis_tracks_deno_env_to_object_as_env_read() {
    let source = write_source_fixture_with_extension(
        r#"
Deno.env.toObject;
globalThis.Deno.env.toObject;
"#,
        "js",
    );

    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert!(
        inference.dynamic_reasons.is_empty(),
        "unexpected dynamic reasons: {:?}",
        inference.dynamic_reasons
    );
    assert!(
        inference
            .effects
            .iter()
            .filter(|effect| effect.kind == "Process.EnvRead")
            .count()
            >= 2,
        "effects: {:?}",
        inference.effects
    );
}

#[test]
fn effect_analysis_tracks_bracketed_deno_env_to_object_as_dynamic_env_read() {
    let source = write_source_fixture_with_extension(
        r#"
Deno["env"]["toObject"];
Deno["env"].toObject;
globalThis["Deno"]["env"]["toObject"];
globalThis.Deno["env"].toObject;
globalThis["Deno"].env.toObject;
globalThis["Deno"].env["toObject"];
"#,
        "js",
    );

    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert_eq!(inference.dynamic_reasons, vec!["computed-host-access"]);
    assert!(inference
        .effects
        .iter()
        .any(|effect| effect.kind == "Process.EnvRead"));
}

#[test]
fn effect_analysis_marks_proxy_constructor_and_revocable_calls_as_dynamic() {
    let source = write_source_fixture(
        r#"
new Proxy({}, {});
new globalThis.Proxy({}, {});
new globalThis["Proxy"]({}, {});
Proxy.revocable({}, {});
Proxy["revocable"]({}, {});
globalThis.Proxy.revocable({}, {});
globalThis.Proxy["revocable"]({}, {});
globalThis["Proxy"].revocable({}, {});
globalThis["Proxy"]["revocable"]({}, {});
"#,
    );

    let inference = infer_effects_from_roots(
        std::slice::from_ref(&source),
        EffectAnalysisContext::new("deno"),
    )
    .expect("infer effects");

    assert_eq!(inference.dynamic_reasons, vec!["proxy-traps"]);
    assert!(
        inference.effects.is_empty(),
        "unexpected observed effects: {inference:?}"
    );

    let report = effect_report_from_inference(
        vec![source.display().to_string()],
        EffectAnalysisContext::new("deno"),
        inference,
    );

    assert!(report.dynamic_effects);
    assert_eq!(report.dynamic_reasons, vec!["proxy-traps"]);
}
