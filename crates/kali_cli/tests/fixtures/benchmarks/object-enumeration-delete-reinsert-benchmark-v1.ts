function hot(seed) {
  const literal = { 1: 4, 2: 2, b: 1 };
  delete literal.b;
  literal.b = 3;
  return (
    Object.keys(literal).length +
    Object.entries(literal).length +
    Object.values(literal).length +
    seed
  );
}

hot(0);
