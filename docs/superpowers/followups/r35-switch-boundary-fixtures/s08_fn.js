function s(x) {
  switch (x) {
    case 10: console.log("hit=100");
    case 20: console.log("hit=200");
    default: console.log("hit=900");
  }
  return 0;
}
console.log("call20=" + s(20));
console.log("call40=" + s(40));
console.log("call10=" + s(10));
