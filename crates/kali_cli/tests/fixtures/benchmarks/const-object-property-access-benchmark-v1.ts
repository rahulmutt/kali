function entry() {
  function layer0(point) { return point.x + 0; }
  function layer1(point) { return layer0(point) + (point.y - 0); }
  function layer2(point) { return layer1(point) + (point.z * 1); }
  function layer3(point) { return layer2(point) + (point.x + 0); }
  const point = { x: 1, y: 2, z: 3 };
  const folded = (1 + 2) + (3 + 4) + (5 + 6);
  return layer3(point) + folded + layer3(point);
}

entry();
