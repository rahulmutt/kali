function s(n) {
  var r = 0;
  for (var i = 0; i < n; i = i + 1) {
    switch (i) {
      case 1: continue;
      default: break;
    }
    r = r + 1;
  }
  return r;
}
console.log("r1=" + s(1));
console.log("r2=" + s(2));
console.log("r4=" + s(4));
