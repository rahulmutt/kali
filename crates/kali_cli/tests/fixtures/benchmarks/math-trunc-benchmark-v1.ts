function dead0(x) { return Math.trunc(x + 0.5); }
function dead1(x) { return Math.trunc(x + 0.5); }
function dead2(x) { return Math.trunc(x + 0.5); }
function dead3(x) { return Math.trunc(x + 0.5); }
function dead4(x) { return Math.trunc(x + 0.5); }
function dead5(x) { return Math.trunc(x + 0.5); }

function hot(x, y) {
  const folded = Math.trunc((1 + 2) + (3 + 4) + (5 + 6));
  return Math.trunc((x + 0) + (y + 0)) + folded;
}

hot(1, 2);
