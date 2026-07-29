function s(x) {
  var y = 20;
  switch (x) {
    case 10: return 100;
    case y: return 200;
    default: return 900;
  }
}
console.log("r20=" + s(20));
console.log("r40=" + s(40));
console.log("r10=" + s(10));
