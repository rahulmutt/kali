function s(x) {
  switch (x) {
    case 10: var a10 = 100; return a10;
    case 20: var a20 = 200; return a20;
    default: var a90 = 900; return a90;
  }
}
console.log("r20=" + s(20));
console.log("r40=" + s(40));
console.log("r0=" + s(0));
console.log("r10=" + s(10));
