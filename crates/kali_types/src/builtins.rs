//! Built-in globals, Node.js specifiers, and binding helpers.

use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum StaticObjectIdentityValue {
    Boolean(bool),
    Number(f64),
    String(String),
    BigInt(i64),
    Null,
    Undefined,
    Reference(String),
}

impl StaticObjectIdentityValue {
    pub(crate) fn same_value(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(left), Self::Boolean(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::BigInt(left), Self::BigInt(right)) => left == right,
            (Self::Null, Self::Null) | (Self::Undefined, Self::Undefined) => true,
            (Self::Number(left), Self::Number(right)) => {
                (left.is_nan() && right.is_nan())
                    || (left == right
                        && (left != &0.0 || left.is_sign_positive() == right.is_sign_positive()))
            }
            (Self::Reference(left), Self::Reference(right)) => left == right,
            _ => false,
        }
    }

    pub(crate) fn is_nullish(&self) -> bool {
        matches!(self, Self::Null | Self::Undefined)
    }

    pub(crate) fn truthiness(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            Self::Number(value) => Some(!value.is_nan() && *value != 0.0),
            Self::String(value) => Some(!value.is_empty()),
            Self::BigInt(value) => Some(*value != 0),
            Self::Null | Self::Undefined => Some(false),
            Self::Reference(_) => None,
        }
    }
}

pub(crate) fn builtin_globals() -> &'static [&'static str] {
    &[
        "AbortController",
        "AbortSignal",
        "Array",
        "Blob",
        "Boolean",
        "atob",
        "btoa",
        "BroadcastChannel",
        "clearInterval",
        "clearTimeout",
        "console",
        "CustomEvent",
        "Date",
        "Deno",
        "decodeURI",
        "decodeURIComponent",
        "encodeURI",
        "encodeURIComponent",
        "Error",
        "eval",
        "File",
        "FileReader",
        "FormData",
        "Event",
        "EventTarget",
        "WebSocket",
        "Worker",
        "indexedDB",
        "localStorage",
        "sessionStorage",
        "fetch",
        "Function",
        "globalThis",
        "Headers",
        "Infinity",
        "Intl",
        "isFinite",
        "isNaN",
        "JSON",
        "Kali",
        "Map",
        "Math",
        "NaN",
        "Object",
        "navigator",
        "parseFloat",
        "parseInt",
        "performance",
        "Promise",
        "Proxy",
        "queueMicrotask",
        "Reflect",
        "RegExp",
        "Request",
        "ReadableStream",
        "Response",
        "Set",
        "setInterval",
        "setTimeout",
        "String",
        "structuredClone",
        "Symbol",
        "TextDecoder",
        "TextEncoder",
        "TransformStream",
        "URL",
        "URLSearchParams",
        "WeakMap",
        "WeakSet",
        "WritableStream",
        "abs",
        "crypto",
    ]
}

pub(crate) fn node_builtin_globals() -> &'static [&'static str] {
    &["Buffer", "exports", "module", "process", "require"]
}

pub(crate) fn node_builtin_specifiers() -> &'static [&'static str] {
    &[
        "assert",
        "buffer",
        "child_process",
        "crypto",
        "events",
        "fs",
        "fs/promises",
        "http",
        "https",
        "os",
        "path",
        "process",
        "stream",
        "timers",
        "url",
        "util",
    ]
}

pub(crate) fn is_node_builtin_specifier(source: &str) -> bool {
    let normalized = source.strip_prefix("node:").unwrap_or(source);
    node_builtin_specifiers().contains(&normalized)
}

pub(crate) fn bind_builtin(scope: &mut Scope, next_binding_id: &mut u32, name: &str) {
    if scope.contains(name) {
        return;
    }

    scope.bind(name, NodeId::new(*next_binding_id));
    *next_binding_id = next_binding_id
        .checked_add(1)
        .expect("binding id overflow is unreachable in stage 1");
}

pub(crate) fn duplicate_binding(name: &str) -> Diagnostic {
    Diagnostic::error(
        e3::DUPLICATE_BINDING as u32,
        format!("duplicate binding '{}'", name),
    )
    .with_suggestion("rename the binding or move it into a nested scope")
}
