function dead0(flag) { return flag ? (1 + 2 + 3) : (4 + 5 + 6); }
function dead1(flag) { return flag ? (1 + 2 + 3) : (4 + 5 + 6); }
function dead2(flag) { return flag ? (1 + 2 + 3) : (4 + 5 + 6); }
function dead3(flag) { return flag ? (1 + 2 + 3) : (4 + 5 + 6); }

function hot(flag) {
  if (flag) {
    return (1 + 2) + (3 + 4) + (5 + 6) + (7 + 8);
  }
  return (9 + 10) + (11 + 12) + (13 + 14) + (15 + 16);
}

hot(true);
