var r = 0;
var n = 4;
for (var i = 0; i < n; i = i + 1) {
  switch (i) {
    case 1: continue;
    default: r = r + 1;
  }
}
console.log("r=" + r);
