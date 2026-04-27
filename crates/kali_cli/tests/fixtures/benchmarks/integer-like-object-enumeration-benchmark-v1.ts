function hot(seed) {
  const literal = {
    10: 10,
    2: 2,
    1: 1,
    0: 0,
    a: 4,
    b: 5,
    z: 6,
  };
  return (
    Object.keys(literal).length +
    Object.entries(literal).length +
    Object.values(literal).length +
    seed
  );
}

hot(0);
