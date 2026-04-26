function hot(seed) {
  const literal = { 1: 4, 2: 2, b: 1 };
  const alias0 = literal;
  const alias1 = alias0;
  const alias2 = alias1;
  return (
    Object.keys(alias2).length +
    Object.entries(alias2).length +
    Object.values(alias2).length +
    seed
  );
}

hot(0);
