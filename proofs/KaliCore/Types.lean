namespace KaliCore

/-- Literal values that appear in the bounded core calculus. -/
inductive LitVal where
  | bool : Bool → LitVal
  | number : Int → LitVal
  | bigint : Int → LitVal
  | string : String → LitVal
  | symbol : String → LitVal
  | null : LitVal
  | undef : LitVal
  deriving Repr

/-- Core Kali types for the provisional Lean model. -/
inductive Ty where
  | TNever : Ty
  | TUnknown : Ty
  | TAny : Ty
  | TVoid : Ty
  | TUndef : Ty
  | TNull : Ty
  | TBool : Ty
  | TNumber : Ty
  | TBigInt : Ty
  | TString : Ty
  | TSymbol : Ty
  | TLit : LitVal → Ty
  | TFun : List Ty → Ty → Ty
  | TObj : List (String × Ty) → Ty
  | TUnion : Ty → Ty → Ty
  | TInter : Ty → Ty → Ty
  deriving Repr

/-- The core surface expression grammar used by the proof model. -/
inductive Expr where
  | ELit : LitVal → Expr
  | EVar : String → Expr
  | EFun : String → Ty → Expr → Expr
  | EApp : Expr → Expr → Expr
  | ESeq : Expr → Expr → Expr
  | EIf : Expr → Expr → Expr → Expr
  | EAssign : String → Expr → Expr
  | ETry : Expr → String → Expr → Expr
  | EThrow : Expr → Expr
  deriving Repr

/-- Type associated with a literal value. -/
def litTy : LitVal → Ty
  | .bool _ => .TBool
  | .number _ => .TNumber
  | .bigint _ => .TBigInt
  | .string _ => .TString
  | .symbol _ => .TSymbol
  | .null => .TNull
  | .undef => .TVoid

end KaliCore
