function hot(seed) {
  return seed / 1 + seed * 1;
}

const literal = { 1: 4, 2: 2, b: 1 };
delete literal.b;
literal.b = 3;

const total =
  Object.keys(literal).length +
  Object.entries(literal).length +
  Object.values(literal).length +
  hot(3);

total;
