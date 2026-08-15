// Replay a stock movement ledger: apply receipts and shipments in order, keep
// the running balance per SKU, flag anything that goes negative or falls under
// its reorder point.

const CATALOGUE = {
  "SKU-100": { name: "hex bolt m6", reorderAt: 40, opening: 120 },
  "SKU-200": { name: "washer m6", reorderAt: 100, opening: 400 },
  "SKU-300": { name: "bearing 608", reorderAt: 10, opening: 24 },
  "SKU-400": { name: "belt 300mm", reorderAt: 5, opening: 6 },
};

const MOVEMENTS = [
  { sku: "SKU-100", kind: "ship", quantity: 60 },
  { sku: "SKU-200", kind: "ship", quantity: 250 },
  { sku: "SKU-300", kind: "ship", quantity: 20 },
  { sku: "SKU-100", kind: "ship", quantity: 30 },
  { sku: "SKU-400", kind: "ship", quantity: 8 },
  { sku: "SKU-300", kind: "receive", quantity: 50 },
  { sku: "SKU-900", kind: "ship", quantity: 1 },
  { sku: "SKU-200", kind: "receive", quantity: 100 },
  { sku: "SKU-100", kind: "adjust", quantity: -5 },
];

function openingBalances(catalogue) {
  const balances = {};
  for (const sku of Object.keys(catalogue)) {
    balances[sku] = catalogue[sku].opening;
  }
  return balances;
}

function applyMovement(balances, movement) {
  if (balances[movement.sku] === undefined) {
    console.warn("movement references unknown sku " + movement.sku);
    return false;
  }
  if (movement.kind === "receive") {
    balances[movement.sku] += movement.quantity;
  } else if (movement.kind === "ship") {
    balances[movement.sku] -= movement.quantity;
  } else {
    balances[movement.sku] += movement.quantity;
  }
  return true;
}

const balances = openingBalances(CATALOGUE);
const lowStock = [];
const negative = [];
let applied = 0;

for (const movement of MOVEMENTS) {
  if (applyMovement(balances, movement)) applied += 1;
}

for (const sku of Object.keys(balances)) {
  const item = CATALOGUE[sku];
  const onHand = balances[sku];
  if (onHand < 0) negative.push(sku);
  if (onHand <= item.reorderAt) lowStock.push(sku);
  console.log(
    sku + "  " + item.name.padEnd(14) + String(onHand).padStart(5) +
      "  reorder at " + item.reorderAt,
  );
}

console.log("movements applied:", applied, "of", MOVEMENTS.length);
console.log(lowStock);
console.log("nothing oversold:", negative.length === 0);

for (const sku of lowStock) {
  const shortfall = CATALOGUE[sku].reorderAt - balances[sku] + 1;
  console.log("reorder " + shortfall + " of " + CATALOGUE[sku].name);
}
