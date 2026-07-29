function s(n) {
  var r = 0;
  for (var i = 0; i < n; i = i + 1) {
    if (i === 1) {
      continue;
    } else {
      r = r + 1;
    }
  }
  return r;
}
console.log("r4=" + s(4));
