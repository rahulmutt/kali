mod compare;
mod inference;
mod report;
mod scan;

pub use compare::compare_effects_to_policy;
pub use inference::infer_effects_from_roots;
pub use report::{
    effect_report_from_inference, package_effects_report, EffectAnalysisContext, EffectInference,
    EffectLocation, EffectOccurrence, EffectReport, ObservedEffect, PackageCoordinate,
    PackageEffectsReport,
};
