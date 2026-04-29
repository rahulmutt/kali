function hot(seed) {
  const literal = { 1: 4, 2: 2, b: 1 };
  const bound = literal;
  return (
    Object.keys(bound).length +
    Object.entries(bound).length +
    Object.values(bound).length +
    seed
  );
}

hot(0);
