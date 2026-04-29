function hot(seed) {
  const literal = { 1: 4, 2: 2, b: 1 };
  const bound = literal;
  return Reflect.ownKeys(bound).length + seed;
}

hot(0);
