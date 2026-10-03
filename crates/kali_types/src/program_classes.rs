//! Program classes, their bases and member names (unresolved-member-call
//! spec A-1, §3.2, §3.3). Collected over the serialized AST so nested
//! functions, methods and class expressions are all covered.

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::Statement;
use serde_json::Value;

struct ClassFacts {
    super_class: Option<String>,
    members: BTreeSet<String>,
    /// A computed key (`["foo"](){}`) the parser skipped: `members` is not
    /// every member, so no member set is known (spec A-6).
    has_computed_members: bool,
}

pub(crate) struct ProgramClasses {
    classes: BTreeMap<String, ClassFacts>,
    /// Names declared more than once (different scopes). Facts are keyed by
    /// bare name, so these are unknowable: no member set, and not provably host.
    ambiguous: BTreeSet<String>,
}

enum Chain<'a> {
    Known(Vec<&'a ClassFacts>),
    /// Leaves the program (a non-program base, or a cycle).
    Host,
    /// Passes through a duplicated class name.
    Ambiguous,
}

impl ProgramClasses {
    pub(crate) fn collect(statements: &[Statement]) -> ProgramClasses {
        let mut classes = BTreeMap::new();
        let mut ambiguous = BTreeSet::new();
        if let Ok(tree) = serde_json::to_value(statements) {
            collect_from(&tree, None, &mut classes, &mut ambiguous);
        }
        ProgramClasses { classes, ambiguous }
    }

    pub(crate) fn is_program_class(&self, name: &str) -> bool {
        self.classes.contains_key(name)
    }

    /// The chain from `name` upward.
    fn chain(&self, name: &str) -> Chain<'_> {
        let mut chain = Vec::new();
        let mut seen = BTreeSet::new();
        let mut current = name;
        loop {
            if self.ambiguous.contains(current) {
                return Chain::Ambiguous;
            }
            if !seen.insert(current) {
                return Chain::Host;
            }
            let Some(facts) = self.classes.get(current) else {
                return Chain::Host;
            };
            chain.push(facts);
            match facts.super_class.as_deref() {
                None => return Chain::Known(chain),
                Some(base) => current = base,
            }
        }
    }

    pub(crate) fn host_derived(&self) -> BTreeSet<String> {
        self.classes
            .keys()
            .filter(|name| matches!(self.chain(name), Chain::Host))
            .cloned()
            .collect()
    }

    pub(crate) fn member_names(&self, name: &str) -> Option<BTreeSet<String>> {
        let Chain::Known(chain) = self.chain(name) else {
            return None;
        };
        if chain.iter().any(|facts| facts.has_computed_members) {
            return None;
        }
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
    let has_computed_members = class
        .get("body")
        .and_then(|body| body.get("has_computed_members"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
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
        has_computed_members,
    }
}

/// `binding` is the declarator name when `value` is a declarator's init, so
/// `const K = class …` is recorded as `K` (and under its own id, if any).
fn collect_from(
    value: &Value,
    binding: Option<&str>,
    out: &mut BTreeMap<String, ClassFacts>,
    ambiguous: &mut BTreeSet<String>,
) {
    let mut insert = |name: &str, facts: ClassFacts| {
        if out.insert(name.to_string(), facts).is_some() {
            ambiguous.insert(name.to_string());
        }
    };
    match value {
        Value::Object(map) => {
            if let Some(class) = map.get("ClassDeclaration") {
                if let Some(name) = class.get("name").and_then(Value::as_str) {
                    insert(name, class_facts(class));
                }
            }
            if let Some(class) = map.get("ClassExpression") {
                // `const K = class Foo …`: the program constructs it as `K`,
                // and `Foo` names it inside its own body. One class under two
                // names is not a duplicate (ruling R6 is about two classes).
                let id = class.get("id").and_then(Value::as_str);
                let mut names: Vec<&str> = id.into_iter().chain(binding).collect();
                names.dedup();
                for name in names {
                    insert(name, class_facts(class));
                }
            }
            // A declarator: `{ "id": "K", "init": … }`.
            let declarator_name = match (map.get("id"), map.get("init")) {
                (Some(Value::String(id)), Some(_)) => Some(id.as_str()),
                _ => None,
            };
            for (key, child) in map {
                let child_binding = if key == "init" { declarator_name } else { None };
                collect_from(child, child_binding, out, ambiguous);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_from(item, binding, out, ambiguous);
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
