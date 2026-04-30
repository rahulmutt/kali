function dead0(x) { return ((x + 0) * 1) + ((0 + x) * 1); }
function dead1(x) { return ((x + 0) * 1) + ((0 + x) * 1); }
function dead2(x) { return ((x + 0) * 1) + ((0 + x) * 1); }
function dead3(x) { return ((x + 0) * 1) + ((0 + x) * 1); }
function dead4(x) { return ((x + 0) * 1) + ((0 + x) * 1); }
function dead5(x) { return ((x + 0) * 1) + ((0 + x) * 1); }

function hot(x, y) {
  const folded = (1 + 2) + (3 + 4) + (5 + 6);
  return (((x + 0) * 1) + ((y + 0) * 1)) + folded;
}

hot(1, 2);
