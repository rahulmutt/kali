function entry() {
  function wrap(value) {
    return value;
  }

  function hot(value) {
    function consume(input) {
      const folded = (1 + 2) + (3 + 4) + (5 + 6) + (7 + 8);
      return wrap(((input + 0) + (input + 0)) + folded);
    }

    const first = consume(value);
    const second = consume(value);
    const third = consume(void 0);
    return wrap(first + second + third);
  }

  return hot(null);
}

entry();
