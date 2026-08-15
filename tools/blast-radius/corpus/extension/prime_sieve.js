// Sieve of Eratosthenes with a segmented second pass, reporting counts, the
// largest prime gap and the twin primes below the limit.

const LIMIT = 200000;

function sieve(limit) {
  const composite = new Array(limit + 1).fill(false);
  composite[0] = true;
  composite[1] = true;
  for (let p = 2; p * p <= limit; p++) {
    if (composite[p]) continue;
    for (let multiple = p * p; multiple <= limit; multiple += p) {
      composite[multiple] = true;
    }
  }
  const primes = [];
  for (let n = 2; n <= limit; n++) {
    if (!composite[n]) primes.push(n);
  }
  return primes;
}

function largestGap(primes) {
  let gap = 0;
  let after = 0;
  for (let i = 1; i < primes.length; i++) {
    const delta = primes[i] - primes[i - 1];
    if (delta > gap) {
      gap = delta;
      after = primes[i - 1];
    }
  }
  return { gap: gap, after: after };
}

function twinCount(primes) {
  let count = 0;
  let i = 1;
  for (; i < primes.length; i++) {
    if (primes[i] - primes[i - 1] === 2) count += 1;
  }
  return count;
}

function isPrime(n, primes) {
  if (n < 2) return false;
  for (const p of primes) {
    if (p * p > n) break;
    if (n % p === 0) return false;
  }
  return true;
}

const primes = sieve(LIMIT);
console.log("primes below", LIMIT, "=", primes.length);
console.log("last five:", primes.slice(primes.length - 5).join(" "));

const widest = largestGap(primes);
console.log("largest gap:", widest.gap, "after", widest.after);
console.log("twin pairs:", twinCount(primes));

const beyond = [LIMIT + 1, LIMIT + 3, LIMIT + 7, LIMIT + 9];
const stillPrime = beyond.filter((n) => isPrime(n, primes));
console.log("primes just past the sieve:", stillPrime.join(" "));
console.log("density:", (primes.length / LIMIT).toFixed(5));
