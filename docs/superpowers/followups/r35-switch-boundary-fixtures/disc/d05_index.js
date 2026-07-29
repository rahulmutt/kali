var a = [1, 2];
function s(x) {
  switch (a[0]) {
    case 1: return "A";
    default: return "D";
  }
}
console.log("v=" + s(1));
