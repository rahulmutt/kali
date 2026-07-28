function s(x) {
  switch (x) {
    case 10: return 100;
    case 20: throw "boom";
    default: return 900;
  }
}
console.log("r10=" + s(10));
console.log("r40=" + s(40));
