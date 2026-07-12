use crate::*;

impl Optimizer {
    pub(crate) fn clone_boolean_literal(&self, program: &mut LirProgram, value: bool) -> LirNodeId {
        let node = LirNode {
            kind: LirNodeKind::Literal,
            text: Some(value.to_string()),
            children: Vec::new(),
            function_flavor: None,
        };
        let new_id = LirNodeId(program.nodes.len() as u32);
        program.nodes.push(node);
        new_id
    }

    pub(crate) fn member_access_name(
        &self,
        program: &LirProgram,
        node: &LirNode,
    ) -> Option<String> {
        if self.is_object_freeze_call(program, node) {
            let inner = node.children.get(1).copied()?;
            let inner = program.nodes.get(inner.0 as usize)?;
            return self.member_access_name(program, inner);
        }

        if let Some(name) = self.callable_selection_member_access_name(program, node) {
            return Some(name);
        }

        let object = node.children.first().copied()?;
        let object = program.nodes.get(object.0 as usize)?;
        let object_name = match object.text.as_deref() {
            Some(text) => text.to_string(),
            None => self.member_access_name(program, object)?,
        };

        Some(format!("{}.{}", object_name, node.text.as_deref()?))
    }

    /// Resolve a short-circuit (`??`/`&&`/`||`) or ternary (`?`) callable
    /// SELECTION to the member-access name of the callable it statically
    /// selects. LIR mirror of the shared static callable oracle
    /// (`kali_types::resolve_static_callable_name` /
    /// `kali_codegen::resolve_transparent_callable_node`) with the identical
    /// discipline: a literal condition picks the winning branch; an unknown
    /// condition resolves ONLY when both branches name the SAME target
    /// (compared canonicalized). A selection between two DIFFERENT callables
    /// stays unresolved — fail-closed, so the enumeration fold declines and
    /// the call routes to codegen's E5506 backstop instead of ever folding
    /// the wrong operator.
    fn callable_selection_member_access_name(
        &self,
        program: &LirProgram,
        node: &LirNode,
    ) -> Option<String> {
        if node.kind != LirNodeKind::Value {
            return None;
        }

        let branch_name = |id: LirNodeId| -> Option<String> {
            let branch = program.nodes.get(id.0 as usize)?;
            self.member_access_name(program, branch)
        };
        let both_branches_same_target = |left: LirNodeId, right: LirNodeId| -> Option<String> {
            let left_name = branch_name(left)?;
            let right_name = branch_name(right)?;
            let canonical = |name: &str| {
                Self::canonicalize_optional_chain_member_access_name(
                    &Self::canonicalize_bracketed_member_access_name(name),
                )
            };
            if canonical(&left_name) == canonical(&right_name) {
                Some(left_name)
            } else {
                None
            }
        };

        match (node.text.as_deref(), node.children.as_slice()) {
            (Some("??"), &[left, right]) => match literal_value(program, left) {
                Some(ConstantValue::Null | ConstantValue::Undefined) => branch_name(right),
                Some(_) => branch_name(left),
                None => both_branches_same_target(left, right),
            },
            (Some("&&"), &[left, right]) => {
                match literal_value(program, left).map(ConstantValue::truthy) {
                    Some(true) => branch_name(right),
                    Some(false) => branch_name(left),
                    None => both_branches_same_target(left, right),
                }
            }
            (Some("||"), &[left, right]) => {
                match literal_value(program, left).map(ConstantValue::truthy) {
                    Some(true) => branch_name(left),
                    Some(false) => branch_name(right),
                    None => both_branches_same_target(left, right),
                }
            }
            // Ternary marker "?" (see the HIR ConditionalExpr lowering note):
            // children are [test, consequent, alternate].
            (Some("?"), &[test, consequent, alternate]) => {
                match literal_value(program, test).map(ConstantValue::truthy) {
                    Some(true) => branch_name(consequent),
                    Some(false) => branch_name(alternate),
                    None => both_branches_same_target(consequent, alternate),
                }
            }
            _ => None,
        }
    }

    pub(crate) fn normalized_member_access_name(
        &self,
        program: &LirProgram,
        node: &LirNode,
    ) -> Option<String> {
        let raw = self.member_access_name(program, node)?;
        Some(Self::canonicalize_optional_chain_member_access_name(
            &Self::canonicalize_bracketed_member_access_name(&raw),
        ))
    }

    pub(crate) fn canonicalize_optional_chain_member_access_name(name: &str) -> String {
        name.replace("globalThis?.", "globalThis.")
    }

    pub(crate) fn canonicalize_bracketed_member_access_name(name: &str) -> String {
        let mut canonical = String::with_capacity(name.len());
        let bytes = name.as_bytes();
        let mut index = 0;

        while index < bytes.len() {
            if bytes[index] == b'[' && index + 2 < bytes.len() {
                let quote = bytes[index + 1];
                if quote == b'"' || quote == b'\'' {
                    let mut end = index + 2;
                    while end < bytes.len() && bytes[end] != quote {
                        end += 1;
                    }

                    if end + 1 < bytes.len() && bytes[end + 1] == b']' {
                        if !canonical.is_empty() && !canonical.ends_with('.') {
                            canonical.push('.');
                        }
                        canonical.push_str(&name[index + 2..end]);
                        index = end + 2;
                        continue;
                    }
                }
            }

            canonical.push(bytes[index] as char);
            index += 1;
        }

        canonical
    }

    pub(crate) fn constant_property_key(
        &self,
        program: &LirProgram,
        id: LirNodeId,
    ) -> Option<String> {
        Some(literal_text(literal_value(program, id)?))
    }

    pub(crate) fn clone_string_literal(&self, program: &mut LirProgram, text: String) -> LirNodeId {
        let node = LirNode {
            kind: LirNodeKind::Literal,
            text: Some(text),
            children: Vec::new(),
            function_flavor: None,
        };
        let new_id = LirNodeId(program.nodes.len() as u32);
        program.nodes.push(node);
        new_id
    }

    pub(crate) fn push_array_literal(
        &self,
        program: &mut LirProgram,
        elements: Vec<LirNodeId>,
    ) -> LirNodeId {
        let new_id = LirNodeId(program.nodes.len() as u32);
        program.nodes.push(LirNode {
            kind: LirNodeKind::Value,
            text: None,
            children: elements,
            function_flavor: None,
        });
        new_id
    }

    pub(crate) fn push_object_literal(
        &self,
        program: &mut LirProgram,
        properties: Vec<(String, LirNodeId)>,
    ) -> LirNodeId {
        let mut property_nodes = Vec::with_capacity(properties.len());
        for (key, value) in properties {
            let key_id = self.clone_string_literal(program, key);
            let property_id = LirNodeId(program.nodes.len() as u32);
            program.nodes.push(LirNode {
                kind: LirNodeKind::Value,
                text: Some("init".to_string()),
                children: vec![key_id, value],
                function_flavor: None,
            });
            property_nodes.push(property_id);
        }

        let new_id = LirNodeId(program.nodes.len() as u32);
        program.nodes.push(LirNode {
            kind: LirNodeKind::Value,
            text: None,
            children: property_nodes,
            function_flavor: None,
        });
        new_id
    }

    pub(crate) fn object_property_order_key(key: &str) -> Option<u64> {
        // Single source of truth (throw-fallout Stage 2, Lane B).
        kali_common::property_order_key(key)
    }
}

pub(crate) fn node_signature(program: &LirProgram, id: LirNodeId) -> String {
    let Some(node) = program.nodes.get(id.0 as usize) else {
        return "<missing>".to_string();
    };

    let mut signature = format!("{:?}:{:?}", node.kind, node.text);
    if !node.children.is_empty() {
        signature.push('(');
        for child in &node.children {
            signature.push_str(&node_signature(program, *child));
            signature.push(',');
        }
        signature.push(')');
    }
    signature
}

pub(crate) fn literal_signature(prefix: &str, kind: LirNodeKind, text: Option<&str>) -> String {
    match parse_literal_text(text) {
        Some(ConstantValue::Number(value)) => format!(
            "{prefix}:number:{}",
            text.map_or_else(|| value.to_string(), str::to_owned)
        ),
        Some(ConstantValue::BigInt(value)) => format!("{prefix}:bigint:{value}"),
        Some(ConstantValue::Boolean(value)) => {
            format!("{prefix}:boolean:{value}")
        }
        Some(ConstantValue::String(_)) => text
            .and_then(|text| string_literal_signature(prefix, text))
            .unwrap_or_else(|| format!("{prefix}:string:<missing>")),
        Some(ConstantValue::RegExp { pattern, flags }) => {
            format!("{prefix}:regexp:pattern={pattern}:flags={flags}")
        }
        Some(ConstantValue::Null) => format!("{prefix}:null"),
        Some(ConstantValue::Undefined) => format!("{prefix}:undefined"),
        Some(ConstantValue::NegativeZero) => format!("{prefix}:number:-0"),
        Some(ConstantValue::Infinity) => format!("{prefix}:number:Infinity"),
        Some(ConstantValue::NegativeInfinity) => format!("{prefix}:number:-Infinity"),
        Some(ConstantValue::NaN) => format!("{prefix}:number:NaN"),
        None => format!("{:?}:{:?}", kind, text),
    }
}

pub(crate) fn string_literal_signature(prefix: &str, text: &str) -> Option<String> {
    let value = parse_string_literal(text)?;
    let literal_kind = if text.starts_with('`') {
        "template"
    } else {
        "quoted"
    };
    Some(format!("{prefix}:string:{literal_kind}:{value}"))
}
