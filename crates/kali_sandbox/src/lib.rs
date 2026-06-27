//! Sandbox and policy system for the Kali compiler.

pub mod effects;

pub use effects::{
    compare_effects_to_policy, effect_report_from_inference, infer_effects_from_roots,
    package_effects_report, EffectAnalysisContext, EffectInference, EffectLocation,
    EffectOccurrence, EffectReport, ObservedEffect, PackageCoordinate, PackageEffectsReport,
};

mod diagnostics;
mod enforcement;
mod loading;
mod matching;
mod operation;
mod policy;
mod predicate;
mod validation;

pub use operation::{HostOperation, PolicyPredicateContext};
pub use policy::{
    AccessRule, EffectsPolicy, FileSystemPolicy, NetworkPolicy, ProcessPolicy, ResourceLimits,
    SandboxPolicy, TimerPolicy,
};
pub use predicate::{HostPredicate, PolicyPredicateRegistry};
pub use validation::PolicyValidation;

pub(crate) use matching::PatternKind;

#[cfg(test)]
use std::path::Path;

#[cfg(test)]
use kali_error::_error_codes::{e4, e5};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
