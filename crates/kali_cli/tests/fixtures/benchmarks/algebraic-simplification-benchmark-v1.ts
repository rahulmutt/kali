function dead0(x) { return (x / 1) + (x + 0); }
function dead1(x) { return (x / 1) + (x + 0); }
function dead2(x) { return (x / 1) + (x + 0); }
function dead3(x) { return (x / 1) + (x + 0); }
function dead4(x) { return (x / 1) + (x + 0); }
function dead5(x) { return (x / 1) + (x + 0); }

function hot(x, y) {
  const folded = (1 + 2) + (3 + 4) + (5 + 6);
  return ((x / 1) + (y + 0)) + folded;
}

hot(1, 2);
