function s(x) {
  switch (x) {
    case "a": return 100;
    case "b": return 200;
    default: return 900;
  }
}
console.log("ra=" + s("a"));
console.log("rb=" + s("b"));
console.log("rz=" + s("z"));
