function s(x) {
  switch (x) {
    case 10: return 100;
    default: return 900;
    case 20: return 200;
  }
}
console.log("r20=" + s(20));
console.log("r40=" + s(40));
console.log("r10=" + s(10));
