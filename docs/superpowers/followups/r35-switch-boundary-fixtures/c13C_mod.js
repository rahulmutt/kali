var r = 0;
var n = 4;
for (var i = 0; i < n; i = i + 1) {
  if (i === 1) {
    continue;
  } else {
    r = r + 1;
  }
}
console.log("r=" + r);
