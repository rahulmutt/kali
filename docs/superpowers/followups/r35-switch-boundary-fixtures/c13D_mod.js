var r = 0;
for (var i = 0; i < 4; i = i + 1) {
  if (i === 1) continue;
  r = r + 1;
}
console.log("r=" + r);
