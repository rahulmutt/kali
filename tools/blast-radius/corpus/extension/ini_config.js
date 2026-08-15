// Parse an INI-style config into sections, apply defaults, and report the
// effective settings. Small tools keep reaching for INI because it survives
// hand editing.

const SOURCE = [
  "; build settings",
  "[build]",
  "target = wasm32",
  "optimize = true",
  "jobs = 4",
  "",
  "[log]",
  "level = debug",
  "color=false",
  "",
  "[paths]",
  "out = ./dist",
].join("\n");

const DEFAULTS = {
  build: { target: "native", optimize: false, jobs: 1 },
  log: { level: "info", color: true },
};

function coerce(raw) {
  if (raw === "true") return true;
  if (raw === "false") return false;
  const asNumber = +raw;
  if (raw.length > 0 && asNumber === asNumber) return asNumber;
  return raw;
}

function parseIni(text) {
  const sections = {};
  let current = "";
  for (const rawLine of text.split("\n")) {
    const line = rawLine.trim();
    if (line.length === 0 || line[0] === ";" || line[0] === "#") continue;
    if (line[0] === "[" && line[line.length - 1] === "]") {
      current = line.slice(1, line.length - 1);
      sections[current] = {};
      continue;
    }
    const eq = line.indexOf("=");
    if (eq < 0) {
      console.warn("ignoring line without '=': " + line);
      continue;
    }
    if (current === "") {
      console.warn("ignoring key outside any section: " + line);
      continue;
    }
    const key = line.slice(0, eq).trim();
    sections[current][key] = coerce(line.slice(eq + 1).trim());
  }
  return sections;
}

function withDefaults(parsed, defaults) {
  const merged = {};
  for (const section of Object.keys(defaults)) {
    merged[section] = {};
    for (const key of Object.keys(defaults[section])) {
      merged[section][key] = defaults[section][key];
    }
  }
  for (const section of Object.keys(parsed)) {
    if (merged[section] === undefined) merged[section] = {};
    for (const key of Object.keys(parsed[section])) {
      merged[section][key] = parsed[section][key];
    }
  }
  return merged;
}

const config = withDefaults(parseIni(SOURCE), DEFAULTS);

for (const section of Object.keys(config)) {
  console.log("[" + section + "]");
  for (const key of Object.keys(config[section])) {
    console.log("  " + key + " = " + String(config[section][key]));
  }
}
