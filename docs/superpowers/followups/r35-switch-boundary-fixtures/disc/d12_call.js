function g(y) { return y; }
function s(x) {
  switch (g(x)) {
    case 1: return "A";
    default: return "D";
  }
}
console.log("v=" + s(1));
