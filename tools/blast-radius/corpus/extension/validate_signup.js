// Validate a batch of signup records against a rule table and report the
// failures per field. Written as data plus small predicates so a new rule is a
// new table entry rather than a new branch.

const RULES = {
  username: {
    required: true,
    check: function (value) {
      if (typeof value !== "string") return "must be text";
      if (value.length < 3) return "must be at least 3 characters";
      if (value.length > 20) return "must be at most 20 characters";
      for (let i = 0; i < value.length; i++) {
        const ch = value[i];
        const ok = (ch >= "a" && ch <= "z") || (ch >= "0" && ch <= "9") || ch === "_";
        if (!ok) return "may only contain a-z, 0-9 and underscore";
      }
      return "";
    },
  },
  email: {
    required: true,
    check: function (value) {
      if (typeof value !== "string") return "must be text";
      const at = value.indexOf("@");
      if (at <= 0) return "must contain a local part and a domain";
      if (value.indexOf("@", at + 1) >= 0) return "must contain exactly one @";
      const domain = value.slice(at + 1);
      if (domain.indexOf(".") <= 0) return "domain must contain a dot";
      return "";
    },
  },
  age: {
    required: false,
    check: (value) => {
      if (value === null || value === undefined) return "";
      const numeric = +value;
      if (numeric !== numeric) return "must be a number";
      if (numeric < 13) return "must be 13 or older";
      return "";
    },
  },
};

const RECORDS = [
  { username: "ada", email: "ada@lovelace.org", age: 36 },
  { username: "x", email: "x@y.z" },
  { username: "Bad Name", email: "not-an-email", age: 11 },
  { username: "grace_h", email: "grace@navy@mil", age: "42" },
  { email: "anon@example.com" },
];

function validate(record, rules) {
  const problems = [];
  for (const field of Object.keys(rules)) {
    const rule = rules[field];
    const value = record[field];
    if (value === undefined || value === null) {
      if (rule.required) problems.push({ field: field, message: "is required" });
      continue;
    }
    const check = rule.check;
    const message = check(value);
    if (message !== "") {
      problems.push({ field: field, message: message });
    }
  }
  return problems;
}

function isValid(record) {
  return validate(record, RULES).length === 0;
}

let accepted = 0;
const failuresByField = {};

for (let i = 0; i < RECORDS.length; i++) {
  const record = RECORDS[i];
  const problems = validate(record, RULES);
  if (problems.length === 0) {
    accepted += 1;
    console.log("record " + i + " ok:", isValid(record));
    continue;
  }
  problems.forEach(function (problem) {
    failuresByField[problem.field] = (failuresByField[problem.field] || 0) + 1;
    console.log("record " + i + " " + problem.field + " " + problem.message);
  });
}

console.log("accepted:", accepted, "of", RECORDS.length);
for (const field of Object.keys(failuresByField).sort()) {
  console.log(field + " failed " + failuresByField[field] + " time(s)");
}
