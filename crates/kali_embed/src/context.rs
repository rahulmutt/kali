use std::{
    collections::BTreeMap,
    sync::Arc,
};

use kali_error::{
    _error_codes::{e4, e5},
    Diagnostic,
};
use kali_sandbox::{
    HostOperation, PolicyPredicateContext, SandboxPolicy,
};

use crate::compiler::{CompilerConfig, KaliCompiler};

/// Decision returned by host-registered sandbox predicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredicateDecision {
    /// Allow the guarded operation to proceed.
    Allow,
    /// Reject the guarded operation with a host-specific note.
    Deny(String),
}

impl PredicateDecision {
    /// Convenience constructor for an allowed operation.
    pub fn allow() -> Self {
        Self::Allow
    }

    /// Convenience constructor for a rejected operation.
    pub fn deny(message: impl Into<String>) -> Self {
        Self::Deny(message.into())
    }
}

impl From<bool> for PredicateDecision {
    fn from(value: bool) -> Self {
        if value {
            Self::Allow
        } else {
            Self::Deny(String::new())
        }
    }
}

/// Canonical operation context observed by host-registered narrowing predicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationContext {
    /// Canonical capability name from the sandbox vocabulary.
    pub capability: String,
    /// Subject/resource string associated with the host operation.
    pub resource: String,
    /// Host operation being evaluated.
    pub operation: HostOperation,
    /// Deterministic extra details for host-specific predicate logic.
    pub details: BTreeMap<String, String>,
}

impl OperationContext {
    /// Create the canonical predicate context for one host operation.
    pub fn from_operation(operation: &HostOperation) -> Self {
        let policy_context = PolicyPredicateContext::from_operation(operation);
        Self {
            capability: policy_context.capability,
            resource: policy_context.subject,
            operation: policy_context.operation,
            details: policy_context.details,
        }
    }
}

#[derive(Clone)]
struct RegisteredPredicate {
    name: String,
    predicate: Arc<dyn Fn(&OperationContext) -> PredicateDecision + Send + Sync + 'static>,
}

/// Embedding context retained for compatibility with the original stub API.
pub struct EmbeddingCtx {
    compiler: KaliCompiler,
    predicates: BTreeMap<String, Vec<RegisteredPredicate>>,
    predicate_registration_enabled: bool,
}

impl Default for EmbeddingCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddingCtx {
    pub fn new() -> Self {
        Self::with_predicate_registration_enabled(true)
    }

    /// Construct an embedding context with explicit host-predicate availability.
    pub fn with_predicate_registration_enabled(enabled: bool) -> Self {
        Self {
            compiler: KaliCompiler::new(CompilerConfig::default()),
            predicates: BTreeMap::new(),
            predicate_registration_enabled: enabled,
        }
    }

    /// Return whether host-registered sandbox predicates may be added in this context.
    pub fn predicate_registration_enabled(&self) -> bool {
        self.predicate_registration_enabled
    }

    /// Register a deterministic narrowing predicate for one canonical capability name.
    pub fn register_sandbox_predicate(
        &mut self,
        capability: impl Into<String>,
        name: impl Into<String>,
        predicate: impl Fn(&OperationContext) -> PredicateDecision + Send + Sync + 'static,
    ) -> Result<&mut Self, Diagnostic> {
        if !self.predicate_registration_enabled {
            return Err(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "host-registered sandbox predicates are unavailable in this embedding context",
            ));
        }

        let capability = capability.into();
        let entry = self.predicates.entry(capability).or_default();
        entry.push(RegisteredPredicate {
            name: name.into(),
            predicate: Arc::new(predicate),
        });
        Ok(self)
    }

    /// Evaluate a host operation against a declarative policy and the registered predicates.
    pub fn check_operation_with_policy(
        &self,
        policy: &SandboxPolicy,
        operation: HostOperation,
    ) -> Result<(), Diagnostic> {
        policy.check_operation(operation.clone())?;

        let context = OperationContext::from_operation(&operation);
        let Some(predicates) = self.predicates.get(&context.capability) else {
            return Ok(());
        };

        for predicate in predicates {
            match (predicate.predicate)(&context) {
                PredicateDecision::Allow => {}
                PredicateDecision::Deny(reason) => {
                    return Err(predicate_violation(&predicate.name, &context, &reason));
                }
            }
        }

        Ok(())
    }

    /// Build a library artifact from raw source text by reusing the stable compiler API.
    pub fn build_library(&self, source: &str) -> Vec<u8> {
        self.compiler
            .compile_lib_source("embedded", source)
            .map(|artifact| artifact.wasm_bytes().to_vec())
            .unwrap_or_default()
    }
}

fn predicate_violation(name: &str, context: &OperationContext, reason: &str) -> Diagnostic {
    let reason_suffix = if reason.trim().is_empty() {
        String::new()
    } else {
        format!(": {}", reason)
    };

    let mut diagnostic = Diagnostic::error(
        e4::EFFECT_NOT_PERMITTED as u32,
        format!(
            "host-registered predicate '{}' rejected {} for resource '{}'{}",
            name, context.capability, context.resource, reason_suffix
        ),
    )
    .note(format!("capability: {}", context.capability))
    .note(format!("resource: {}", context.resource));

    for (key, value) in &context.details {
        diagnostic = diagnostic.note(format!("detail {}: {}", key, value));
    }

    diagnostic
}