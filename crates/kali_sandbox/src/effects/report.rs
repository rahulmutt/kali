use std::collections::{BTreeMap, BTreeSet, HashSet};

use serde::{Deserialize, Serialize};

/// Analysis context recorded in effect reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectAnalysisContext {
    pub api_surface: String,
    pub runtime_profiles: Vec<String>,
    pub compat_features: Vec<String>,
}

impl EffectAnalysisContext {
    pub fn new(api_surface: impl Into<String>) -> Self {
        Self {
            api_surface: api_surface.into(),
            runtime_profiles: Vec::new(),
            compat_features: Vec::new(),
        }
    }

    /// Return a normalized copy with sorted, deduplicated semantic axes.
    pub fn normalized(mut self) -> Self {
        self.runtime_profiles = normalize_semantic_axis(self.runtime_profiles);
        self.compat_features = normalize_semantic_axis(self.compat_features);
        self
    }
}

/// Location attached to an observed effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
}

/// One occurrence of a built-in effect kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectOccurrence {
    pub kind: String,
    pub locations: Vec<EffectLocation>,
}

/// Public reusable effect-report payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectReport {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub analysis_context: EffectAnalysisContext,
    pub entry_points: Vec<String>,
    pub effects: Vec<EffectOccurrence>,
    pub dynamic_effects: bool,
    pub dynamic_reasons: Vec<String>,
}

/// Package coordinate used by the package-effects report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageCoordinate {
    pub name: String,
    pub version: String,
    pub registry: String,
}

/// Package-effects wrapper payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageEffectsReport {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub package: PackageCoordinate,
    pub report: EffectReport,
}

/// Internal observed effect with optional target details for policy comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedEffect {
    pub kind: String,
    pub location: EffectLocation,
    pub target: Option<String>,
}

/// Result of inferring effects across one or more source roots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectInference {
    pub effects: Vec<ObservedEffect>,
    pub dynamic_reasons: Vec<String>,
}

/// Convert inferred effects into the public reusable effect-report payload.
pub fn effect_report_from_inference(
    mut entry_points: Vec<String>,
    context: EffectAnalysisContext,
    inference: EffectInference,
) -> EffectReport {
    let EffectInference {
        effects,
        dynamic_reasons,
    } = inference;
    let context = context.normalized();

    normalize_entry_points(&mut entry_points);

    let mut grouped = BTreeMap::<String, Vec<EffectLocation>>::new();
    for effect in effects {
        grouped
            .entry(effect.kind)
            .or_default()
            .push(effect.location);
    }

    let mut effect_groups = grouped
        .into_iter()
        .map(|(kind, mut locations)| {
            locations.sort_by(location_sort_key);
            EffectOccurrence { kind, locations }
        })
        .collect::<Vec<_>>();
    effect_groups.sort_by(|a, b| a.kind.cmp(&b.kind));

    let mut dynamic_reasons = dynamic_reasons;
    dynamic_reasons.sort();
    dynamic_reasons.dedup();

    EffectReport {
        schema_version: 1,
        analysis_context: context,
        entry_points,
        effects: effect_groups,
        dynamic_effects: !dynamic_reasons.is_empty(),
        dynamic_reasons,
    }
}

/// Wrap a public effect report in the package-effects envelope.
pub fn package_effects_report(
    package: PackageCoordinate,
    report: EffectReport,
) -> PackageEffectsReport {
    PackageEffectsReport {
        schema_version: 1,
        package,
        report,
    }
}

fn normalize_semantic_axis(values: Vec<String>) -> Vec<String> {
    let mut normalized = BTreeSet::new();
    for value in values {
        let value = value.trim();
        if !value.is_empty() {
            normalized.insert(value.to_string());
        }
    }
    normalized.into_iter().collect()
}

fn normalize_entry_points(entry_points: &mut Vec<String>) {
    let mut seen = HashSet::<String>::new();
    entry_points.retain(|entry_point| seen.insert(entry_point.clone()));
}

pub(crate) fn location_sort_key(a: &EffectLocation, b: &EffectLocation) -> std::cmp::Ordering {
    a.file
        .cmp(&b.file)
        .then_with(|| a.line.cmp(&b.line))
        .then_with(|| a.column.cmp(&b.column))
        .then_with(|| a.function.cmp(&b.function))
}
