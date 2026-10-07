// Consumed by repr inference (Task 6); unused until then.
#![allow(dead_code)]

//! Growable-runtime-arrays spec §3.3, §3.5-§3.6, A-3, A-6, A-8: every
//! refusal the solved growable property implies, as message text. Pure: the
//! facts and the solution in, strings out; repr inference turns each into a
//! shape conflict (E5506), so `check` and `run` report them alike.

use std::collections::{BTreeMap, BTreeSet};

use super::flow::{GrowConflict, GrowFacts, GrowNode, GrowSolution, TempKind, UseKind};

pub(crate) fn growable_refusals(
    facts: &GrowFacts,
    solution: &GrowSolution,
    params: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut messages: Vec<String> = Vec::new();
    for conflict in &solution.conflicts {
        messages.push(match conflict {
            GrowConflict::MixedLayout(node) => {
                kali_common::growable_mixed_layout_message(&subject(node, facts))
            }
            GrowConflict::NonArrayWrite(node) => {
                kali_common::growable_non_array_write_message(&subject(node, facts))
            }
            GrowConflict::LiteralExpression(site) => {
                kali_common::growable_literal_expression_message(site)
            }
        });
    }
    for occurrence in &facts.uses {
        if !solution.is_growable(&occurrence.node) {
            continue;
        }
        let named = matches!(occurrence.node, GrowNode::Binding(..));
        let node = &occurrence.node;
        let message = match &occurrence.kind {
            UseKind::Push | UseKind::Pop | UseKind::IndexRead | UseKind::IndexWrite if !named => {
                Some(kali_common::growable_temporary_use_message(&source(node, facts)))
            }
            UseKind::Flow
            | UseKind::Push
            | UseKind::Pop
            | UseKind::IndexRead
            | UseKind::IndexWrite
            | UseKind::LengthRead
            | UseKind::ForOf
            | UseKind::Join
            | UseKind::Slice
            | UseKind::Search { from_index: false, .. } => None,
            UseKind::Search { method, from_index: true } => {
                Some(kali_common::growable_from_index_message(method))
            }
            UseKind::LengthWrite => {
                Some(kali_common::growable_length_write_message(&subject(node, facts)))
            }
            UseKind::Console => {
                Some(kali_common::runtime_array_print_unavailable_message().to_string())
            }
            UseKind::Method(operation) => Some(kali_common::growable_unsupported_operation_message(
                operation,
                &subject(node, facts),
            )),
            UseKind::Plain => Some(kali_common::growable_plain_use_message(&subject(node, facts))),
            UseKind::Captured => Some(kali_common::growable_capture_message(&subject(node, facts))),
            UseKind::ModuleRead => match node {
                GrowNode::Binding(_, name) => {
                    Some(kali_common::growable_module_read_message(name, &occurrence.site))
                }
                _ => None,
            },
        };
        messages.extend(message);
    }
    for stack in &facts.opaque_sites {
        for node in solution.growable_members() {
            if let GrowNode::Binding(func, _) = node {
                if stack.iter().any(|frame| frame == func) {
                    messages.push(kali_common::growable_capture_message(&subject(node, facts)));
                }
            }
        }
    }
    let mutating = length_mutating_params(facts, solution, params);
    for frame in &facts.loops {
        if !solution.is_growable(&frame.iterable) {
            continue;
        }
        let direct = frame
            .mutations
            .iter()
            .any(|receiver| solution.same_component(receiver, &frame.iterable));
        let through_call = frame.calls.iter().any(|call| {
            solution.same_component(&call.node, &frame.iterable)
                && mutating.contains(&(call.callee.clone(), call.index))
        });
        if direct || through_call {
            messages.push(kali_common::growable_for_of_mutation_message(&subject(
                &frame.iterable,
                facts,
            )));
        }
    }
    let mut seen = BTreeSet::new();
    messages.retain(|message| seen.insert(message.clone()));
    messages
}

/// Spec A-8: `(callee, index)` pairs whose parameter may have `push` or
/// `pop` applied to the array it receives: in the callee's own body, on the
/// parameter or on a local in its growable component, or through a further
/// call that passes such a node on. A least fixed point; it only grows.
pub(crate) fn length_mutating_params(
    facts: &GrowFacts,
    solution: &GrowSolution,
    params: &BTreeMap<String, Vec<String>>,
) -> BTreeSet<(String, usize)> {
    let indices_reaching = |site: &str, node: &GrowNode| -> Vec<usize> {
        params
            .get(site)
            .map(|names| {
                names
                    .iter()
                    .enumerate()
                    .filter(|(_, name)| {
                        solution.same_component(
                            &GrowNode::Binding(site.to_string(), (*name).clone()),
                            node,
                        )
                    })
                    .map(|(index, _)| index)
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut mutating = BTreeSet::new();
    for occurrence in &facts.uses {
        if matches!(occurrence.kind, UseKind::Push | UseKind::Pop) {
            for index in indices_reaching(&occurrence.site, &occurrence.node) {
                mutating.insert((occurrence.site.clone(), index));
            }
        }
    }
    loop {
        let mut changed = false;
        for call in &facts.calls {
            if !mutating.contains(&(call.callee.clone(), call.index)) {
                continue;
            }
            for index in indices_reaching(&call.site, &call.node) {
                changed |= mutating.insert((call.site.clone(), index));
            }
        }
        if !changed {
            return mutating;
        }
    }
}

fn subject(node: &GrowNode, facts: &GrowFacts) -> String {
    match node {
        GrowNode::Binding(func, name) => kali_common::growable_binding_subject(name, func),
        GrowNode::Return(func) => kali_common::growable_return_subject(func),
        GrowNode::Temp(n) => match facts.temp_kinds.get(n) {
            Some((TempKind::Slice, site)) => format!(
                "{} {}",
                kali_common::growable_slice_result_source(),
                kali_common::growable_scope_phrase(site)
            ),
            Some((_, site)) => format!(
                "a `?:`, `||` or `&&` value {}",
                kali_common::growable_scope_phrase(site)
            ),
            None => "a temporary array".to_string(),
        },
    }
}

/// Spec A-6: what a refusal of direct indexing or mutation names.
fn source(node: &GrowNode, facts: &GrowFacts) -> String {
    match node {
        GrowNode::Return(func) => kali_common::growable_call_result_source(func),
        GrowNode::Temp(n) if matches!(facts.temp_kinds.get(n), Some((TempKind::Slice, _))) => {
            kali_common::growable_slice_result_source().to_string()
        }
        other => subject(other, facts),
    }
}

#[cfg(test)]
#[path = "positions_tests.rs"]
mod positions_tests;
