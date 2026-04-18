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

/-- The release-only helper's live-reference list is exactly the target-filtered
original live set. -/
theorem releaseRefLiveRefsFiltered (snapshot : RcSnapshot) (ref : String) :
    (releaseRef snapshot ref).liveRefs = snapshot.liveRefs.filter (fun r => decide (r ≠ ref)) := by
  rfl

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

/-- The decrement helper's live-reference list is exactly the target-filtered
original live set. -/
theorem releaseAndDecrementLiveRefsFiltered (snapshot : RcSnapshot) (ref : String) :
    (releaseAndDecrement snapshot ref).liveRefs = snapshot.liveRefs.filter (fun r => decide (r ≠ ref)) := by
  rfl

/-- Release a live reference, decrement its target cell, and collect any zero-count
heap cells. This adds the local freeing step that the later Stage 4.2 memory
story will eventually widen further. -/
def releaseAndCollect (snapshot : RcSnapshot) (ref : String) : RcSnapshot :=
  let decremented := releaseAndDecrement snapshot ref
  { decremented with
    heap := decremented.heap.filter (fun cell => cell.refCount > 0)
  }

/-- The local collection helper's live-reference list is still exactly the target-filtered
original live set. -/
theorem releaseAndCollectLiveRefsFiltered (snapshot : RcSnapshot) (ref : String) :
    (releaseAndCollect snapshot ref).liveRefs = snapshot.liveRefs.filter (fun r => decide (r ≠ ref)) := by
  rfl

/-- Release-only, decrement, and collection helpers leave the ownership map untouched. -/
theorem releaseRefPreservesOwnership (snapshot : RcSnapshot) (ref : String) :
    (releaseRef snapshot ref).ownership = snapshot.ownership := by
  rfl

theorem releaseAndDecrementPreservesOwnership (snapshot : RcSnapshot) (ref : String) :
    (releaseAndDecrement snapshot ref).ownership = snapshot.ownership := by
  rfl

theorem releaseAndCollectPreservesOwnership (snapshot : RcSnapshot) (ref : String) :
    (releaseAndCollect snapshot ref).ownership = snapshot.ownership := by
  rfl

/-- Release-only, decrement, and collection helpers preserve the set of already-released references. -/
theorem releaseRefPreservesReleasedRefs (snapshot : RcSnapshot) (ref : String) :
    ∀ r, r ∈ snapshot.releasedRefs → r ∈ (releaseRef snapshot ref).releasedRefs := by
  intro r hr
  simpa [releaseRef] using (List.mem_cons_of_mem ref hr)

/-- The release-and-decrement helper preserves the set of already-released references. -/
theorem releaseAndDecrementPreservesReleasedRefs (snapshot : RcSnapshot) (ref : String) :
    ∀ r, r ∈ snapshot.releasedRefs → r ∈ (releaseAndDecrement snapshot ref).releasedRefs := by
  intro r hr
  simpa [releaseAndDecrement] using (List.mem_cons_of_mem ref hr)

/-- The local release-and-collect helper preserves the set of already-released references. -/
theorem releaseAndCollectPreservesReleasedRefs (snapshot : RcSnapshot) (ref : String) :
    ∀ r, r ∈ snapshot.releasedRefs → r ∈ (releaseAndCollect snapshot ref).releasedRefs := by
  intro r hr
  simpa [releaseAndCollect, releaseAndDecrement] using (List.mem_cons_of_mem ref hr)

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

/-- The release-and-decrement helper keeps the surviving live references anchored in ownership and allocation. -/
theorem releaseAndDecrementLiveRefsAreOwnedAndAllocated (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) :
    ∀ r, r ∈ (releaseAndDecrement snapshot ref).liveRefs →
      hasOwnership (releaseAndDecrement snapshot ref).ownership r ∧
      allocated (releaseAndDecrement snapshot ref) r := by
  intro r hr
  have hwf : WellFormed (releaseAndDecrement snapshot ref) :=
    releaseAndDecrementPreservesWellFormed snapshot ref h
  exact liveRefsAreOwnedAndAllocated (releaseAndDecrement snapshot ref) hwf r hr

/-- A release-and-decrement step still records the released reference. -/
theorem releaseAndDecrementRecorded (snapshot : RcSnapshot) (ref : String) :
    ref ∈ (releaseAndDecrement snapshot ref).releasedRefs := by
  simp [releaseAndDecrement]

/-- The release-and-decrement helper's released-reference list is exactly the released reference followed by the original released set. -/
theorem releaseAndDecrementReleasedRefsCons (snapshot : RcSnapshot) (ref : String) :
    (releaseAndDecrement snapshot ref).releasedRefs = ref :: snapshot.releasedRefs := by
  rfl

/-- A release-and-decrement step decrements the targeted heap cell when it is present. -/
theorem releaseAndDecrementDecrementsTargetCell (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.name = ref →
      { cell with refCount := cell.refCount - 1 } ∈ (releaseAndDecrement snapshot ref).heap := by
  intro cell hmem hname
  exact List.mem_map.mpr ⟨cell, hmem, by simp [hname]⟩

/-- A release-and-decrement step keeps the targeted heap cell when its decremented count stays positive. -/
theorem releaseAndDecrementKeepsTargetCellWhenPositiveCount (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.name = ref → cell.refCount > 1 →
      { cell with refCount := cell.refCount - 1 } ∈ (releaseAndDecrement snapshot ref).heap ∧
      { cell with refCount := cell.refCount - 1 }.refCount > 0 := by
  intro cell hmem hname hgt1
  constructor
  · exact List.mem_map.mpr ⟨cell, hmem, by simp [hname]⟩
  · simpa using Nat.sub_pos_of_lt hgt1

/-- A release-and-decrement step keeps the targeted reference allocated when its decremented count stays positive. -/
theorem releaseAndDecrementTargetCellAllocatedWhenPositiveCount (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.name = ref → cell.refCount > 1 →
      allocated (releaseAndDecrement snapshot ref) ref := by
  intro cell hmem hname hgt1
  refine ⟨{ cell with refCount := cell.refCount - 1 }, ?_, ?_, ?_⟩
  · exact (releaseAndDecrementKeepsTargetCellWhenPositiveCount snapshot ref cell hmem hname hgt1).1
  · simp [hname]
  · simpa using Nat.sub_pos_of_lt hgt1

/-- A release-and-decrement step keeps the live target reference anchored in ownership and allocation when its decremented count stays positive. -/
theorem releaseAndDecrementTargetCellOwnedAndAllocatedWhenPositiveCount (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) (href : ref ∈ snapshot.liveRefs) :
    ∀ cell, cell ∈ snapshot.heap → cell.name = ref → cell.refCount > 1 →
      hasOwnership (releaseAndDecrement snapshot ref).ownership ref ∧
      allocated (releaseAndDecrement snapshot ref) ref := by
  intro cell hmem hname hgt1
  have hown : hasOwnership snapshot.ownership ref := (h ref href).1
  constructor
  · simpa [releaseAndDecrement] using hown
  · exact releaseAndDecrementTargetCellAllocatedWhenPositiveCount snapshot ref cell hmem hname hgt1

/-- Every release-and-decrement heap cell comes from the original heap, with only the released target decremented or left unchanged. -/
theorem releaseAndDecrementHeapCellOrigin (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseAndDecrement snapshot ref).heap →
      ∃ cell0, cell0 ∈ snapshot.heap ∧
        (cell = { cell0 with refCount := cell0.refCount - 1 } ∨ cell = cell0) := by
  intro cell hmem
  rcases List.mem_map.mp hmem with ⟨cell0, hmem0, hcell⟩
  by_cases hname : cell0.name = ref
  · refine ⟨cell0, hmem0, Or.inl ?_⟩
    simpa [releaseAndDecrement, hname] using hcell.symm
  · refine ⟨cell0, hmem0, Or.inr ?_⟩
    simpa [releaseAndDecrement, hname] using hcell.symm

/-- Every release-and-decrement heap cell also preserves its original name and ownership tag. -/
theorem releaseAndDecrementHeapCellOriginAndOwnership (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseAndDecrement snapshot ref).heap →
      ∃ cell0, cell0 ∈ snapshot.heap ∧
        (cell = { cell0 with refCount := cell0.refCount - 1 } ∨ cell = cell0) ∧
        cell.name = cell0.name ∧
        cell.owner = cell0.owner := by
  intro cell hmem
  rcases releaseAndDecrementHeapCellOrigin snapshot ref cell hmem with ⟨cell0, hmem0, hshape⟩
  refine ⟨cell0, hmem0, hshape, ?_⟩
  cases hshape with
  | inl h =>
      subst h
      simp
  | inr h =>
      subst h
      simp

/-- A release-and-decrement step keeps every surviving heap cell traceable to the original heap and positive-count. -/
theorem releaseAndDecrementHeapCellOriginAndPositiveCount (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseAndDecrement snapshot ref).heap →
      cell.refCount > 0 →
      ∃ cell0, cell0 ∈ snapshot.heap ∧
        (cell = { cell0 with refCount := cell0.refCount - 1 } ∨ cell = cell0) ∧
        cell.refCount > 0 := by
  intro cell hmem hpos
  rcases releaseAndDecrementHeapCellOrigin snapshot ref cell hmem with ⟨cell0, hmem0, hshape⟩
  exact ⟨cell0, hmem0, hshape, hpos⟩

/-- A release-and-decrement step keeps any surviving positive-count cell traceable to the original heap with its original name and ownership tag. -/
theorem releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseAndDecrement snapshot ref).heap →
      cell.refCount > 0 →
      ∃ cell0, cell0 ∈ snapshot.heap ∧
        (cell = { cell0 with refCount := cell0.refCount - 1 } ∨ cell = cell0) ∧
        cell.name = cell0.name ∧
        cell.owner = cell0.owner ∧
        cell.refCount > 0 := by
  intro cell hmem hpos
  rcases releaseAndDecrementHeapCellOriginAndOwnership snapshot ref cell hmem with ⟨cell0, hmem0, hshape, hname, howner⟩
  exact ⟨cell0, hmem0, hshape, hname, howner, hpos⟩

/-- The release-and-decrement helper's heap is exactly the original heap with the released target decremented and every other cell unchanged. -/
theorem releaseAndDecrementHeapCharacterisation (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseAndDecrement snapshot ref).heap ↔
      ∃ cell0, cell0 ∈ snapshot.heap ∧
        ((cell0.name = ref ∧ cell = { cell0 with refCount := cell0.refCount - 1 }) ∨
         (cell0.name ≠ ref ∧ cell = cell0)) := by
  intro cell
  constructor
  · intro hmem
    rcases List.mem_map.mp hmem with ⟨cell0, hmem0, hcell⟩
    by_cases hname : cell0.name = ref
    · refine ⟨cell0, hmem0, Or.inl ⟨hname, ?_⟩⟩
      simpa [releaseAndDecrement, hname] using hcell.symm
    · refine ⟨cell0, hmem0, Or.inr ⟨hname, ?_⟩⟩
      simpa [releaseAndDecrement, hname] using hcell.symm
  · intro hmem
    rcases hmem with ⟨cell0, hmem0, hcase⟩
    rcases hcase with ⟨hname, hcell⟩ | ⟨hname, hcell⟩
    · subst hname
      exact List.mem_map.mpr ⟨cell0, hmem0, by simpa [releaseAndDecrement] using hcell.symm⟩
    · exact List.mem_map.mpr ⟨cell0, hmem0, by simp [hname, hcell]⟩

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

/-- A release-and-collect step drops every zero-count cell from the decrement pass. -/
theorem releaseAndCollectDropsZeroCountCells (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseAndDecrement snapshot ref).heap → cell.refCount = 0 →
      cell ∉ (releaseAndCollect snapshot ref).heap := by
  intro cell hmem hcount hpresent
  simp [releaseAndCollect, hcount] at hpresent

/-- A release-and-collect step keeps every positive-count cell from the
decrement pass, so the local helper only drops zero-count entries. -/
theorem releaseAndCollectKeepsPositiveCountCells (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseAndDecrement snapshot ref).heap → cell.refCount > 0 →
      cell ∈ (releaseAndCollect snapshot ref).heap := by
  intro cell hmem hpos
  dsimp [releaseAndCollect]
  exact List.mem_filter.mpr ⟨hmem, by simpa using hpos⟩

/-- A release-and-collect step keeps the released target when its decremented
count stays positive. -/
theorem releaseAndCollectKeepsTargetCellWhenPositiveCount (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.name = ref → cell.refCount > 1 →
      { cell with refCount := cell.refCount - 1 } ∈ (releaseAndCollect snapshot ref).heap := by
  intro cell hmem hname hgt1
  have hmem' : { cell with refCount := cell.refCount - 1 } ∈ (releaseAndDecrement snapshot ref).heap := by
    exact List.mem_map.mpr ⟨cell, hmem, by simp [hname]⟩
  have hpos : { cell with refCount := cell.refCount - 1 }.refCount > 0 := by
    simpa using Nat.sub_pos_of_lt hgt1
  exact releaseAndCollectKeepsPositiveCountCells snapshot ref { cell with refCount := cell.refCount - 1 } hmem' hpos

/-- A release-and-collect step keeps the targeted reference allocated when its decremented count stays positive. -/
theorem releaseAndCollectTargetCellAllocatedWhenPositiveCount (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.name = ref → cell.refCount > 1 →
      allocated (releaseAndCollect snapshot ref) ref := by
  intro cell hmem hname hgt1
  refine ⟨{ cell with refCount := cell.refCount - 1 }, ?_, ?_, ?_⟩
  · exact releaseAndCollectKeepsTargetCellWhenPositiveCount snapshot ref cell hmem hname hgt1
  · simp [hname]
  · simpa using Nat.sub_pos_of_lt hgt1

/-- A release-and-collect step keeps the live target reference anchored in ownership and allocation when its decremented count stays positive. -/
theorem releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) (href : ref ∈ snapshot.liveRefs) :
    ∀ cell, cell ∈ snapshot.heap → cell.name = ref → cell.refCount > 1 →
      hasOwnership (releaseAndCollect snapshot ref).ownership ref ∧
      allocated (releaseAndCollect snapshot ref) ref := by
  intro cell hmem hname hgt1
  have hown : hasOwnership snapshot.ownership ref := (h ref href).1
  constructor
  · simpa [releaseAndCollect, releaseAndDecrement] using hown
  · exact releaseAndCollectTargetCellAllocatedWhenPositiveCount snapshot ref cell hmem hname hgt1

/-- A release-and-collect step keeps positive-count cells from the original heap
when they are not the released target, and those survivors remain positive-count
after collection. This makes the helper-level no-leak story explicit.
-/
theorem releaseAndCollectKeepsOtherPositiveCountCells (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.name ≠ ref → cell.refCount > 0 →
      cell ∈ (releaseAndCollect snapshot ref).heap ∧ cell.refCount > 0 := by
  intro cell hmem hname hpos
  have hmem' : cell ∈ (releaseAndDecrement snapshot ref).heap := by
    exact List.mem_map.mpr ⟨cell, hmem, by simp [hname]⟩
  have hkeep : cell ∈ (releaseAndCollect snapshot ref).heap := by
    exact releaseAndCollectKeepsPositiveCountCells snapshot ref cell hmem' hpos
  exact ⟨hkeep, hpos⟩

/-- The local release-and-collect helper keeps every original positive-count cell alive: non-target cells survive unchanged, and the released target survives when its decremented count stays positive. This packages the helper-level no-leak story explicitly. -/
theorem releaseAndCollectKeepsOriginalPositiveCountCells (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.refCount > 0 →
      (cell.name = ref → cell.refCount > 1 →
        { cell with refCount := cell.refCount - 1 } ∈ (releaseAndCollect snapshot ref).heap ∧
        { cell with refCount := cell.refCount - 1 }.refCount > 0) ∧
      (cell.name ≠ ref →
        cell ∈ (releaseAndCollect snapshot ref).heap ∧ cell.refCount > 0) := by
  intro cell hmem hpos
  constructor
  · intro hname hgt1
    constructor
    · exact releaseAndCollectKeepsTargetCellWhenPositiveCount snapshot ref cell hmem hname hgt1
    · simpa using Nat.sub_pos_of_lt hgt1
  · intro hname
    exact releaseAndCollectKeepsOtherPositiveCountCells snapshot ref cell hmem hname hpos

/-- A release-and-collect step keeps unrelated positive-count heap entries in the collected heap. -/
theorem releaseAndCollectKeepsOtherHeapEntries (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.name ≠ ref → cell.refCount > 0 →
      cell ∈ (releaseAndCollect snapshot ref).heap := by
  intro cell hmem hname hpos
  exact (releaseAndCollectKeepsOtherPositiveCountCells snapshot ref cell hmem hname hpos).1

/-- A release-and-collect step drops any original zero-count cell from the final heap. -/
theorem releaseAndCollectDropsOriginalZeroCountCells (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.refCount = 0 →
      cell ∉ (releaseAndCollect snapshot ref).heap := by
  intro cell hmem hzero hpresent
  have hfilter : cell ∈ (releaseAndDecrement snapshot ref).heap.filter (fun cell => cell.refCount > 0) := by
    simpa [releaseAndCollect] using hpresent
  have hpos : decide (cell.refCount > 0) = true := (List.mem_filter.mp hfilter).2
  have hfalse : False := by
    simp [hzero] at hpos
  exact False.elim hfalse

/-- The release-and-collect helper's heap is exactly the positive-count filter of the decrement pass. -/
theorem releaseAndCollectHeapIsPositiveCountFilter (snapshot : RcSnapshot) (ref : String) :
    (releaseAndCollect snapshot ref).heap =
      (releaseAndDecrement snapshot ref).heap.filter (fun cell => cell.refCount > 0) := by
  simp [releaseAndCollect]

/-- The release-and-collect helper's final heap contains only positive-count cells. -/
theorem releaseAndCollectHeapCellsHavePositiveCount (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseAndCollect snapshot ref).heap → cell.refCount > 0 := by
  intro cell hmem
  have hfilter : cell ∈ (releaseAndDecrement snapshot ref).heap.filter (fun cell => cell.refCount > 0) := by
    simpa [releaseAndCollect] using hmem
  have hdec : decide (cell.refCount > 0) = true := (List.mem_filter.mp hfilter).2
  exact of_decide_eq_true hdec

/-- Every surviving release-and-collect heap cell comes from the original heap, with only the released target decremented. -/
theorem releaseAndCollectHeapCellOrigin (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseAndCollect snapshot ref).heap →
      ∃ cell0, cell0 ∈ snapshot.heap ∧
        (cell = { cell0 with refCount := cell0.refCount - 1 } ∨ cell = cell0) := by
  intro cell hmem
  have hfilter : cell ∈ (releaseAndDecrement snapshot ref).heap.filter (fun cell => cell.refCount > 0) := by
    simpa [releaseAndCollect] using hmem
  rcases List.mem_filter.mp hfilter with ⟨hdecr, _⟩
  rcases List.mem_map.mp hdecr with ⟨cell0, hmem0, hcell⟩
  by_cases hname : cell0.name = ref
  · refine ⟨cell0, hmem0, Or.inl ?_⟩
    simpa [releaseAndDecrement, hname] using hcell.symm
  · refine ⟨cell0, hmem0, Or.inr ?_⟩
    simpa [releaseAndDecrement, hname] using hcell.symm

/-- A release-and-collect step keeps every surviving heap cell traceable to the original heap with the same name and ownership tag. -/
theorem releaseAndCollectHeapCellOriginAndOwnership (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseAndCollect snapshot ref).heap →
      ∃ cell0, cell0 ∈ snapshot.heap ∧
        (cell = { cell0 with refCount := cell0.refCount - 1 } ∨ cell = cell0) ∧
        cell.name = cell0.name ∧
        cell.owner = cell0.owner := by
  intro cell hmem
  rcases releaseAndCollectHeapCellOrigin snapshot ref cell hmem with ⟨cell0, hmem0, hshape⟩
  refine ⟨cell0, hmem0, hshape, ?_⟩
  cases hshape with
  | inl h =>
      subst h
      simp
  | inr h =>
      subst h
      simp

/-- A release-and-collect step keeps every surviving heap cell traceable to the original heap, with its original name, ownership tag, and positive count. -/
theorem releaseAndCollectHeapCellOriginOwnershipAndPositiveCount (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseAndCollect snapshot ref).heap →
      ∃ cell0, cell0 ∈ snapshot.heap ∧
        (cell = { cell0 with refCount := cell0.refCount - 1 } ∨ cell = cell0) ∧
        cell.name = cell0.name ∧
        cell.owner = cell0.owner ∧
        cell.refCount > 0 := by
  intro cell hmem
  rcases releaseAndCollectHeapCellOriginAndOwnership snapshot ref cell hmem with ⟨cell0, hmem0, hshape, hname, howner⟩
  exact ⟨cell0, hmem0, hshape, hname, howner, releaseAndCollectHeapCellsHavePositiveCount snapshot ref cell hmem⟩

/-- The release-and-collect helper's heap is exactly the original heap with the released target decremented, all other cells unchanged, and only positive-count survivors retained. -/
theorem releaseAndCollectHeapCharacterisation (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseAndCollect snapshot ref).heap ↔
      ∃ cell0, cell0 ∈ snapshot.heap ∧
        ((cell0.name = ref ∧ cell = { cell0 with refCount := cell0.refCount - 1 }) ∨
         (cell0.name ≠ ref ∧ cell = cell0)) ∧
        cell.refCount > 0 := by
  intro cell
  constructor
  · intro hmem
    have hpos : cell.refCount > 0 := releaseAndCollectHeapCellsHavePositiveCount snapshot ref cell hmem
    have hfilter : cell ∈ (releaseAndDecrement snapshot ref).heap.filter (fun cell => cell.refCount > 0) := by
      simpa [releaseAndCollect] using hmem
    rcases List.mem_filter.mp hfilter with ⟨hdecr, _⟩
    rcases (releaseAndDecrementHeapCharacterisation snapshot ref cell).mp hdecr with ⟨cell0, hmem0, hcase⟩
    exact ⟨cell0, hmem0, hcase, hpos⟩
  · intro hmem
    rcases hmem with ⟨cell0, hmem0, hcase, hpos⟩
    have hdecr : cell ∈ (releaseAndDecrement snapshot ref).heap :=
      (releaseAndDecrementHeapCharacterisation snapshot ref cell).mpr ⟨cell0, hmem0, hcase⟩
    exact List.mem_filter.mpr ⟨hdecr, by simpa using hpos⟩

/-- A release-and-collect step keeps every surviving heap cell both positively counted and traceable to the original heap. -/
theorem releaseAndCollectHeapCellOriginAndPositiveCount (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseAndCollect snapshot ref).heap →
      ∃ cell0, cell0 ∈ snapshot.heap ∧
        (cell = { cell0 with refCount := cell0.refCount - 1 } ∨ cell = cell0) ∧
        cell.refCount > 0 := by
  intro cell hmem
  rcases releaseAndCollectHeapCellOrigin snapshot ref cell hmem with ⟨cell0, hmem0, hshape⟩
  exact ⟨cell0, hmem0, hshape, releaseAndCollectHeapCellsHavePositiveCount snapshot ref cell hmem⟩

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

/-- The release-and-collect helper keeps the surviving live references anchored in ownership and allocation. -/
theorem releaseAndCollectLiveRefsAreOwnedAndAllocated (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) :
    ∀ r, r ∈ (releaseAndCollect snapshot ref).liveRefs →
      hasOwnership (releaseAndCollect snapshot ref).ownership r ∧
      allocated (releaseAndCollect snapshot ref) r := by
  intro r hr
  have hwf : WellFormed (releaseAndCollect snapshot ref) :=
    releaseAndCollectPreservesWellFormed snapshot ref h
  exact liveRefsAreOwnedAndAllocated (releaseAndCollect snapshot ref) hwf r hr

/-- A release-and-collect step still records the released reference. -/
theorem releaseAndCollectRecorded (snapshot : RcSnapshot) (ref : String) :
    ref ∈ (releaseAndCollect snapshot ref).releasedRefs := by
  simp [releaseAndCollect, releaseAndDecrement]

/-- The release-and-collect helper's released-reference list is exactly the released reference followed by the original released set. -/
theorem releaseAndCollectReleasedRefsCons (snapshot : RcSnapshot) (ref : String) :
    (releaseAndCollect snapshot ref).releasedRefs = ref :: snapshot.releasedRefs := by
  rfl

/-- Released references stay disjoint from the live set after a release-and-collect step. -/
theorem releaseAndCollectReleasedNotLiveRef (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) :
    ∀ r, r ∈ (releaseAndCollect snapshot ref).releasedRefs → r ∉ (releaseAndCollect snapshot ref).liveRefs := by
  intro r hr hlive
  have hwf : WellFormed (releaseAndCollect snapshot ref) :=
    releaseAndCollectPreservesWellFormed snapshot ref h
  exact releasedNotLiveRef (releaseAndCollect snapshot ref) hwf r hr hlive

/-- Live references other than the released target remain live after the local release-and-collect helper runs. -/
theorem releaseAndCollectPreservesOtherLiveRefs (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) :
    ∀ r, r ∈ snapshot.liveRefs → r ≠ ref → liveAnnotated (releaseAndCollect snapshot ref) r := by
  intro r hr hneq
  have hr' : r ∈ (releaseAndCollect snapshot ref).liveRefs := by
    simp [releaseAndCollect, releaseAndDecrement, hr, hneq]
  have hwf : WellFormed (releaseAndCollect snapshot ref) :=
    releaseAndCollectPreservesWellFormed snapshot ref h
  exact hwf r hr'

/-- A release-and-decrement step leaves unrelated heap entries untouched. -/
theorem releaseAndDecrementKeepsOtherHeapEntries (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.name ≠ ref →
      cell ∈ (releaseAndDecrement snapshot ref).heap := by
  intro cell hmem hname
  exact List.mem_map.mpr ⟨cell, hmem, by simp [hname]⟩

/-- A release-and-decrement step keeps positive-count cells from the original heap
when they are not the released target. -/
theorem releaseAndDecrementKeepsOtherPositiveCountCells (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.name ≠ ref → cell.refCount > 0 →
      cell ∈ (releaseAndDecrement snapshot ref).heap ∧ cell.refCount > 0 := by
  intro cell hmem hname hpos
  constructor
  · exact releaseAndDecrementKeepsOtherHeapEntries snapshot ref cell hmem hname
  · exact hpos

/-- A release-and-decrement step keeps every original positive-count cell alive:
non-target cells survive unchanged, and the released target survives when its
decremented count stays positive. -/
theorem releaseAndDecrementKeepsOriginalPositiveCountCells (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ snapshot.heap → cell.refCount > 0 →
      (cell.name = ref → cell.refCount > 1 →
        { cell with refCount := cell.refCount - 1 } ∈ (releaseAndDecrement snapshot ref).heap ∧
        { cell with refCount := cell.refCount - 1 }.refCount > 0) ∧
      (cell.name ≠ ref →
        cell ∈ (releaseAndDecrement snapshot ref).heap ∧ cell.refCount > 0) := by
  intro cell hmem hpos
  constructor
  · intro hname hgt1
    constructor
    · exact (releaseAndDecrementKeepsTargetCellWhenPositiveCount snapshot ref cell hmem hname hgt1).1
    · simpa using Nat.sub_pos_of_lt hgt1
  · intro hname
    exact ⟨releaseAndDecrementKeepsOtherHeapEntries snapshot ref cell hmem hname, hpos⟩

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

/-- A release-only step keeps the surviving live references anchored in ownership and allocation. -/
theorem releaseRefLiveRefsAreOwnedAndAllocated (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) :
    ∀ r, r ∈ (releaseRef snapshot ref).liveRefs →
      hasOwnership (releaseRef snapshot ref).ownership r ∧
      allocated (releaseRef snapshot ref) r := by
  intro r hr
  have hwf : WellFormed (releaseRef snapshot ref) :=
    releasePreservesWellFormed snapshot ref h
  exact liveRefsAreOwnedAndAllocated (releaseRef snapshot ref) hwf r hr

/-- Released references stay disjoint from the live set after a release-only step. -/
theorem releaseRefReleasedNotLiveRef (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) :
    ∀ r, r ∈ (releaseRef snapshot ref).releasedRefs → r ∉ (releaseRef snapshot ref).liveRefs := by
  intro r hr hlive
  have hwf : WellFormed (releaseRef snapshot ref) :=
    releasePreservesWellFormed snapshot ref h
  exact releasedNotLiveRef (releaseRef snapshot ref) hwf r hr hlive

/-- A released reference is recorded in the released set after the release step. -/
theorem releaseRecorded (snapshot : RcSnapshot) (ref : String) :
    ref ∈ (releaseRef snapshot ref).releasedRefs := by
  simp [releaseRef]

/-- The release-only helper's released-reference list is exactly the released reference followed by the original released set. -/
theorem releaseRefReleasedRefsCons (snapshot : RcSnapshot) (ref : String) :
    (releaseRef snapshot ref).releasedRefs = ref :: snapshot.releasedRefs := by
  rfl

/-- The release-only helper's heap is unchanged. -/
theorem releaseRefHeapCharacterisation (snapshot : RcSnapshot) (ref : String) :
    (releaseRef snapshot ref).heap = snapshot.heap := by
  rfl

/-- The release-only helper's surviving heap cells come from the original heap unchanged. -/
theorem releaseRefHeapCellOrigin (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseRef snapshot ref).heap →
      ∃ cell0, cell0 ∈ snapshot.heap ∧
        cell = cell0 := by
  intro cell hmem
  refine ⟨cell, ?_, rfl⟩
  simpa [releaseRefHeapCharacterisation] using hmem

/-- The release-only helper's surviving heap cells come from the original heap with the same name and ownership tag. -/
theorem releaseRefHeapCellOriginAndOwnership (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseRef snapshot ref).heap →
      ∃ cell0, cell0 ∈ snapshot.heap ∧
        cell = cell0 ∧
        cell.name = cell0.name ∧
        cell.owner = cell0.owner := by
  intro cell hmem
  rcases releaseRefHeapCellOrigin snapshot ref cell hmem with ⟨cell0, hmem0, hcell⟩
  refine ⟨cell0, hmem0, hcell, ?_, ?_⟩
  · subst hcell
    simp
  · subst hcell
    simp

/-- The release-only helper's surviving heap cells are traceable to the original heap with their original name, ownership tag, and positive count. -/
theorem releaseRefHeapCellOriginOwnershipAndPositiveCount (snapshot : RcSnapshot) (ref : String) :
    ∀ cell, cell ∈ (releaseRef snapshot ref).heap →
      cell.refCount > 0 →
      ∃ cell0, cell0 ∈ snapshot.heap ∧
        cell = cell0 ∧
        cell.name = cell0.name ∧
        cell.owner = cell0.owner ∧
        cell.refCount > 0 := by
  intro cell hmem hpos
  rcases releaseRefHeapCellOriginAndOwnership snapshot ref cell hmem with ⟨cell0, hmem0, hcell, hname, howner⟩
  exact ⟨cell0, hmem0, hcell, hname, howner, hpos⟩

/-- A release-only step preserves the no-dangling-reference property on well-formed snapshots. -/
theorem releaseRefNoDanglingReference (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) :
    ¬ DanglingReference (releaseRef snapshot ref) := by
  exact noDanglingReference (releaseRef snapshot ref) (releasePreservesWellFormed snapshot ref h)

/-- A release-and-decrement step preserves the no-dangling-reference property on well-formed snapshots. -/
theorem releaseAndDecrementNoDanglingReference (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) :
    ¬ DanglingReference (releaseAndDecrement snapshot ref) := by
  exact noDanglingReference (releaseAndDecrement snapshot ref) (releaseAndDecrementPreservesWellFormed snapshot ref h)

/-- A release-and-collect step preserves the no-dangling-reference property on well-formed snapshots. -/
theorem releaseAndCollectNoDanglingReference (snapshot : RcSnapshot) (ref : String)
    (h : WellFormed snapshot) :
    ¬ DanglingReference (releaseAndCollect snapshot ref) := by
  exact noDanglingReference (releaseAndCollect snapshot ref) (releaseAndCollectPreservesWellFormed snapshot ref h)

end KaliCore
