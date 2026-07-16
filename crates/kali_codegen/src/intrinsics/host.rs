//! Host environment intrinsic call recognition and code emission (console, env, process, semver).
use crate::*;

pub(crate) fn semver_min_version(range: &str) -> Option<String> {
    let trimmed = range.trim();
    let candidate = trimmed
        .trim_start_matches(|c: char| {
            c.is_whitespace() || matches!(c, '^' | '~' | '=' | 'v' | '>' | '<')
        })
        .split(|c: char| c.is_whitespace() || c == ',' || c == '|')
        .next()?;
    Version::parse(candidate)
        .ok()
        .map(|version| version.to_string())
}

/// Stage D scheduling surfaces codegen emits real registrations for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SchedulingSurface {
    QueueMicrotask,
    SetTimeout,
    SetInterval,
    ClearTimeout,
    ClearInterval,
}

/// How a scheduling call's callback argument resolved (Stage D).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SchedulingCallback {
    /// Stable provenance to a compiled function: its raw wasm index.
    Resolved(u32),
    /// An identifier resolving to NOTHING in any codegen namespace — the
    /// flattened block-arrow lane (`Value("unknown")`). Kept on the
    /// pre-existing placeholder warning path until Task 7 (the un-flatten)
    /// deletes this variant; after that every arrow is a real function.
    LegacyPlaceholder,
    /// Everything else: unresolvable/unstable provenance — fail closed E5506.
    Deny,
}

impl<'a> FunctionEmitter<'a> {
    pub(crate) fn emit_coverage_hit(&mut self, function: &mut Function, coverage_id: Option<u32>) {
        if let Some(coverage_id) = coverage_id {
            function.instruction(&Instruction::I32Const(coverage_id as i32));
            function.instruction(&Instruction::Call(crate::COVERAGE_HIT_IMPORT_INDEX));
        }
    }

    pub(crate) fn console_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        let object = callee_node.children.first().copied()?;
        let object_name = self.node(object).text.as_deref()?;
        if object_name != "console" {
            return None;
        }

        match method {
            "log" => Some(crate::CONSOLE_LOG_IMPORT_INDEX),
            "error" => Some(crate::CONSOLE_ERROR_IMPORT_INDEX),
            "warn" => Some(crate::CONSOLE_WARN_IMPORT_INDEX),
            "info" => Some(crate::CONSOLE_INFO_IMPORT_INDEX),
            "debug" => Some(crate::CONSOLE_DEBUG_IMPORT_INDEX),
            _ => None,
        }
    }

    /// `Promise.resolve` (or `globalThis.Promise.resolve`) callee recognizer.
    /// `Promise.resolve(v)` synchronously settles to `v`; a bare `Promise.resolve()`
    /// settles to unit. Consumed by the await value-passthrough lane (Stage 3
    /// Task 4). Scoped to the exact `Promise` receiver so no other `.resolve`
    /// member call is affected.
    pub(crate) fn is_promise_resolve(&self, callee_node: &LirNode) -> bool {
        if callee_node.text.as_deref() != Some("resolve") {
            return false;
        }
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        self.is_promise_root(object)
    }

    /// The `Promise` global, spelled either as the bare `Promise` identifier or as
    /// `globalThis.Promise`.
    fn is_promise_root(&self, id: LirNodeId) -> bool {
        let node = self.node(id);
        if node.text.as_deref() != Some("Promise") {
            return false;
        }
        match node.children.first().copied() {
            None => true,
            Some(root) => self.node(root).text.as_deref() == Some("globalThis"),
        }
    }

    pub(crate) fn is_console_assert(&self, callee_node: &LirNode) -> bool {
        let Some(method) = callee_node.text.as_deref() else {
            return false;
        };
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        self.node(object).text.as_deref() == Some("console") && method == "assert"
    }

    pub(crate) fn env_set_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if method != "set" {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        let object_node = self.node(object);
        if object_node.text.as_deref() != Some("env") {
            return None;
        }

        let root = object_node.children.first().copied()?;
        if !self.is_deno_pid(root) {
            return None;
        }

        self.env_set_import_index
    }

    pub(crate) fn env_delete_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if method != "delete" {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        let object_node = self.node(object);
        if object_node.text.as_deref() != Some("env") {
            return None;
        }

        let root = object_node.children.first().copied()?;
        if !self.is_deno_pid(root) {
            return None;
        }

        self.env_delete_import_index
    }

    pub(crate) fn env_get_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if method != "get" {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        let object_node = self.node(object);
        if object_node.text.as_deref() != Some("env") {
            return None;
        }

        let root = object_node.children.first().copied()?;
        if !self.is_deno_pid(root) {
            return None;
        }

        self.env_get_import_index
    }

    pub(crate) fn env_has_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if method != "has" {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        let object_node = self.node(object);
        if object_node.text.as_deref() != Some("env") {
            return None;
        }

        let root = object_node.children.first().copied()?;
        if !self.is_deno_pid(root) {
            return None;
        }

        self.env_has_import_index
    }

    pub(crate) fn cwd_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if method != "cwd" {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        if !self.is_deno_pid(object) && !self.is_process_cwd(object) {
            return None;
        }

        Some(crate::CWD_IMPORT_INDEX)
    }

    pub(crate) fn cwd_set_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if method != "chdir" {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        if !self.is_deno_pid(object) && !self.is_process_cwd(object) {
            return None;
        }

        self.cwd_set_import_index
    }

    pub(crate) fn process_exit_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if method != "exit" {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        if !self.is_process_exit(object) && !self.is_deno_exit(object) {
            return None;
        }

        self.process_exit_import_index
    }

    pub(crate) fn performance_now_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if method != "now" {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        let object_node = self.node(object);
        if object_node.text.as_deref() != Some("performance") {
            return None;
        }

        self.performance_now_import_index
    }

    /// Recognize `crypto.getRandomValues(<buffer>)` (throw-fallout Stage 3 bucket
    /// #6): callee method text `"getRandomValues"`, object text `"crypto"`.
    /// Mirrors the `program_uses_crypto_get_random_values` probe and the
    /// kali_types admission arm (`resolve_crypto_call`).
    pub(crate) fn crypto_get_random_values_import_index(
        &self,
        callee_node: &LirNode,
    ) -> Option<u32> {
        if callee_node.text.as_deref()? != "getRandomValues" {
            return None;
        }
        let object = callee_node.children.first().copied()?;
        if self.node(object).text.as_deref() != Some("crypto") {
            return None;
        }
        self.crypto_get_random_values_import_index
    }

    /// Recognize `crypto.randomUUID()` (throw-fallout Stage 3 bucket #6): callee
    /// method text `"randomUUID"`, object text `"crypto"`.
    pub(crate) fn crypto_random_uuid_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        if callee_node.text.as_deref()? != "randomUUID" {
            return None;
        }
        let object = callee_node.children.first().copied()?;
        if self.node(object).text.as_deref() != Some("crypto") {
            return None;
        }
        self.crypto_random_uuid_import_index
    }

    /// Recognize `crypto.subtle.digest(algo, bytes)` (throw-fallout Stage 3 bucket
    /// #6 part 2): callee method text `"digest"`, object text `"subtle"`,
    /// grand-object text `"crypto"`. Mirrors the `program_uses_crypto_subtle_digest`
    /// probe and the kali_types admission arm (`resolve_crypto_call`).
    pub(crate) fn crypto_subtle_digest_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        if callee_node.text.as_deref()? != "digest" {
            return None;
        }
        let subtle = callee_node.children.first().copied()?;
        let subtle_node = self.node(subtle);
        if subtle_node.text.as_deref() != Some("subtle") {
            return None;
        }
        let crypto = subtle_node.children.first().copied()?;
        if self.node(crypto).text.as_deref() != Some("crypto") {
            return None;
        }
        self.crypto_subtle_digest_import_index
    }

    /// Recognize `new TextEncoder().encode(<string>)` (throw-fallout Stage 3 bucket
    /// #6 part 2): callee method text `"encode"` whose object is a
    /// `new TextEncoder()` construction (a `Call` node whose own callee text is
    /// `"TextEncoder"`). A pure GUEST-SIDE reinterpret — no host import — so this
    /// returns a bool rather than an import index. Mirrors the raw-node arm in
    /// `declarator_init_is_crypto_call` and the kali_types admission arm.
    pub(crate) fn is_text_encoder_encode(&self, callee_node: &LirNode) -> bool {
        if callee_node.text.as_deref() != Some("encode") {
            return false;
        }
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        let object_node = self.node(object);
        if object_node.kind != LirNodeKind::Call {
            return false;
        }
        object_node
            .children
            .first()
            .map(|&ctor| self.node(ctor).text.as_deref() == Some("TextEncoder"))
            .unwrap_or(false)
    }

    pub(crate) fn render_console_call(&self, node: &LirNode) -> Option<String> {
        let args = node.children.iter().skip(1).copied().collect::<Vec<_>>();
        self.render_console_arguments(&args)
    }

    pub(crate) fn render_console_arguments(&self, args: &[LirNodeId]) -> Option<String> {
        let mut rendered = Vec::new();
        for arg in args {
            rendered.push(self.render_static_value(*arg)?);
        }
        Some(rendered.join(" "))
    }

    pub(crate) fn render_static_value(&self, id: LirNodeId) -> Option<String> {
        let node = self.node(id);
        match node.kind {
            LirNodeKind::Literal => match node.text.as_deref() {
                Some("true") => Some("true".to_string()),
                Some("false") => Some("false".to_string()),
                Some("null") => Some("null".to_string()),
                Some("undefined") => Some("undefined".to_string()),
                Some(text) => {
                    if parse_number_literal(text).is_some() {
                        Some(text.to_string())
                    } else {
                        Some(strip_string_delimiters(text).to_string())
                    }
                }
                None => Some("0".to_string()),
            },
            LirNodeKind::Call => {
                if self.is_object_freeze_call(node) {
                    return self.render_static_value(*node.children.get(1)?);
                }

                if let Some(result) = self.resolve_static_array_at_call(node) {
                    return match result {
                        StaticArrayAtResult::Value(value) => self.render_static_value(value),
                        StaticArrayAtResult::OutOfRange => Some("undefined".to_string()),
                    };
                }

                if let Some(result) = self.resolve_static_string_at_call(node) {
                    return match result {
                        StaticStringAtResult::Value(value) => Some(value),
                        StaticStringAtResult::OutOfRange => Some("undefined".to_string()),
                    };
                }

                if let Some(result) = self.resolve_static_string_code_point_at_call(node) {
                    return match result {
                        StaticStringAtResult::Value(value) => Some(value),
                        StaticStringAtResult::OutOfRange => Some("undefined".to_string()),
                    };
                }

                let callee = node.children.first().copied()?;
                let callee_node = self.node(callee);
                let callee_name = callee_node.text.as_deref()?;
                if callee_name == "require" {
                    if let Some(specifier) = self.render_static_value(*node.children.get(1)?) {
                        if let Some(version) = self.render_package_json_version(&specifier) {
                            return Some(version);
                        }
                    }
                }
                self.render_semver_intrinsic(callee_name, node)
            }
            LirNodeKind::Value => {
                if node.children.is_empty() {
                    let text = node.text.as_deref()?;
                    if let Some(bound) = self.bindings.get(text).copied() {
                        return self.render_static_value(bound);
                    }
                    if self.locals.contains_key(text) {
                        // Mutable `let`/`var` locals are runtime values, not statically
                        // known; bail out so the caller falls back to dynamic emission
                        // (`emit_node` -> `LocalGet`) instead of baking in a wrong constant.
                        return None;
                    }
                    if let Some(number) = parse_number_literal(text) {
                        return Some(number.to_string());
                    }
                    if parse_numeric_literal_value(text).is_some() {
                        return Some(text.to_string());
                    }
                    match text {
                        "true" | "false" | "null" | "undefined" => Some(text.to_string()),
                        _ => None,
                    }
                } else if node.children.len() == 1
                    && matches!(node.text.as_deref(), Some("+") | Some("-"))
                {
                    let rendered = self.render_static_value(node.children[0])?;
                    if let Some(value) = parse_number_literal(&rendered) {
                        Some(if node.text.as_deref() == Some("-") {
                            (-value).to_string()
                        } else {
                            value.to_string()
                        })
                    } else {
                        let value = parse_numeric_literal_value(&rendered)?;
                        Some(if node.text.as_deref() == Some("-") {
                            (-value).to_string()
                        } else {
                            value.to_string()
                        })
                    }
                } else if node.text.as_deref().is_some_and(|text| text == "length") {
                    if self.is_process_argv(node.children[0]) {
                        None
                    } else {
                        self.render_length(&node.children[0])
                    }
                } else if let Some(result) = self.resolve_static_index_member(node) {
                    match result {
                        StaticIndexMemberResult::Node(value) => self.render_static_value(value),
                        StaticIndexMemberResult::String(value) => Some(value),
                        StaticIndexMemberResult::Undefined => Some("undefined".to_string()),
                    }
                } else if node
                    .text
                    .as_deref()
                    // A text-less wrapper renders as its child (1 child) or the
                    // aggregate element count. The `"await"` marker (Stage 3
                    // Task 4) is a synchronously-settled passthrough — it always
                    // wraps a single operand, so it tunnels to that child for
                    // static rendering (e.g. `Math.atan2(await 0, await 1)`).
                    .is_none_or(|text| text.is_empty() || text == "await")
                {
                    if node.children.len() == 1 {
                        self.render_static_value(node.children[0])
                    } else {
                        Some(node.children.len().to_string())
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(crate) fn has_semver_import(&self) -> bool {
        self.program
            .nodes
            .iter()
            .any(|node| node.text.as_deref() == Some("semver"))
    }

    pub(crate) fn render_semver_intrinsic(
        &self,
        callee_name: &str,
        node: &LirNode,
    ) -> Option<String> {
        if !self.has_semver_import() {
            return None;
        }

        match callee_name {
            "valid" => {
                let arg = *node.children.get(1)?;
                let version = self.render_static_value(arg)?;
                Version::parse(&version)
                    .ok()
                    .map(|parsed| parsed.to_string())
            }
            "satisfies" => {
                let version = self.render_static_value(*node.children.get(1)?)?;
                let range = self.render_static_value(*node.children.get(2)?)?;
                let version = Version::parse(&version).ok()?;
                let range = VersionReq::parse(&range).ok()?;
                Some(if range.matches(&version) { "1" } else { "0" }.to_string())
            }
            "minVersion" => {
                let range = self.render_static_value(*node.children.get(1)?)?;
                semver_min_version(&range)
            }
            _ => None,
        }
    }

    pub(crate) fn render_package_json_version(&self, specifier: &str) -> Option<String> {
        let source_path = self.source_path.as_ref()?;
        let package_json_path = source_path
            .parent()?
            .join(strip_string_delimiters(specifier));
        if package_json_path.file_name().and_then(|name| name.to_str()) != Some("package.json") {
            return None;
        }

        let raw = std::fs::read_to_string(package_json_path).ok()?;
        let package_json: serde_json::Value = serde_json::from_str(&raw).ok()?;
        package_json
            .get("version")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string())
    }

    pub(crate) fn render_package_json_version_access(&self, id: LirNodeId) -> Option<String> {
        let node = self.node(id);
        if node.kind != LirNodeKind::Call {
            return None;
        }

        let callee = node.children.first().copied()?;
        if self.node(callee).text.as_deref() != Some("require") {
            return None;
        }

        let specifier = self.render_static_value(*node.children.get(1)?)?;
        self.render_package_json_version(&specifier)
    }

    pub(crate) fn is_deno_pid(&self, id: LirNodeId) -> bool {
        let node = self.node(id);
        if node.text.as_deref() == Some("Deno") {
            return true;
        }

        node.text.as_deref() == Some("globalThis")
            && node
                .children
                .first()
                .is_some_and(|child| self.node(*child).text.as_deref() == Some("Deno"))
    }

    pub(crate) fn is_deno_exit(&self, id: LirNodeId) -> bool {
        self.is_deno_pid(id)
    }

    pub(crate) fn is_process_pid(&self, id: LirNodeId) -> bool {
        let node = self.node(id);
        if node.text.as_deref() == Some("process") {
            return true;
        }

        node.text.as_deref() == Some("globalThis")
            && node
                .children
                .first()
                .is_some_and(|child| self.node(*child).text.as_deref() == Some("process"))
    }

    pub(crate) fn is_process_cwd(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent_value_node(id);
        let node = self.node(id);
        if node.text.as_deref() == Some("process") {
            return true;
        }

        node.text.as_deref() == Some("globalThis")
            && node
                .children
                .first()
                .is_some_and(|child| self.node(*child).text.as_deref() == Some("process"))
    }

    pub(crate) fn is_process_exit(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent_value_node(id);
        let node = self.node(id);
        if self.is_object_freeze_call(node) {
            return node
                .children
                .get(1)
                .copied()
                .is_some_and(|child| self.is_process_exit(child));
        }

        if node.text.as_deref() == Some("process") {
            return true;
        }

        node.text.as_deref() == Some("globalThis")
            && node
                .children
                .first()
                .is_some_and(|child| self.node(*child).text.as_deref() == Some("process"))
    }

    pub(crate) fn is_process_kill(&self, callee_node: &LirNode) -> bool {
        let Some(method) = callee_node.text.as_deref() else {
            return false;
        };
        if method != "kill" {
            return false;
        }

        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        if !self.is_process_exit(object) {
            return false;
        }

        true
    }

    pub(crate) fn is_process_argv(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent_value_node(id);
        let node = self.node(id);
        if node.text.as_deref() != Some("argv") || node.children.len() != 1 {
            return false;
        }

        let object = self.unwrap_transparent_value_node(node.children[0]);
        let object = self.node(object);
        if object.text.as_deref() == Some("process") {
            return true;
        }

        object.text.as_deref() == Some("globalThis")
            && object
                .children
                .first()
                .is_some_and(|child| self.node(*child).text.as_deref() == Some("process"))
    }

    /// `process.argv[<int literal>]` — a computed element read on the argv
    /// receiver. Returns the static index. CONFIRMED LIR shape (Spec 5 Task 5,
    /// verified with a `dbg!` probe): a TWO-child `Value` node `[object, index]`
    /// (`object` = the `process.argv` receiver, `index` = the index node),
    /// with the stringified index ALSO carried in `node.text` — a computed
    /// bracket read, distinct from the one-child dot-access shape (e.g.
    /// `process.argv.length`). Only a static non-negative integer-literal index
    /// is supported; anything else (negative, non-literal, non-integer) fails
    /// closed (falls through to the placeholder, which the caller must not treat
    /// as a string).
    pub(crate) fn is_process_argv_element(&self, node: &LirNode) -> Option<i64> {
        if node.children.len() != 2 {
            return None;
        }
        if is_binary_operator_text(node.text.as_deref().unwrap_or_default()) {
            return None;
        }
        if !self.is_process_argv(node.children[0]) {
            return None;
        }
        let index = parse_number_literal(self.node(node.children[1]).text.as_deref()?)?;
        (index >= 0).then_some(index)
    }

    pub(crate) fn is_deno_args(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent_value_node(id);
        let node = self.node(id);
        if node.text.as_deref() != Some("args") || node.children.len() != 1 {
            return false;
        }

        let object = self.unwrap_transparent_value_node(node.children[0]);
        let object = self.node(object);
        if object.text.as_deref() == Some("Deno") {
            return true;
        }

        object.text.as_deref() == Some("globalThis")
            && object
                .children
                .first()
                .is_some_and(|child| self.node(*child).text.as_deref() == Some("Deno"))
    }

    pub(crate) fn process_argv_slice_start(&self, id: LirNodeId) -> Option<i64> {
        let id = self.resolve_bound_node(id);
        let node = self.node(id);
        if node.kind != LirNodeKind::Call {
            return None;
        }

        let callee = node.children.first().copied()?;
        let callee = self.resolve_transparent_callable_node(callee)?;
        let callee_node = self.node(callee);
        if callee_node.text.as_deref() != Some("slice") {
            return None;
        }

        let object = callee_node.children.first().copied()?;
        if !(self.is_process_argv(object) || self.is_deno_args(object)) {
            return None;
        }

        let start = *node.children.get(1)?;
        let start_node = self.node(start);
        parse_number_literal(start_node.text.as_deref()?)
    }

    pub(crate) fn render_length(&self, id: &LirNodeId) -> Option<String> {
        if self.process_argv_slice_start(*id).is_some() {
            return None;
        }

        if let Some(parts) = self.resolve_static_string_split_parts_from_id(*id) {
            return Some(parts.len().to_string());
        }

        // Array literal (inline, via a `const` binding, or through transparent
        // wrappers): `[x].length` is the ELEMENT COUNT, not the string length of
        // a lone element `x`. This MUST precede the string-identity fold below —
        // that fold tunnels a single-element array `[x]` straight into element
        // `x` and would report `x`'s UTF-16 length (e.g. `["abcdef"].length` →
        // 6, or the folded `Object.keys(singleKeyObject).length` → the key's
        // length) instead of 1. Placing the carve-out in this `.length` consumer
        // (rather than in the shared `unwrap_transparent` /
        // `resolve_static_object_identity_value` helpers) keeps every other
        // consumer's legitimate one-child-wrapper tunneling intact
        // (throw-fallout Stage 2 checkpoint regression fix).
        if let Some(aggregate_id) = self.resolve_literal_aggregate(*id) {
            let aggregate = self.node(aggregate_id);
            if self.is_array_literal(aggregate) {
                return Some(aggregate.children.len().to_string());
            }
        }

        if let Some(StaticObjectIdentityValue::String(value)) =
            self.resolve_static_object_identity_value(*id)
        {
            return Some(value.encode_utf16().count().to_string());
        }

        if self.is_string_valued(*id) {
            // Runtime string receiver (a `let` string, string param, substring
            // result, …) not caught by the static-identity fold above: its
            // length lives in the handle at runtime. Defer to dynamic emission
            // (control_flow.rs's string-length arm) instead of falling through
            // to the identifier branch below, which would bake in a wrong
            // static `0`.
            return None;
        }

        let node = self.node(*id);
        if node.text.is_none() {
            return Some(node.children.len().to_string());
        }

        if node.children.is_empty() {
            if let Some(text) = node.text.as_deref() {
                if let Some(bound) = self.bindings.get(text).copied() {
                    return self.render_length(&bound);
                }
                if self.array_bindings.contains(text) || self.is_growable_array(text) {
                    // Runtime (plain or growable) array: the length isn't
                    // statically known; defer to dynamic emission (the
                    // respective `.length` header-load lane) instead of
                    // baking in a wrong constant.
                    return None;
                }
                return Some("0".to_string());
            }
        }

        if node.children.len() == 1 {
            self.render_length(&node.children[0])
        } else {
            Some(node.children.len().to_string())
        }
    }

    pub(crate) fn is_kali_test_call(&self, callee_node: &LirNode) -> bool {
        if callee_node.text.as_deref() != Some("test") {
            return false;
        }

        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        self.node(object).text.as_deref() == Some("Kali")
    }

    pub(crate) fn kali_test_callback_index(&self, node: &LirNode) -> Option<u32> {
        let callback_node = node.children.get(2).copied()?;
        let callback_name = self.node(callback_node).text.as_deref()?;
        self.functions.get(callback_name).copied()
    }

    /// Recognize a bare, UNSHADOWED global scheduling callee (Stage D
    /// provenance rule: "bare unshadowed global callee only"). Any user
    /// binding, local, or function of the same name shadows the global and
    /// the call takes the normal user-call lane.
    pub(crate) fn scheduling_surface(&self, callee_node: &LirNode) -> Option<SchedulingSurface> {
        if !callee_node.children.is_empty() {
            return None;
        }
        let name = callee_node.text.as_deref()?;
        let surface = match name {
            "queueMicrotask" => SchedulingSurface::QueueMicrotask,
            "setTimeout" => SchedulingSurface::SetTimeout,
            "setInterval" => SchedulingSurface::SetInterval,
            "clearTimeout" => SchedulingSurface::ClearTimeout,
            "clearInterval" => SchedulingSurface::ClearInterval,
            _ => return None,
        };
        if self.locals.contains_key(name)
            || self.bindings.contains_key(name)
            || self.module_binding_names.contains(name)
            || self.fn_valued_locals.contains_key(name)
            || self.functions.contains_key(name)
        {
            return None;
        }
        Some(surface)
    }

    /// Resolve a scheduling call's callback argument (`children[1]`) by
    /// STABLE provenance — the same rules as the Stage C
    /// `scheduling_call_args_provably_safe` guard, but yielding the function
    /// index for the registration emit. Capturing callbacks resolve too:
    /// their soundness is `env_safety`'s job (registration edges), not this
    /// resolver's.
    pub(crate) fn scheduling_callback(&self, node: &LirNode) -> SchedulingCallback {
        let Some(&cb) = node.children.get(1) else {
            return SchedulingCallback::Deny;
        };
        let cb = self.unwrap_transparent(cb);
        let cb_node = self.node(cb);
        let Some(text) = cb_node.text.as_deref() else {
            return SchedulingCallback::Deny;
        };
        match cb_node.kind {
            // Inline function expression/declaration lowered as a plan: its
            // node text is the `__kali_fn_N` / declared plan key.
            LirNodeKind::Instruction => match self.functions.get(text) {
                Some(&index) => SchedulingCallback::Resolved(index),
                None => SchedulingCallback::Deny,
            },
            LirNodeKind::Value if cb_node.children.is_empty() => {
                if self.unstable_provenance_names.contains(text) {
                    return SchedulingCallback::Deny;
                }
                if let Some(key) = self.fn_valued_locals.get(text) {
                    return match self.functions.get(key) {
                        Some(&index) => SchedulingCallback::Resolved(index),
                        None => SchedulingCallback::Deny,
                    };
                }
                if self.locals.contains_key(text)
                    || self.bindings.contains_key(text)
                    || self.module_binding_names.contains(text)
                {
                    // A live binding without function provenance: unknown value.
                    return SchedulingCallback::Deny;
                }
                if let Some(&index) = self.functions.get(text) {
                    // Bare unshadowed function name.
                    return SchedulingCallback::Resolved(index);
                }
                SchedulingCallback::LegacyPlaceholder
            }
            _ => SchedulingCallback::Deny,
        }
    }

    pub(crate) fn is_kali_write_stdout_bytes_call(&self, callee_node: &LirNode) -> bool {
        if callee_node.text.as_deref() != Some("writeStdoutBytes") {
            return false;
        }
        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        self.node(object).text.as_deref() == Some("Kali")
    }
}

#[cfg(test)]
#[path = "host_tests.rs"]
mod host_tests;
