# Stage 1.3 Implementation: Expression Parsing Core

## Overview

This document tracks the implementation of the expression parser for Stage 1.3 of Kali's compiler.

## Current State

The `Expression` enum in `crates/kali_ast/src/lib.rs` has ~70 variants but they need proper
field definitions, spans, and the parser implementation needs to be completed.

## Implementation Plan

### Priority 1: Core Expression Types (Highest Priority)

Implement comprehensive field definitions for existing Expression variants:

1. **Identifier expressions** - add name field
   ```rust
   Identifier { name: String }
   ```

2. **Literal expressions** - add value field with proper enum
   ```rust
   Literal { literal: LiteralValue }
   ```

3. **Operator expressions** - add proper span and operand fields
   ```rust
   BinaryExpression { 
       operator: BinaryOperator,
       left: Box<Expression>,
       right: Box<Expression>
   }
   ```

4. **Call/Member/Function expressions** - add proper fields
   ```rust
   CallExpression { 
       callee: Expression,
       args: Vec<Expression>,
       span: Span
   }
   ```

### Priority 2: Parser Core Implementation

Implement recursive descent parsing with Pratt parsing:

1. **TokenStream extension** - add peeking, advancing, checking methods
2. **Primary expression parsing** - identifiers, literals, grouping, function calls
3. **Unary expression parsing** - prefix/postfix operators
4. **Binary operator parsing** - precedence climbing algorithm
5. **Function parsing** - function declarations, arrow functions
6. **Object/array literal parsing** - with support for spread/rest

### Priority 3: Error Recovery

1. **Simple error recovery** - skip to next statement boundary
2. **PANICMode parsing** - resync after parse errors
3. **Diagnostic emission** - proper E2xxx codes

## Next Steps

Start with expression type definitions, then implement primary expression parsing,
then move to operators and advanced expressions.

