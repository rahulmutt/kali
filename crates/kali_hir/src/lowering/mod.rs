//! AST → HIR lowering passes (one `impl HirLowerer` per responsibility).

mod expression;
mod function;
mod module;
mod object;
mod statement;
