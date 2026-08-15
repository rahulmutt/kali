// Build a table of contents from a markdown document: pull the ATX headings,
// slugify them into anchors, and emit an indented list.

const DOC = [
  "# Kali",
  "",
  "Some intro prose.",
  "",
  "## Build from source",
  "",
  "```sh",
  "# not a heading -- this line is inside a fence",
  "cargo build",
  "```",
  "",
  "## Use the CLI",
  "",
  "### Checking a project",
  "",
  "### Building a project",
  "",
  "## License",
].join("\n");

function slugify(title) {
  let slug = "";
  for (let i = 0; i < title.length; i++) {
    const ch = title[i].toLowerCase();
    if ((ch >= "a" && ch <= "z") || (ch >= "0" && ch <= "9")) {
      slug += ch;
    } else if (slug.length > 0 && slug[slug.length - 1] !== "-") {
      slug += "-";
    }
  }
  if (slug[slug.length - 1] === "-") {
    slug = slug.slice(0, slug.length - 1);
  }
  return slug;
}

function headings(text) {
  const found = [];
  let inFence = false;
  for (const line of text.split("\n")) {
    if (line.slice(0, 3) === "```") {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;
    let level = 0;
    while (level < line.length && line[level] === "#") {
      level += 1;
    }
    if (level === 0 || line[level] !== " ") continue;
    const title = line.slice(level + 1).trim();
    found.push({ level: level, title: title, slug: slugify(title) });
  }
  return found;
}

const toc = headings(DOC);
console.log("headings found:", toc.length);

for (const heading of toc) {
  if (heading.level === 1) continue;
  const indent = "  ".repeat(heading.level - 2);
  console.log(indent + "- [" + heading.title + "](#" + heading.slug + ")");
}
