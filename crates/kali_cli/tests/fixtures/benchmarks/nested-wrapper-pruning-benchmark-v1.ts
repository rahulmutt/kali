function entry() {
  const layer0 = (x) => x + 0;
  const layer1 = (x) => layer0(x);
  const layer2 = (x) => layer1(x);
  const layer3 = (x) => layer2(x);
  const folded = (1 + 2) + (3 + 4) + (5 + 6);
  return layer3(1) + folded + layer3(1);
}

entry();
