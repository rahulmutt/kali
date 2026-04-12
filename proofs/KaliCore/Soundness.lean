import KaliCore.Semantics

namespace KaliCore

/-- Typing judgment for the bounded core fragment. -/
inductive Typing : Context → Expr → Ty → Prop where
  | lit : ∀ {Γ v}, Typing Γ (.ELit v) (litTy v)
  | var : ∀ {Γ x T}, Context.lookup Γ x = some T → Typing Γ (.EVar x) T
  | fun : ∀ {Γ x ty body retTy}, Typing ((x, ty) :: Γ) body retTy → Typing Γ (.EFun x ty body) (.TFun [ty] retTy)
  | app : ∀ {Γ fn arg paramTy retTy}, Typing Γ fn (.TFun [paramTy] retTy) → Typing Γ arg paramTy → Typing Γ (.EApp fn arg) retTy
  | seq : ∀ {Γ e1 e2 ty2}, Typing Γ e1 .TVoid → Typing Γ e2 ty2 → Typing Γ (.ESeq e1 e2) ty2
  | if : ∀ {Γ c t e ty}, Typing Γ c .TBool → Typing Γ t ty → Typing Γ e ty → Typing Γ (.EIf c t e) ty

/-- Progress for the bounded core calculus. The theorem is stated for the
closed fragment captured by the typing rules above; the additional runtime forms
carried in `Expr` are excluded by the typing judgment and remain staging stubs. -/
theorem progress : ∀ (e : Expr) (T : Ty), Typing [] e T → Value e ∨ ∃ e', step e e' := by
  sorry

/-- Preservation for the bounded core calculus. -/
theorem preservation : ∀ (e e' : Expr) (T : Ty), Typing [] e T → step e e' → Typing [] e' T := by
  sorry

end KaliCore
