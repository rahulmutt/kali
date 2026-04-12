import KaliCore.Types

namespace KaliCore

abbrev Context := List (String × Ty)

namespace Context

/-- Lookup the first binding for a name in a typing context. -/
def lookup : Context → String → Option Ty
  | [], _ => none
  | (y, ty) :: Γ, x => if x = y then some ty else lookup Γ x

/-- Remove the first binding for a name in a typing context. -/
def remove : Context → String → Context
  | [], _ => []
  | (y, ty) :: Γ, x => if x = y then Γ else (y, ty) :: remove Γ x

end Context

/-- Runtime value predicate for the small-step semantics. -/
inductive Value : Expr → Prop where
  | lit : ∀ v, Value (Expr.ELit v)
  | closure : ∀ x ty body, Value (Expr.EFun x ty body)

/-- Capture-avoiding substitution used by the beta-reduction rule. -/
def subst (x : String) (replacement : Expr) : Expr → Expr
  | .ELit lit => .ELit lit
  | .EVar y => if y = x then replacement else .EVar y
  | .EFun y ty body =>
      if y = x then .EFun y ty body else .EFun y ty (subst x replacement body)
  | .EApp fn arg => .EApp (subst x replacement fn) (subst x replacement arg)
  | .ESeq e1 e2 => .ESeq (subst x replacement e1) (subst x replacement e2)
  | .EIf cond tBranch fBranch =>
      .EIf (subst x replacement cond) (subst x replacement tBranch) (subst x replacement fBranch)
  | .EAssign y e => .EAssign y (subst x replacement e)
  | .ETry e catchVar handler =>
      if catchVar = x then .ETry (subst x replacement e) catchVar handler
      else .ETry (subst x replacement e) catchVar (subst x replacement handler)
  | .EThrow e => .EThrow (subst x replacement e)

/-- Small-step evaluation for the bounded core model.  The soundness theorems only
cover the typed core fragment (literals, variables, functions, application,
sequencing, and conditionals); the extra forms are carried as syntax and runtime
stubs for later phase work. -/
inductive step : Expr → Expr → Prop where
  | app_left : ∀ {f f' a}, step f f' → step (.EApp f a) (.EApp f' a)
  | app_right : ∀ {f a a'}, Value f → step a a' → step (.EApp f a) (.EApp f a')
  | app_beta : ∀ {x ty body v}, Value v → step (.EApp (.EFun x ty body) v) (subst x v body)
  | seq_left : ∀ {e1 e1' e2}, step e1 e1' → step (.ESeq e1 e2) (.ESeq e1' e2)
  | seq_value : ∀ {v e2}, Value v → step (.ESeq v e2) e2
  | if_cond : ∀ {c c' t e}, step c c' → step (.EIf c t e) (.EIf c' t e)
  | if_true : ∀ {t e}, step (.EIf (.ELit (.bool true)) t e) t
  | if_false : ∀ {t e}, step (.EIf (.ELit (.bool false)) t e) e
  | throw_step : ∀ {e e'}, step e e' → step (.EThrow e) (.EThrow e')
  | try_step : ∀ {e e' x h}, step e e' → step (.ETry e x h) (.ETry e' x h)
  | try_catch : ∀ {v x h}, Value v → step (.ETry (.EThrow v) x h) (subst x v h)
  | try_value : ∀ {v x h}, Value v → step (.ETry v x h) v
  | assign_step : ∀ {x e e'}, step e e' → step (.EAssign x e) (.EAssign x e')
  | assign_value : ∀ {x v}, Value v → step (.EAssign x v) v

end KaliCore
