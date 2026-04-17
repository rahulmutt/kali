import KaliIR.HIRModel

namespace KaliIR

open KaliCore

/--
A small proof-backed lowering-correctness fragment for the provisional HIR
model. The current theorem family covers the structural HIR forms already
present in `HIRModel`, including assignment, bare throw, and try/catch,
plus the simple `let1` beta case where the body is a core expression. That
keeps the model honest about the subset we have mechanized while still giving
a real preservation bridge from HIR to the core semantics.
-/
inductive Step : HIRExpr → HIRExpr → Prop where
  | core : ∀ {e e'}, KaliCore.step e e' → Step (.core e) (.core e')
  | let1_value : ∀ {x v v' body}, Step v v' → Step (.let1 x v body) (.let1 x v' body)
  | let1_beta : ∀ {x v body}, KaliCore.Value v → Step (.let1 x (.core v) (.core body)) (.core (KaliCore.subst x v body))
  | seq_left : ∀ {e1 e1' e2}, Step e1 e1' → Step (.seq e1 e2) (.seq e1' e2)
  | seq_value : ∀ {v e2}, KaliCore.Value v → Step (.seq (.core v) e2) e2
  | if_cond : ∀ {c c' t e}, Step c c' → Step (.if c t e) (.if c' t e)
  | if_true : ∀ {t e}, Step (.if (.core (.ELit (.bool true))) t e) t
  | if_false : ∀ {t e}, Step (.if (.core (.ELit (.bool false))) t e) e
  | assign_step : ∀ {x e e'}, Step e e' → Step (.assign x e) (.assign x e')
  | assign_value : ∀ {x v}, KaliCore.Value v → Step (.assign x (.core v)) (.core (.EAssign x v))
  | throw_step : ∀ {e e'}, Step e e' → Step (.throw e) (.throw e')
  | tr_step : ∀ {e e' x h}, Step e e' → Step (.tr e x h) (.tr e' x h)
  | tr_catch : ∀ {v x h}, KaliCore.Value v → Step (.tr (.core (.EThrow v)) x (.core h)) (.core (KaliCore.subst x v h))
  | tr_value : ∀ {v x h}, KaliCore.Value v → Step (.tr (.core v) x h) (.core v)

/-- Reflexive transitive closure over the current HIR step relation. -/
inductive Steps : HIRExpr → HIRExpr → Prop where
  | refl : ∀ {h}, Steps h h
  | step : ∀ {h h' h''}, Step h h' → Steps h' h'' → Steps h h''

/-- Reflexive transitive closure over the current core step relation. -/
inductive CoreSteps : KaliCore.Expr → KaliCore.Expr → Prop where
  | refl : ∀ {e}, CoreSteps e e
  | step : ∀ {e e' e''}, KaliCore.step e e' → CoreSteps e' e'' → CoreSteps e e''

/-- Lowering preserves the small-step relation for the current HIR subset. -/
theorem lower_preserves_step : ∀ {h h' : HIRExpr}, Step h h' → KaliCore.step (lower h) (lower h') := by
  intro h h' hs
  induction hs with
  | core hstep =>
      simpa [lower] using hstep
  | let1_value (x := x) (v := v) (v' := v') (body := body) hstep ih =>
      simpa [lower] using
        (KaliCore.step.app_right
          (f := .EFun x .TAny (lower body))
          (a := lower v)
          (a' := lower v')
          (Value.closure x .TAny (lower body))
          ih)
  | let1_beta (x := x) (v := v) (body := body) hv =>
      simpa [lower] using (KaliCore.step.app_beta (x := x) (ty := .TAny) (body := body) (v := v) hv)
  | seq_left (e1 := e1) (e1' := e1') (e2 := e2) hstep ih =>
      simpa [lower] using (KaliCore.step.seq_left ih)
  | seq_value (v := v) (e2 := e2) hv =>
      simpa [lower] using (KaliCore.step.seq_value (v := v) (e2 := lower e2) hv)
  | if_cond (c := c) (c' := c') (t := t) (e := e) hstep ih =>
      simpa [lower] using (KaliCore.step.if_cond ih)
  | if_true (t := t) (e := e) =>
      simpa [lower] using (KaliCore.step.if_true (t := lower t) (e := lower e))
  | if_false (t := t) (e := e) =>
      simpa [lower] using (KaliCore.step.if_false (t := lower t) (e := lower e))
  | assign_step (x := x) (e := e) (e' := e') hstep ih =>
      simpa [lower] using (KaliCore.step.assign_step (x := x) (e := lower e) (e' := lower e') ih)
  | assign_value (x := x) (v := v) hv =>
      simpa [lower] using (KaliCore.step.assign_value (x := x) (v := v) hv)
  | throw_step (e := e) (e' := e') hstep ih =>
      simpa [lower] using (KaliCore.step.throw_step ih)
  | tr_step (e := e) (e' := e') (x := x) (h := h) hstep ih =>
      simpa [lower] using (KaliCore.step.try_step (x := x) (h := lower h) ih)
  | tr_catch (v := v) (x := x) (h := h) hv =>
      simpa [lower] using (KaliCore.step.try_catch (x := x) (h := lower h) hv)
  | tr_value (v := v) (x := x) (h := h) hv =>
      simpa [lower] using (KaliCore.step.try_value (x := x) (h := lower h) hv)

/-- Lowering also preserves finite HIR traces in the current model. -/
theorem lower_preserves_steps : ∀ {h h' : HIRExpr}, Steps h h' → CoreSteps (lower h) (lower h') := by
  intro h h' hs
  induction hs with
  | refl =>
      exact CoreSteps.refl
  | step (h := h₁) (h' := h₂) (h'' := h₃) hstep ih =>
      exact CoreSteps.step (lower_preserves_step hstep) ih

end KaliIR
