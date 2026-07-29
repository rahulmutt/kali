function s(x) {
  switch (x) {
    case 1:
    case 2: return 200;
    default: return 900;
  }
}
console.log("r1=" + s(1));
console.log("r2=" + s(2));
console.log("r5=" + s(5));
