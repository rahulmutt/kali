function s(b) {
  var r = -1;
  switch (b) {
    case true: r = 100; break;
    case false: r = 200; break;
    default: r = 900; break;
  }
  return r;
}
console.log("rtrue=" + s(true));
console.log("rfalse=" + s(false));
