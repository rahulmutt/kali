function hot(seed) {
  return (
    Object.keys('ab').length +
    Object.values('ab').length +
    Object.entries('ab').length +
    globalThis["Object"]["keys"]('ab').length +
    globalThis["Object"]["values"]('ab').length +
    globalThis["Object"]["entries"]('ab').length +
    seed
  );
}

hot(0);
