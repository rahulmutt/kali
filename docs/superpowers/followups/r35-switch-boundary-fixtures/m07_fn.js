function s(x) {
  let v = 1;
  switch (x) {
    case 10: let v2 = 100; return v2;
    default: let v3 = 900; return v3;
  }
}
console.log("r10=" + s(10));
console.log("r0=" + s(0));
