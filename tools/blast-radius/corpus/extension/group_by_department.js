// Group a flat employee list by department, compute headcount and payroll per
// group, and print the groups largest first. The shape every reporting script
// converges on.

const EMPLOYEES = [
  { name: "Ada", department: "compiler", salary: 141000, years: 6 },
  { name: "Grace", department: "runtime", salary: 136000, years: 9 },
  { name: "Alan", department: "compiler", salary: 129000, years: 3 },
  { name: "Barbara", department: "tooling", salary: 122000, years: 4 },
  { name: "Ken", department: "runtime", salary: 118000, years: 2 },
  { name: "Edsger", department: "compiler", salary: 155000, years: 12 },
  { name: "Margaret", department: "tooling", salary: 147000, years: 11 },
];

function groupBy(records, key) {
  const groups = {};
  for (const record of records) {
    const bucket = record[key];
    if (groups[bucket] === undefined) {
      groups[bucket] = [];
    }
    groups[bucket].push(record);
  }
  return groups;
}

function summarise(members) {
  let payroll = 0;
  let seniorityYears = 0;
  let longest = members[0];
  members.forEach(function (member) {
    payroll += member.salary;
    seniorityYears += member.years;
    if (member.years > longest.years) longest = member;
  });
  return {
    headcount: members.length,
    payroll: payroll,
    averageSalary: Math.round(payroll / members.length),
    averageYears: seniorityYears / members.length,
    longestServing: longest.name,
  };
}

const groups = groupBy(EMPLOYEES, "department");
const departments = Object.keys(groups).sort(function (left, right) {
  return groups[right].length - groups[left].length;
});

console.log("departments:", departments.length, "of", EMPLOYEES.length, "people");

for (const department of departments) {
  const summary = summarise(groups[department]);
  console.log(
    department.padEnd(10) +
      "n=" + summary.headcount +
      "  payroll=" + summary.payroll +
      "  avg=" + summary.averageSalary +
      "  tenure=" + summary.averageYears.toFixed(1) +
      "  longest=" + summary.longestServing,
  );
}

const everyone = [...groups.compiler, ...groups.runtime, ...groups.tooling];
console.log("regrouped covers everyone:", everyone.length === EMPLOYEES.length);
console.log("names:", everyone.map((person) => person.name).sort().join(", "));
