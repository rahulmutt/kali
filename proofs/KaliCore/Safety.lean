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

/-- Release a live reference from the snapshot by moving it out of the live set. -/
def releaseRef (snapshot : RcSnapshot) (ref : String) : RcSnapshot :=
  { snapshot with
    liveRefs := snapshot.liveRefs.filter (fun r => decide (r ≠ ref))
    releasedRefs := ref :: snapshot.releasedRefs
  }

/-- Release a live reference while also decrementing the targeted heap cell's
reference count. This models the current local RC update slice without yet
claiming the fuller freeing story. -/
def releaseAndDecrement (snapshot : RcSnapshot) (ref : String) : RcSnapshot :=
  { snapshot with
    liveRefs := snapshot.liveRefs.filter (fun r => decide (r ≠ ref))
    releasedRefs := ref :: snapshot.releasedRefs
    heap := snapshot.heap.map (fun cell =>
      if cell.name = ref then { cell with refCount := cell.refCount - 1 } else cell)
  }

/-- Release a live reference, decrement its target cell, and collect any zero-count
heap cells. This adds the local freeing step that the later Stage 4.2 memory
story will eventually widen further. -/
def releaseAndCollect (snapshot : RcSnapshot) (ref : String) : RcSnapshot :=
  let decremented := releaseAndDecrement snapshot ref
  { decremented with
    heap := decremented.heap.filter (fun cell => cell.refCount > 0)
  }

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

/-- Well-formed snapshots keep each live reference anchored in ownership and allocation. -/
theorem liveRefsAreOwnedAndAllocated (snapshot : RcSnapshot) (h : WellFormed snapshot) :
    ∀ ref, ref ∈ snapshot.liveRefs → hasOwnership snapshot.ownership ref ∧ allocated snapshot ref := by
  intro ref href
  exact ⟨(h ref href).1, (h ref href).2.1⟩

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

/-- Releasing a live reference preserves the well-formedness of the remaining live set. -/
theorem releasePreservesWellFormed (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) :
    WellFormed (releaseRef snapshot ref) := by
  intro r hr
  simp [releaseRef] at hr ⊢
  rcases hr with ⟨hrLive, hneq⟩
  have hannotated : liveAnnotated snapshot r := h r hrLive
  constructor
  · exact hannotated.1
  · constructor
    · exact hannotated.2.1
    · simpa [hneq] using hannotated.2.2

/-- A release-and-decrement step preserves the well-formedness of the remaining
live set because only the released reference's heap cell is updated. -/
theorem releaseAndDecrementPreservesWellFormed (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) :
    WellFormed (releaseAndDecrement snapshot ref) := by
  intro r hr
  simp [releaseAndDecrement] at hr ⊢
  rcases hr with ⟨hrLive, hneq⟩
  have hannotated : liveAnnotated snapshot r := h r hrLive
  constructor
  · exact hannotated.1
  · constructor
    · rcases hannotated.2.1 with ⟨cell, hmem, hname, hpos⟩
      refine ⟨cell, ?_, hname, hpos⟩
      have hcell : (fun cell => if cell.name = ref then { cell with refCount := cell.refCount - 1 } else cell) cell = cell := by
        simp [hname, hneq]
      exact List.mem_map.mpr ⟨cell, hmem, hcell⟩
    · simpa [hneq] using hannotated.2.2

/-- A release-and-decrement step still records the released reference. -/
theorem releaseAndDecrementRecorded (snapshot : RcSnapshot) (ref : String) :
    ref ∈ (releaseAndDecrement snapshot ref).releasedRefs := by
  simp [releaseAndDecrement]

/-- A release-and-decrement step decrements the targeted heap cell when it is present. -/
theorem releaseAndDecrementDecrementsTargetCell (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.name = ref →
      { cell with refCount := cell.refCount - 1 } ∈ (releaseAndDecrement snapshot ref).heap := by
  intro cell hmem hname
  exact List.mem_map.mpr ⟨cell, hmem, by simp [hname]⟩

/-- A release-and-decrement step zeroes the target cell when the released reference was the last live count. -/
theorem releaseAndDecrementZeroesLastTargetCell (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.name = ref → cell.refCount = 1 →
      { cell with refCount := cell.refCount - 1 } ∈ (releaseAndDecrement snapshot ref).heap ∧
      { cell with refCount := cell.refCount - 1 }.refCount = 0 := by
  intro cell hmem hname hcount
  constructor
  · exact List.mem_map.mpr ⟨cell, hmem, by simp [hname]⟩
  · simp [hcount]

/-- A release-and-collect step removes zero-count cells after the decrement pass. -/
theorem releaseAndCollectRemovesZeroCountCells (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.name = ref → cell.refCount = 1 →
      { cell with refCount := cell.refCount - 1 } ∉ (releaseAndCollect snapshot ref).heap := by
  intro cell hmem hname hcount hpresent
  simp [releaseAndCollect, releaseAndDecrement, hname, hcount] at hpresent

/-- A release-and-collect step preserves the well-formedness of the remaining
live set because zero-count cells are collected after the decrement pass. -/
theorem releaseAndCollectPreservesWellFormed (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) :
    WellFormed (releaseAndCollect snapshot ref) := by
  intro r hr
  have hr' : r ∈ snapshot.liveRefs ∧ r ≠ ref := by
    simpa [releaseAndCollect, releaseAndDecrement] using hr
  have hrLive : r ∈ snapshot.liveRefs := hr'.1
  have hneq : r ≠ ref := hr'.2
  have hannotated : liveAnnotated snapshot r := h r hrLive
  constructor
  · exact hannotated.1
  · constructor
    · rcases hannotated.2.1 with ⟨cell, hmem, hname, hpos⟩
      refine ⟨cell, ?_, hname, hpos⟩
      have hcell : (fun cell => if cell.name = ref then { cell with refCount := cell.refCount - 1 } else cell) cell = cell := by
        simp [hname, hneq]
      have hmem' : cell ∈ (releaseAndDecrement snapshot ref).heap :=
        List.mem_map.mpr ⟨cell, hmem, hcell⟩
      exact List.mem_filter.mpr ⟨hmem', by simpa using hpos⟩
    · simpa [releaseAndCollect, releaseAndDecrement, hneq] using hannotated.2.2

/-- Released references stay disjoint from the live set after a release-and-collect step. -/
theorem releaseAndCollectReleasedNotLiveRef (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) :
    ∀ r, r ∈ (releaseAndCollect snapshot ref).releasedRefs → r ∉ (releaseAndCollect snapshot ref).liveRefs := by
  intro r hr hlive
  have hwf : WellFormed (releaseAndCollect snapshot ref) :=
    releaseAndCollectPreservesWellFormed snapshot ref h
  exact releasedNotLiveRef (releaseAndCollect snapshot ref) hwf r hr hlive

/-- A release-and-decrement step leaves unrelated heap entries untouched. -/
theorem releaseAndDecrementKeepsOtherHeapEntries (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.name ≠ ref →
      cell ∈ (releaseAndDecrement snapshot ref).heap := by
  intro cell hmem hname
  exact List.mem_map.mpr ⟨cell, hmem, by simp [hname]⟩

/-- Live references other than the released target remain live after a release-and-decrement step. -/
theorem releaseAndDecrementPreservesOtherLiveRefs (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) :
    ∀ r, r ∈ snapshot.liveRefs → r ≠ ref → liveAnnotated (releaseAndDecrement snapshot ref) r := by
  intro r hr hneq
  have hannotated : liveAnnotated snapshot r := h r hr
  constructor
  · exact hannotated.1
  · constructor
    · rcases hannotated.2.1 with ⟨cell, hmem, hname, hpos⟩
      have hcellname : cell.name ≠ ref := by
        simpa [hname] using hneq
      refine ⟨cell, ?_, hname, hpos⟩
      exact releaseAndDecrementKeepsOtherHeapEntries snapshot ref cell hmem hcellname
    · simp [releaseAndDecrement, hneq, hannotated.2.2]

/-- Released references remain disjoint from the live set after a release-and-decrement step. -/
theorem releaseAndDecrementReleasedNotLiveRef (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) :
    ∀ r, r ∈ (releaseAndDecrement snapshot ref).releasedRefs → r ∉ (releaseAndDecrement snapshot ref).liveRefs := by
  intro r hr hlive
  have hwf : WellFormed (releaseAndDecrement snapshot ref) :=
    releaseAndDecrementPreservesWellFormed snapshot ref h
  exact releasedNotLiveRef (releaseAndDecrement snapshot ref) hwf r hr hlive

/-- A released reference is recorded in the released set after the release step. -/
theorem releaseRecorded (snapshot : RcSnapshot) (ref : String) :
    ref ∈ (releaseRef snapshot ref).releasedRefs := by
  simp [releaseRef]

end KaliCore
