// Run-length encode and decode a string, then check the round trip. RLE is the
// first compression anybody writes for a tile map or a terminal frame buffer.

function encode(input) {
  if (input.length === 0) return "";
  let out = "";
  let run = 1;
  for (let i = 1; i <= input.length; i++) {
    if (i < input.length && input[i] === input[i - 1]) {
      run += 1;
      continue;
    }
    out += run > 1 ? String(run) + input[i - 1] : input[i - 1];
    run = 1;
  }
  return out;
}

function decode(input) {
  let out = "";
  let digits = "";
  for (let i = 0; i < input.length; i++) {
    const ch = input[i];
    if (ch >= "0" && ch <= "9") {
      digits += ch;
      continue;
    }
    const count = digits.length === 0 ? 1 : +digits;
    out += ch.repeat(count);
    digits = "";
  }
  return out;
}

const SAMPLES = [
  "aaabbbcccd",
  "wwwwwwwwwwwwbbbwwwwwwwwwwwwbbbwwwwwwwwwwww",
  "abcdef",
  "",
];

let shrunk = 0;
for (const sample of SAMPLES) {
  const packed = encode(sample);
  const back = decode(packed);
  const ok = back === sample;
  if (!ok) {
    console.warn("round trip failed for: " + sample);
  }
  if (packed.length < sample.length) shrunk += 1;
  console.log(
    "in=" + sample.length + " out=" + packed.length + " roundtrip=" + String(ok),
  );
}

console.log("samples that got smaller:", shrunk, "of", SAMPLES.length);
console.log(encode("aaabbbcccd").slice(0, 4) + "...");
