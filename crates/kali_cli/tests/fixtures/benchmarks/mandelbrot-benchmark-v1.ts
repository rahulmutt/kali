// The Computer Language Benchmarks Game
// https://benchmarksgame-team.pages.debian.net/benchmarksgame/
// mandelbrot — TS port normalized to Kali (no intrinsic tuning). Retains upstream attribution.
function mandelbrot(n) {
  const out = new Array(11 + (n * n >> 3));
  out[0] = 80; out[1] = 52; out[2] = 10;                 // "P4\n"
  out[3] = 49; out[4] = 50; out[5] = 56; out[6] = 32;    // "128 "
  out[7] = 49; out[8] = 50; out[9] = 56; out[10] = 10;   // "128\n"
  let p = 11;
  for (let y = 0; y < n; y = y + 1) {
    const Ci = 2.0 * y / n - 1.0;
    let byte = 0;
    let bits = 0;
    for (let x = 0; x < n; x = x + 1) {
      const Cr = 2.0 * x / n - 1.5;
      let Zr = 0.0; let Zi = 0.0; let Tr = 0.0; let Ti = 0.0;
      for (let i = 0; i < 50; i = i + 1) {
        Zi = 2.0 * Zr * Zi + Ci;
        Zr = Tr - Ti + Cr;
        Tr = Zr * Zr;
        Ti = Zi * Zi;
        if (Tr + Ti > 4.0) { break; }
      }
      let bit = 0;
      if (Tr + Ti <= 4.0) { bit = 1; }
      byte = (byte << 1) | bit;
      bits = bits + 1;
      if (bits === 8) {
        out[p] = byte;
        p = p + 1;
        byte = 0;
        bits = 0;
      }
    }
  }
  Kali.writeStdoutBytes(out);
}
mandelbrot(128);
