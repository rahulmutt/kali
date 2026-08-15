// Base64 encode and decode without leaning on any runtime helper, because the
// moment a payload has to cross a text-only boundary this is what gets
// inlined.

const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

function encode(bytes) {
  let out = "";
  for (let i = 0; i < bytes.length; i += 3) {
    const b0 = bytes[i];
    const b1 = i + 1 < bytes.length ? bytes[i + 1] : 0;
    const b2 = i + 2 < bytes.length ? bytes[i + 2] : 0;
    const triple = (b0 << 16) | (b1 << 8) | b2;
    out += ALPHABET[(triple >> 18) & 63];
    out += ALPHABET[(triple >> 12) & 63];
    out += i + 1 < bytes.length ? ALPHABET[(triple >> 6) & 63] : "=";
    out += i + 2 < bytes.length ? ALPHABET[triple & 63] : "=";
  }
  return out;
}

function decode(text) {
  const bytes = [];
  let accumulator = 0;
  let bits = 0;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (ch === "=") break;
    const value = ALPHABET.indexOf(ch);
    if (value < 0) {
      console.warn("skipping non-base64 character: " + ch);
      continue;
    }
    accumulator <<= 6;
    accumulator |= value;
    bits += 6;
    if (bits >= 8) {
      bits -= 8;
      bytes.push((accumulator >> bits) & 255);
    }
  }
  return bytes;
}

function toBytes(text) {
  const bytes = [];
  for (let i = 0; i < text.length; i++) {
    bytes.push(text.charCodeAt(i) & 255);
  }
  return bytes;
}

function toText(bytes) {
  let out = "";
  for (const byte of bytes) {
    out += String.fromCharCode(byte);
  }
  return out;
}

const CASES = ["", "f", "fo", "foo", "foob", "fooba", "foobar", "any carnal pleasure."];

for (const text of CASES) {
  const encoded = encode(toBytes(text));
  const decoded = toText(decode(encoded));
  console.log(JSON.stringify(text).padEnd(24) + encoded.padEnd(32) + String(decoded === text));
}

console.log("one byte pads with two '=':", encode(toBytes("f")).slice(2) === "==");
console.log("output length is always a multiple of 4:", encode(toBytes("fooba")).length % 4 === 0);
