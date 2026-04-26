function entry() {
  let total = 0;

  if (true) {
    total += 1 + 2 + 3;
  } else {
    total += 4 + 5 + 6;
  }

  if (false) {
    total += 7 + 8 + 9;
  } else {
    total += 10 + 11 + 12;
  }

  const add = (value) => value + 0;
  const alias = add;
  return alias(total) + alias(total);
}

entry();
