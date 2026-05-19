function dead0(x) {
  return (x + 0) + (0 + x) + Math.floor(2.9);
}
function dead1(x) {
  return (x + 0) + (0 + x) + Math.floor(3.9);
}
function dead2(x) {
  return (x + 0) + (0 + x) + Math.floor(4.9);
}
function dead3(x) {
  return (x + 0) + (0 + x) + Math.floor(5.9);
}
function dead4(x) {
  return (x + 0) + (0 + x) + Math.floor(6.9);
}
function dead5(x) {
  return (x + 0) + (0 + x) + Math.floor(7.9);
}

function hot(x, y) {
  const folded = (1 + 2) + (3 + 4) + (5 + 6);
  return ((x + 0) + (y + 0)) + folded + Math.floor(2.9);
}

hot(1, 2);
