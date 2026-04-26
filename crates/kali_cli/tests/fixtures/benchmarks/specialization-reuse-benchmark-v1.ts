function entry() {
  function hot(input) {
    const folded = (1 + 2) + (3 + 4) + (5 + 6) + (7 + 8);
    return ((input + 0) + (input + 0)) + folded;
  }

  const first = hot(1);
  const second = hot(1);
  const third = hot(1);
  return first + second + third;
}

entry();
