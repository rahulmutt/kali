// Dense matrix multiply, transpose and trace over flat arrays, plus a
// Gauss-Seidel-free check that A * I == A. Flat storage because that is what
// survives a trip through a compiled kernel.

function makeMatrix(rows, cols, fill = 0) {
  const data = new Array(rows * cols);
  for (let i = 0; i < data.length; i++) {
    data[i] = fill;
  }
  return { rows: rows, cols: cols, data: data };
}

function identity(size) {
  const matrix = makeMatrix(size, size);
  for (let i = 0; i < size; i++) {
    matrix.data[i * size + i] = 1;
  }
  return matrix;
}

function fromRows(rows) {
  const height = rows.length;
  const width = rows[0].length;
  const matrix = makeMatrix(height, width);
  for (let r = 0; r < height; r++) {
    for (let c = 0; c < width; c++) {
      matrix.data[r * width + c] = rows[r][c];
    }
  }
  return matrix;
}

function multiply(left, right) {
  if (left.cols !== right.rows) {
    console.warn("dimension mismatch: " + left.cols + " vs " + right.rows);
    return makeMatrix(0, 0);
  }
  const out = makeMatrix(left.rows, right.cols);
  for (let r = 0; r < left.rows; r++) {
    for (let c = 0; c < right.cols; c++) {
      let sum = 0;
      for (let k = 0; k < left.cols; k++) {
        sum += left.data[r * left.cols + k] * right.data[k * right.cols + c];
      }
      out.data[r * out.cols + c] = sum;
    }
  }
  return out;
}

function transpose(matrix) {
  const out = makeMatrix(matrix.cols, matrix.rows);
  for (let r = 0; r < matrix.rows; r++) {
    for (let c = 0; c < matrix.cols; c++) {
      out.data[c * out.cols + r] = matrix.data[r * matrix.cols + c];
    }
  }
  return out;
}

function trace(matrix) {
  let sum = 0;
  const limit = Math.min(matrix.rows, matrix.cols);
  for (let i = 0; i < limit; i++) {
    sum += matrix.data[i * matrix.cols + i];
  }
  return sum;
}

function equal(left, right) {
  if (left.rows !== right.rows || left.cols !== right.cols) return false;
  for (let i = 0; i < left.data.length; i++) {
    if (Math.abs(left.data[i] - right.data[i]) > 1e-9) return false;
  }
  return true;
}

function render(matrix) {
  const lines = [];
  for (let r = 0; r < matrix.rows; r++) {
    const cells = [];
    for (let c = 0; c < matrix.cols; c++) {
      cells.push(String(matrix.data[r * matrix.cols + c]).padStart(6));
    }
    lines.push(cells.join(""));
  }
  return lines.join("\n");
}

const a = fromRows([
  [1, 2, 3],
  [4, 5, 6],
]);
const b = fromRows([
  [7, 8],
  [9, 10],
  [11, 12],
]);

const product = multiply(a, b);
console.log(render(product));
console.log("trace:", trace(product));
console.log("A * I equals A:", equal(multiply(a, identity(3)), a));
console.log("(AB)^T equals B^T A^T:", equal(transpose(product), multiply(transpose(b), transpose(a))));
