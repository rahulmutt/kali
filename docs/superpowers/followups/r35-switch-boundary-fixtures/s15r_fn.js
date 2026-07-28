function s(x) {
  switch (x) {
    case 1.5: return 100;
    case 2.5: return 200;
    default: return 900;
  }
}
console.log("r25=" + s(2.5));
console.log("r35=" + s(3.5));
console.log("r15=" + s(1.5));
