function s(x) {
  var r = -1;
  switch (x) {
    case 1:
    case 2: r = 200; break;
    default: r = 900; break;
  }
  return r;
}
console.log("r1=" + s(1));
console.log("r2=" + s(2));
console.log("r5=" + s(5));
