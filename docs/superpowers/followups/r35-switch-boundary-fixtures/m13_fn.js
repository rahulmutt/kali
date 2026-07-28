function s(x) {
  switch (x) {
    case 1: return 100;
    default: return 900;
  }
}
console.log("r1=" + s(1));
console.log("r0=" + s(0));
