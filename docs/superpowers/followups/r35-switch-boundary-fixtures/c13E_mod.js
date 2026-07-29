let r = 0;
for (let i = 0; i < 4; i++) {
  switch (i) {
    case 1: continue;
    default: r = r + 1;
  }
}
console.log("r=" + r);
