function entry() {
  function hot(value) {
    const folded = (1n + 2n) + (3n + 4n) + (5n + 6n) + (7n + 8n);
    return (value + 0n) + folded;
  }

  return hot(1n) + hot(2n) + hot(1n);
}

entry();
