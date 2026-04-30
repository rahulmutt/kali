function outer() {
  function middle() {
    return (1 + 2) + (3 + 4) + (5 + 6);
  }

  return middle() + middle();
}

outer();
