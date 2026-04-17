import KaliCore.Types

namespace KaliIR

open KaliCore

/-- A provisional HIR model used to anchor later lowering-correctness work. -/
inductive HIRExpr where
  | core : KaliCore.Expr → HIRExpr
  | let1 : String → HIRExpr → HIRExpr → HIRExpr
  | seq : HIRExpr → HIRExpr → HIRExpr
  | if : HIRExpr → HIRExpr → HIRExpr → HIRExpr
  deriving Repr

/-- Provisional lowering from HIR into the small core expression language. -/
def lower : HIRExpr → KaliCore.Expr
  | .core e => e
  | .let1 x value body => .EApp (.EFun x .TAny (lower body)) (lower value)
  | .seq e1 e2 => .ESeq (lower e1) (lower e2)
  | .if c t e => .EIf (lower c) (lower t) (lower e)

@[simp] theorem lower_core (e : KaliCore.Expr) : lower (.core e) = e := rfl

end KaliIR
