// The Computer Language Benchmarks Game
// https://benchmarksgame-team.pages.debian.net/benchmarksgame/
// binary-trees — idiomatic TS port of the canonical CLBG shape, normalized to
// Kali's pipeline (no intrinsic tuning). Retains upstream attribution.
function bottomUpTree(depth) {
  if (depth > 0) {
    return { left: bottomUpTree(depth - 1), right: bottomUpTree(depth - 1) };
  }
  return { left: null, right: null };
}

function itemCheck(t) {
  if (t.left === null) {
    return 1;
  }
  return 1 + itemCheck(t.left) + itemCheck(t.right);
}

function main() {
  const n = 21;
  const minDepth = 4;
  const maxDepth = n;
  const stretchDepth = maxDepth + 1;
  console.log(`stretch tree of depth ${stretchDepth}\t check: ${itemCheck(bottomUpTree(stretchDepth))}`);
  const longLivedTree = bottomUpTree(maxDepth);
  for (let depth = minDepth; depth <= maxDepth; depth = depth + 2) {
    const iterations = 1 << (maxDepth - depth + minDepth);
    let check = 0;
    for (let i = 1; i <= iterations; i = i + 1) {
      check = check + itemCheck(bottomUpTree(depth));
    }
    console.log(`${iterations}\t trees of depth ${depth}\t check: ${check}`);
  }
  console.log(`long lived tree of depth ${maxDepth}\t check: ${itemCheck(longLivedTree)}`);
}

main();
