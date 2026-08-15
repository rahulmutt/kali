// A bit-flag permission set: grant, revoke, test and describe. Bit flags keep
// showing up because a role fits in one integer and compares in one
// instruction.

const READ = 1 << 0;
const WRITE = 1 << 1;
const EXECUTE = 1 << 2;
const DELETE = 1 << 3;
const ADMIN = 1 << 4;

const FLAG_NAMES = [
  { bit: READ, name: "read" },
  { bit: WRITE, name: "write" },
  { bit: EXECUTE, name: "execute" },
  { bit: DELETE, name: "delete" },
  { bit: ADMIN, name: "admin" },
];

const ROLES = {
  viewer: READ,
  editor: READ | WRITE,
  operator: READ | WRITE | EXECUTE,
  owner: READ | WRITE | EXECUTE | DELETE | ADMIN,
};

function grant(mask, flag) {
  let updated = mask;
  updated |= flag;
  return updated;
}

function revoke(mask, flag) {
  let updated = mask;
  updated &= ~flag;
  return updated;
}

function toggle(mask, flag) {
  let updated = mask;
  updated ^= flag;
  return updated;
}

function has(mask, flag) {
  return (mask & flag) === flag;
}

function describe(mask) {
  const names = [];
  for (const entry of FLAG_NAMES) {
    if (has(mask, entry.bit)) names.push(entry.name);
  }
  return names.length === 0 ? "none" : names.join(",");
}

function bits(mask) {
  return mask.toString(2).padStart(5, "0");
}

for (const role of Object.keys(ROLES)) {
  console.log(role.padEnd(9) + bits(ROLES[role]) + "  " + describe(ROLES[role]));
}

let session = ROLES.viewer;
session = grant(session, WRITE);
console.log("after granting write:", describe(session));

session = toggle(session, EXECUTE);
console.log("after toggling execute:", describe(session));

session = revoke(session, WRITE);
console.log("after revoking write:", describe(session));
console.log("can delete:", has(session, DELETE));
console.log("owner covers session:", (ROLES.owner & session) === session);
