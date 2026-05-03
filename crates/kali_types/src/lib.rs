//! Type system and name-resolution infrastructure for TypeScript/JavaScript.
//!
//! Stage 1.4 focuses on the deterministic scope model and name resolver that
//! downstream compiler stages use to catch unresolved names and duplicate
//! bindings before lowering.

use indexmap::IndexMap;
use kali_ast::{
    ArrayExpression, ArrowFunctionExpression, AssignmentExpression, AssignmentOperator,
    BlockStatement, BreakStatement, CallExpression, CatchClause, ClassBody, ClassDeclaration,
    ClassExpression, ContinueStatement, DecoratedExpression, DoWhileStatement, EnumDeclaration,
    EnumMember, Expression, ExpressionOrSpread, ExpressionStatement, ForInLefthand, ForInStatement,
    ForInit, ForOfLefthand, ForOfStatement, ForStatement, FunctionDeclaration, FunctionExpression,
    FunctionParam, IfStatement, ImportDeclaration, ImportExpression, ImportSpecifier,
    InterfaceDeclaration, JsxChild, JsxElement, JsxFragment, LabeledStatement, LiteralValue,
    MemberExpression, NodeId, ObjectExpression, ObjectProperty, ObjectPropertyKind,
    OptionalChainExpression, OptionalChainInner, PropertyName, ReturnStatement, Statement,
    SwitchCase, SwitchStatement, TemplateLiteral, ThrowStatement, TryStatement,
    TypeAliasDeclaration, TypeAssertion, UpdateExpression, VariableDeclaration, WhileStatement,
    WithStatement,
};
use kali_common::template::resolve_interpolated_template_literal;
use kali_error::{
    _error_codes::e3, _error_codes::e4, _error_codes::e5, _error_codes::e6, diagnostic::Diagnostic,
};
use kali_lexer::Lexer;
use kali_parser::Parser;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Scope types recognized by the stage-1 resolver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeType {
    Global,
    Module,
    Block,
    Function,
    Class,
    TypeAlias,
    Interface,
    Catch,
}

/// A lexical scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    pub scope_type: ScopeType,
    pub parent: Option<NodeId>,
    pub bindings: IndexMap<String, NodeId>,
    pub mutable_bindings: IndexMap<String, bool>,
    pub static_values: IndexMap<String, String>,
    pub static_numeric_values: IndexMap<String, String>,
    pub static_arrays: IndexMap<String, bool>,
    pub static_objects: IndexMap<String, bool>,
}

impl Scope {
    pub fn new(scope_type: ScopeType, parent: Option<NodeId>) -> Self {
        Self {
            scope_type,
            parent,
            bindings: IndexMap::new(),
            mutable_bindings: IndexMap::new(),
            static_values: IndexMap::new(),
            static_numeric_values: IndexMap::new(),
            static_arrays: IndexMap::new(),
            static_objects: IndexMap::new(),
        }
    }

    pub fn bind(&mut self, name: impl Into<String>, node_id: NodeId) {
        let name = name.into();
        self.bindings.insert(name.clone(), node_id);
        self.mutable_bindings.insert(name, false);
    }

    pub fn lookup(&self, name: &str) -> Option<&NodeId> {
        self.bindings.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }
}

/// Result of name resolution over a source file/module.
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    pub diagnostics: Vec<Diagnostic>,
    pub scopes: IndexMap<NodeId, Scope>,
    pub global_scope: Scope,
}

/// Type / name-resolution context.
pub struct TypeContext {
    pub global_scope: Scope,
    pub scopes: IndexMap<NodeId, Scope>,
    pub scope_stack: Vec<NodeId>,
    pub type_env: IndexMap<NodeId, String>,
    diagnostics: Vec<Diagnostic>,
    next_scope_id: u32,
    next_binding_id: u32,
    base_path: Option<PathBuf>,
    api_surface: String,
    runtime_profiles: Vec<String>,
    in_generator_function: bool,
}

impl TypeContext {
    pub fn new() -> Self {
        let mut global_scope = Scope::new(ScopeType::Global, None);
        let mut next_binding_id = 0u32;
        for builtin in builtin_globals() {
            bind_builtin(&mut global_scope, &mut next_binding_id, builtin);
        }

        Self {
            global_scope,
            scopes: IndexMap::new(),
            scope_stack: Vec::new(),
            type_env: IndexMap::new(),
            diagnostics: Vec::new(),
            next_scope_id: 1,
            next_binding_id,
            base_path: None,
            api_surface: "deno".to_string(),
            runtime_profiles: Vec::new(),
            in_generator_function: false,
        }
    }

    pub fn with_base_path(base_path: impl AsRef<Path>) -> Self {
        let mut ctx = Self::new();
        ctx.base_path = Some(base_path.as_ref().to_path_buf());
        ctx
    }

    pub fn with_base_path_and_api_surface(
        base_path: impl AsRef<Path>,
        api_surface: impl Into<String>,
    ) -> Self {
        let mut ctx = Self::with_base_path(base_path);
        ctx.set_api_surface(api_surface);
        ctx
    }

    pub fn with_base_path_and_api_surface_and_runtime_profiles(
        base_path: impl AsRef<Path>,
        api_surface: impl Into<String>,
        runtime_profiles: Vec<String>,
    ) -> Self {
        let mut ctx = Self::with_base_path_and_api_surface(base_path, api_surface);
        ctx.set_runtime_profiles(runtime_profiles);
        ctx
    }

    pub fn with_api_surface(api_surface: impl Into<String>) -> Self {
        let mut ctx = Self::new();
        ctx.set_api_surface(api_surface);
        ctx
    }

    pub fn with_api_surface_and_runtime_profiles(
        api_surface: impl Into<String>,
        runtime_profiles: Vec<String>,
    ) -> Self {
        let mut ctx = Self::with_api_surface(api_surface);
        ctx.set_runtime_profiles(runtime_profiles);
        ctx
    }

    pub fn api_surface(&self) -> &str {
        &self.api_surface
    }

    pub fn set_api_surface(&mut self, api_surface: impl Into<String>) {
        self.api_surface = api_surface.into();
        if self.api_surface == "node" {
            for builtin in node_builtin_globals() {
                bind_builtin(&mut self.global_scope, &mut self.next_binding_id, builtin);
            }
        }
    }

    pub fn set_runtime_profiles(&mut self, runtime_profiles: Vec<String>) {
        self.runtime_profiles = runtime_profiles;
    }

    fn has_threaded_runtime_profile(&self) -> bool {
        self.runtime_profiles
            .iter()
            .any(|profile| profile.trim() == "wasm-threads")
    }

    pub fn push_scope(&mut self, scope_type: ScopeType) -> NodeId {
        let parent = self.scope_stack.last().copied();
        let scope_id = NodeId::new(self.next_scope_id);
        self.next_scope_id = self
            .next_scope_id
            .checked_add(1)
            .expect("scope id overflow is unreachable in stage 1");
        self.scopes.insert(scope_id, Scope::new(scope_type, parent));
        self.scope_stack.push(scope_id);
        scope_id
    }

    pub fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    pub fn push_block_scope(&mut self) -> NodeId {
        self.push_scope(ScopeType::Block)
    }

    pub fn push_function_scope(&mut self) -> NodeId {
        self.push_scope(ScopeType::Function)
    }

    pub fn is_defined(&self, name: &str) -> bool {
        self.global_scope.contains(name)
    }

    pub fn define(&mut self, name: impl Into<String>) -> ScopeRef<'_> {
        let name = name.into();
        let binding_id = self.next_binding_id();
        self.global_scope.bind(&name, binding_id);
        ScopeRef {
            scope: &self.global_scope,
            name,
            binding_id,
        }
    }

    pub fn resolve_statements(&mut self, statements: &[Statement]) -> ResolutionResult {
        self.resolve_statements_at_path(None::<&Path>, statements)
    }

    pub fn resolve_statements_at_path(
        &mut self,
        base_path: Option<impl AsRef<Path>>,
        statements: &[Statement],
    ) -> ResolutionResult {
        self.clear_diagnostics();
        self.scopes.clear();
        self.scope_stack.clear();
        self.type_env.clear();
        self.next_scope_id = 1;
        self.base_path = base_path.map(|path| path.as_ref().to_path_buf());

        self.push_scope(ScopeType::Module);
        self.resolve_statement_list(statements);
        self.scope_stack.clear();

        ResolutionResult {
            diagnostics: self.diagnostics.clone(),
            scopes: self.scopes.clone(),
            global_scope: self.global_scope.clone(),
        }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn drain_diagnostics(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    pub fn check_type_annotation(&mut self, _node_id: NodeId, annotation: &str) {
        self.resolve_type_annotation_text(annotation);
    }

    pub fn check_node(&mut self, _node_id: NodeId) {}

    pub fn typecheck(&mut self, _program_root: NodeId) -> Vec<Diagnostic> {
        self.clear_diagnostics();
        self.diagnostics.clone()
    }

    pub fn resolve_name(&self, name: &str) -> Option<NodeId> {
        let mut current = self.scope_stack.last().copied();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id)?;
            if let Some(binding) = scope.lookup(name) {
                return Some(*binding);
            }
            current = scope.parent;
        }

        self.global_scope.lookup(name).copied()
    }

    pub fn resolve_statements_in_file(
        &mut self,
        file_path: impl AsRef<Path>,
        statements: &[Statement],
    ) -> ResolutionResult {
        let file_path = file_path.as_ref();
        self.resolve_statements_at_path(Some(file_path), statements)
    }

    fn next_binding_id(&mut self) -> NodeId {
        let id = NodeId::new(self.next_binding_id);
        self.next_binding_id = self
            .next_binding_id
            .checked_add(1)
            .expect("binding id overflow is unreachable in stage 1");
        id
    }

    fn current_scope_id(&self) -> Option<NodeId> {
        self.scope_stack.last().copied()
    }

    fn scope_mut(&mut self, scope_id: NodeId) -> Option<&mut Scope> {
        self.scopes.get_mut(&scope_id)
    }

    fn bind_current_scope(&mut self, name: impl Into<String>) {
        let name = name.into();
        let binding_id = self.next_binding_id();
        match self.current_scope_id() {
            Some(scope_id) => {
                let scope = self.scope_mut(scope_id).expect("active scope exists");
                if scope.contains(&name) {
                    self.diagnostics.push(duplicate_binding(&name));
                    return;
                }
                scope.bind(name, binding_id);
            }
            None => {
                if self.global_scope.contains(&name) {
                    self.diagnostics.push(duplicate_binding(&name));
                    return;
                }
                self.global_scope.bind(name, binding_id);
            }
        }
    }

    fn bind_in_scope(&mut self, scope_id: NodeId, name: impl Into<String>) {
        let name = name.into();
        let binding_id = self.next_binding_id();
        let scope = self.scope_mut(scope_id).expect("scope exists");
        if scope.contains(&name) {
            self.diagnostics.push(duplicate_binding(&name));
            return;
        }
        scope.bind(name, binding_id);
    }

    fn resolve_statement_list(&mut self, statements: &[Statement]) {
        for statement in statements {
            self.resolve_statement(statement);
        }
    }

    fn resolve_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::ExpressionStatement(ExpressionStatement { expression }) => {
                self.resolve_expression(expression)
            }
            Statement::BreakStatement(BreakStatement { .. }) => {}
            Statement::ContinueStatement(ContinueStatement { .. }) => {}
            Statement::WithStatement(WithStatement { object, body }) => {
                self.resolve_expression(object);
                self.resolve_statement(body);
            }
            Statement::ReturnStatement(ReturnStatement { argument }) => {
                if let Some(argument) = argument {
                    self.resolve_expression(argument);
                }
            }
            Statement::LabeledStatement(LabeledStatement { body, .. }) => {
                self.resolve_statement(body)
            }
            Statement::IfStatement(IfStatement {
                test,
                consequent,
                alternate,
            }) => {
                self.resolve_expression(test);
                self.resolve_block_statement(consequent);
                if let Some(alternate) = alternate {
                    self.resolve_block_statement(alternate);
                }
            }
            Statement::SwitchStatement(SwitchStatement {
                discriminant,
                cases,
            }) => {
                self.resolve_expression(discriminant);
                self.resolve_switch_cases(cases);
            }
            Statement::ThrowStatement(ThrowStatement { argument }) => {
                self.resolve_expression(argument)
            }
            Statement::TryStatement(TryStatement {
                block,
                handler,
                finalizer,
            }) => {
                self.resolve_block_statement(block);
                if let Some(CatchClause { param, body }) = handler {
                    self.push_scope(ScopeType::Catch);
                    self.bind_current_scope(param.clone());
                    self.resolve_block_body(body);
                    self.pop_scope();
                }
                if let Some(finalizer) = finalizer {
                    self.resolve_block_statement(finalizer);
                }
            }
            Statement::DebuggerStatement(_) => {}
            Statement::BlockStatement(block) => self.resolve_block_statement(block),
            Statement::ForStatement(ForStatement {
                init,
                test,
                update,
                body,
            }) => {
                self.push_scope(ScopeType::Block);
                if let Some(init) = init {
                    match init {
                        ForInit::VariableDeclaration(decl) => {
                            self.resolve_variable_declaration(decl)
                        }
                        ForInit::Expression(expr) => self.resolve_expression(expr),
                    }
                }
                if let Some(test) = test {
                    self.resolve_expression(test);
                }
                if let Some(update) = update {
                    self.resolve_expression(update);
                }
                self.resolve_block_body(body);
                self.pop_scope();
            }
            Statement::ForInStatement(ForInStatement { left, right, body }) => {
                self.push_scope(ScopeType::Block);
                match left {
                    ForInLefthand::VariableDeclaration(decl) => {
                        self.resolve_variable_declaration(decl)
                    }
                    ForInLefthand::Expression(expr) => self.resolve_expression(expr),
                }
                self.resolve_expression(right);
                self.resolve_loop_body(body);
                self.pop_scope();
            }
            Statement::ForOfStatement(ForOfStatement {
                left,
                right,
                body,
                is_await: _,
            }) => {
                let left_is_supported = match left {
                    ForOfLefthand::VariableDeclaration(_) => true,
                    ForOfLefthand::Expression(expression) => {
                        self.is_simple_for_of_binding_expression(expression)
                    }
                };
                if !left_is_supported || !self.is_static_array_iteration_target(right) {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "for-of array iteration lowering is unavailable unless the iterable is a literal array with literal elements and the loop target is a variable declaration or simple identifier binding; use a supported loop form or the later compatibility path",
                    ));
                    return;
                }

                self.push_scope(ScopeType::Block);
                if let ForOfLefthand::VariableDeclaration(decl) = left {
                    self.resolve_variable_declaration(decl)
                }
                self.resolve_expression(right);
                self.resolve_loop_body(body);
                self.pop_scope();
            }
            Statement::WhileStatement(WhileStatement { test, body }) => {
                self.resolve_expression(test);
                self.resolve_block_statement(body);
            }
            Statement::DoWhileStatement(DoWhileStatement { body, test }) => {
                self.resolve_block_statement(body);
                self.resolve_expression(test);
            }
            Statement::FunctionDeclaration(FunctionDeclaration {
                name,
                params,
                body,
                generator,
                ..
            }) => {
                self.bind_current_scope(name.clone());
                self.push_scope(ScopeType::Function);
                let previous_generator = self.in_generator_function;
                self.in_generator_function = *generator;
                if *generator {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "generator function lowering is unavailable in the current phase; use a synchronous function or the later compatibility path",
                    ));
                }
                self.bind_name_list(params);
                self.resolve_block_body(body);
                self.in_generator_function = previous_generator;
                self.pop_scope();
            }
            Statement::ClassDeclaration(ClassDeclaration { name, body }) => {
                self.bind_current_scope(name.clone());
                self.resolve_class_body(body);
            }
            Statement::VariableDeclaration(declaration) => {
                self.resolve_variable_declaration(declaration)
            }
            Statement::ImportDeclaration(declaration) => {
                self.resolve_import_declaration(declaration)
            }
            Statement::ExportNamed(declaration) => self.resolve_export_named(declaration),
            Statement::ExportDefault(declaration) => self.resolve_export_default(declaration),
            Statement::EnumDeclaration(EnumDeclaration { name, members }) => {
                self.bind_current_scope(name.clone());
                self.push_scope(ScopeType::Class);
                for EnumMember { name, value } in members {
                    self.bind_current_scope(name.clone());
                    if let Some(value) = value {
                        self.resolve_expression(value);
                    }
                }
                self.pop_scope();
            }
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name,
                type_params,
                type_annotation,
            }) => {
                self.bind_current_scope(name.clone());
                self.push_scope(ScopeType::TypeAlias);
                self.bind_type_params(type_params);
                self.resolve_type_annotation_text(type_annotation);
                self.pop_scope();
            }
            Statement::InterfaceDeclaration(InterfaceDeclaration { name, properties }) => {
                self.bind_current_scope(name.clone());
                self.push_scope(ScopeType::Interface);
                for property in properties {
                    self.bind_current_scope(property.name.clone());
                    self.resolve_type_annotation_text(&property.type_annotation);
                }
                self.pop_scope();
            }
        }
    }

    fn resolve_block_statement(&mut self, block: &BlockStatement) {
        self.push_scope(ScopeType::Block);
        self.resolve_block_body(block);
        self.pop_scope();
    }

    fn resolve_block_body(&mut self, block: &BlockStatement) {
        for statement in &block.body {
            self.resolve_statement(statement);
        }
    }

    fn resolve_loop_body(&mut self, body: &Statement) {
        match body {
            Statement::BlockStatement(block) => self.resolve_block_body(block),
            other => self.resolve_statement(other),
        }
    }

    fn is_static_array_iteration_target(&self, expression: &Expression) -> bool {
        match expression {
            Expression::ParenthesizedExpression(parenthesized) => {
                self.is_static_array_iteration_target(&parenthesized.expression)
            }
            Expression::ArrayExpression(array) => array.elements.iter().all(|element| {
                matches!(
                    element,
                    Some(ExpressionOrSpread::Expression(expr))
                        if self.is_static_array_iteration_element(expr)
                )
            }),
            Expression::Identifier(name) => self.resolve_static_array_binding_name(name),
            Expression::TypeAssertion(expr) => {
                self.is_static_array_iteration_target(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.is_static_array_iteration_target(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.is_static_array_iteration_target(&expr.expression)
            }
            _ => false,
        }
    }

    fn is_static_array_iteration_element(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Literal(_) => true,
            Expression::Identifier(_) => {
                self.resolve_static_numeric_literal_value(expression)
                    .is_some()
                    || self.resolve_static_string_expression(expression).is_some()
            }
            Expression::ParenthesizedExpression(expr) => {
                self.is_static_array_iteration_element(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                self.is_static_array_iteration_element(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.is_static_array_iteration_element(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.is_static_array_iteration_element(&expr.expression)
            }
            _ => false,
        }
    }

    fn is_simple_for_of_binding_expression(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Identifier(_) => true,
            Expression::ParenthesizedExpression(expr) => {
                self.is_simple_for_of_binding_expression(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                self.is_simple_for_of_binding_expression(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.is_simple_for_of_binding_expression(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.is_simple_for_of_binding_expression(&expr.expression)
            }
            _ => false,
        }
    }

    fn is_simple_update_target_expression(&self, expression: &Expression) -> bool {
        matches!(
            expression,
            Expression::Identifier(_)
                | Expression::ParenthesizedExpression(_)
                | Expression::TypeAssertion(_)
                | Expression::SatisfiesExpression(_)
        )
    }

    fn resolve_update_binding_name(&self, expression: &Expression) -> Option<String> {
        match expression {
            Expression::Identifier(name) => Some(name.clone()),
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_update_binding_name(&expr.expression)
            }
            Expression::TypeAssertion(expr) => self.resolve_update_binding_name(&expr.expression),
            Expression::SatisfiesExpression(expr) => {
                self.resolve_update_binding_name(&expr.expression)
            }
            _ => None,
        }
    }

    fn resolve_switch_cases(&mut self, cases: &[SwitchCase]) {
        self.push_scope(ScopeType::Block);
        for case in cases {
            if let Some(test) = &case.test {
                self.resolve_expression(test);
            }
            for statement in &case.consequent {
                self.resolve_statement(statement);
            }
        }
        self.pop_scope();
    }

    fn resolve_variable_declaration(&mut self, declaration: &VariableDeclaration) {
        let target_scope = self.variable_binding_scope(&declaration.kind);
        for declarator in &declaration.declarations {
            self.bind_in_scope(target_scope, declarator.id.clone());
        }
        for declarator in &declaration.declarations {
            if let Some(init) = &declarator.init {
                self.resolve_expression(init);
                if let Some(scope) = self.scopes.get_mut(&target_scope) {
                    scope
                        .mutable_bindings
                        .insert(declarator.id.clone(), declaration.kind != "const");
                } else if self.global_scope.contains(&declarator.id) {
                    self.global_scope
                        .mutable_bindings
                        .insert(declarator.id.clone(), declaration.kind != "const");
                }
                if declaration.kind == "const" {
                    if let Some(value) = self.resolve_static_string_expression(init) {
                        if let Some(scope) = self.scopes.get_mut(&target_scope) {
                            scope.static_values.insert(declarator.id.clone(), value);
                        }
                    }
                    if let Some(value) = self.resolve_static_numeric_literal_value(init) {
                        if let Some(scope) = self.scopes.get_mut(&target_scope) {
                            scope
                                .static_numeric_values
                                .insert(declarator.id.clone(), value.to_string());
                        }
                    }
                    if self.is_static_array_iteration_target(init) {
                        if let Some(scope) = self.scopes.get_mut(&target_scope) {
                            scope.static_arrays.insert(declarator.id.clone(), true);
                        }
                    }
                    if self.resolve_static_object_model_target(init) {
                        if let Some(scope) = self.scopes.get_mut(&target_scope) {
                            scope.static_objects.insert(declarator.id.clone(), true);
                        }
                    }
                }
            }
        }
    }

    fn variable_binding_scope(&self, kind: &str) -> NodeId {
        if kind != "var" {
            return self.current_scope_id().unwrap_or_else(|| NodeId::new(0));
        }

        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id).expect("scope exists");
            match scope.scope_type {
                ScopeType::Function | ScopeType::Module | ScopeType::Global => return scope_id,
                _ => current = scope.parent,
            }
        }

        self.current_scope_id().unwrap_or_else(|| NodeId::new(0))
    }

    fn resolve_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Identifier(name) => self.resolve_identifier(name),
            Expression::Literal(_) => {}
            Expression::BinaryExpression(expr) => {
                self.resolve_expression(&expr.left);
                self.resolve_expression(&expr.right);
            }
            Expression::UnaryExpression(expr) => {
                if expr.operator == "delete" {
                    if let Expression::MemberExpression(member) = &expr.argument {
                        if self.resolve_late_process_env_mutation_member(member) {
                            return;
                        }
                    }
                }
                self.resolve_expression(&expr.argument)
            }
            Expression::CallExpression(expr) => self.resolve_call_expression(expr),
            Expression::MemberExpression(expr) => self.resolve_member_expression(expr),
            Expression::ArrayExpression(ArrayExpression { elements }) => {
                for element in elements.iter().flatten() {
                    match element {
                        ExpressionOrSpread::Expression(expr) => self.resolve_expression(expr),
                        ExpressionOrSpread::Spread(spread) => {
                            self.resolve_expression(&spread.argument)
                        }
                        ExpressionOrSpread::Empty => {}
                    }
                }
            }
            Expression::ObjectExpression(ObjectExpression { properties }) => {
                for property in properties {
                    self.resolve_object_property(property);
                }
            }
            Expression::FunctionExpression(expr) => self.resolve_function_expression(expr),
            Expression::ArrowFunctionExpression(expr) => self.resolve_arrow_function(expr),
            Expression::ClassExpression(expr) => self.resolve_class_expression(expr),
            Expression::NewExpression(expr) => {
                self.resolve_expression(&expr.callee);
                for arg in &expr.args {
                    self.resolve_expression(arg);
                }
            }
            Expression::MetaProperty(_) => {}
            Expression::TemplateLiteral(template) => self.resolve_template_literal(template),
            Expression::TaggedTemplateExpression(expr) => {
                self.resolve_expression(&expr.tag);
                self.resolve_template_literal(&expr.template);
            }
            Expression::UpdateExpression(expr) => self.resolve_update_expression(expr),
            Expression::AssignmentExpression(expr) => {
                self.resolve_expression(&expr.left);
                self.resolve_expression(&expr.right);

                if self.resolve_late_env_assignment_mutation(expr) {
                    return;
                }

                if matches!(expr.operator, AssignmentOperator::Assign) {
                    if let Expression::MemberExpression(member) = &expr.left {
                        let dotted = Self::member_access_name(member)
                            .unwrap_or_else(|| member.property.clone());
                        if self.api_surface == "node"
                            && Self::is_process_env_mutation_path(&dotted)
                            && !Self::is_process_env_root_path(&dotted)
                        {
                            return;
                        }
                    }
                }

                if matches!(expr.operator, AssignmentOperator::Assign) {
                    return;
                }

                let Some(name) = self.resolve_update_binding_name(&expr.left) else {
                    let message = if matches!(expr.operator, AssignmentOperator::NullishAssign) {
                        "nullish assignment lowering is unavailable unless the target is a mutable local binding; use a mutable variable or the later compatibility path".to_string()
                    } else {
                        "compound assignment lowering is unavailable unless the target is a mutable local binding; use a mutable variable or the later compatibility path".to_string()
                    };
                    self.diagnostics
                        .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
                    return;
                };

                if !self.binding_is_mutable(&name) {
                    let message = if matches!(expr.operator, AssignmentOperator::NullishAssign) {
                        format!(
                            "nullish assignment lowering is unavailable for binding '{}' unless it was declared with a mutable binding kind; use a mutable variable or the later compatibility path",
                            name
                        )
                    } else {
                        format!(
                            "compound assignment lowering is unavailable for binding '{}' unless it was declared with a mutable binding kind; use a mutable variable or the later compatibility path",
                            name
                        )
                    };
                    self.diagnostics
                        .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
                }
            }
            Expression::LogicalExpression(expr) => {
                self.resolve_expression(&expr.left);
                self.resolve_expression(&expr.right);
            }
            Expression::ConditionalExpression(expr) => {
                self.resolve_expression(&expr.test);
                self.resolve_expression(&expr.consequent);
                self.resolve_expression(&expr.alternate);
            }
            Expression::SequenceExpression(expr) => {
                for subexpr in &expr.expressions {
                    self.resolve_expression(subexpr);
                }
            }
            Expression::ParenthesizedExpression(expr) => self.resolve_expression(&expr.expression),
            Expression::YieldExpression(expr) => {
                if !self.in_generator_function {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "generator function lowering is unavailable in the current phase; use a synchronous function or the later compatibility path",
                    ));
                }
                if let Some(argument) = &expr.argument {
                    self.resolve_expression(argument);
                }
            }
            Expression::AwaitExpression(expr) => self.resolve_expression(&expr.argument),
            Expression::OptionalChainExpression(expr) => self.resolve_optional_chain(expr),
            Expression::ChainExpression(expr) => self.resolve_expression(&expr.expression),
            Expression::SpreadElement(expr) => self.resolve_expression(&expr.argument),
            Expression::RestElement(expr) => self.resolve_expression(&expr.argument),
            Expression::ImportExpression(expr) => self.resolve_import_expression(expr),
            Expression::DecoratedExpression(DecoratedExpression { expression }) => {
                self.resolve_expression(expression)
            }
            Expression::JsxElement(expr) => self.resolve_jsx_element(expr),
            Expression::JsxFragment(expr) => self.resolve_jsx_fragment(expr),
            Expression::JsxEmptyExpression => {}
            Expression::TypeAssertion(expr) => self.resolve_type_assertion(expr),
            Expression::SatisfiesExpression(expr) => self.resolve_satisfies_expression(expr),
            Expression::ThisExpression | Expression::SuperExpression => {}
            Expression::PrivateIdentifier(_) | Expression::BigIntLiteral(_) => {}
        }
    }

    fn resolve_update_expression(&mut self, expr: &UpdateExpression) {
        self.resolve_expression(&expr.argument);

        if !self.is_simple_update_target_expression(&expr.argument) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "update expression lowering is unavailable unless the target is a mutable local binding; use a local binding or the later compatibility path",
            ));
            return;
        }

        let Some(name) = self.resolve_update_binding_name(&expr.argument) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "update expression lowering is unavailable unless the target is a mutable local binding; use a local binding or the later compatibility path",
            ));
            return;
        };

        if !self.binding_is_mutable(&name) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "update expression lowering is unavailable for binding '{}' unless it was declared with a mutable binding kind; use a mutable local binding or the later compatibility path",
                    name
                ),
            ));
        }
    }

    fn binding_is_mutable(&self, name: &str) -> bool {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id).expect("scope exists");
            if scope.bindings.contains_key(name) {
                return scope.mutable_bindings.get(name).copied().unwrap_or(false);
            }
            current = scope.parent;
        }

        self.global_scope.bindings.contains_key(name)
            && self
                .global_scope
                .mutable_bindings
                .get(name)
                .copied()
                .unwrap_or(false)
    }

    fn resolve_import_expression(&mut self, expr: &ImportExpression) {
        self.resolve_expression(&expr.source);

        if let Some(source) = self.resolve_static_import_source(&expr.source) {
            match self.resolve_import_source(&source) {
                Ok(true) => {}
                Ok(false) => {
                    self.diagnostics.push(
                        Diagnostic::error(
                            e4::DYNAMIC_IMPORT_NOT_IN_LINKED_GRAPH as u32,
                            format!(
                                "dynamic import target '{}' could not be resolved in the linked graph",
                                source
                            ),
                        )
                        .with_suggestion(
                            "use a statically known import specifier or link the module in the build graph",
                        ),
                    );
                }
                Err(diagnostic) => self.diagnostics.push(diagnostic),
            }
        } else {
            self.diagnostics.push(
                Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "non-literal dynamic import() is unavailable in the current phase; use a statically known import specifier that can be resolved in the linked graph".to_string(),
                )
                .with_suggestion(
                    "rewrite the import() target so the compiler can determine a linked-graph module at compile time",
                ),
            );
        }
    }

    fn resolve_static_import_source(&self, expression: &Expression) -> Option<String> {
        self.resolve_static_string_expression(expression)
    }

    fn resolve_static_string_expression(&self, expression: &Expression) -> Option<String> {
        match expression {
            Expression::Literal(LiteralValue::String(value)) => {
                if let Some(rendered) = resolve_interpolated_template_literal(value, |segment| {
                    self.resolve_static_string_from_source(segment)
                }) {
                    Some(rendered)
                } else {
                    Some(Self::normalize_import_segment(value))
                }
            }
            Expression::Literal(LiteralValue::Number(value)) => Some(value.to_string()),
            Expression::Literal(LiteralValue::Boolean(value)) => Some(value.to_string()),
            Expression::Literal(LiteralValue::Null) => Some("null".to_string()),
            Expression::BinaryExpression(expr) if expr.operator == "+" => {
                let left = self.resolve_static_string_expression(&expr.left)?;
                let right = self.resolve_static_string_expression(&expr.right)?;
                Some(format!("{}{}", left, right))
            }
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_string_expression(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                self.resolve_static_string_expression(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_string_expression(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_static_string_expression(&expr.expression)
            }
            Expression::TemplateLiteral(template) => {
                let mut rendered = String::new();
                for (idx, quasi) in template.quasis.iter().enumerate() {
                    rendered.push_str(&quasi.value);
                    if let Some(expr) = template.expressions.get(idx) {
                        rendered.push_str(&self.resolve_static_string_expression(expr)?);
                    }
                }
                Some(rendered)
            }
            Expression::Identifier(name) => self.resolve_static_string_binding(name),
            _ => None,
        }
    }

    fn resolve_static_string_binding(&self, name: &str) -> Option<String> {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id)?;
            if let Some(value) = scope.static_values.get(name) {
                return Some(value.clone());
            }
            current = scope.parent;
        }

        self.global_scope.static_values.get(name).cloned()
    }

    fn resolve_static_string_from_source(&self, source: &str) -> Option<String> {
        let wrapped = format!("const __kali_template__ = ({source});");
        let lexer = Lexer::new(kali_common::FileId::new(0), wrapped);
        let tokens = lexer.lex_all().tokens;
        let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;
        let Statement::VariableDeclaration(declaration) = statements.first()? else {
            return None;
        };
        let initializer = declaration.declarations.first()?.init.as_ref()?;
        self.resolve_static_string_expression(initializer)
    }

    fn resolve_static_numeric_binding(&self, name: &str) -> Option<f64> {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id)?;
            if let Some(value) = scope.static_numeric_values.get(name) {
                return parse_numeric_literal_value(value);
            }
            current = scope.parent;
        }

        self.global_scope
            .static_numeric_values
            .get(name)
            .and_then(|value| parse_numeric_literal_value(value))
    }

    fn resolve_static_array_binding_name(&self, name: &str) -> bool {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id).expect("scope exists");
            if scope.static_arrays.contains_key(name) {
                return true;
            }
            current = scope.parent;
        }

        self.global_scope.static_arrays.contains_key(name)
    }

    fn resolve_static_object_binding_name(&self, name: &str) -> bool {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id).expect("scope exists");
            if scope.static_objects.contains_key(name) {
                return true;
            }
            current = scope.parent;
        }

        self.global_scope.static_objects.contains_key(name)
    }

    fn resolve_static_object_model_target(&self, expression: &Expression) -> bool {
        match expression {
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_object_model_target(&expr.expression)
            }
            Expression::TypeAssertion(expr) => {
                self.resolve_static_object_model_target(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_object_model_target(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_static_object_model_target(&expr.expression)
            }
            Expression::ObjectExpression(ObjectExpression { properties }) => {
                properties.iter().all(|property| {
                    matches!(property.kind, ObjectPropertyKind::Init)
                        && matches!(
                            property.key,
                            PropertyName::Identifier(_)
                                | PropertyName::Number(_)
                                | PropertyName::String(_)
                        )
                })
            }
            Expression::Identifier(name) => self.resolve_static_object_binding_name(name),
            _ => false,
        }
    }

    fn normalize_import_segment(value: &str) -> String {
        let trimmed = value.trim();
        if trimmed.len() >= 2 {
            let mut chars = trimmed.chars();
            let first = chars.next().unwrap();
            let last = chars.next_back().unwrap();
            if matches!((first, last), ('"', '"') | ('\'', '\'') | ('`', '`')) {
                return trimmed[1..trimmed.len() - 1].to_string();
            }
        }
        trimmed.to_string()
    }

    fn resolve_identifier(&mut self, name: &str) {
        if name == "unknown" {
            return;
        }

        if matches!(name, "SharedArrayBuffer" | "Atomics") {
            if self.has_threaded_runtime_profile() {
                return;
            }
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "threaded runtime global '{}' is unavailable until the WASM-threaded profile is enabled",
                    name
                ),
            ));
            return;
        }

        if name == "Intl" {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "broader Intl support is unavailable until the later web/Intl compatibility path is enabled".to_string(),
            ));
            return;
        }

        if matches!(
            name,
            "Proxy" | "WeakMap" | "WeakSet" | "WeakRef" | "FinalizationRegistry"
        ) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "late object-model API '{}' is unavailable until the later object-model compatibility path is enabled",
                    name
                ),
            ));
            return;
        }

        if self.resolve_name(name).is_none() {
            self.diagnostics.push(
                Diagnostic::error(
                    e3::UNDEFINED_IDENTIFIER as u32,
                    format!("undefined identifier '{}'", name),
                )
                .with_suggestion("declare the name in the current module or import it"),
            );
        }
    }

    fn resolve_call_expression(&mut self, expr: &CallExpression) {
        if self.resolve_static_object_model_call(expr) {
            return;
        }

        if self.resolve_static_object_identity_call(expr) {
            return;
        }

        self.resolve_expression(&expr.callee);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }
        self.resolve_permission_query_call(expr);
        self.resolve_math_member_call(expr);
        self.resolve_promise_member_call(expr);
    }

    fn resolve_member_expression(&mut self, expr: &MemberExpression) {
        if self.resolve_late_intl_member(expr) {
            return;
        }

        if self.resolve_late_object_model_member(expr) {
            return;
        }

        if self.resolve_late_env_object_member(expr) {
            return;
        }

        if self.resolve_late_env_mutation_member(expr) {
            return;
        }

        self.resolve_expression(&expr.object);
        self.resolve_threaded_runtime_member(expr);
        self.resolve_late_host_control_member(expr);
        self.resolve_late_permission_escalation_member(expr);
    }

    fn resolve_permission_query_call(&mut self, expr: &CallExpression) {
        let Some(callee_name) = Self::member_access_name(match &expr.callee {
            Expression::MemberExpression(member) => member,
            _ => return,
        }) else {
            return;
        };

        if !matches!(
            callee_name.as_str(),
            "Deno.permissions.query" | "globalThis.Deno.permissions.query"
        ) {
            return;
        }

        let Some(descriptor_name) = expr
            .args
            .first()
            .and_then(|expr| self.resolve_permissions_query_descriptor_name(expr))
        else {
            return;
        };

        if matches!(descriptor_name.as_str(), "read" | "write" | "net" | "env") {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "permission query descriptor '{}' is unavailable in the Phase-1 Deno permission facade",
                descriptor_name
            ),
        ));
    }

    fn resolve_static_object_model_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) = Self::member_access_name(match &expr.callee {
            Expression::MemberExpression(member) => member,
            _ => return false,
        }) else {
            return false;
        };

        let is_object_has_own = matches!(
            callee_name.as_str(),
            "Object.hasOwn" | "globalThis.Object.hasOwn"
        );
        let is_has_own_property_call = matches!(
            callee_name.as_str(),
            "Object.prototype.hasOwnProperty.call"
                | "globalThis.Object.prototype.hasOwnProperty.call"
        );
        if !is_object_has_own && !is_has_own_property_call {
            return false;
        }

        let Some(object_arg) = expr.args.first() else {
            return false;
        };
        let Some(key_arg) = expr.args.get(1) else {
            return false;
        };

        if !self.resolve_static_object_model_target(object_arg) {
            return false;
        }
        if self.resolve_static_string_expression(key_arg).is_none() {
            return false;
        }

        self.resolve_expression(object_arg);
        self.resolve_expression(key_arg);
        for arg in expr.args.iter().skip(2) {
            self.resolve_expression(arg);
        }
        true
    }

    fn resolve_static_object_identity_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) = Self::member_access_name(match &expr.callee {
            Expression::MemberExpression(member) => member,
            _ => return false,
        }) else {
            return false;
        };

        if !matches!(callee_name.as_str(), "Object.is" | "globalThis.Object.is") {
            return false;
        }

        let Some(left) = expr.args.first() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Object.is requires at least two statically-known numeric literal arguments in the current phase; use explicit constants or the later compatibility path",
            ));
            return true;
        };
        let Some(right) = expr.args.get(1) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Object.is requires at least two statically-known numeric literal arguments in the current phase; use explicit constants or the later compatibility path",
            ));
            return true;
        };

        if self.resolve_static_numeric_literal_value(left).is_none()
            || self.resolve_static_numeric_literal_value(right).is_none()
        {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Object.is is unavailable unless both arguments are statically-known numeric literals in the current phase; use explicit constants or the later compatibility path",
            ));
            return true;
        }

        self.resolve_expression(left);
        self.resolve_expression(right);
        for arg in expr.args.iter().skip(2) {
            self.resolve_expression(arg);
        }
        true
    }

    fn resolve_math_member_call(&mut self, expr: &CallExpression) {
        let Some(callee_name) = Self::member_access_name(match &expr.callee {
            Expression::MemberExpression(member) => member,
            _ => return,
        }) else {
            return;
        };

        let Some(method) = callee_name
            .strip_prefix("Math.")
            .or_else(|| callee_name.strip_prefix("globalThis.Math."))
        else {
            return;
        };

        if method == "hypot" {
            if self
                .resolve_math_hypot_static_literal_root(&expr.args)
                .is_some()
            {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Math.hypot is unavailable unless every argument is a statically-known integer literal whose squared sum is a perfect-square integer literal in the current phase; use explicit constants or the later compatibility path",
            ));
            return;
        }

        if method == "sqrt" || method == "cbrt" || method == "log2" || method == "log10" {
            let literal_root = expr.args.first().and_then(|arg| {
                if method == "sqrt" {
                    self.resolve_math_sqrt_static_literal_root(arg)
                } else if method == "cbrt" {
                    self.resolve_math_cbrt_static_literal_root(arg)
                } else if method == "log2" {
                    self.resolve_math_log2_static_literal_exponent(arg)
                } else {
                    self.resolve_math_log10_static_literal_exponent(arg)
                }
            });
            if literal_root.is_some() {
                return;
            }

            let shape = if method == "sqrt" {
                "perfect-square"
            } else if method == "cbrt" {
                "perfect-cube"
            } else if method == "log2" {
                "positive power-of-two"
            } else {
                "positive power-of-ten"
            };
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known {shape} integer literal in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return;
        }

        if method == "exp" || method == "log" {
            let Some(value) = expr
                .args
                .first()
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                        if method == "exp" { "zero" } else { "one" }
                    ),
                ));
                return;
            };

            if (method == "exp" && value == 0.0) || (method == "log" && value == 1.0) {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                    if method == "exp" { "zero" } else { "one" }
                ),
            ));
            return;
        }

        if method == "expm1" || method == "log1p" {
            let Some(value) = expr
                .args
                .first()
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                    ),
                ));
                return;
            };

            if value == 0.0 {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return;
        }

        if method == "asin" || method == "acos" || method == "atan" {
            let Some(value) = expr
                .args
                .first()
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                        if method == "acos" { "one" } else { "zero" }
                    ),
                ));
                return;
            };

            if (method == "acos" && value == 1.0) || (method != "acos" && value == 0.0) {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                    if method == "acos" { "one" } else { "zero" }
                ),
            ));
            return;
        }

        if method == "atan2" {
            let atan2_message = "Math.atan2 is unavailable unless the first argument is a statically-known zero numeric literal and the second argument is a statically-known non-negative numeric literal in the current phase; use explicit constants or the later compatibility path".to_string();
            let Some(y) = expr
                .args
                .first()
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    atan2_message,
                ));
                return;
            };

            let Some(x) = expr
                .args
                .get(1)
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    atan2_message,
                ));
                return;
            };

            if y == 0.0 && x.is_finite() && x >= 0.0 {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                atan2_message,
            ));
            return;
        }

        if method == "asinh" || method == "acosh" || method == "atanh" {
            let Some(argument) = expr.args.first() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                        if method == "acosh" { "one" } else { "zero" }
                    ),
                ));
                return;
            };

            if self
                .resolve_math_inverse_hyperbolic_constant_value(method, argument)
                .is_some()
            {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                    if method == "acosh" { "one" } else { "zero" }
                ),
            ));
            return;
        }

        if method == "sinh" || method == "cosh" || method == "tanh" {
            let Some(argument) = expr.args.first() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                    ),
                ));
                return;
            };

            if self
                .resolve_math_hyperbolic_zero_constant_value(method, argument)
                .is_some()
            {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return;
        }

        if method == "atan2" {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Math.atan2 is unavailable in the current phase; use a supported Math builtin or the later compatibility path",
            ));
            return;
        }

        if method == "sin" || method == "cos" || method == "tan" {
            let Some(value) = expr
                .args
                .first()
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                    ),
                ));
                return;
            };

            if value == 0.0 {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return;
        }

        if method == "max" || method == "min" {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                    ),
                ));
                return;
            }

            if let Some(_folded) =
                self.resolve_math_extrema_static_literal_value(method, &expr.args)
            {
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path"
                    ),
                ));
            }
            return;
        }

        if method == "pow" {
            if expr.args.len() < 2 {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.pow requires at least two arguments in the current phase; use explicit operands or the later compatibility path",
                ));
                return;
            }

            let exponent_is_static_zero = self
                .resolve_static_numeric_literal_value(expr.args.get(1).unwrap())
                .is_some_and(|value| value == 0.0);

            if !exponent_is_static_zero
                && expr
                    .args
                    .iter()
                    .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.pow is unavailable for non-integer numeric literals in the current phase; use an integer-valued exponent or the later compatibility path",
                ));
                return;
            }

            if expr
                .args
                .get(1)
                .is_some_and(|arg| self.contains_negative_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.pow is unavailable for negative numeric literals in the current phase; use a non-negative exponent or the later compatibility path",
                ));
            }
            return;
        }

        if method == "round" {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.round requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                return;
            }

            if self
                .resolve_math_round_like_static_literal_value(method, expr.args.first())
                .is_some()
            {
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.round is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path",
                ));
            }
            return;
        }

        if method == "floor" {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.floor requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                return;
            }

            if self
                .resolve_math_round_like_static_literal_value(method, expr.args.first())
                .is_some()
            {
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.floor is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path",
                ));
            }
            return;
        }

        if matches!(method, "trunc" | "ceil") {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                    ),
                ));
                return;
            }

            if self
                .resolve_math_round_like_static_literal_value(method, expr.args.first())
                .is_some()
            {
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path"
                    ),
                ));
            }
            return;
        }

        if matches!(
            method,
            "max" | "min" | "abs" | "sign" | "tan" | "asinh" | "acosh" | "atanh"
        ) {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                    ),
                ));
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path"
                    ),
                ));
            }
            return;
        }

        if method == "imul" {
            if expr.args.len() < 2 {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.imul requires at least two arguments in the current phase; use explicit operands or the later compatibility path",
                ));
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.imul is unavailable for non-integer numeric literals in the current phase; use integer-valued operands or the later compatibility path",
                ));
            }
            return;
        }

        if method == "clz32" {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "Math.{method} is unavailable in the current phase; use a supported Math builtin or the later compatibility path"
            ),
        ));
    }

    fn resolve_promise_member_call(&mut self, _expr: &CallExpression) {}

    fn contains_non_integer_numeric_literal(&self, expression: &Expression) -> bool {
        self.resolve_static_numeric_literal_value(expression)
            .is_some_and(|value| value.fract() != 0.0)
    }

    fn resolve_static_numeric_literal_value(&self, expression: &Expression) -> Option<f64> {
        match expression {
            Expression::Literal(LiteralValue::Number(value)) => Some(*value),
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_numeric_literal_value(&expr.expression)
            }
            Expression::UnaryExpression(expr) if expr.operator == "+" => {
                self.resolve_static_numeric_literal_value(&expr.argument)
            }
            Expression::UnaryExpression(expr) if expr.operator == "-" => self
                .resolve_static_numeric_literal_value(&expr.argument)
                .map(|value| -value),
            Expression::TypeAssertion(expr) => {
                self.resolve_static_numeric_literal_value(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_numeric_literal_value(&expr.expression)
            }
            Expression::ChainExpression(expr) => {
                self.resolve_static_numeric_literal_value(&expr.expression)
            }
            Expression::Identifier(name) => self.resolve_static_numeric_binding(name),
            _ => None,
        }
    }

    fn resolve_math_round_like_static_literal_value(
        &self,
        method: &str,
        expression: Option<&Expression>,
    ) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression?)?;
        let folded = match method {
            "round" => (value + 0.5).floor(),
            "trunc" => value.trunc(),
            "ceil" => value.ceil(),
            "floor" => value.floor(),
            _ => return None,
        };

        if !folded.is_finite() || folded < i64::MIN as f64 || folded > i64::MAX as f64 {
            return None;
        }

        Some(folded as i64)
    }

    fn contains_negative_numeric_literal(&self, expression: &Expression) -> bool {
        self.resolve_static_numeric_literal_value(expression)
            .is_some_and(|value| value < 0.0)
    }

    fn resolve_math_extrema_static_literal_value(
        &self,
        method: &str,
        expressions: &[Expression],
    ) -> Option<i64> {
        let mut values = expressions.iter().map(|expression| {
            let value = self.resolve_static_numeric_literal_value(expression)?;
            if !value.is_finite()
                || value.fract() != 0.0
                || value < i64::MIN as f64
                || value > i64::MAX as f64
            {
                return None;
            }
            Some(value as i64)
        });

        let first = values.next().flatten()?;
        let mut folded = first;

        for value in values {
            let value = value?;
            folded = if method == "max" {
                folded.max(value)
            } else {
                folded.min(value)
            };
        }

        Some(folded)
    }

    fn resolve_math_inverse_hyperbolic_constant_value(
        &self,
        method: &str,
        expression: &Expression,
    ) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;

        match method {
            "acosh" if value == 1.0 => Some(0),
            "asinh" | "atanh" if value == 0.0 => Some(0),
            _ => None,
        }
    }

    fn resolve_math_hyperbolic_zero_constant_value(
        &self,
        method: &str,
        expression: &Expression,
    ) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;
        if value != 0.0 {
            return None;
        }

        Some(if method == "cosh" { 1 } else { 0 })
    }

    fn resolve_math_sqrt_static_literal_root(&self, expression: &Expression) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;
        if !value.is_finite() || value.fract() != 0.0 || value < 0.0 || value > i64::MAX as f64 {
            return None;
        }

        let value = value as i64;
        let root = (value as f64).sqrt() as i64;
        if root.checked_mul(root) == Some(value) {
            Some(root)
        } else {
            None
        }
    }

    fn resolve_math_cbrt_static_literal_root(&self, expression: &Expression) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;
        if !value.is_finite()
            || value.fract() != 0.0
            || value < i64::MIN as f64
            || value > i64::MAX as f64
        {
            return None;
        }

        let value = value as i64;
        let root = (value as f64).cbrt().round() as i64;
        if i128::from(root).pow(3) == i128::from(value) {
            Some(root)
        } else {
            None
        }
    }

    fn resolve_math_log2_static_literal_exponent(&self, expression: &Expression) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;
        if !value.is_finite() || value.fract() != 0.0 || value <= 0.0 || value > u64::MAX as f64 {
            return None;
        }

        let value = value as u64;
        if value.is_power_of_two() {
            Some(i64::from(value.trailing_zeros()))
        } else {
            None
        }
    }

    fn resolve_math_log10_static_literal_exponent(&self, expression: &Expression) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;
        if !value.is_finite() || value.fract() != 0.0 || value <= 0.0 || value > i64::MAX as f64 {
            return None;
        }

        let mut value = value as i64;
        let mut exponent = 0;
        while value % 10 == 0 {
            value /= 10;
            exponent += 1;
        }

        if value == 1 {
            Some(exponent)
        } else {
            None
        }
    }

    fn resolve_math_hypot_static_literal_root(&self, expressions: &[Expression]) -> Option<i64> {
        if expressions.is_empty() {
            return Some(0);
        }

        let mut sum = 0_i128;
        for expression in expressions {
            let value = self.resolve_static_numeric_literal_value(expression)?;
            if !value.is_finite()
                || value.fract() != 0.0
                || value < i64::MIN as f64
                || value > i64::MAX as f64
            {
                return None;
            }

            let value = value as i128;
            sum = sum.checked_add(value.checked_mul(value)?)?;
        }

        self.resolve_perfect_square_i128(sum)
    }

    fn resolve_perfect_square_i128(&self, value: i128) -> Option<i64> {
        if value < 0 {
            return None;
        }

        let mut low = 0_i128;
        let mut high = i128::from(i64::MAX).min(value);
        while low <= high {
            let mid = low + (high - low) / 2;
            let square = mid.checked_mul(mid)?;
            if square == value {
                return Some(mid as i64);
            }
            if square < value {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }

        None
    }

    fn resolve_permissions_query_descriptor_name(&self, expr: &Expression) -> Option<String> {
        let Expression::ObjectExpression(ObjectExpression { properties }) = expr else {
            return None;
        };

        for property in properties {
            if !matches!(property.kind, ObjectPropertyKind::Init) {
                continue;
            }

            let key_name = match &property.key {
                PropertyName::Identifier(name) | PropertyName::String(name) => name.as_str(),
                PropertyName::Number(_) => continue,
            };

            if key_name != "name" {
                continue;
            }

            return self.resolve_static_string_expression(&property.value);
        }

        None
    }

    fn resolve_threaded_runtime_member(&mut self, expr: &MemberExpression) {
        let Expression::Identifier(object_name) = &expr.object else {
            return;
        };

        if object_name != "globalThis" {
            return;
        }

        if !matches!(expr.property.as_str(), "SharedArrayBuffer" | "Atomics") {
            return;
        }

        if self.has_threaded_runtime_profile() {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "threaded runtime global 'globalThis.{}' is unavailable until the WASM-threaded profile is enabled",
                expr.property
            ),
        ));
    }

    fn resolve_late_host_control_member(&mut self, expr: &MemberExpression) {
        if !matches!(expr.property.as_str(), "pid" | "cwd" | "chdir" | "exit") {
            return;
        }

        let Some(object_name) = Self::member_object_name(&expr.object) else {
            return;
        };

        if expr.property == "pid" && object_name == "Deno" {
            return;
        }

        if expr.property == "exit" && object_name == "Deno" && self.api_surface == "deno" {
            return;
        }

        if expr.property == "cwd" && object_name == "Deno" && self.api_surface == "deno" {
            return;
        }

        if expr.property == "chdir" && object_name == "Deno" && self.api_surface == "deno" {
            return;
        }

        if expr.property == "cwd" && object_name == "process" && self.api_surface == "node" {
            return;
        }

        if expr.property == "chdir" && object_name == "process" && self.api_surface == "node" {
            return;
        }

        if expr.property == "pid" && object_name == "process" && self.api_surface == "node" {
            return;
        }

        if expr.property == "exit" && object_name == "process" && self.api_surface == "node" {
            return;
        }

        if !matches!(object_name.as_str(), "Deno" | "process") {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "late host-control API '{}' is unavailable until the later host-control compatibility path is enabled",
                Self::member_access_name(expr).unwrap_or_else(|| format!("{}.{}", object_name, expr.property))
            ),
        ));
    }

    fn resolve_late_permission_escalation_member(&mut self, expr: &MemberExpression) -> bool {
        if !matches!(
            Self::member_access_name(expr).as_deref(),
            Some("Deno.permissions.request")
                | Some("Deno.permissions.revoke")
                | Some("globalThis.Deno.permissions.request")
                | Some("globalThis.Deno.permissions.revoke")
        ) {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "permission escalation API '{}' (aka {}) is unavailable in the Phase-1 Deno permission facade",
                dotted, bracketed
            ),
        ));
        true
    }

    fn resolve_late_env_object_member(&mut self, expr: &MemberExpression) -> bool {
        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        if !matches!(
            dotted.as_str(),
            "Deno.env.toObject" | "globalThis.Deno.env.toObject"
        ) {
            return false;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "environment snapshot materialization API '{}' (aka {}) is unavailable until the later env-object materialization and object-aggregate lowering path is enabled",
                dotted, bracketed
            ),
        ));
        true
    }

    fn resolve_late_env_mutation_member(&mut self, expr: &MemberExpression) -> bool {
        if self.api_surface != "browser" {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        if !matches!(
            dotted.as_str(),
            "Deno.env.set"
                | "Deno.env.delete"
                | "globalThis.Deno.env.set"
                | "globalThis.Deno.env.delete"
        ) {
            return false;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "environment mutation API '{}' (aka {}) is unavailable in the browser API surface until the later mutable env path is enabled",
                dotted, bracketed
            ),
        ));
        true
    }

    fn resolve_late_env_assignment_mutation(&mut self, expr: &AssignmentExpression) -> bool {
        let Expression::MemberExpression(member) = &expr.left else {
            return false;
        };

        let dotted = Self::member_access_name(member).unwrap_or_else(|| member.property.clone());
        let bracketed =
            Self::member_access_name_bracketed(member).unwrap_or_else(|| dotted.clone());

        if Self::is_process_env_root_path(&dotted) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "environment mutation API '{}' (aka {}) is unavailable on the {} API surface until the later mutable env path is enabled",
                    dotted, bracketed, self.api_surface
                ),
            ));
            return true;
        }

        if self.api_surface != "node" && Self::is_process_env_mutation_path(&dotted) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "environment mutation API '{}' (aka {}) is unavailable on the {} API surface until the later mutable env path is enabled",
                    dotted, bracketed, self.api_surface
                ),
            ));
            return true;
        }

        false
    }

    fn resolve_late_process_env_mutation_member(&mut self, member: &MemberExpression) -> bool {
        let dotted = Self::member_access_name(member).unwrap_or_else(|| member.property.clone());
        let bracketed =
            Self::member_access_name_bracketed(member).unwrap_or_else(|| dotted.clone());

        if Self::is_process_env_root_path(&dotted) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "environment mutation API '{}' (aka {}) is unavailable on the {} API surface until the later mutable env path is enabled",
                    dotted, bracketed, self.api_surface
                ),
            ));
            return true;
        }

        if self.api_surface != "node" && Self::is_process_env_mutation_path(&dotted) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "environment mutation API '{}' (aka {}) is unavailable on the {} API surface until the later mutable env path is enabled",
                    dotted, bracketed, self.api_surface
                ),
            ));
            return true;
        }

        false
    }

    fn is_process_env_root_path(path: &str) -> bool {
        matches!(path, "process.env" | "globalThis.process.env")
    }

    fn is_process_env_mutation_path(path: &str) -> bool {
        Self::is_process_env_root_path(path)
            || path.starts_with("process.env.")
            || path.starts_with("process.env[")
            || path.starts_with("globalThis.process.env.")
            || path.starts_with("globalThis.process.env[")
    }

    fn resolve_late_intl_member(&mut self, expr: &MemberExpression) -> bool {
        let is_intl_root = matches!(&expr.object, Expression::Identifier(name) if name == "Intl")
            || matches!(
                &expr.object,
                Expression::Identifier(name) if name == "globalThis" && expr.property == "Intl"
            )
            || matches!(
                &expr.object,
                Expression::MemberExpression(member)
                    if matches!(&member.object, Expression::Identifier(name) if name == "globalThis")
                        && member.property == "Intl"
            );

        if !is_intl_root {
            return false;
        }

        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr)
            .unwrap_or_else(|| format!("globalThis[\"{}\"]", expr.property));

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "broader Intl support via '{}' (aka {}) is unavailable until the later web/Intl compatibility path is enabled",
                dotted, bracketed
            ),
        ));
        true
    }

    fn resolve_late_object_model_member(&mut self, expr: &MemberExpression) -> bool {
        let dotted = Self::member_access_name(expr).unwrap_or_else(|| expr.property.clone());
        let bracketed = Self::member_access_name_bracketed(expr).unwrap_or_else(|| dotted.clone());

        if matches!(
            dotted.as_str(),
            "Proxy.revocable" | "globalThis.Proxy.revocable"
        ) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "late object-model API '{}' (aka {}) is unavailable until the later object-model compatibility path is enabled",
                    dotted, bracketed
                ),
            ));
            return true;
        }

        if matches!(
            dotted.as_str(),
            "Object.hasOwn"
                | "globalThis.Object.hasOwn"
                | "Object.prototype.hasOwnProperty.call"
                | "globalThis.Object.prototype.hasOwnProperty.call"
        ) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "late object-model API '{}' (aka {}) is unavailable until the later object-model compatibility path is enabled",
                    dotted, bracketed
                ),
            ));
            return true;
        }

        if !matches!(
            expr.property.as_str(),
            "Proxy" | "WeakMap" | "WeakSet" | "WeakRef" | "FinalizationRegistry"
        ) {
            return false;
        }

        let Some(object_name) = Self::member_object_name(&expr.object) else {
            return false;
        };

        if object_name != "globalThis" {
            return false;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "late object-model API '{}' (aka {}) is unavailable until the later object-model compatibility path is enabled",
                dotted, bracketed
            ),
        ));
        true
    }

    fn member_access_name(expr: &MemberExpression) -> Option<String> {
        let object_name = match &expr.object {
            Expression::Identifier(name) => Some(name.clone()),
            Expression::MemberExpression(member) => Self::member_access_name(member),
            _ => None,
        }?;

        Some(format!("{}.{}", object_name, expr.property))
    }

    fn member_access_name_bracketed(expr: &MemberExpression) -> Option<String> {
        let object_name = match &expr.object {
            Expression::Identifier(name) => Some(name.clone()),
            Expression::MemberExpression(member) => Self::member_access_name_bracketed(member),
            _ => None,
        }?;

        Some(format!("{}[\"{}\"]", object_name, expr.property))
    }

    fn member_object_name(object: &Expression) -> Option<String> {
        match object {
            Expression::Identifier(name) => Some(name.clone()),
            Expression::MemberExpression(member) if matches!(&member.object, Expression::Identifier(name) if name == "globalThis") => {
                Some(member.property.clone())
            }
            _ => None,
        }
    }

    fn resolve_function_expression(&mut self, expr: &FunctionExpression) {
        self.push_scope(ScopeType::Function);
        let previous_generator = self.in_generator_function;
        self.in_generator_function = expr.generator;
        if expr.generator {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "generator function lowering is unavailable in the current phase; use a synchronous function or the later compatibility path",
            ));
        }
        if let Some(name) = &expr.id {
            self.bind_current_scope(name.clone());
        }
        self.bind_function_params(&expr.params);
        if let Some(body) = &expr.body {
            self.resolve_block_body(body);
        }
        self.in_generator_function = previous_generator;
        self.pop_scope();
    }

    fn resolve_arrow_function(&mut self, expr: &ArrowFunctionExpression) {
        self.push_scope(ScopeType::Function);
        self.bind_function_params(&expr.params);
        if let Some(return_type) = &expr.returnType {
            self.resolve_type_annotation_text(return_type);
        }
        self.resolve_expression(&expr.body);
        self.pop_scope();
    }

    fn resolve_class_expression(&mut self, expr: &ClassExpression) {
        self.push_scope(ScopeType::Class);
        if let Some(name) = &expr.id {
            self.bind_current_scope(name.clone());
        }
        self.resolve_class_body(&expr.body);
        self.pop_scope();
    }

    fn resolve_class_body(&mut self, body: &ClassBody) {
        self.push_scope(ScopeType::Class);
        for method in &body.methods {
            self.bind_current_scope(method.name.clone());
            self.push_scope(ScopeType::Function);
            self.bind_name_list(&method.params);
            if let Some(body) = &method.body {
                self.resolve_block_body(body);
            }
            self.pop_scope();
        }
        self.pop_scope();
    }

    fn resolve_import_declaration(&mut self, declaration: &ImportDeclaration) {
        match self.resolve_import_source(&declaration.source) {
            Ok(true) => {}
            Ok(false) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        e3::IMPORT_NOT_FOUND as u32,
                        format!(
                            "import source '{}' could not be resolved",
                            declaration.source
                        ),
                    )
                    .with_suggestion("check the relative path or package specifier"),
                );
                return;
            }
            Err(diagnostic) => {
                self.diagnostics.push(diagnostic);
                return;
            }
        }

        for specifier in &declaration.specifiers {
            match specifier {
                ImportSpecifier::Default(local) => {
                    self.bind_current_scope(local.clone());
                }
                ImportSpecifier::Named(specifiers) | ImportSpecifier::Type(specifiers) => {
                    for specifier in specifiers {
                        self.bind_current_scope(specifier.local.clone());
                    }
                }
                ImportSpecifier::Namespace(local) => {
                    self.bind_current_scope(local.clone());
                }
                ImportSpecifier::SideEffect => {}
            }
        }
    }

    fn resolve_export_named(&mut self, declaration: &kali_ast::ExportNamedDeclaration) {
        if let Some(source) = &declaration.source {
            match self.resolve_import_source(source) {
                Ok(true) => {}
                Ok(false) => {
                    self.diagnostics.push(
                        Diagnostic::error(
                            e3::IMPORT_NOT_FOUND as u32,
                            format!("re-export source '{}' could not be resolved", source),
                        )
                        .with_suggestion("check the relative path or package specifier"),
                    );
                }
                Err(diagnostic) => {
                    self.diagnostics.push(diagnostic);
                }
            }
            return;
        }

        for specifier in &declaration.specifiers {
            self.resolve_identifier(&specifier.local);
        }
    }

    fn resolve_export_default(&mut self, declaration: &kali_ast::ExportDefaultDeclaration) {
        match declaration {
            kali_ast::ExportDefaultDeclaration::Expression(expr) => self.resolve_expression(expr),
            kali_ast::ExportDefaultDeclaration::FunctionDeclaration(func) => {
                self.push_scope(ScopeType::Function);
                self.bind_current_scope(func.name.clone());
                self.bind_function_params(
                    &func
                        .params
                        .iter()
                        .cloned()
                        .map(|name| FunctionParam { name })
                        .collect::<Vec<_>>(),
                );
                self.resolve_block_body(&func.body);
                self.pop_scope();
            }
            kali_ast::ExportDefaultDeclaration::ClassDeclaration(class) => {
                self.push_scope(ScopeType::Class);
                self.bind_current_scope(class.name.clone());
                self.resolve_class_body(&class.body);
                self.pop_scope();
            }
        }
    }

    fn resolve_optional_chain(&mut self, expr: &OptionalChainExpression) {
        match expr.inner.as_ref() {
            OptionalChainInner::NonNull { object, .. } => self.resolve_expression(object),
        }
    }

    fn resolve_template_literal(&mut self, template: &TemplateLiteral) {
        for expr in &template.expressions {
            self.resolve_expression(expr);
        }
    }

    fn resolve_object_property(&mut self, property: &ObjectProperty) {
        self.resolve_property_name(&property.key);
        self.resolve_expression(&property.value);
    }

    fn resolve_property_name(&mut self, name: &PropertyName) {
        match name {
            PropertyName::Identifier(_) | PropertyName::Number(_) | PropertyName::String(_) => {}
        }
    }

    fn resolve_type_assertion(&mut self, expr: &TypeAssertion) {
        self.resolve_type_annotation_text(&expr.type_name);
        self.resolve_expression(&expr.expression);
    }

    fn resolve_satisfies_expression(&mut self, expr: &kali_ast::SatisfiesExpression) {
        self.resolve_type_annotation_text(&expr.type_name);
        self.resolve_expression(&expr.expression);
    }

    fn resolve_jsx_element(&mut self, expr: &JsxElement) {
        for child in &expr.children {
            self.resolve_jsx_child(child);
        }
    }

    fn resolve_jsx_fragment(&mut self, expr: &JsxFragment) {
        for child in &expr.children {
            self.resolve_jsx_child(child);
        }
    }

    fn resolve_jsx_child(&mut self, child: &JsxChild) {
        match child {
            JsxChild::JsxText(_) => {}
            JsxChild::JsxExpression(container) => {
                if let Some(expr) = &container.expression {
                    self.resolve_expression(expr);
                }
            }
            JsxChild::JsxElement(child) => self.resolve_jsx_element(child),
            JsxChild::JsxFragment(child) => self.resolve_jsx_fragment(child),
        }
    }

    fn bind_function_params(&mut self, params: &[FunctionParam]) {
        for param in params {
            self.bind_current_scope(param.name.clone());
        }
    }

    fn bind_name_list(&mut self, names: &[String]) {
        for name in names {
            self.bind_current_scope(name.clone());
        }
    }

    fn bind_type_params(&mut self, type_params: &[String]) {
        self.bind_name_list(type_params)
    }

    fn resolve_type_annotation_text(&mut self, annotation: &str) {
        let annotation = annotation.trim();
        if annotation.is_empty() {
            return;
        }

        let chars: Vec<char> = annotation.chars().collect();
        let mut index = 0;
        while index < chars.len() {
            let ch = chars[index];
            if matches!(ch, '\'' | '"' | '`') {
                index = skip_quoted_annotation_segment(&chars, index);
                continue;
            }

            if is_ident_start(ch) {
                let start = index;
                index += 1;
                while index < chars.len() && is_ident_continue(chars[index]) {
                    index += 1;
                }

                let ident: String = chars[start..index].iter().collect();
                if !is_type_annotation_keyword(&ident)
                    && !is_property_name_context(&chars, start, index)
                    && self.resolve_name(&ident).is_none()
                {
                    self.diagnostics.push(
                        Diagnostic::error(
                            e3::UNDEFINED_IDENTIFIER as u32,
                            format!("undefined type reference '{}'", ident),
                        )
                        .with_suggestion(
                            "declare the type or import it before using it in an annotation",
                        ),
                    );
                }
                continue;
            }

            index += 1;
        }
    }

    fn resolve_relative_import_source(&self, base_dir: &Path, source: &str) -> bool {
        let candidate = base_dir.join(source);
        if candidate.is_file() {
            return true;
        }

        if candidate.is_dir() && self.resolve_directory_index_candidate(&candidate) {
            return true;
        }

        let extensions = [
            "ts", "tsx", "js", "jsx", "mts", "cts", "d.ts", "d.mts", "d.cts",
        ];
        extensions.iter().any(|extension| {
            let candidate = if source.ends_with(extension) {
                base_dir.join(source)
            } else {
                base_dir.join(format!("{}.{}", source, extension))
            };
            candidate.is_file()
                || (candidate.is_dir() && self.resolve_directory_index_candidate(&candidate))
        })
    }

    fn resolve_directory_index_candidate(&self, directory: &Path) -> bool {
        for index_name in [
            "index.ts",
            "index.tsx",
            "index.js",
            "index.jsx",
            "index.mts",
            "index.mjs",
            "index.cts",
            "index.cjs",
            "index.d.ts",
            "index.d.mts",
            "index.d.cts",
        ] {
            if directory.join(index_name).is_file() {
                return true;
            }
        }

        false
    }

    fn resolve_import_source(&self, source: &str) -> Result<bool, Diagnostic> {
        if self.api_surface == "node" && is_node_builtin_specifier(source) {
            return Ok(true);
        }

        if self.api_surface == "node" && source.starts_with("node:") {
            return Err(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "node builtin '{}' is not available on the explicit Node API surface",
                    source
                ),
            ));
        }

        let base_dir = self
            .base_path
            .as_ref()
            .and_then(|path| path.parent())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let project_root =
            kali_npm::discover_project_root(&base_dir).unwrap_or_else(|| base_dir.clone());

        if self.resolve_relative_import_source(&base_dir, source) {
            return Ok(true);
        }

        let Some(resolved) = kali_npm::resolve_materialized_import_with_browser_context(
            project_root,
            source,
            self.api_surface == "browser",
        ) else {
            return Ok(false);
        };

        if self.api_surface != "node" {
            if let Ok(contents) = fs::read_to_string(&resolved) {
                if let Some(builtin) = kali_npm::source_mentions_node_only_host_api(&contents) {
                    return Err(Diagnostic::error(
                        e6::NODE_ONLY_HOST_APIS as u32,
                        format!(
                            "package uses Node-only host API '{}' in '{}' and falls outside the default standalone context; use the Phase-3 Node compatibility target",
                            builtin,
                            resolved.display()
                        ),
                    ));
                }
            }
        }

        Ok(true)
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Reference to a scope binding.
pub struct ScopeRef<'a> {
    scope: &'a Scope,
    name: String,
    binding_id: NodeId,
}

impl<'a> ScopeRef<'a> {
    pub fn binding_id(&self) -> NodeId {
        self.binding_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn scope(&self) -> &Scope {
        self.scope
    }
}

/// A lightweight type-checking facade.
#[derive(Default)]
pub struct TypeChecker {
    context: TypeContext,
    diagnostics: Vec<Diagnostic>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn clear_diagnostics(&mut self) {
        self.context.clear_diagnostics();
        self.diagnostics.clear();
    }

    pub fn check_type_annotation(&mut self, _node_id: NodeId, annotation: &str) {
        self.context.resolve_type_annotation_text(annotation);
        self.diagnostics.extend(self.context.drain_diagnostics());
    }

    pub fn check_node(&mut self, _node_id: NodeId) {
        let _ = &self.context;
    }

    pub fn typecheck(&mut self, _program_root: NodeId) -> Vec<Diagnostic> {
        self.diagnostics.extend(self.context.drain_diagnostics());
        self.diagnostics.clone()
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit()
}

fn is_type_annotation_keyword(ident: &str) -> bool {
    matches!(
        ident,
        "any"
            | "as"
            | "bigint"
            | "boolean"
            | "const"
            | "extends"
            | "false"
            | "infer"
            | "in"
            | "intrinsic"
            | "is"
            | "keyof"
            | "never"
            | "null"
            | "number"
            | "object"
            | "out"
            | "readonly"
            | "string"
            | "symbol"
            | "this"
            | "true"
            | "typeof"
            | "undefined"
            | "unique"
            | "unknown"
            | "void"
    )
}

fn is_property_name_context(chars: &[char], start: usize, end: usize) -> bool {
    if matches!(next_non_whitespace_char(chars, end), Some(':')) {
        return true;
    }

    if matches!(next_non_whitespace_char(chars, end), Some('?')) {
        let mut index = end + 1;
        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }
        return matches!(chars.get(index), Some(':'));
    }

    if start > 0 {
        let mut index = start;
        while index > 0 {
            index -= 1;
            if chars[index].is_whitespace() {
                continue;
            }
            return matches!(chars.get(index), Some('.'));
        }
    }

    false
}

fn next_non_whitespace_char(chars: &[char], mut index: usize) -> Option<char> {
    while index < chars.len() {
        let ch = chars[index];
        if !ch.is_whitespace() {
            return Some(ch);
        }
        index += 1;
    }
    None
}

fn skip_quoted_annotation_segment(chars: &[char], start: usize) -> usize {
    let quote = chars[start];
    let mut index = start + 1;
    while index < chars.len() {
        let ch = chars[index];
        if ch == '\\' {
            index = index.saturating_add(2);
            continue;
        }
        if ch == quote {
            return index + 1;
        }
        index += 1;
    }
    chars.len()
}

fn parse_numeric_literal_value(text: &str) -> Option<f64> {
    if let Some(stripped) = text.strip_suffix('n') {
        return stripped.parse::<f64>().ok();
    }
    text.parse::<f64>().ok()
}

fn builtin_globals() -> &'static [&'static str] {
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

fn node_builtin_globals() -> &'static [&'static str] {
    &["Buffer", "exports", "module", "process", "require"]
}

fn node_builtin_specifiers() -> &'static [&'static str] {
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

fn is_node_builtin_specifier(source: &str) -> bool {
    let normalized = source.strip_prefix("node:").unwrap_or(source);
    node_builtin_specifiers().contains(&normalized)
}

fn bind_builtin(scope: &mut Scope, next_binding_id: &mut u32, name: &str) {
    if scope.contains(name) {
        return;
    }

    scope.bind(name, NodeId::new(*next_binding_id));
    *next_binding_id = next_binding_id
        .checked_add(1)
        .expect("binding id overflow is unreachable in stage 1");
}

fn duplicate_binding(name: &str) -> Diagnostic {
    Diagnostic::error(
        e3::DUPLICATE_BINDING as u32,
        format!("duplicate binding '{}'", name),
    )
    .with_suggestion("rename the binding or move it into a nested scope")
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
