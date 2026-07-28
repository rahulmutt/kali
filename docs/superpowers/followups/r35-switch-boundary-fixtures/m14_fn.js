function s(x) {
  switch (x) {
    case 1: return 100;
    case 2: return 200;
    default: return 900;
  }
}
console.log("r25=" + s(2.5));
console.log("r0=" + s(0.0));
