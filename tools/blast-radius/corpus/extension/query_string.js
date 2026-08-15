// Parse and rebuild a URL query string: repeated keys collapse into arrays,
// flag keys with no value become true, and the output is sorted so two
// equivalent URLs compare equal.

function decodeComponent(text) {
  let out = "";
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (ch === "+") {
      out += " ";
    } else if (ch === "%" && i + 2 < text.length) {
      const hex = text.slice(i + 1, i + 3);
      out += String.fromCharCode(parseInt(hex, 16));
      i += 2;
    } else {
      out += ch;
    }
  }
  return out;
}

function encodeComponent(text) {
  let out = "";
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    const safe =
      (ch >= "a" && ch <= "z") || (ch >= "A" && ch <= "Z") || (ch >= "0" && ch <= "9") ||
      ch === "-" || ch === "_" || ch === "." || ch === "~";
    if (safe) {
      out += ch;
    } else if (ch === " ") {
      out += "+";
    } else {
      out += "%" + ch.charCodeAt(0).toString(16).toUpperCase().padStart(2, "0");
    }
  }
  return out;
}

function parseQuery(query) {
  const params = {};
  const start = query[0] === "?" ? 1 : 0;
  for (const pair of query.slice(start).split("&")) {
    if (pair.length === 0) continue;
    const eq = pair.indexOf("=");
    const key = decodeComponent(eq < 0 ? pair : pair.slice(0, eq));
    const value = eq < 0 ? true : decodeComponent(pair.slice(eq + 1));
    const existing = params[key];
    if (existing === undefined) {
      params[key] = value;
    } else if (Array.isArray(existing)) {
      existing.push(value);
    } else {
      params[key] = [existing, value];
    }
  }
  return params;
}

function stringifyQuery(params) {
  const pieces = [];
  for (const key of Object.keys(params).sort()) {
    const value = params[key];
    if (Array.isArray(value)) {
      for (const item of value) {
        pieces.push(encodeComponent(key) + "=" + encodeComponent(item));
      }
    } else if (value === true) {
      pieces.push(encodeComponent(key));
    } else {
      pieces.push(encodeComponent(key) + "=" + encodeComponent(value));
    }
  }
  return pieces.join("&");
}

const QUERY = "?tag=wasm&tag=compiler&q=blast+radius&verbose&limit=25";
const parsed = parseQuery(QUERY);

console.log("keys:", Object.keys(parsed).length);
console.log("tags:", parsed.tag);
console.log("verbose flag is set:", parsed.verbose === true);
console.log("canonical:", stringifyQuery(parsed));
console.log("stable:", stringifyQuery(parseQuery(stringifyQuery(parsed))) === stringifyQuery(parsed));
