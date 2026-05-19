function dead0(x, y) { return Math.imul(x + 0, y + 0); }
function dead1(x, y) { return Math.imul(x + 0, y + 0); }
function dead2(x, y) { return Math.imul(x + 0, y + 0); }
function dead3(x, y) { return Math.imul(x + 0, y + 0); }
function dead4(x, y) { return Math.imul(x + 0, y + 0); }
function dead5(x, y) { return Math.imul(x + 0, y + 0); }

function hot(x, y) {
  const folded = Math.imul((1 + 2), (3 + 4));
  const extra = Math.imul((5 + 6), (7 + 8));
  return Math.imul(x + 0, y + 0) + folded + extra;
}

hot(1, 2);
