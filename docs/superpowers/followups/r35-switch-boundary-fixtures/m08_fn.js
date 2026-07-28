function s(x) {
  let v = 1;
  switch (x) {
    case 10: let v = 100; return v;
    default: return v;
  }
}
console.log("r10=" + s(10));
console.log("r0=" + s(0));
