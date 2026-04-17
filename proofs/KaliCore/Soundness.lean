import KaliCore.Semantics

namespace KaliCore

/-- Typing judgment for the bounded core fragment modelled in Lean. The current
proof boundary keeps this intentionally small: literals, variables, and closed
functions are enough to exercise the value/progress and preservation shape while
leaving richer control-flow and application reasoning to later mechanisation.
-/
inductive Typing : Context → Expr → Ty → Prop where
  | lit : ∀ {Γ v}, Typing Γ (.ELit v) (litTy v)
  | var : ∀ {Γ x T}, Context.lookup Γ x = some T → Typing Γ (.EVar x) T
  | lam : ∀ {Γ x ty body retTy}, Typing Γ body retTy → Typing Γ (.EFun x ty body) (.TFun [ty] retTy)

/-- Progress for the bounded core calculus. -/
theorem progress : ∀ (e : Expr) (T : Ty), Typing [] e T → Value e ∨ ∃ e', step e e' := by
  intro e T hty
  cases hty with
  | lit =>
      exact Or.inl (Value.lit _)
  | var hlookup =>
      simp [Context.lookup] at hlookup
  | lam hbody =>
      exact Or.inl (Value.closure _ _ _)

/-- Preservation for the bounded core calculus. -/
theorem preservation : ∀ (e e' : Expr) (T : Ty), Typing [] e T → step e e' → Typing [] e' T := by
  intro e e' T hty hstep
  cases hstep <;> cases hty

end KaliCore
