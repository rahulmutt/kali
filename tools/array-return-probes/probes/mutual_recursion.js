function f(n) { if (n === 0) { return [5]; } return g(n - 1); } function g(n) { return f(n); } console.log(f(2)[0]);
