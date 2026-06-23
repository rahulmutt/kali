//! Type system and name-resolution infrastructure for TypeScript/JavaScript.
//!
//! Stage 1.4 focuses on the deterministic scope model and name resolver that
//! downstream compiler stages use to catch unresolved names and duplicate
//! bindings before lowering.

mod builtins;
mod context;
mod late_host;
mod package;
mod resolve;
mod scope;
mod typecheck;
mod static_analysis;

use builtins::*;
use package::*;
pub use context::{ResolutionResult, TypeContext};
pub use scope::{Scope, ScopeRef, ScopeType};
pub use typecheck::TypeChecker;
use typecheck::*;

use indexmap::IndexMap;
use kali_ast::{
    ArrayExpression, ArrowFunctionExpression, AssignmentExpression, AssignmentOperator,
    BlockStatement, BreakStatement, CallExpression, CatchClause, ClassBody, ClassDeclaration,
    ClassExpression, ContinueStatement, DecoratedExpression, DoWhileStatement, EnumDeclaration,
    EnumMember, ExportAllDeclaration, Expression, ExpressionOrSpread, ExpressionStatement,
    ForInLefthand, ForInStatement, ForInit, ForOfLefthand, ForOfStatement, ForStatement,
    FunctionDeclaration, FunctionExpression, FunctionParam, IfStatement, ImportDeclaration,
    ImportExpression, ImportSpecifier, InterfaceDeclaration, JsxChild, JsxElement, JsxFragment,
    LabeledStatement, LiteralValue, LogicalOperator, MemberExpression, NewExpression, NodeId,
    ObjectExpression, ObjectProperty, ObjectPropertyKind, OptionalChainExpression,
    OptionalChainInner, PropertyName, ReturnStatement, Statement, SwitchCase, SwitchStatement,
    TemplateLiteral, ThrowStatement, TryStatement, TypeAliasDeclaration, TypeAssertion,
    UpdateExpression, VariableDeclaration, WhileStatement, WithStatement,
};
use kali_common::{
    generator_class_method_yield_lowering_unavailable_message_for_flavors,
    generator_function_lowering_unavailable_message_for_flavors,
    generator_function_yield_lowering_unavailable_message,
    late_process_control_single_quoted_exit_aliases,
    late_process_control_single_quoted_kill_aliases, process_kill_zero_probe_wrapped_zero_aliases,
    template::resolve_interpolated_template_literal,
};
use kali_error::{
    _error_codes::e3, _error_codes::e4, _error_codes::e5, _error_codes::e6, diagnostic::Diagnostic,
};
use kali_lexer::Lexer;
use kali_parser::Parser;
use std::{
    fs,
    path::{Path, PathBuf},
};


#[cfg(test)]
#[path = "test_support.rs"]
mod test_support;
