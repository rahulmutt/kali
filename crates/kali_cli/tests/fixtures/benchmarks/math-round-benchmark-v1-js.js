function dead0(x) {
  return (x + 0) + (0 + x) + Math.round(2.4);
}
function dead1(x) {
  return (x + 0) + (0 + x) + Math.round(3.4);
}
function dead2(x) {
  return (x + 0) + (0 + x) + Math.round(4.4);
}
function dead3(x) {
  return (x + 0) + (0 + x) + Math.round(5.4);
}
function dead4(x) {
  return (x + 0) + (0 + x) + Math.round(6.4);
}
function dead5(x) {
  return (x + 0) + (0 + x) + Math.round(7.4);
}

function hot(x, y) {
  const folded = (1 + 2) + (3 + 4) + (5 + 6);
  return ((x + 0) + (y + 0)) + folded + Math.round(2.4);
}

hot(1, 2);
