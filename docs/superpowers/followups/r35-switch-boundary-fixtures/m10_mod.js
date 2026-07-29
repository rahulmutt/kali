var r = 0;
for (var i = 0; i < 5; i = i + 1) {
  switch (i) {
    case 99: r = r + 1000; break;
    default: r = r + 1; break;
  }
  r = r + 100;
}
console.log("r=" + r);
