function entry() {
  function layer0(x) { return x + 0; }
  function layer1(x) { return layer0(x); }
  function layer2(x) { return layer1(x); }
  function layer3(x) { return layer2(x); }
  const folded = (1 + 2) + (3 + 4) + (5 + 6);
  return layer3(1) + folded + layer3(1);
}

entry();
