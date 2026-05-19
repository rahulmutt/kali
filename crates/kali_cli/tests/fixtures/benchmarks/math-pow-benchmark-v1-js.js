function dead0(x) {
  return (x + 0) + (0 + x) + Math.pow(2, 1);
}
function dead1(x) {
  return (x + 0) + (0 + x) + Math.pow(3, 1);
}
function dead2(x) {
  return (x + 0) + (0 + x) + Math.pow(4, 1);
}
function dead3(x) {
  return (x + 0) + (0 + x) + Math.pow(5, 1);
}
function dead4(x) {
  return (x + 0) + (0 + x) + Math.pow(6, 1);
}
function dead5(x) {
  return (x + 0) + (0 + x) + Math.pow(7, 1);
}

function hot(x, y) {
  const folded = (1 + 2) + (3 + 4) + (5 + 6);
  return ((x + 0) + (y + 0)) + folded + Math.pow(2, 1);
}

hot(1, 2);
