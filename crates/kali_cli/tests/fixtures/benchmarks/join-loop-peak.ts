// Bounded-peak reclamation proof (fasta Spec 7 Task 4e).
//
// `run` builds a 60-element string array once, then loops `n` times. Each
// iteration allocates a per-iteration heap `marker` object literal (so the
// escape gate's `allocates` bit fires for THIS while loop and it is granted
// its own per-iteration loop arena -- `line.join("")` and string `+` are, by
// design, invisible to that bit on their own: `arena_note_alloc` only fires
// on `ObjectExpr`/`ArrayExpr` literal nodes, see
// crates/kali_mir/src/analysis/walk.rs and arena_gate.rs's `"join"` arm),
// then joins `line` into a fresh string and concatenates a `"!"` onto it,
// dropping the result straight into `console.log`.
//
// Both the join and the `+` concat are proven iteration-local (Task 4b) and
// route to the `__join_arena`/`string_concat_arena` twins (Task 4c/4d),
// which allocate into the loop's own per-iteration arena -- reset every
// iteration -- instead of the never-reset global arena. Peak memory should
// therefore stay O(1) in `n`, not O(n): see reclamation_bounded_peak.rs.
function run(n) {
  var line = new Array(60);
  for (var i = 0; i < 60; i = i + 1) {
    line[i] = "x";
  }
  while (n > 0) {
    var marker = { tag: n };
    console.log(line.join("") + "!");
    n = n - 1;
  }
}
run(+process.argv[2]);
