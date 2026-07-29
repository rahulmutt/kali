function s(b) {
  switch (b) {
    case false: return 200;
    case true: return 100;
    default: return 900;
  }
}
console.log("rtrue=" + s(true));
console.log("rfalse=" + s(false));
