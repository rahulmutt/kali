function main(){ let n = 0; function bump() { n = n + 1; return 5; } let b = (bump(), 7); console.log("b=" + b); } main();
