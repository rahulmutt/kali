function entry() {
  const layer0 = (point) => point.x + 0;
  const layer1 = (point) => layer0(point) + (point.y - 0);
  const layer2 = (point) => layer1(point) + (point.z * 1);
  const layer3 = (point) => layer2(point) + (point.x + 0);
  const point = { x: 1, y: 2, z: 3 };
  const folded = (1 + 2) + (3 + 4) + (5 + 6);
  return layer3(point) + folded + layer3(point);
}

entry();
