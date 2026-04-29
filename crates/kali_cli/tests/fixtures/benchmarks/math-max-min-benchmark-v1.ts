function dead0() { return Math.max(1, 2) + Math.min(3, 4) + (1 + 2) + (3 + 4); }
function dead1() { return Math.max(5, 6) + Math.min(7, 8) + (5 + 6) + (7 + 8); }
function dead2() { return Math.max(9, 10) + Math.min(11, 12) + (9 + 10) + (11 + 12); }
function dead3() { return Math.max(13, 14) + Math.min(15, 16) + (13 + 14) + (15 + 16); }
function dead4() { return Math.max(17, 18) + Math.min(19, 20) + (17 + 18) + (19 + 20); }
function dead5() { return Math.max(21, 22) + Math.min(23, 24) + (21 + 22) + (23 + 24); }

function hot(x, y) {
  const folded =
    (1 + 2) +
    (3 + 4) +
    (5 + 6) +
    (7 + 8) +
    (9 + 10) +
    (11 + 12) +
    (13 + 14) +
    (15 + 16);

  return folded + (x + 0) + (y + 0) + Math.max(1, 2) + Math.min(3, 4);
}

hot(1, 2);
