# Stage 1.3: Parser & AST Implementation Progress

## Status: IN PROGRESS

### Completed Tasks

#### AST Cleanup (Partially Complete)
- ✅ Removed NodeKind enum duplication - now using typed enums for ModuleItem, Statement
- ✅ Separated ASTBuilder from AST with proper lifecycle (builder → ast conversion)
- ✅ Added clean ImportDeclaration with ImportSpecifier types
- ✅ Added ExportDeclaration variants (NamedExport, ExportAll, Default, TypeExport)
- ✅ Fixed ASTBuilder API to use borrowed storage

#### Still Needed for AST
- ⏳ Expand Expression enum beyond basic types (needs ~50 variants)
  - Missing: ArrayLit, ObjectLit, Spread, Template, TaggedTemplate, NewExpr, MetaProperty
  - Missing: AwaitExpr, YieldExpr, YieldExpression, ClassExpr, SequenceExpr
  - Missing: OptionalChainExpr, LogicalExpr, AssignmentExpr, UpdateExpr
  - Missing: UnaryExpr variants (Delete, Void, Typeof, Plus, Minus, BitwiseNot, Not)
- ⏳ Expand Statement enum (needs more variety)
  - Missing: DoWhileStmt, SwitchStmt cases
  - Need to verify all ECMA-262 statements covered
- ⏳ Add TypeScript type variants
  - Missing: TsUnionType, TsIntersectionType, TsTupleType, TsArrayType
  - Missing: TsFunctionType, TsConstructorType, TsConditionalType, TsMappedType
  - Missing: TsIndexedAccessType, TsInferType, TsTypeQuery
- ⏳ Add JSX node types
  - Missing: JsxElement, JsxFragment, JsxOpeningElement, JsxClosingElement
  - Missing: JsxSelfClosingElement, JsxAttribute, JsxSpreadAttribute
  - Missing: JsxExpressionContainer, JsxText
- ⏳ Add Kali-specific annotations
  - Missing: EffectAnnotation, PureModifier
  - Missing: ErrorNode for error recovery

### Phase 2: Parser Implementation (Not Started)

#### Core Parser Infrastructure
- ⏳ Update ParseSource to consume Vec<Token> from lexer
- ⏳ Implement TokenIterator with peek/push-back capabilities  
- ⏳ Add parse error collection to ParseSource
- ⏳ Implement ASI (Automatic Semicolon Insertion) logic
- ⏳ Track strict mode state

#### Expression Parsing (Pratt/precedence climbing)
- ⏳ Implement primary expressions
  - Identifiers, literals, this, super
- ⏳ Implement member/access expressions
  - MemberExpr, CallExpr, NewExpr
- ⏳ Implement update expressions
  - Increment/decrement prefix and postfix
- ⏳ Implement call expressions
  - CallExpr with arguments array
- ⏳ Implement member expressions
  - MemberExpr with optional chain support
- ⏳ Implement member chaining
  - Multiple member access
- ⏳ Implement call chaining
  - Multiple function calls

#### Operator Precedence Parsing
- ⏳ Implement operator table
- ⏳ Implement unary/binary operators at correct precedences
- ⏳ Handle assignment operators (with LHS validation)
- ⏳ Handle logical operators (||, &&)
- ⏳ Handle logical AND/OR operators

#### Statement Parsing
- ⏳ Block statements
- ⏳ Variable declarations (var, let, const)
- ⏳ Function declarations
- ⏳ Class declarations
- ⏳ If/else statements
- ⏳ Switch statements
- ⏳ Loop constructs (for, for-in, for-of, while, do-while)
- ⏳ Try/catch/finally
- ⏳ Throw statements
- /continue statements
- ⏳ Return statements
- ⏳ Labeled statements
- ⏳ Debugger statements
- ⏳ Empty statements
- ⏳ Expression statements

#### Module System
- ⏳ Import declarations
  - Default imports
  - Named imports with aliases
  - Namespace imports
  - Side-effect imports
  - Type-only imports
- ⏳ Export declarations
  - Named exports
  - Default exports
  - Re-export all
  - Re-export with alias
  - Type exports

#### TypeScript Extensions
- ⏳ Type annotations on variables
- ⏳ Function parameter types
- ⏳ Return type annotations
- ⏳ Generic type parameters
- ⏳ Interface declarations
- ⏳ Type alias declarations
- ⏳ Enum declarations
- ⏳ Decorators (@decorator)
- ⏳ Parameter properties
- ⏳ Abstract classes/methods
- ⏳ Override modifier
- ⏳ Satisfies operator
- ⏳ Non-null assertion (!)
- ⏳ Triple-slash directives

#### JSX Support
- ⏳ JSX element parsing
- ⏳ JSX fragment parsing
- ⏳ JSX attributes and spread attributes
- ⏳ JSX expression containers
- ⏳ Handle `<T>` vs JSX tag ambiguity

#### Error Recovery
- ⏳ Implement panic-mode recovery
- ⏳ Define synchronization tokens
- ⏳ Create ErrorNode placeholder
- ⏳ Collect and report E2xxx diagnostics

### Phase 3: Testing
- ⏳ Create test fixture files (JS, TS, TSX, D.TS)
- ⏳ Write snapshot tests using insta
- ⏳ Write error recovery tests
- ⏳ Document error codes and expected behavior

### Files to Modify
1. `crates/kali_ast/src/lib.rs` - Expand with more expression types
2. `crates/kali_parser/src/lib.rs` - Implement actual parser
3. `crates/kali_lexer/src/lib.rs` - Verify Token stream matches parser expectations
4. `stats/15-errors.md` - Add E2xxx error codes (if needed)

### Dependencies to Verify
1. `crates/kali_common` - Span structure used by AST
2. `crates/kali_error` - Diagnostic emission for errors
3. `crates/kali_lexer` - Token stream for parser input

### Recommended Order
1. ✅ Clean up AST structure (DONE for main skeleton)
2. ⏳ Expand AST with all required node types
3. ⏳ Implement minimal parser (expressions + basic statements)
4. ⏳ Add statement parsing
5. ⏳ Add TypeScript support
6. ⏳ Add JSX support
7. ⏳ Add error recovery
8. ⏳ Write tests

### Next Immediate Task
Start implementing basic expression parsing with primary expressions and member expressions, then add the precedence climbing/Pratt parsing for operators.

### Notes
- Parser uses recursive descent with Pratt parsing for expressions
- Per-file arena allocation pattern planned but not yet implemented
- Error recovery uses panic-mode with synchronization tokens
- JSX ambiguity handled via file extension checking
