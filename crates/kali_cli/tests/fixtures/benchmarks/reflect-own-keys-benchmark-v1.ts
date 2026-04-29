function hot(seed) {
  const literal = { 1: 4, 2: 2, b: 1 };
  return Reflect.ownKeys(literal).length + seed;
}

hot(0);
