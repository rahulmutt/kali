function hot(seed) {
  const literal = { b: 1, 2: 2, 1: 4 };
  return (
    Object.keys(literal).length +
    Object.entries(literal).length +
    Object.values(literal).length +
    seed
  );
}

hot(0);
