function s(x) {
  switch (x) {
    case 1: return 100;
    case 2: return 200;
    case 3: return 300;
    case 4: return 400;
    default: return 900;
  }
}
console.log("r1=" + s(1));
console.log("r2=" + s(2));
console.log("r3=" + s(3));
console.log("r4=" + s(4));
console.log("r9=" + s(9));
console.log("r0=" + s(0));
