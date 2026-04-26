function entry() {
  function hot(value) {
    const folded = (1 + 2) + (3 + 4) + (5 + 6) + (7 + 8);
    return ((value + 0) + (value + 0)) + folded;
  }

  return hot(null);
}

entry();
