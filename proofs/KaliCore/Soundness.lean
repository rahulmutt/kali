import KaliCore.Semantics

namespace KaliCore

/-- Typing judgment for the bounded core fragment modelled in Lean. The current
proof boundary now covers literals, variables, closed functions, application,
sequencing, and conditionals. That keeps the progress and preservation story
mechanised while still leaving assignment, exceptions, and the wider memory /
lowering proofs for later work.
-/
inductive Typing : Context → Expr → Ty → Prop where
  | lit : ∀ {Γ v}, Typing Γ (.ELit v) (litTy v)
  | var : ∀ {Γ x T}, Context.lookup Γ x = some T → Typing Γ (.EVar x) T
  | lam : ∀ {Γ x ty body retTy}, Typing [] body retTy → Typing Γ (.EFun x ty body) (.TFun [ty] retTy)
  | app : ∀ {Γ x ty body a argTy retTy}, Typing [] body retTy → Typing Γ a argTy → Typing Γ (.EApp (.EFun x ty body) a) retTy
  | seq : ∀ {Γ e1 e2 t1 t2}, Typing Γ e1 t1 → Typing Γ e2 t2 → Typing Γ (.ESeq e1 e2) t2
  | ite : ∀ {Γ c t f tRet}, Typing Γ c .TBool → Typing Γ t tRet → Typing Γ f tRet → Typing Γ (.EIf c t f) tRet
  | assign : ∀ {Γ name e t}, Typing Γ e t → Typing Γ (.EAssign name e) t
  | tr : ∀ {Γ e name h t}, Typing Γ e t → Typing Γ h t → Typing Γ (.ETry e name h) t

/-- Closed typed expressions do not change under the substitution used by beta
reduction. -/
theorem subst_closed : ∀ {e : Expr} {T : Ty} {x : String} {v : Expr}, Typing [] e T → subst x v e = e := by
  intro e
  induction e with
  | ELit lit =>
      intro T x v hty
      cases hty
      rfl
  | EVar name =>
      intro T x v hty
      cases hty with
      | var hlookup =>
          have hfalse : False := by
            simp [Context.lookup] at hlookup
          exact False.elim hfalse
  | EFun name ty body ih =>
      intro T x v hty
      cases hty with
      | lam hbody =>
          rfl
  | EApp fn arg ihFn ihArg =>
      intro T x v hty
      cases hty with
      | app hbody harg =>
          simp [subst, ihArg harg]
  | ESeq e1 e2 ih1 ih2 =>
      intro T x v hty
      cases hty with
      | seq h1 h2 =>
          simp [subst, ih1 h1, ih2 h2]
  | EIf c t f ihC ihT ihF =>
      intro T x v hty
      cases hty with
      | ite hcond ht hf =>
          simp [subst, ihC hcond, ihT ht, ihF hf]
  | EAssign name e ih =>
      intro T x v hty
      cases hty with
      | assign h =>
          simp [subst, ih h]
  | ETry e name h ihE ihH =>
      intro T x v hty
      cases hty with
      | tr hbody hhandler =>
          simp [subst, ihE hbody, ihH hhandler]
  | EThrow e ih =>
      intro T x v hty
      cases hty

/-- Progress for the bounded core calculus. -/
theorem progress : ∀ (e : Expr) (T : Ty), Typing [] e T → Value e ∨ ∃ e', step e e' := by
  intro e
  induction e with
  | ELit lit =>
      intro T hty
      exact Or.inl (Value.lit _)
  | EVar name =>
      intro T hty
      cases hty with
      | var hlookup =>
          have hfalse : False := by
            simp [Context.lookup] at hlookup
          exact False.elim hfalse
  | EFun name ty body ih =>
      intro T hty
      exact Or.inl (Value.closure _ _ _)
  | EApp fn arg ihFn ihArg =>
      intro T hty
      cases hty with
      | app hbody harg =>
          rcases ihArg _ harg with hargval | ⟨a', ha⟩
          · exact Or.inr ⟨_, step.app_beta hargval⟩
          · exact Or.inr ⟨_, step.app_right (Value.closure _ _ _) ha⟩
  | ESeq e1 e2 ih1 ih2 =>
      intro T hty
      cases hty with
      | seq h1 h2 =>
          rcases ih1 _ h1 with h1val | ⟨e1', he1⟩
          · exact Or.inr ⟨_, step.seq_value h1val⟩
          · exact Or.inr ⟨_, step.seq_left he1⟩
  | EIf c t f ihC ihT ihF =>
      intro T hty
      cases hty with
      | ite hcond ht hf =>
          rcases ihC _ hcond with hcondval | ⟨c', hc'⟩
          · cases hcondval with
            | lit lit =>
                cases lit with
                | bool b =>
                    cases b with
                    | true => exact Or.inr ⟨_, step.if_true⟩
                    | false => exact Or.inr ⟨_, step.if_false⟩
                | number n =>
                    cases hcond
                | bigint n =>
                    cases hcond
                | string s =>
                    cases hcond
                | symbol s =>
                    cases hcond
                | null =>
                    cases hcond
                | undef =>
                    cases hcond
            | closure x ty body =>
                cases hcond
          · exact Or.inr ⟨_, step.if_cond hc'⟩
  | EAssign name e ih =>
      intro T hty
      cases hty with
      | assign hrhs =>
          rcases ih _ hrhs with hval | ⟨e', he⟩
          · exact Or.inr ⟨_, step.assign_value hval⟩
          · exact Or.inr ⟨_, step.assign_step he⟩
  | ETry e name h ihE ihH =>
      intro T hty
      cases hty with
      | tr hbody hhandler =>
          rcases ihE _ hbody with hval | ⟨e', he⟩
          · exact Or.inr ⟨_, step.try_value hval⟩
          · exact Or.inr ⟨_, step.try_step he⟩
  | EThrow e ih =>
      intro T hty
      cases hty

/-- Preservation for the bounded core calculus. -/
theorem preservation : ∀ (e e' : Expr) (T : Ty), Typing [] e T → step e e' → Typing [] e' T := by
  intro e
  induction e with
  | ELit lit =>
      intro e' T hty hstep
      cases hstep
  | EVar name =>
      intro e' T hty hstep
      cases hty with
      | var hlookup =>
          have hfalse : False := by
            simp [Context.lookup] at hlookup
          exact False.elim hfalse
  | EFun name ty body ih =>
      intro e' T hty hstep
      cases hty with
      | lam hbody =>
          cases hstep
  | EApp fn arg ihFn ihArg =>
      intro e' T hty hstep
      cases hty with
      | app hbody harg =>
          cases hstep with
          | app_right hv hs =>
              exact Typing.app hbody (ihArg _ _ harg hs)
          | app_beta hv =>
              simpa [subst_closed hbody] using hbody
          | app_left hs =>
              cases hs
  | ESeq e1 e2 ih1 ih2 =>
      intro e' T hty hstep
      cases hty with
      | seq h1 h2 =>
          cases hstep with
          | seq_left hs =>
              exact Typing.seq (ih1 _ _ h1 hs) h2
          | seq_value hv =>
              exact h2
  | EIf c t f ihC ihT ihF =>
      intro e' T hty hstep
      cases hty with
      | ite hcond ht hf =>
          cases hstep with
          | if_cond hs =>
              exact Typing.ite (ihC _ _ hcond hs) ht hf
          | if_true =>
              exact ht
          | if_false =>
              exact hf
  | EAssign name e ih =>
      intro e' T hty hstep
      cases hty with
      | assign hrhs =>
          cases hstep with
          | assign_step hs =>
              exact Typing.assign (ih _ _ hrhs hs)
          | assign_value hv =>
              exact hrhs
  | ETry e name h ihE ihH =>
      intro e' T hty hstep
      cases hty with
      | tr hbody hhandler =>
          cases hstep with
          | try_step hs =>
              exact Typing.tr (ihE _ _ hbody hs) hhandler
          | try_catch hv =>
              cases hbody
          | try_value hv =>
              exact hbody
  | EThrow e ih =>
      intro e' T hty hstep
      cases hty

end KaliCore
