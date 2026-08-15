// CRC-32 (IEEE) over byte arrays, with the table built once at startup, plus
// an Adler-32 for comparison. This is what gets written when a format needs a
// checksum and the runtime does not ship one.

const POLYNOMIAL = 0xedb88320;

function buildTable() {
  const table = new Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) {
      if (c & 1) {
        c = POLYNOMIAL ^ (c >>> 1);
      } else {
        c >>>= 1;
      }
    }
    table[n] = c >>> 0;
  }
  return table;
}

const TABLE = buildTable();

function crc32(bytes) {
  let crc = 0xffffffff;
  for (let i = 0; i < bytes.length; i++) {
    let index = crc ^ bytes[i];
    index &= 0xff;
    crc = TABLE[index] ^ (crc >>> 8);
  }
  crc ^= 0xffffffff;
  return crc >>> 0;
}

function adler32(bytes) {
  let a = 1;
  let b = 0;
  for (let i = 0; i < bytes.length; i++) {
    a = (a + bytes[i]) % 65521;
    b = (b + a) % 65521;
  }
  let combined = b;
  combined <<= 16;
  combined |= a;
  return combined >>> 0;
}

function toBytes(text) {
  const bytes = [];
  for (let i = 0; i < text.length; i++) {
    const code = text.charCodeAt(i);
    if (code < 128) {
      bytes.push(code);
    } else {
      bytes.push(0xc0 | (code >> 6));
      bytes.push(0x80 | (code & 0x3f));
    }
  }
  return bytes;
}

function hex32(value) {
  return value.toString(16).padStart(8, "0");
}

const SAMPLES = ["", "a", "abc", "The quick brown fox jumps over the lazy dog", "123456789"];

for (const sample of SAMPLES) {
  const bytes = toBytes(sample);
  console.log(
    "len=" + String(bytes.length).padStart(3) +
      " crc32=" + hex32(crc32(bytes)) +
      " adler32=" + hex32(adler32(bytes)) +
      "  " + JSON.stringify(sample.slice(0, 20)),
  );
}

console.log("check vector 123456789 matches 0xcbf43926:", crc32(toBytes("123456789")) === 0xcbf43926);
