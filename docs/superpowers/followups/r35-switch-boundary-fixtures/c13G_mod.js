var r = 0;
for (var i = 0; i < 4; i = i + 1) {
  switch (i) {
    case 1: continue;
    default: break;
  }
  r = r + 1;
  console.log("iter=" + i);
}
console.log("r=" + r);
