const K = 2.5;
function f() {
  let s = 0;
  for (const K of [1, 2, 3]) { s = s + K; }
  return s;
}
console.log(f());
