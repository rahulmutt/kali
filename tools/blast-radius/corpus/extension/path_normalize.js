// Normalise POSIX-style paths and resolve one against another, without
// touching the filesystem -- what a bundler does to turn an import specifier
// into a canonical module id.

function normalize(path) {
  const absolute = path[0] === "/";
  const parts = path.split("/");
  const stack = [];
  for (const part of parts) {
    if (part === "" || part === ".") continue;
    if (part === "..") {
      if (stack.length > 0 && stack[stack.length - 1] !== "..") {
        stack.pop();
        continue;
      }
      if (absolute) continue;
    }
    stack.push(part);
  }
  const joined = stack.join("/");
  if (absolute) return "/" + joined;
  return joined.length === 0 ? "." : joined;
}

function dirname(path) {
  const normalized = normalize(path);
  const cut = normalized.lastIndexOf("/");
  if (cut < 0) return ".";
  if (cut === 0) return "/";
  return normalized.slice(0, cut);
}

function extname(path) {
  const base = normalize(path).split("/").pop();
  const dot = base.lastIndexOf(".");
  return dot <= 0 ? "" : base.slice(dot);
}

function resolve(from, specifier) {
  if (specifier[0] === "/") return normalize(specifier);
  if (specifier[0] !== ".") return specifier;
  return normalize(dirname(from) + "/" + specifier);
}

const CASES = [
  "/usr/local/../bin/./kali",
  "src//lib/../main.js",
  "../../up/two",
  "./same",
];

for (const path of CASES) {
  console.log(path + " -> " + normalize(path));
}

console.log("dirname:", dirname("/src/compiler/parser.js"));
console.log("extname:", extname("/src/compiler/parser.js"));
console.log("resolved:", resolve("/src/compiler/parser.js", "../runtime/gc.js"));
console.log("bare specifier passes through:", resolve("/src/main.js", "acorn"));
