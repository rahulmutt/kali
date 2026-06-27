use std::{collections::BTreeMap, sync::Arc};

use kali_error::Diagnostic;

use crate::diagnostics::{host_predicate_violation, unavailable_capability};
use crate::PolicyPredicateContext;

/// Deterministic host-registered narrowing predicate registry.
#[derive(Clone)]
pub struct PolicyPredicateRegistry {
    enabled: bool,
    predicates: BTreeMap<String, Vec<RegisteredPredicate>>,
}

#[derive(Clone)]
struct RegisteredPredicate {
    name: String,
    predicate: HostPredicate,
}

/// Host predicate function used by the embedding narrowing layer.
pub type HostPredicate = Arc<dyn Fn(&PolicyPredicateContext) -> bool + Send + Sync + 'static>;

impl Default for PolicyPredicateRegistry {
    fn default() -> Self {
        Self::enabled()
    }
}

impl PolicyPredicateRegistry {
    /// Create an enabled predicate registry.
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            predicates: BTreeMap::new(),
        }
    }

    /// Create a disabled registry that deterministically rejects predicate evaluation.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            predicates: BTreeMap::new(),
        }
    }

    /// Return whether host-registered predicate evaluation is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Register a deterministic narrowing predicate for one canonical capability name.
    pub fn register(
        &mut self,
        capability: impl Into<String>,
        name: impl Into<String>,
        predicate: impl Fn(&PolicyPredicateContext) -> bool + Send + Sync + 'static,
    ) -> &mut Self {
        let capability = capability.into();
        let entry = self.predicates.entry(capability).or_default();
        entry.push(RegisteredPredicate {
            name: name.into(),
            predicate: Arc::new(predicate),
        });
        self
    }

    pub(crate) fn evaluate(&self, context: &PolicyPredicateContext) -> Result<(), Diagnostic> {
        if !self.enabled {
            return Err(unavailable_capability("host-registered sandbox predicates"));
        }

        let Some(predicates) = self.predicates.get(&context.capability) else {
            return Ok(());
        };

        for predicate in predicates {
            if !(predicate.predicate)(context) {
                return Err(host_predicate_violation(
                    format!(
                        "host-registered predicate '{}' rejected {} for subject '{}'",
                        predicate.name, context.capability, context.subject
                    ),
                    context,
                ));
            }
        }

        Ok(())
    }
}
