function dead0(x) { return (x + 0) + (0 + x); }
function dead1(x) { return (x + 0) + (0 + x); }
function dead2(x) { return (x + 0) + (0 + x); }
function dead3(x) { return (x + 0) + (0 + x); }

function hot(x, y) {
  const folded = (1 + 2) + (3 + 4) + (5 + 6) + (7 + 8);
  return ((x + 0) + (y + 0)) + folded;
}

hot(1, 2);
