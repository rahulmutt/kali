// Bounded-peak reclamation proof for the string-site-triggered loop arena
// (fasta Spec 7 Task 4f) — the real `fastaRepeat`/`fastaRandom` loop shape.
//
// Unlike Task 4e's `join-loop-peak.ts`, this loop contains NO per-iteration
// object/array literal (`marker`). Its `while` body's ONLY reclaimable
// allocation is the granted `line.join("") + "!"` string site, and the `line`
// buffer is a `new Array(60)` allocated ONCE, OUTSIDE the loop. So the object
// arena channel (`loop_arena`) never fires for this loop — `arena_note_alloc`
// only sets `allocates` on `ObjectExpr`/`ArrayExpr` literal nodes, and a
// `new Array(n)` is a `NewExpr` (see crates/kali_mir/src/analysis/walk.rs and
// arena_gate.rs). The loop opens its per-iteration arena SOLELY through the
// Task 4f `string_arena_loop` channel (granted string site + not arena_eligible
// + no unknown call + no allocating callee), routing the join and `+` concat to
// the resettable `__join_arena`/`string_concat_arena` twins that allocate into
// the loop's own per-iteration arena — reset every iteration — instead of the
// never-reset global arena.
//
// Peak memory therefore stays O(1) in `n`, not O(n): see the discrimination
// note in reclamation_bounded_peak.rs. Without the Task 4f channel this exact
// loop leaks every line's join/concat garbage into the boot arena and traps
// E4000 at large N under the small fixed budget.
function run(n) {
  var line = new Array(60);
  for (var i = 0; i < 60; i = i + 1) {
    line[i] = "x";
  }
  while (n > 0) {
    console.log(line.join("") + "!");
    n = n - 1;
  }
}
run(+process.argv[2]);
