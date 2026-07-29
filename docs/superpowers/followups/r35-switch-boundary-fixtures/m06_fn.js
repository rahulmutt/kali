function s(x) {
  let a = 1;
  switch (x) {
    case 10: let a10 = 100; return a10 + a;
    default: let a90 = 900; return a90 + a;
  }
}
console.log("r10=" + s(10));
console.log("r0=" + s(0));
