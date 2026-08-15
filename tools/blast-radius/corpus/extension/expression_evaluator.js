// Evaluate arithmetic expressions: hand-written lexer, shunting-yard to RPN,
// then a stack machine. The calculator that every config format eventually
// needs so a field can say "1024 * 64".

const PRECEDENCE = { "+": 1, "-": 1, "*": 2, "/": 2, "%": 2, "^": 3 };
const RIGHT_ASSOCIATIVE = { "^": true };

function tokenize(source) {
  const tokens = [];
  let i = 0;
  while (i < source.length) {
    const ch = source[i];
    switch (ch) {
      case " ":
      case "\t":
        i += 1;
        continue;
      case "(":
      case ")":
        tokens.push({ type: "paren", text: ch });
        i += 1;
        continue;
      case "+":
      case "-":
      case "*":
      case "/":
      case "%":
      case "^":
        tokens.push({ type: "operator", text: ch });
        i += 1;
        continue;
      default:
        break;
    }

    if ((ch >= "0" && ch <= "9") || ch === ".") {
      let text = "";
      for (; i < source.length && ((source[i] >= "0" && source[i] <= "9") || source[i] === "."); i++) {
        text += source[i];
      }
      tokens.push({ type: "number", text: text, value: +text });
      continue;
    }

    console.warn("skipping unexpected character: " + ch);
    i += 1;
  }
  return tokens;
}

function toRpn(tokens) {
  const output = [];
  const operators = [];
  for (const token of tokens) {
    if (token.type === "number") {
      output.push(token);
      continue;
    }
    if (token.type === "paren") {
      if (token.text === "(") {
        operators.push(token);
      } else {
        while (operators.length > 0 && operators[operators.length - 1].text !== "(") {
          output.push(operators.pop());
        }
        operators.pop();
      }
      continue;
    }
    while (operators.length > 0) {
      const top = operators[operators.length - 1];
      if (top.text === "(") break;
      const higher = PRECEDENCE[top.text] > PRECEDENCE[token.text];
      const equal = PRECEDENCE[top.text] === PRECEDENCE[token.text];
      if (higher || (equal && !RIGHT_ASSOCIATIVE[token.text])) {
        output.push(operators.pop());
        continue;
      }
      break;
    }
    operators.push(token);
  }
  while (operators.length > 0) {
    output.push(operators.pop());
  }
  return output;
}

function apply(operator, left, right) {
  switch (operator) {
    case "+":
      return left + right;
    case "-":
      return left - right;
    case "*":
      return left * right;
    case "/":
      return right === 0 ? 0 : left / right;
    case "%":
      return right === 0 ? 0 : left % right;
    default:
      return Math.pow(left, right);
  }
}

function evaluate(source) {
  const stack = [];
  for (const token of toRpn(tokenize(source))) {
    if (token.type === "number") {
      stack.push(token.value);
      continue;
    }
    const right = stack.pop();
    const left = stack.pop();
    stack.push(apply(token.text, left, right));
  }
  return stack.length === 0 ? 0 : stack[stack.length - 1];
}

const EXPRESSIONS = [
  "1 + 2 * 3",
  "(1 + 2) * 3",
  "2 ^ 3 ^ 2",
  "1024 * 64",
  "10 % 4 + 0.5",
  "7 / 0",
  "3 + $ 4",
];

for (const expression of EXPRESSIONS) {
  console.log(expression.padEnd(14) + "= " + evaluate(expression));
}

console.log("token count for the first expression:", tokenize(EXPRESSIONS[0]).length);
console.log("rpn:", toRpn(tokenize("(1 + 2) * 3")).map((token) => token.text).join(" "));
