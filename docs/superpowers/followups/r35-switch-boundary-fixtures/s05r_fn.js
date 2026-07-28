function s(x) {
  switch (x) {
    case 10: let a10 = 100; return a10;
    case 20: let a20 = 200; return a20;
    default: let a90 = 900; return a90;
  }
}
console.log("r20=" + s(20));
console.log("r40=" + s(40));
console.log("r0=" + s(0));
console.log("r10=" + s(10));
