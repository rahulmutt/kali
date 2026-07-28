function s(b) {
  switch (b) {
    case true: return 100;
    case false: return 200;
    default: return 900;
  }
}
console.log("rtrue=" + s(true));
console.log("rfalse=" + s(false));
