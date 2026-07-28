var r = 0;
var n = 2;
for (var i = 0; i < n; i = i + 1) {
  switch (i) {
    case 1: r = r + 10; break;
    default: r = r + 1; break;
  }
}
console.log("r=" + r);
