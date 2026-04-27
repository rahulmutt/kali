function hot(seed) {
  const left = [seed, seed + 1, 1, 2];
  const right = [seed + 2, seed + 3, 3, 4];
  const folded = (1 + 2) + (3 + 4) + (5 + 6) + (7 + 8);
  return (
    left[0] +
    right[0] +
    left[1] +
    right[1] +
    left[2] +
    right[2] +
    left[3] +
    right[3] +
    (left[0] + right[0]) +
    (left[1] + right[1]) +
    folded
  );
}

hot(1);
