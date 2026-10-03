//! Program classes, their bases and member names (unresolved-member-call
//! spec A-1, §3.2, §3.3). Collected over the serialized AST so nested
//! functions, methods and class expressions are all covered.

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::Statement;
use serde_json::Value;

struct ClassFacts {
    super_class: Option<String>,
    members: BTreeSet<String>,
}

pub(crate) struct ProgramClasses {
    classes: BTreeMap<String, ClassFacts>,
}

impl ProgramClasses {
    pub(crate) fn collect(statements: &[Statement]) -> ProgramClasses {
        let mut classes = BTreeMap::new();
        if let Ok(tree) = serde_json::to_value(statements) {
            collect_from(&tree, None, &mut classes);
        }
        ProgramClasses { classes }
    }

    pub(crate) fn is_program_class(&self, name: &str) -> bool {
        self.classes.contains_key(name)
    }

    /// The chain from `name` upward, or `None` when it leaves the program
    /// (a base that is no program class, or a cycle).
    fn chain(&self, name: &str) -> Option<Vec<&ClassFacts>> {
        let mut chain = Vec::new();
        let mut seen = BTreeSet::new();
        let mut current = name;
        loop {
            if !seen.insert(current) {
                return None;
            }
            let facts = self.classes.get(current)?;
            chain.push(facts);
            match facts.super_class.as_deref() {
                None => return Some(chain),
                Some(base) => current = base,
            }
        }
    }

    pub(crate) fn host_derived(&self) -> BTreeSet<String> {
        self.classes
            .keys()
            .filter(|name| self.chain(name).is_none())
            .cloned()
            .collect()
    }

    pub(crate) fn member_names(&self, name: &str) -> Option<BTreeSet<String>> {
        let chain = self.chain(name)?;
        Some(
            chain
                .iter()
                .flat_map(|facts| facts.members.iter().cloned())
                .collect(),
        )
    }
}

fn class_facts(class: &Value) -> ClassFacts {
    let super_class = class
        .get("super_class")
        .and_then(Value::as_str)
        .map(str::to_string);
    let mut members = BTreeSet::new();
    if let Some(body) = class.get("body") {
        for method in body
            .get("methods")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(name) = method.get("name").and_then(Value::as_str) {
                members.insert(name.to_string());
            }
        }
        for field in body
            .get("field_names")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(name) = field.as_str() {
                members.insert(name.to_string());
            }
        }
    }
    ClassFacts {
        super_class,
        members,
    }
}

/// `binding` is the declarator name when `value` is a declarator's init, so a
/// nameless `const K = class …` is recorded as `K`.
fn collect_from(value: &Value, binding: Option<&str>, out: &mut BTreeMap<String, ClassFacts>) {
    match value {
        Value::Object(map) => {
            if let Some(class) = map.get("ClassDeclaration") {
                if let Some(name) = class.get("name").and_then(Value::as_str) {
                    out.insert(name.to_string(), class_facts(class));
                }
            }
            if let Some(class) = map.get("ClassExpression") {
                let name = class.get("id").and_then(Value::as_str).or(binding);
                if let Some(name) = name {
                    out.insert(name.to_string(), class_facts(class));
                }
            }
            // A declarator: `{ "id": "K", "init": … }`.
            let declarator_name = match (map.get("id"), map.get("init")) {
                (Some(Value::String(id)), Some(_)) => Some(id.as_str()),
                _ => None,
            };
            for (key, child) in map {
                let child_binding = if key == "init" { declarator_name } else { None };
                collect_from(child, child_binding, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_from(item, binding, out);
            }
        }
        _ => {}
    }
}

/// Every property name some assignment writes, on any receiver, in any scope
/// (`o.f = …`, `this.cb = …`, `x.h += 1`).
pub(crate) fn assigned_property_names(statements: &[Statement]) -> BTreeSet<String> {
    fn walk(value: &Value, out: &mut BTreeSet<String>) {
        match value {
            Value::Object(map) => {
                if let Some(assign) = map.get("AssignmentExpression") {
                    if let Some(property) = assign
                        .get("left")
                        .and_then(|left| left.get("MemberExpression"))
                        .and_then(|member| member.get("property"))
                        .and_then(Value::as_str)
                    {
                        out.insert(property.to_string());
                    }
                }
                map.values().for_each(|child| walk(child, out));
            }
            Value::Array(items) => items.iter().for_each(|item| walk(item, out)),
            _ => {}
        }
    }
    let mut out = BTreeSet::new();
    if let Ok(tree) = serde_json::to_value(statements) {
        walk(&tree, &mut out);
    }
    out
}
