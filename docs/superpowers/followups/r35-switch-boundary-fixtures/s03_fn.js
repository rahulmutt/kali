function s(x) {
  var r = -1;
  switch (x) {
    case 10: r = 100; break;
    case 20: r = 200; break;
    default: r = 900; break;
  }
  return r;
}
console.log("r20=" + s(20));
console.log("r40=" + s(40));
console.log("r0=" + s(0));
console.log("r10=" + s(10));
