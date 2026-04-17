import KaliCore.Types

namespace KaliIR

open KaliCore

/-- A provisional HIR model used to anchor later lowering-correctness work. -/
inductive HIRExpr where
  | core : KaliCore.Expr → HIRExpr
  | let1 : String → HIRExpr → HIRExpr → HIRExpr
  | seq : HIRExpr → HIRExpr → HIRExpr
  | if : HIRExpr → HIRExpr → HIRExpr → HIRExpr
  | assign : String → HIRExpr → HIRExpr
  | throw : HIRExpr → HIRExpr
  | tr : HIRExpr → String → HIRExpr → HIRExpr
  deriving Repr

/-- Provisional lowering from HIR into the small core expression language. -/
def lower : HIRExpr → KaliCore.Expr
  | .core e => e
  | .let1 x value body => .EApp (.EFun x .TAny (lower body)) (lower value)
  | .seq e1 e2 => .ESeq (lower e1) (lower e2)
  | .if c t e => .EIf (lower c) (lower t) (lower e)
  | .assign x e => .EAssign x (lower e)
  | .throw e => .EThrow (lower e)
  | .tr e x h => .ETry (lower e) x (lower h)

@[simp] theorem lower_core (e : KaliCore.Expr) : lower (.core e) = e := rfl

@[simp] theorem lower_let1 (x : String) (value body : HIRExpr) :
    lower (.let1 x value body) = .EApp (.EFun x .TAny (lower body)) (lower value) := rfl

@[simp] theorem lower_seq (e1 e2 : HIRExpr) :
    lower (.seq e1 e2) = .ESeq (lower e1) (lower e2) := rfl

@[simp] theorem lower_if (c t e : HIRExpr) :
    lower (.if c t e) = .EIf (lower c) (lower t) (lower e) := rfl

@[simp] theorem lower_assign (x : String) (e : HIRExpr) :
    lower (.assign x e) = .EAssign x (lower e) := rfl

@[simp] theorem lower_throw (e : HIRExpr) :
    lower (.throw e) = .EThrow (lower e) := rfl

@[simp] theorem lower_tr (e : HIRExpr) (x : String) (h : HIRExpr) :
    lower (.tr e x h) = .ETry (lower e) x (lower h) := rfl

end KaliIR
