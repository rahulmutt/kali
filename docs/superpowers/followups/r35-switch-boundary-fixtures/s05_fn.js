function s(x) {
  var r = -1;
  switch (x) {
    case 10: let a10 = 100; r = a10; break;
    case 20: let a20 = 200; r = a20; break;
    default: let a90 = 900; r = a90; break;
  }
  return r;
}
console.log("r20=" + s(20));
console.log("r40=" + s(40));
console.log("r0=" + s(0));
console.log("r10=" + s(10));
