use kali_error::{
    _error_codes::{e4, e5},
    Diagnostic,
};

use crate::PolicyPredicateContext;

pub(crate) fn unavailable_capability(name: &str) -> Diagnostic {
    Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "{} is unavailable in the current phase or availability context",
            name
        ),
    )
}

pub(crate) fn host_predicate_violation(
    message: impl Into<String>,
    context: &PolicyPredicateContext,
) -> Diagnostic {
    let mut diagnostic = Diagnostic::error(e4::EFFECT_NOT_PERMITTED as u32, message.into())
        .note(format!("capability: {}", context.capability))
        .note(format!("subject: {}", context.subject));

    for (key, value) in &context.details {
        diagnostic = diagnostic.note(format!("detail {}: {}", key, value));
    }

    diagnostic
}

pub(crate) fn sandbox_violation(message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(e4::EFFECT_NOT_PERMITTED as u32, message.into())
}

pub(crate) fn resource_limit_violation(message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(e4::RESOURCE_LIMIT_EXCEEDED as u32, message.into())
}
