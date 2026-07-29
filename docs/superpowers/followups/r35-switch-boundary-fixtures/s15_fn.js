function s(x) {
  var r = -1;
  switch (x) {
    case 1.5: r = 100; break;
    case 2.5: r = 200; break;
    default: r = 900; break;
  }
  return r;
}
console.log("r25=" + s(2.5));
console.log("r35=" + s(3.5));
console.log("r15=" + s(1.5));
