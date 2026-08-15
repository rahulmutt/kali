// Compare semantic versions and answer whether one satisfies a caret or tilde
// range. Any tool that pins a toolchain version ends up owning this function.

function parse(version) {
  const build = version.split("+")[0];
  const dash = build.indexOf("-");
  const core = dash < 0 ? build : build.slice(0, dash);
  const prerelease = dash < 0 ? "" : build.slice(dash + 1);
  const parts = core.split(".");
  return {
    major: +parts[0],
    minor: parts.length > 1 ? +parts[1] : 0,
    patch: parts.length > 2 ? +parts[2] : 0,
    prerelease: prerelease,
  };
}

function comparePrerelease(left, right) {
  if (left === right) return 0;
  if (left === "") return 1;
  if (right === "") return -1;
  return left < right ? -1 : 1;
}

function compare(leftVersion, rightVersion) {
  const left = parse(leftVersion);
  const right = parse(rightVersion);
  if (left.major !== right.major) return left.major < right.major ? -1 : 1;
  if (left.minor !== right.minor) return left.minor < right.minor ? -1 : 1;
  if (left.patch !== right.patch) return left.patch < right.patch ? -1 : 1;
  return comparePrerelease(left.prerelease, right.prerelease);
}

function satisfies(version, range) {
  const operator = range[0] === "^" || range[0] === "~" ? range[0] : "=";
  const bound = operator === "=" ? range : range.slice(1);
  if (operator === "=") return compare(version, bound) === 0;
  if (compare(version, bound) < 0) return false;

  const target = parse(bound);
  const candidate = parse(version);
  if (operator === "^") {
    return target.major === 0 ? candidate.minor === target.minor : candidate.major === target.major;
  }
  return candidate.major === target.major && candidate.minor === target.minor;
}

const VERSIONS = ["1.2.3", "1.2.10", "1.3.0", "2.0.0", "1.2.3-rc.1", "0.9.4"];

const sorted = VERSIONS.slice().sort(compare);
console.log("sorted:", sorted.join(" < "));
console.log("newest:", sorted[sorted.length - 1]);

const CHECKS = [
  { version: "1.2.10", range: "^1.2.3" },
  { version: "2.0.0", range: "^1.2.3" },
  { version: "1.2.10", range: "~1.2.3" },
  { version: "1.3.0", range: "~1.2.3" },
  { version: "1.2.3", range: "1.2.3" },
];

for (const check of CHECKS) {
  console.log(check.version + " satisfies " + check.range + ": " + satisfies(check.version, check.range));
}

console.log(satisfies("1.2.10", "^1.2.3"));
console.log("prerelease sorts before its release:", compare("1.2.3-rc.1", "1.2.3") < 0);
