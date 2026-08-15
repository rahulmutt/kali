// Amortisation schedule for a fixed-rate loan: monthly payment, the split
// between interest and principal, and the total cost. Every mortgage
// spreadsheet is this function with nicer fonts.

function monthlyPayment(principal, annualRate, months) {
  const monthlyRate = annualRate / 12;
  if (monthlyRate === 0) return principal / months;
  const growth = Math.pow(1 + monthlyRate, months);
  return (principal * monthlyRate * growth) / (growth - 1);
}

function schedule(principal, annualRate, months) {
  const payment = monthlyPayment(principal, annualRate, months);
  const monthlyRate = annualRate / 12;
  const rows = [];
  let balance = principal;
  let totalInterest = 0;

  for (let month = 1; month <= months; month++) {
    const interest = balance * monthlyRate;
    let principalPart = payment - interest;
    if (principalPart > balance) principalPart = balance;
    balance -= principalPart;
    totalInterest += interest;
    rows.push({
      month: month,
      payment: principalPart + interest,
      interest: interest,
      principal: principalPart,
      balance: balance < 0.005 ? 0 : balance,
    });
  }

  return { payment: payment, rows: rows, totalInterest: totalInterest };
}

function money(value) {
  return value.toFixed(2).padStart(12);
}

function crossoverMonth(rows) {
  for (const row of rows) {
    if (row.principal > row.interest) return row.month;
  }
  return 0;
}

const PRINCIPAL = 320000;
const RATE = 0.0525;
const YEARS = 25;
const months = YEARS * 12;

const plan = schedule(PRINCIPAL, RATE, months);

console.log("principal " + money(PRINCIPAL));
console.log("payment   " + money(plan.payment));
console.log("interest  " + money(plan.totalInterest));
console.log("total     " + money(PRINCIPAL + plan.totalInterest));
console.log("month     payment     interest    principal      balance");

for (const row of plan.rows) {
  if (row.month % 60 !== 0 && row.month !== 1 && row.month !== months) continue;
  console.log(
    String(row.month).padStart(5) + money(row.payment) + money(row.interest) +
      money(row.principal) + money(row.balance),
  );
}

console.log("principal overtakes interest in month:", crossoverMonth(plan.rows));
console.log("loan is repaid:", plan.rows[plan.rows.length - 1].balance === 0);

const interestFree = schedule(12000, 0, 24);
console.log("zero-rate payment:", interestFree.payment.toFixed(2), "total interest:", interestFree.totalInterest);
