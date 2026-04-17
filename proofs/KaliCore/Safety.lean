import KaliCore.Types

namespace KaliCore

/-- Ownership classes used by the proof-backed memory-safety model. -/
inductive OwnershipClass where
  | stack
  | ownedHeap
  | sharedHeap
  | borrowed
  deriving Repr, DecidableEq

/-- A lightweight ownership annotation map for the proof-backed memory model. -/
abbrev OwnershipEnv := List (String × OwnershipClass)

/-- One cell in the modelled reference-counted heap. -/
structure RcCell where
  name : String
  owner : OwnershipClass
  refCount : Nat
  deriving Repr

/-- A bounded snapshot of the modelled ownership / RC state. -/
structure RcSnapshot where
  ownership : OwnershipEnv
  heap : List RcCell
  liveRefs : List String
  releasedRefs : List String
  deriving Repr

/-- A reference is owned when it has an explicit ownership annotation. -/
def hasOwnership (ownership : OwnershipEnv) (ref : String) : Prop :=
  ∃ owner, (ref, owner) ∈ ownership

/-- A reference is allocated when it has a live heap cell with a positive count. -/
def allocated (snapshot : RcSnapshot) (ref : String) : Prop :=
  ∃ cell, cell ∈ snapshot.heap ∧ cell.name = ref ∧ cell.refCount > 0

/-- A reference is live when it is both owned and allocated and has not been released. -/
def liveAnnotated (snapshot : RcSnapshot) (ref : String) : Prop :=
  hasOwnership snapshot.ownership ref ∧ allocated snapshot ref ∧ ref ∉ snapshot.releasedRefs

/-- Dangling references are live references that fail the ownership/heap/liveness test. -/
def DanglingReference (snapshot : RcSnapshot) : Prop :=
  ∃ ref, ref ∈ snapshot.liveRefs ∧ ¬ liveAnnotated snapshot ref

/-- The well-formedness condition for the modelled RC snapshot. -/
def WellFormed (snapshot : RcSnapshot) : Prop :=
  ∀ ref, ref ∈ snapshot.liveRefs → liveAnnotated snapshot ref

/-- In the proof-backed memory model, well-formed snapshots cannot contain dangling references. -/
theorem noDanglingReference (snapshot : RcSnapshot) (h : WellFormed snapshot) : ¬ DanglingReference snapshot := by
  intro hd
  rcases hd with ⟨ref, href, hbad⟩
  exact hbad (h ref href)

/-- Released references are not considered live in the model. -/
theorem releasedNotLive (snapshot : RcSnapshot) :
    ∀ ref, ref ∈ snapshot.releasedRefs → ¬ liveAnnotated snapshot ref := by
  intro ref href hlive
  exact hlive.2.2 href

/-- Well-formed snapshots keep the live and released reference sets disjoint. -/
theorem releasedNotLiveRef (snapshot : RcSnapshot) (h : WellFormed snapshot) :
    ∀ ref, ref ∈ snapshot.releasedRefs → ref ∉ snapshot.liveRefs := by
  intro ref href hlive
  have hliveAnnotated : liveAnnotated snapshot ref := h ref hlive
  exact hliveAnnotated.2.2 href

end KaliCore
