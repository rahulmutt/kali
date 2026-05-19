function dead0(x) { return Math.abs((x + 0) + (0 + x)); }
function dead1(x) { return Math.abs((x + 0) + (0 + x)); }
function dead2(x) { return Math.abs((x + 0) + (0 + x)); }
function dead3(x) { return Math.abs((x + 0) + (0 + x)); }
function dead4(x) { return Math.abs((x + 0) + (0 + x)); }
function dead5(x) { return Math.abs((x + 0) + (0 + x)); }

function hot(x) {
  const folded = Math.abs((1 + 2) + (3 + 4) + (5 + 6));
  return Math.sign(Math.abs(((x + 0) + (0 + x)) + folded));
}

hot(1);
