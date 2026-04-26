function entry() {
  function hot(value) {
    const literal = { 1: 4, 2: 2, b: 1 };
    const enumerated =
      Object.keys(literal).length +
      Object.entries(literal).length +
      Object.values(literal).length;
    const folded = (1 + 2) + (3 + 4) + (5 + 6) + (7 + 8);
    return ((value + 0) + (value + 0)) + folded + enumerated;
  }

  return hot(null);
}

entry();
