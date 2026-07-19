function entry() {
  function layer0(bag) { return bag[0] + 0; }
  function layer1(bag) { return layer0(bag); }
  function layer2(bag) { return layer1(bag); }
  function layer3(bag) { return layer2(bag); }
  const folded = (1 + 2) + (3 + 4) + (5 + 6);
  const bag = new Array(3);
  bag[0] = 1;
  bag[1] = 2;
  bag[2] = 3;
  return layer3(bag) + folded + layer3(bag);
}

entry();
