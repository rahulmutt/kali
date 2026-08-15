// A minimal {{placeholder}} template renderer, the kind that gets written once
// per project to fill in a report or a config file without pulling in a
// templating dependency.

function lookup(context, path) {
  const parts = path.split(".");
  let value = context;
  for (let i = 0; i < parts.length; i++) {
    if (value === null || value === undefined) return undefined;
    value = value[parts[i]];
  }
  return value;
}

function render(template, context, missing = "") {
  let out = "";
  let index = 0;
  while (index < template.length) {
    const open = template.indexOf("{{", index);
    if (open < 0) {
      out += template.slice(index);
      break;
    }
    const close = template.indexOf("}}", open);
    if (close < 0) {
      out += template.slice(index);
      break;
    }
    out += template.slice(index, open);
    const key = template.slice(open + 2, close).trim();
    const value = lookup(context, key);
    if (value === undefined) {
      console.warn("unresolved placeholder: " + key);
      out += missing;
    } else {
      out += String(value);
    }
    index = close + 2;
  }
  return out;
}

const TEMPLATE = [
  "Release {{ version }} of {{ project.name }}",
  "Maintainer: {{ project.owner }}",
  "Targets: {{ targets }}",
  "Notes: {{ notes }}",
].join("\n");

const context = {
  version: "1.4.0",
  project: { name: "kali", owner: "the compiler team" },
  targets: 3,
};

console.log(render(TEMPLATE, context, "(none)"));
console.log("rendered length:", render(TEMPLATE, context).length);
