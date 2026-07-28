var x = 20;
switch (x) {
  case 10: console.log("hit=100"); throw "t100";
  case 20: console.log("hit=200"); throw "t200";
  default: console.log("hit=900"); throw "t900";
}
