function hot(seed) {
  const literal = { 1: 4, 2: 2, b: 1 };
  const bound = literal;
  const alias = bound;
  return Reflect.ownKeys(alias).length + seed;
}

hot(0);
