function hot(seed) {
  const consumeArray = (items, value) => items[0] + items[1] + value;
  const first = consumeArray([1, 2], 1);
  const second = consumeArray([1, 2, 3], 1);
  return first + second + seed;
}

hot(0);
