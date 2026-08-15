// Join orders to customers on customer id, the way a report does before it has
// a database: build an index, walk the fact table, and account for the rows
// that do not match.

const CUSTOMERS = [
  { id: 101, name: "Northwind", tier: "gold", region: "north" },
  { id: 102, name: "Southgate", tier: "silver", region: "south" },
  { id: 103, name: "Eastfield", tier: "gold", region: "east" },
  { id: 104, name: "Westhaven", tier: "bronze", region: "west" },
];

const ORDERS = [
  { id: 9001, customerId: 101, total: 1420.5, items: 6 },
  { id: 9002, customerId: 103, total: 990, items: 2 },
  { id: 9003, customerId: 101, total: 310.25, items: 1 },
  { id: 9004, customerId: 999, total: 75, items: 1 },
  { id: 9005, customerId: 102, total: 2210.75, items: 11 },
  { id: 9006, customerId: 104, total: 55.5, items: 1 },
  { id: 9007, customerId: 103, total: 1810, items: 8 },
];

function indexBy(records, key) {
  const index = {};
  for (const record of records) {
    index[record[key]] = record;
  }
  return index;
}

function innerJoin(facts, index, foreignKey) {
  const joined = [];
  const orphans = [];
  for (const fact of facts) {
    const match = index[fact[foreignKey]];
    if (match === undefined) {
      orphans.push(fact);
      continue;
    }
    joined.push({
      orderId: fact.id,
      customer: match.name,
      tier: match.tier,
      region: match.region,
      total: fact.total,
      items: fact.items,
    });
  }
  return { joined: joined, orphans: orphans };
}

function totalsByTier(rows) {
  const totals = {};
  for (const row of rows) {
    totals[row.tier] = (totals[row.tier] === undefined ? 0 : totals[row.tier]) + row.total;
  }
  return totals;
}

const customerIndex = indexBy(CUSTOMERS, "id");
const result = innerJoin(ORDERS, customerIndex, "customerId");

console.log("joined rows:", result.joined.length, "orphaned rows:", result.orphans.length);
for (const orphan of result.orphans) {
  console.warn("order " + orphan.id + " references unknown customer " + orphan.customerId);
}

result.joined
  .filter((row) => row.total > 500)
  .sort(function (left, right) {
    return right.total - left.total;
  })
  .forEach(function (row) {
    console.log(
      String(row.orderId) + "  " + row.customer.padEnd(11) + row.tier.padEnd(8) +
        row.total.toFixed(2).padStart(9),
    );
  });

const tiers = totalsByTier(result.joined);
for (const tier of Object.keys(tiers).sort()) {
  console.log(tier + " revenue: " + tiers[tier].toFixed(2));
}
