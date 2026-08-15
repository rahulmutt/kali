// Convert identifiers between snake_case, kebab-case, camelCase and
// PascalCase, and title-case a heading. Code generators and CLI flag parsers
// need this pair of functions constantly.

const ACRONYMS = ["id", "url", "api", "io", "http"];
const SMALL_WORDS = ["a", "an", "and", "the", "of", "for", "to", "in", "on"];

function words(identifier) {
  const parts = [];
  let current = "";
  for (let i = 0; i < identifier.length; i++) {
    const ch = identifier[i];
    if (ch === "_" || ch === "-" || ch === " ") {
      if (current.length > 0) parts.push(current.toLowerCase());
      current = "";
      continue;
    }
    const isUpper = ch >= "A" && ch <= "Z";
    const previous = current.length === 0 ? "" : current[current.length - 1];
    const previousIsUpper = previous >= "A" && previous <= "Z";
    const next = i + 1 < identifier.length ? identifier[i + 1] : "";
    const startsNewWord = isUpper && current.length > 0 && (!previousIsUpper || (next >= "a" && next <= "z"));
    if (startsNewWord) {
      parts.push(current.toLowerCase());
      current = ch;
      continue;
    }
    current += ch;
  }
  if (current.length > 0) parts.push(current.toLowerCase());
  return parts;
}

function capitalise(word) {
  return word.charAt(0).toUpperCase() + word.slice(1);
}

function toSnake(identifier) {
  return words(identifier).join("_");
}

function toKebab(identifier) {
  return words(identifier).join("-");
}

function toCamel(identifier) {
  const parts = words(identifier);
  let out = parts[0];
  for (let i = 1; i < parts.length; i++) {
    out += ACRONYMS.indexOf(parts[i]) >= 0 ? parts[i].toUpperCase() : capitalise(parts[i]);
  }
  return out;
}

function toPascal(identifier) {
  return capitalise(toCamel(identifier));
}

function titleCase(heading) {
  const parts = heading.split(" ");
  const out = [];
  for (let i = 0; i < parts.length; i++) {
    const word = parts[i].toLowerCase();
    if (i > 0 && SMALL_WORDS.indexOf(word) >= 0) {
      out.push(word);
      continue;
    }
    out.push(capitalise(word));
  }
  return out.join(" ");
}

const IDENTIFIERS = [
  "user_id",
  "parse-http-header",
  "blastRadiusScore",
  "HTTPServerPort",
  "already_snake_case",
];

for (const identifier of IDENTIFIERS) {
  console.log(
    identifier.padEnd(22) + toSnake(identifier).padEnd(22) + toKebab(identifier).padEnd(22) +
      toCamel(identifier).padEnd(22) + toPascal(identifier),
  );
}

console.log(titleCase("a definition of blast radius for the compiler frontier"));
console.log("snake of camel is stable:", toSnake(toCamel("user_id")) === "user_id");
console.log("first letter of a heading is always upper:", titleCase("of mice and men").charAt(0) === "O");
