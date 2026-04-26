function entry() {
  const layer0 = (bag) => bag[0] + 0;
  const layer1 = (bag) => layer0(bag);
  const layer2 = (bag) => layer1(bag);
  const layer3 = (bag) => layer2(bag);
  const folded = (1 + 2) + (3 + 4) + (5 + 6);
  const bag = [1, 2, 3];
  return layer3(bag) + folded + layer3(bag);
}

entry();
