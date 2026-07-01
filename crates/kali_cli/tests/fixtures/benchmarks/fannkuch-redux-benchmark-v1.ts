// The Computer Language Benchmarks Game
// https://benchmarksgame-team.pages.debian.net/benchmarksgame/
// fannkuch-redux — idiomatic TS port of the Node.js / JavaScript submission,
// normalized to Kali's pipeline (integer-only, no intrinsic tuning).
// Retains upstream attribution per the CLBG license terms.
function fannkuch(n) {
  const perm = new Array(n);
  const perm1 = new Array(n);
  const count = new Array(n);
  for (let i = 0; i < n; i = i + 1) {
    perm1[i] = i;
  }
  let maxFlipsCount = 0;
  let permCount = 0;
  let checksum = 0;
  let r = n;
  while (true) {
    while (r !== 1) {
      count[r - 1] = r;
      r = r - 1;
    }
    for (let i = 0; i < n; i = i + 1) {
      perm[i] = perm1[i];
    }
    let flipsCount = 0;
    let k = perm[0];
    while (k !== 0) {
      let i = 0;
      let j = k;
      while (i < j) {
        const temp = perm[i];
        perm[i] = perm[j];
        perm[j] = temp;
        i = i + 1;
        j = j - 1;
      }
      flipsCount = flipsCount + 1;
      k = perm[0];
    }
    if (flipsCount > maxFlipsCount) {
      maxFlipsCount = flipsCount;
    }
    if (permCount % 2 === 0) {
      checksum = checksum + flipsCount;
    } else {
      checksum = checksum - flipsCount;
    }
    let done = false;
    while (true) {
      if (r === n) {
        done = true;
        break;
      }
      const perm0 = perm1[0];
      let i = 0;
      while (i < r) {
        perm1[i] = perm1[i + 1];
        i = i + 1;
      }
      perm1[r] = perm0;
      count[r] = count[r] - 1;
      if (count[r] > 0) {
        break;
      }
      r = r + 1;
    }
    if (done) {
      break;
    }
    permCount = permCount + 1;
  }
  console.log(checksum);
  console.log("Pfannkuchen(" + n + ") = " + maxFlipsCount);
}
fannkuch(7);
