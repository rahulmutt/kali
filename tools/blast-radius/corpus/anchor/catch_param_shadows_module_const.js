const K = 2.5;
function f() {
  try {
    throw 1;
  } catch (K) {
    return K + 1;
  }
}
console.log(f());
