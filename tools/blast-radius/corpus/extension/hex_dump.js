// Format a byte buffer as a classic hex dump: offset, sixteen hex columns, and
// a printable-ASCII gutter. Every binary format debugging session starts here.

function encodeUtf8(text) {
  const bytes = [];
  for (let i = 0; i < text.length; i++) {
    const code = text.charCodeAt(i);
    if (code < 0x80) {
      bytes.push(code);
    } else if (code < 0x800) {
      bytes.push(0xc0 | (code >> 6), 0x80 | (code & 0x3f));
    } else {
      bytes.push(0xe0 | (code >> 12), 0x80 | ((code >> 6) & 0x3f), 0x80 | (code & 0x3f));
    }
  }
  return bytes;
}

function hexByte(value) {
  return value.toString(16).padStart(2, "0");
}

function printable(value) {
  return value >= 0x20 && value < 0x7f ? String.fromCharCode(value) : ".";
}

function dumpLine(bytes, offset, width = 16) {
  const columns = [];
  const gutter = [];
  for (let i = 0; i < width; i++) {
    const index = offset + i;
    if (index < bytes.length) {
      columns.push(hexByte(bytes[index]));
      gutter.push(printable(bytes[index]));
    } else {
      columns.push("  ");
      gutter.push(" ");
    }
    if (i === width / 2 - 1) columns.push("");
  }
  return offset.toString(16).padStart(8, "0") + "  " + columns.join(" ") + " |" + gutter.join("") + "|";
}

function dump(bytes, width = 16) {
  const lines = [];
  for (let offset = 0; offset < bytes.length; offset += width) {
    lines.push(dumpLine(bytes, offset, width));
  }
  lines.push(bytes.length.toString(16).padStart(8, "0"));
  return lines;
}

const HEADER = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
const payload = HEADER.concat(encodeUtf8("kali build --target wasm32 → module.wasm"));

for (const line of dump(payload)) {
  console.log(line);
}

console.log("bytes:", payload.length);
console.log("looks like a wasm module:", payload[0] === 0x00 && payload[1] === 0x61);
