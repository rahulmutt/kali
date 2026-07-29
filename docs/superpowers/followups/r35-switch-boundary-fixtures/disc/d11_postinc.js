function s(x) {
  switch (x++) {
    case 1: return "A";
    default: return "D";
  }
}
console.log("v=" + s(1));
