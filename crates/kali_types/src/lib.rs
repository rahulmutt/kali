//! Type system and name-resolution infrastructure for TypeScript/JavaScript.
//!
//! Stage 1.4 focuses on the deterministic scope model and name resolver that
//! downstream compiler stages use to catch unresolved names and duplicate
//! bindings before lowering.

use indexmap::IndexMap;
use kali_ast::{
    ArrayExpression, ArrowFunctionExpression, BlockStatement, BreakStatement, CallExpression,
    CatchClause, ClassBody, ClassDeclaration, ClassExpression, ContinueStatement,
    DecoratedExpression, DoWhileStatement, EnumDeclaration, EnumMember, Expression,
    ExpressionOrSpread, ExpressionStatement, ForInLefthand, ForInStatement, ForInit, ForOfLefthand,
    ForOfStatement, ForStatement, FunctionDeclaration, FunctionExpression, FunctionParam,
    IfStatement, ImportDeclaration, ImportExpression, ImportSpecifier, InterfaceDeclaration,
    JsxChild, JsxElement, JsxFragment, LabeledStatement, LiteralValue, MemberExpression, NodeId,
    ObjectExpression, ObjectProperty, OptionalChainExpression, OptionalChainInner, PropertyName,
    ReturnStatement, Statement, SwitchCase, SwitchStatement, TemplateLiteral, ThrowStatement,
    TryStatement, TypeAliasDeclaration, TypeAssertion, VariableDeclaration, WhileStatement,
    WithStatement,
};
use kali_error::{_error_codes::e3, _error_codes::e4, diagnostic::Diagnostic};
use std::path::{Path, PathBuf};

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
    pub static_values: IndexMap<String, String>,
}

impl Scope {
    pub fn new(scope_type: ScopeType, parent: Option<NodeId>) -> Self {
        Self {
            scope_type,
            parent,
            bindings: IndexMap::new(),
            static_values: IndexMap::new(),
        }
    }

    pub fn bind(&mut self, name: impl Into<String>, node_id: NodeId) {
        self.bindings.insert(name.into(), node_id);
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

    pub fn with_api_surface(api_surface: impl Into<String>) -> Self {
        let mut ctx = Self::new();
        ctx.set_api_surface(api_surface);
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
            Statement::ForOfStatement(ForOfStatement { left, right, body }) => {
                self.push_scope(ScopeType::Block);
                match left {
                    ForOfLefthand::VariableDeclaration(decl) => {
                        self.resolve_variable_declaration(decl)
                    }
                    ForOfLefthand::Expression(expr) => self.resolve_expression(expr),
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
            Statement::FunctionDeclaration(FunctionDeclaration { name, params, body }) => {
                self.bind_current_scope(name.clone());
                self.push_scope(ScopeType::Function);
                self.bind_name_list(params);
                self.resolve_block_body(body);
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
                if declaration.kind == "const" {
                    if let Some(value) = self.resolve_static_string_expression(init) {
                        if let Some(scope) = self.scopes.get_mut(&target_scope) {
                            scope.static_values.insert(declarator.id.clone(), value);
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
            Expression::UnaryExpression(expr) => self.resolve_expression(&expr.argument),
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
            Expression::UpdateExpression(expr) => self.resolve_expression(&expr.argument),
            Expression::AssignmentExpression(expr) => {
                self.resolve_expression(&expr.left);
                self.resolve_expression(&expr.right);
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

    fn resolve_import_expression(&mut self, expr: &ImportExpression) {
        self.resolve_expression(&expr.source);

        if let Some(source) = self.resolve_static_import_source(&expr.source) {
            if !self.resolve_import_source(&source) {
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
        }
    }

    fn resolve_static_import_source(&self, expression: &Expression) -> Option<String> {
        self.resolve_static_string_expression(expression)
    }

    fn resolve_static_string_expression(&self, expression: &Expression) -> Option<String> {
        match expression {
            Expression::Literal(LiteralValue::String(value)) => {
                Some(Self::normalize_import_segment(value))
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
        self.resolve_expression(&expr.callee);
        for arg in &expr.args {
            self.resolve_expression(arg);
        }
    }

    fn resolve_member_expression(&mut self, expr: &MemberExpression) {
        self.resolve_expression(&expr.object);
    }

    fn resolve_function_expression(&mut self, expr: &FunctionExpression) {
        self.push_scope(ScopeType::Function);
        if let Some(name) = &expr.id {
            self.bind_current_scope(name.clone());
        }
        self.bind_function_params(&expr.params);
        if let Some(body) = &expr.body {
            self.resolve_block_body(body);
        }
        self.pop_scope();
    }

    fn resolve_arrow_function(&mut self, expr: &ArrowFunctionExpression) {
        self.push_scope(ScopeType::Function);
        self.bind_function_params(&expr.params);
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
        if !self.resolve_import_source(&declaration.source) {
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
            if !self.resolve_import_source(source) {
                self.diagnostics.push(
                    Diagnostic::error(
                        e3::IMPORT_NOT_FOUND as u32,
                        format!("re-export source '{}' could not be resolved", source),
                    )
                    .with_suggestion("check the relative path or package specifier"),
                );
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
        self.resolve_expression(&expr.expression);
    }

    fn resolve_satisfies_expression(&mut self, expr: &kali_ast::SatisfiesExpression) {
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

    fn resolve_import_source(&self, source: &str) -> bool {
        if self.api_surface == "node" && is_node_builtin_specifier(source) {
            return true;
        }

        let base_dir = self
            .base_path
            .as_ref()
            .and_then(|path| path.parent())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let project_root =
            kali_npm::discover_project_root(&base_dir).unwrap_or_else(|| base_dir.clone());

        let candidate = base_dir.join(source);
        if candidate.exists() {
            return true;
        }

        let extensions = [
            "ts", "tsx", "js", "jsx", "mts", "cts", "d.ts", "d.mts", "d.cts",
        ];
        if extensions.iter().any(|extension| {
            let candidate = if source.ends_with(extension) {
                base_dir.join(source)
            } else {
                base_dir.join(format!("{}.{}", source, extension))
            };
            candidate.exists()
        }) {
            return true;
        }

        kali_npm::resolve_materialized_import(project_root, source).is_some()
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
        self.diagnostics.clear();
    }

    pub fn check_type_annotation(&mut self, _node_id: NodeId, annotation: &str) {
        self.context.resolve_type_annotation_text(annotation);
    }

    pub fn check_node(&mut self, _node_id: NodeId) {
        let _ = &self.context;
    }

    pub fn typecheck(&mut self, _program_root: NodeId) -> Vec<Diagnostic> {
        self.clear_diagnostics();
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

fn builtin_globals() -> &'static [&'static str] {
    &[
        "AbortController",
        "AbortSignal",
        "Array",
        "Boolean",
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
        "Event",
        "EventTarget",
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
        "parseFloat",
        "parseInt",
        "performance",
        "Promise",
        "Proxy",
        "queueMicrotask",
        "Reflect",
        "RegExp",
        "Request",
        "Response",
        "Set",
        "setInterval",
        "setTimeout",
        "String",
        "structuredClone",
        "Symbol",
        "TextDecoder",
        "TextEncoder",
        "URL",
        "URLSearchParams",
        "WeakMap",
        "WeakSet",
        "abs",
        "crypto",
    ]
}

fn node_builtin_globals() -> &'static [&'static str] {
    &["Buffer", "process"]
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
mod tests {
    use super::*;
    use kali_ast::{
        BinaryExpression, LiteralValue, ParenthesizedExpression, TypeAliasDeclaration,
        VariableDeclarator,
    };
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_scope_creation() {
        let scope = Scope::new(ScopeType::Global, None);
        assert_eq!(scope.scope_type, ScopeType::Global);
        assert!(scope.parent.is_none());
    }

    #[test]
    fn test_scope_binding() {
        let mut scope = Scope::new(ScopeType::Module, None);
        scope.bind("x", NodeId::new(1));
        scope.bind("y", NodeId::new(2));

        assert!(scope.contains("x"));
        assert!(scope.contains("y"));
        assert!(!scope.contains("z"));
    }

    #[test]
    fn test_type_context() {
        let mut ctx = TypeContext::new();
        assert!(ctx.is_defined("Kali"));
        assert!(!ctx.is_defined("x"));

        let _module = ctx.push_scope(ScopeType::Module);
        let binding = ctx.define("x");
        assert_eq!(binding.name(), "x");
        assert!(ctx.resolve_name("x").is_some());
    }

    #[test]
    fn test_type_annotation_resolution_accepts_known_names() {
        let mut ctx = TypeContext::new();
        let statements = vec![
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name: "Foo".to_string(),
                type_params: vec![],
                type_annotation: "string".to_string(),
            }),
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name: "Box".to_string(),
                type_params: vec![],
                type_annotation: "Foo | Array<string>".to_string(),
            }),
        ];

        let result = ctx.resolve_statements(&statements);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    }

    #[test]
    fn test_type_annotation_resolution_reports_unknown_names() {
        let mut ctx = TypeContext::new();
        let statements = vec![Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Box".to_string(),
            type_params: vec![],
            type_annotation: "Missing | string".to_string(),
        })];

        let result = ctx.resolve_statements(&statements);
        assert!(result
            .diagnostics
            .iter()
            .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)));
    }

    #[test]
    fn test_resolution_finds_bound_names() {
        let mut ctx = TypeContext::new();
        let statements = vec![Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        })];

        let result = ctx.resolve_statements(&statements);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert!(result.scopes.values().any(|scope| scope.contains("value")));
    }

    #[test]
    fn test_resolution_reports_unresolved_identifiers() {
        let mut ctx = TypeContext::new();
        let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("missing".to_string())),
        })];

        let result = ctx.resolve_statements(&statements);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code,
            Some(e3::UNDEFINED_IDENTIFIER as u32)
        );
    }

    #[test]
    fn test_resolution_reports_duplicate_bindings() {
        let mut ctx = TypeContext::new();
        let statements = vec![Statement::BlockStatement(BlockStatement {
            body: vec![
                Statement::VariableDeclaration(VariableDeclaration {
                    kind: "let".to_string(),
                    declarations: vec![VariableDeclarator {
                        id: "x".to_string(),
                        init: None,
                    }],
                }),
                Statement::VariableDeclaration(VariableDeclaration {
                    kind: "let".to_string(),
                    declarations: vec![VariableDeclarator {
                        id: "x".to_string(),
                        init: None,
                    }],
                }),
            ],
        })];

        let result = ctx.resolve_statements(&statements);
        assert!(result
            .diagnostics
            .iter()
            .any(|diag| diag.code == Some(e3::DUPLICATE_BINDING as u32)));
    }

    #[test]
    fn test_resolution_reports_missing_imports() {
        let mut ctx = TypeContext::with_base_path(".");
        let statements = vec![Statement::ImportDeclaration(ImportDeclaration {
            specifiers: vec![ImportSpecifier::Default("value".to_string())],
            source: "./definitely-missing-file.ts".to_string(),
        })];

        let result = ctx.resolve_statements_at_path(Some("."), &statements);
        assert_eq!(
            result.diagnostics[0].code,
            Some(e3::IMPORT_NOT_FOUND as u32)
        );
    }

    #[test]
    fn test_resolution_allows_node_builtin_imports_in_node_context() {
        let mut ctx = TypeContext::with_base_path_and_api_surface(".", "node");
        assert!(ctx.is_defined("process"));

        let statements = vec![Statement::ImportDeclaration(ImportDeclaration {
            specifiers: vec![ImportSpecifier::Default("fs".to_string())],
            source: "node:fs/promises".to_string(),
        })];

        let result = ctx.resolve_statements_at_path(Some("."), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_resolution_rejects_node_builtin_imports_outside_node_context() {
        let mut ctx = TypeContext::with_base_path(".");
        let statements = vec![Statement::ImportDeclaration(ImportDeclaration {
            specifiers: vec![ImportSpecifier::Default("fs".to_string())],
            source: "node:fs/promises".to_string(),
        })];

        let result = ctx.resolve_statements_at_path(Some("."), &statements);
        assert_eq!(
            result.diagnostics[0].code,
            Some(e3::IMPORT_NOT_FOUND as u32)
        );
    }

    #[test]
    fn test_resolution_allows_static_dynamic_import_targets() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.ts");
        fs::write(dir.path().join("lazy.ts"), "export const lazy = 7;").unwrap();
        fs::write(&source_path, "const lazy = import(\"./\" + \"lazy.ts\");").unwrap();

        let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: Expression::BinaryExpression(Box::new(BinaryExpression {
                    operator: "+".to_string(),
                    left: Expression::Literal(LiteralValue::String("./".to_string())),
                    right: Expression::Literal(LiteralValue::String("lazy.ts".to_string())),
                })),
            }))),
        })];

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_resolution_allows_const_bound_dynamic_import_targets() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.ts");
        fs::write(dir.path().join("lazy.ts"), "export const lazy = 7;").unwrap();
        fs::write(
            &source_path,
            "const name = \"lazy.ts\"; const root = \"./\"; import(root + name);",
        )
        .unwrap();

        let statements = vec![
            Statement::VariableDeclaration(VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "name".to_string(),
                    init: Some(Expression::Literal(LiteralValue::String(
                        "lazy.ts".to_string(),
                    ))),
                }],
            }),
            Statement::VariableDeclaration(VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "root".to_string(),
                    init: Some(Expression::Literal(LiteralValue::String("./".to_string()))),
                }],
            }),
            Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                    source: Expression::BinaryExpression(Box::new(BinaryExpression {
                        operator: "+".to_string(),
                        left: Expression::Identifier("root".to_string()),
                        right: Expression::Identifier("name".to_string()),
                    })),
                }))),
            }),
        ];

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_resolution_allows_parenthesized_dynamic_import_targets() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.ts");
        fs::write(dir.path().join("lazy.ts"), "export const lazy = 7;").unwrap();
        fs::write(
            &source_path,
            "const name = \"lazy.ts\"; const root = \"./\"; import((root + name));",
        )
        .unwrap();

        let statements = vec![
            Statement::VariableDeclaration(VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "name".to_string(),
                    init: Some(Expression::Literal(LiteralValue::String(
                        "lazy.ts".to_string(),
                    ))),
                }],
            }),
            Statement::VariableDeclaration(VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "root".to_string(),
                    init: Some(Expression::Literal(LiteralValue::String("./".to_string()))),
                }],
            }),
            Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                    source: Expression::ParenthesizedExpression(Box::new(
                        ParenthesizedExpression {
                            expression: Box::new(Expression::BinaryExpression(Box::new(
                                BinaryExpression {
                                    operator: "+".to_string(),
                                    left: Expression::Identifier("root".to_string()),
                                    right: Expression::Identifier("name".to_string()),
                                },
                            ))),
                        },
                    )),
                }))),
            }),
        ];

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_resolution_reports_unknown_dynamic_import_targets() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            "const name = \"lazy.ts\"; import(\"./\" + name);",
        )
        .unwrap();

        let statements = vec![
            Statement::VariableDeclaration(VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "name".to_string(),
                    init: Some(Expression::Literal(LiteralValue::String(
                        "lazy.ts".to_string(),
                    ))),
                }],
            }),
            Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                    source: Expression::BinaryExpression(Box::new(BinaryExpression {
                        operator: "+".to_string(),
                        left: Expression::Literal(LiteralValue::String("./".to_string())),
                        right: Expression::Identifier("name".to_string()),
                    })),
                }))),
            }),
        ];

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert_eq!(
            result.diagnostics[0].code,
            Some(e4::DYNAMIC_IMPORT_NOT_IN_LINKED_GRAPH as u32)
        );
    }

    #[test]
    fn test_resolution_uses_project_root_for_materialized_packages() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("kali.json"),
            r#"{
  "schemaVersion": 1,
  "devDependencies": {
    "@types/lodash": "1.0.0"
  }
}"#,
        )
        .unwrap();

        let src_dir = dir.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let source_path = src_dir.join("main.ts");
        fs::write(&source_path, "import lodash from 'lodash';\n").unwrap();

        let types_dir = dir.path().join("node_modules/@types/lodash");
        fs::create_dir_all(&types_dir).unwrap();
        fs::write(
            types_dir.join("package.json"),
            r#"{"name":"@types/lodash","types":"index.d.ts"}"#,
        )
        .unwrap();
        fs::write(types_dir.join("index.d.ts"), "declare const _: number;").unwrap();

        let mut ctx = TypeContext::with_base_path(&source_path);
        let statements = vec![Statement::ImportDeclaration(ImportDeclaration {
            specifiers: vec![ImportSpecifier::Default("lodash".to_string())],
            source: "lodash".to_string(),
        })];

        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
    }
}
