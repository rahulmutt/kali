function s(x) {
  switch (x ? 1 : 2) {
    case 1: return "A";
    default: return "D";
  }
}
console.log("v=" + s(1));
