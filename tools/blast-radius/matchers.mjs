// tools/blast-radius/matchers.mjs
//
// One matcher per countable predicate in predicates.json. The matcher name here
// MUST equal the `matcher` field there; count.mjs checks that and refuses to run
// if they disagree.
//
// acorn, not kali_parser. Counting the constructs kali miscompiles with kali's
// own parser is the confounded-instrument trap sweep-common.md rule 3 exists to
// prevent, and R-49 is the proof it is not hypothetical: parse_switch_statement
// silently reparented every post-switch statement for weeks with the suite green.
//
// Each matcher implements what its catalogue record's `description` says, no
// more and no less. Where a record states an upper bound with a disclosure
// clause naming what the AST cannot see, the matcher implements the shape as
// written and does not narrow it on the strength of the disclosure.

import * as acorn from "acorn";

const PARSE_OPTIONS = { ecmaVersion: 2024, sourceType: "script", allowReturnOutsideFunction: false };

export function parse(source) {
  try {
    return acorn.parse(source, PARSE_OPTIONS);
  } catch (cause) {
    // Never swallow this into a zero count: a file that fails to parse would
    // otherwise report "this construct does not appear here", which is a
    // measurement it did not make.
    throw new Error(`parse failed: ${cause.message}`, { cause });
  }
}

// --------------------------------------------------------------------------
// Generic traversal
// --------------------------------------------------------------------------

const SKIP_KEYS = new Set(["start", "end", "loc", "range", "type", "raw", "regex", "bigint"]);

/** Direct child nodes of `node`, in source order of its own keys. */
function children(node) {
  const out = [];
  for (const key of Object.keys(node)) {
    if (SKIP_KEYS.has(key)) continue;
    const value = node[key];
    if (Array.isArray(value)) {
      for (const each of value) {
        if (each && typeof each.type === "string") out.push(each);
      }
    } else if (value && typeof value.type === "string") {
      out.push(value);
    }
  }
  return out;
}

const FUNCTION_TYPES = new Set(["FunctionDeclaration", "FunctionExpression", "ArrowFunctionExpression"]);
const LOOP_TYPES = new Set(["ForStatement", "ForInStatement", "ForOfStatement", "WhileStatement", "DoWhileStatement"]);
const COMPARISON_OPERATORS = new Set(["==", "!=", "===", "!==", "<", "<=", ">", ">="]);
const NULL_LIKE_EQUALITY_OPERATORS = new Set(["==", "!=", "===", "!=="]);
const BITWISE_COMPOUND_OPERATORS = new Set(["&=", "|=", "^=", "<<=", ">>=", ">>>="]);

/** Names bound by a binding pattern. */
function declaredNames(node, out) {
  if (!node) return out;
  if (node.type === "Identifier") out.add(node.name);
  else if (node.type === "ObjectPattern") node.properties.forEach((p) => declaredNames(p.value ?? p.argument, out));
  else if (node.type === "ArrayPattern") node.elements.forEach((e) => declaredNames(e, out));
  else if (node.type === "AssignmentPattern") declaredNames(node.left, out);
  else if (node.type === "RestElement") declaredNames(node.argument, out);
  return out;
}

// --------------------------------------------------------------------------
// Scope analysis
//
// Several records are stated in terms of what a name is bound to (R-02's
// declarator kind and binding role, R-10's enclosing binding, R-12's alias
// chain, R-29's const target, R-30's producers, R-47's let-bound array). A
// binding walk over the AST is still pure syntax -- it reads nothing the source
// does not say -- so it stays inside the "decidable from an acorn AST" line the
// catalogue drew.
// --------------------------------------------------------------------------

class Scope {
  constructor(parent, kind) {
    this.parent = parent;
    this.kind = kind; // "function" | "block"
    this.bindings = new Map();
  }

  declare(name, binding) {
    if (!this.bindings.has(name)) this.bindings.set(name, binding);
  }

  lookup(name) {
    for (let scope = this; scope; scope = scope.parent) {
      const found = scope.bindings.get(name);
      if (found) return found;
    }
    return null;
  }
}

/** `var` and function declarations hoisted to the nearest function scope. */
function collectHoisted(node, scope) {
  for (const child of children(node)) {
    if (FUNCTION_TYPES.has(child.type)) {
      if (child.type === "FunctionDeclaration" && child.id) {
        scope.declare(child.id.name, { kind: "function", init: child, node: child });
      }
      continue; // a nested function's own vars belong to its own scope
    }
    if (child.type === "VariableDeclaration" && child.kind === "var") {
      for (const declarator of child.declarations) {
        for (const name of declaredNames(declarator.id, new Set())) {
          scope.declare(name, {
            kind: "var",
            init: declarator.id.type === "Identifier" ? declarator.init : null,
            node: declarator,
          });
        }
      }
    }
    collectHoisted(child, scope);
  }
}

/** `let`/`const`/class/function declarations directly in a statement list. */
function collectLexical(statements, scope) {
  for (const statement of statements) {
    if (statement.type === "VariableDeclaration" && statement.kind !== "var") {
      for (const declarator of statement.declarations) {
        for (const name of declaredNames(declarator.id, new Set())) {
          scope.declare(name, {
            kind: statement.kind,
            init: declarator.id.type === "Identifier" ? declarator.init : null,
            node: declarator,
          });
        }
      }
    } else if (statement.type === "FunctionDeclaration" && statement.id) {
      scope.declare(statement.id.name, { kind: "function", init: statement, node: statement });
    } else if (statement.type === "ClassDeclaration" && statement.id) {
      scope.declare(statement.id.name, { kind: "class", init: statement, node: statement });
    }
  }
}

/**
 * One walk that records, for every node: its parent, the scope its identifier
 * references resolve in, the nearest enclosing function (null at module scope),
 * and its position in document order.
 */
function analyze(ast) {
  const parentOf = new Map();
  const scopeOf = new Map();
  const functionOf = new Map();
  const nodesByType = new Map();
  const all = [];

  const record = (node) => {
    all.push(node);
    const bucket = nodesByType.get(node.type);
    if (bucket) bucket.push(node);
    else nodesByType.set(node.type, [node]);
  };

  const visit = (node, parent, scope, fn) => {
    parentOf.set(node, parent);
    scopeOf.set(node, scope);
    functionOf.set(node, fn);
    record(node);

    let childScope = scope;
    let childFn = fn;

    if (node.type === "Program") {
      childScope = new Scope(null, "function");
      collectHoisted(node, childScope);
      collectLexical(node.body, childScope);
      scopeOf.set(node, childScope);
    } else if (FUNCTION_TYPES.has(node.type)) {
      childFn = node;
      childScope = new Scope(scope, "function");
      for (const param of node.params) {
        for (const name of declaredNames(param, new Set())) {
          childScope.declare(name, { kind: "param", init: null, node: param });
        }
      }
      if (node.body.type === "BlockStatement") {
        collectHoisted(node.body, childScope);
        collectLexical(node.body.body, childScope);
      }
    } else if (node.type === "BlockStatement" && !(parent && FUNCTION_TYPES.has(parent.type))) {
      childScope = new Scope(scope, "block");
      collectLexical(node.body, childScope);
    } else if (node.type === "SwitchStatement") {
      childScope = new Scope(scope, "block");
      for (const each of node.cases) collectLexical(each.consequent, childScope);
    } else if (node.type === "ForStatement" || node.type === "ForInStatement" || node.type === "ForOfStatement") {
      const head = node.type === "ForStatement" ? node.init : node.left;
      if (head && head.type === "VariableDeclaration" && head.kind !== "var") {
        childScope = new Scope(scope, "block");
        collectLexical([head], childScope);
      }
    } else if (node.type === "CatchClause" && node.param) {
      childScope = new Scope(scope, "block");
      for (const name of declaredNames(node.param, new Set())) {
        childScope.declare(name, { kind: "catch", init: null, node: node.param });
      }
    }

    for (const child of children(node)) visit(child, node, childScope, childFn);
  };

  visit(ast, null, null, null);

  return {
    parentOf,
    scopeOf,
    functionOf,
    all,
    of(type) {
      return nodesByType.get(type) ?? [];
    },
    /** The binding an identifier node resolves to, or null. */
    binding(identifier) {
      if (!identifier || identifier.type !== "Identifier") return null;
      const scope = scopeOf.get(identifier);
      return scope ? scope.lookup(identifier.name) : null;
    },
  };
}

const ANALYSES = new WeakMap();

/** Memoized per-AST analysis, so each matcher can be written independently. */
function analysisOf(ast) {
  let found = ANALYSES.get(ast);
  if (!found) {
    found = analyze(ast);
    ANALYSES.set(ast, found);
  }
  return found;
}

// --------------------------------------------------------------------------
// Shared shape predicates
// --------------------------------------------------------------------------

const isLiteral = (n) => Boolean(n) && n.type === "Literal";
const isNumericLiteral = (n) => isLiteral(n) && typeof n.value === "number";
const isStringLiteral = (n) => isLiteral(n) && typeof n.value === "string";
const isBooleanLiteral = (n) => isLiteral(n) && typeof n.value === "boolean";
const isNullLiteral = (n) => isLiteral(n) && n.raw === "null";
const isUndefinedIdentifier = (n) => Boolean(n) && n.type === "Identifier" && n.name === "undefined";
const isSubstitutionFreeTemplate = (n) => Boolean(n) && n.type === "TemplateLiteral" && n.expressions.length === 0;

/** `recv.name(...)` in the non-computed spelling. */
function isNamedProperty(node, name) {
  return (
    Boolean(node) &&
    node.type === "MemberExpression" &&
    !node.computed &&
    node.property.type === "Identifier" &&
    node.property.name === name
  );
}

/** `recv.name` or `recv["name"]`. */
function isNamedPropertyEitherSpelling(node, name) {
  if (!node || node.type !== "MemberExpression") return false;
  return node.computed ? isStringLiteral(node.property) && node.property.value === name : isNamedProperty(node, name);
}

/** A call to `console.<sink>` in the non-computed spelling. */
function isConsoleCall(node, sink) {
  if (!node || node.type !== "CallExpression") return false;
  const callee = node.callee;
  return (
    callee.type === "MemberExpression" &&
    !callee.computed &&
    callee.object.type === "Identifier" &&
    callee.object.name === "console" &&
    callee.property.type === "Identifier" &&
    (sink === undefined || callee.property.name === sink)
  );
}

/** Is this node an operand of a `+` concatenation? */
function isConcatOperand(node, analysis) {
  const parent = analysis.parentOf.get(node);
  return Boolean(parent) && parent.type === "BinaryExpression" && parent.operator === "+";
}

/** Follow `const b = a` alias chains to the declarator whose init is an array literal. */
function resolvesToArrayLiteral(identifier, analysis, seen = new Set()) {
  const binding = analysis.binding(identifier);
  if (!binding || !binding.init || seen.has(binding)) return false;
  seen.add(binding);
  if (binding.init.type === "ArrayExpression") return true;
  if (binding.init.type === "Identifier") return resolvesToArrayLiteral(binding.init, analysis, seen);
  return false;
}

/** Is this binding an alias declarator -- `const b = a` where `a` is an array literal binding? */
function isArrayAliasBinding(binding, analysis) {
  return Boolean(binding) && Boolean(binding.init) && binding.init.type === "Identifier" && resolvesToArrayLiteral(binding.init, analysis);
}

/** The function node an identifier callee resolves to, or null if not a user function. */
function userFunctionOf(identifier, analysis) {
  const binding = analysis.binding(identifier);
  if (!binding || !binding.init) return null;
  if (binding.kind === "function") return binding.init;
  if (FUNCTION_TYPES.has(binding.init.type)) return binding.init;
  return null;
}

/** The expressions a function node returns, ignoring nested functions. */
function returnedExpressions(fn) {
  if (fn.body.type !== "BlockStatement") return [fn.body];
  const out = [];
  const walk = (node) => {
    for (const child of children(node)) {
      if (FUNCTION_TYPES.has(child.type)) continue;
      if (child.type === "ReturnStatement") {
        if (child.argument) out.push(child.argument);
        continue;
      }
      walk(child);
    }
  };
  walk(fn.body);
  return out;
}

/** The operand a `??` statically selects, or null when the choice is not decidable. */
function nullishSelectedOperand(node) {
  const left = node.left;
  if (isNullLiteral(left) || isUndefinedIdentifier(left)) return node.right;
  if (isLiteral(left) || left.type === "ObjectExpression" || left.type === "ArrayExpression") return left;
  return null;
}

// --------------------------------------------------------------------------
// The matchers
// --------------------------------------------------------------------------

export const MATCHERS = {
  // R-01: a `function` declaration or function expression with a default-valued
  // parameter (arrow forms excluded -- they fail closed).
  functionWithDefaultParameter(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const type of ["FunctionDeclaration", "FunctionExpression"]) {
      for (const node of analysis.of(type)) {
        if (node.params.some((p) => p.type === "AssignmentPattern")) count += 1;
      }
    }
    return count;
  },

  // R-02: a call whose callee identifier resolves to a `let`/`var` binding, a
  // parameter, or a `const` aliasing another name, rather than to a function
  // declaration or a `const` bound directly to a function literal.
  //
  // The record states the shape twice -- once as a list of binding roles, once
  // as the complement of the two correct ones -- and the complement is the
  // operative half. The register's broken list includes "a function RETURNED
  // from a function and called" (`const f = mk(); f()`), a `const` bound to
  // neither a name nor a function literal, and the record's exclusion clause
  // says "a `const` bound DIRECTLY to a function literal" rather than "a
  // `const`". Reading the list as exhaustive would drop a shape the register
  // names as broken and the record pointedly does not exclude. So: any resolved
  // binding that is neither a function declaration nor a `const` initialized
  // directly with a function literal.
  callThroughNonConstFunctionBinding(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("CallExpression")) {
      if (node.callee.type !== "Identifier") continue;
      const binding = analysis.binding(node.callee);
      if (!binding) continue; // an unresolved name is a host global, not a binding
      if (binding.kind === "function") continue;
      if (binding.kind === "const" && binding.init && FUNCTION_TYPES.has(binding.init.type)) continue;
      count += 1;
    }
    return count;
  },

  // R-03: a `.forEach(...)` call with any callback, or a `.filter(...)` call
  // whose callback is an expression-bodied arrow.
  forEachOrExpressionBodiedFilterCall(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("CallExpression")) {
      if (isNamedProperty(node.callee, "forEach")) count += 1;
      else if (isNamedProperty(node.callee, "filter")) {
        const callback = node.arguments[0];
        if (callback && callback.type === "ArrowFunctionExpression" && callback.expression) count += 1;
      }
    }
    return count;
  },

  // R-04: a call to any `console` sink with two or more arguments where at
  // least one argument is not a literal or a substitution-free template.
  consoleCallWithNonLiteralArgument(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("CallExpression")) {
      if (!isConsoleCall(node)) continue;
      if (node.arguments.length < 2) continue;
      if (node.arguments.some((a) => !isLiteral(a) && !isSubstitutionFreeTemplate(a))) count += 1;
    }
    return count;
  },

  // R-05: an object literal property whose value is a function expression or
  // arrow function (shorthand methods excluded -- they fail closed).
  objectLiteralFunctionProperty(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("ObjectExpression")) {
      for (const property of node.properties) {
        if (property.type !== "Property" || property.kind !== "init" || property.method) continue;
        if (FUNCTION_TYPES.has(property.value.type)) count += 1;
      }
    }
    return count;
  },

  // R-49: a `switch` with at least one following sibling statement, inside a
  // function body or nested block. A top-level `switch` is harmless: the damage
  // is reparenting the remaining statements INTO module scope, where a
  // top-level switch's siblings already are.
  statementAfterSwitchInNestedBlock(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("SwitchStatement")) {
      const parent = analysis.parentOf.get(node);
      if (!parent || parent.type !== "BlockStatement") continue;
      const index = parent.body.indexOf(node);
      if (index >= 0 && index < parent.body.length - 1) count += 1;
    }
    return count;
  },

  // R-51: an optional call `f?.(...)`.
  optionalCallExpression(ast) {
    const analysis = analysisOf(ast);
    return analysis.of("CallExpression").filter((node) => node.optional === true).length;
  },

  // R-52: a C-style `for` that omits a clause in any arity the count-based
  // classifier misreads -- every omission except `for(;test;)`,
  // `for(init;test;)` and `for(;;)`.
  forWithMisclassifiedClauseArity(ast) {
    const analysis = analysisOf(ast);
    const EXEMPT = new Set(["110", "010", "000"]); // for(init;test;), for(;test;), for(;;)
    let count = 0;
    for (const node of analysis.of("ForStatement")) {
      const shape = `${node.init ? 1 : 0}${node.test ? 1 : 0}${node.update ? 1 : 0}`;
      if (shape === "111") continue; // omits nothing
      if (EXEMPT.has(shape)) continue;
      count += 1;
    }
    return count;
  },

  // R-06: a `var` or `let` declarator whose initializer is an object literal or
  // an array literal.
  nonConstObjectOrArrayLiteralInitializer(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("VariableDeclaration")) {
      if (node.kind === "const") continue;
      for (const declarator of node.declarations) {
        const init = declarator.init;
        if (init && (init.type === "ObjectExpression" || init.type === "ArrayExpression")) count += 1;
      }
    }
    return count;
  },

  // R-07: a `const` declarator whose initializer is not a literal. The register
  // bounds the damage the other way round -- "a `const` bound to a literal is
  // correct", "every non-literal initializer form is affected" -- so the
  // predicate is the negation of literal, not the enumeration that illustrates
  // it.
  constWithNonLiteralInitializer(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("VariableDeclaration")) {
      if (node.kind !== "const") continue;
      for (const declarator of node.declarations) {
        const init = declarator.init;
        if (init && !isLiteral(init) && !isSubstitutionFreeTemplate(init)) count += 1;
      }
    }
    return count;
  },

  // R-08: an `==`/`!=`/`===`/`!==` comparison with a `null`, `undefined`,
  // boolean-literal or numeric-literal operand, or any `??` expression.
  equalityOrNullishWithNullLikeOperand(ast) {
    const analysis = analysisOf(ast);
    const nullLike = (n) => isNullLiteral(n) || isUndefinedIdentifier(n) || isBooleanLiteral(n) || isNumericLiteral(n);
    let count = 0;
    for (const node of analysis.of("BinaryExpression")) {
      if (!NULL_LIKE_EQUALITY_OPERATORS.has(node.operator)) continue;
      if (nullLike(node.left) || nullLike(node.right)) count += 1;
    }
    for (const node of analysis.of("LogicalExpression")) {
      if (node.operator === "??") count += 1;
    }
    return count;
  },

  // R-09: a `continue` whose innermost enclosing loop is a C-style `for` with
  // an update clause, a `do...while`, or a `for...in`.
  continueInUnfaithfulLoop(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("ContinueStatement")) {
      let loop = null;
      for (let each = analysis.parentOf.get(node); each; each = analysis.parentOf.get(each)) {
        if (FUNCTION_TYPES.has(each.type)) break;
        if (LOOP_TYPES.has(each.type)) {
          loop = each;
          break;
        }
      }
      if (!loop) continue;
      if (loop.type === "DoWhileStatement" || loop.type === "ForInStatement") count += 1;
      else if (loop.type === "ForStatement" && loop.update) count += 1;
    }
    return count;
  },

  // R-10: a let/const declaration in a nested block whose declared name is also
  // bound in an enclosing scope.
  //
  // A function body is a nested block for this purpose. Every R-10 occurrence
  // in the extension corpus is function-body shadowing, so a matcher restricted
  // to module-scope blocks would report zero for matcher reasons rather than
  // corpus reasons.
  shadowingBlockDeclaration(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("VariableDeclaration")) {
      if (node.kind === "var") continue;
      const parent = analysis.parentOf.get(node);
      // "in a nested block": a statement list that is not the Program body.
      // A function body block and any other block both qualify.
      if (!parent || parent.type === "Program") continue;
      if (parent.type !== "BlockStatement" && parent.type !== "SwitchCase") continue;
      const scope = analysis.scopeOf.get(node);
      for (const declarator of node.declarations) {
        for (const name of declaredNames(declarator.id, new Set())) {
          // The declaration's own scope holds this name; an enclosing one
          // holding it too is the shadow.
          const outer = scope && scope.parent ? scope.parent.lookup(name) : null;
          if (outer) count += 1;
        }
      }
    }
    return count;
  },

  // R-11: an assignment using `&=`, `|=`, `^=`, `<<=`, `>>=` or `>>>=`.
  bitwiseCompoundAssignment(ast) {
    const analysis = analysisOf(ast);
    return analysis.of("AssignmentExpression").filter((node) => BITWISE_COMPOUND_OPERATORS.has(node.operator)).length;
  },

  // R-12: a computed element store to an array-literal binding on either silent
  // lane -- through an alias declarator (`const b = a; b[0] = ...`) in either
  // scope, or un-aliased at module scope. The un-aliased in-function form fails
  // closed E5506, so scope, not the declarator, is the discriminator there.
  silentArrayElementStore(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    const targets = [];
    for (const node of analysis.of("AssignmentExpression")) targets.push([node, node.left]);
    for (const node of analysis.of("UpdateExpression")) targets.push([node, node.argument]);
    for (const [node, target] of targets) {
      if (!target || target.type !== "MemberExpression" || !target.computed) continue;
      if (target.object.type !== "Identifier") continue;
      const binding = analysis.binding(target.object);
      if (!binding) continue;
      if (isArrayAliasBinding(binding, analysis)) {
        count += 1; // the alias lane is silent in both scopes
      } else if (binding.init && binding.init.type === "ArrayExpression" && analysis.functionOf.get(node) === null) {
        count += 1; // the un-aliased lane is silent only at module scope
      }
    }
    return count;
  },

  // R-13: computed member access whose key expression is not a literal.
  computedMemberNonLiteralKey(ast) {
    const analysis = analysisOf(ast);
    return analysis.of("MemberExpression").filter((node) => node.computed && node.property.type !== "Literal").length;
  },

  // R-14: a member or computed read applied directly to a call expression's
  // result.
  memberReadOnCallResult(ast) {
    const analysis = analysisOf(ast);
    return analysis.of("MemberExpression").filter((node) => node.object.type === "CallExpression").length;
  },

  // R-15: an indexed element read of a `.split(...)` call whose receiver,
  // separator argument and index are ALL literals -- the whole access folds
  // statically -- used as a `+` concat operand.
  staticSplitElementInConcatPosition(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("MemberExpression")) {
      if (!node.computed || !isLiteral(node.property)) continue;
      const call = node.object;
      if (!call || call.type !== "CallExpression" || !isNamedProperty(call.callee, "split")) continue;
      if (!isLiteral(call.callee.object)) continue;
      if (call.arguments.length === 0 || !call.arguments.every(isLiteral)) continue;
      if (isConcatOperand(node, analysis)) count += 1;
    }
    return count;
  },

  // R-16: a `.slice(...)`, `.charAt(...)`, `.toUpperCase(...)` or
  // `.repeat(...)` call used as a `+` concat operand. `.substring(...)` is
  // excluded -- it is correct, as is every non-concat position.
  stringMethodResultInConcatPosition(ast) {
    const analysis = analysisOf(ast);
    const METHODS = ["slice", "charAt", "toUpperCase", "repeat"];
    let count = 0;
    for (const node of analysis.of("CallExpression")) {
      if (!METHODS.some((name) => isNamedProperty(node.callee, name))) continue;
      if (isConcatOperand(node, analysis)) count += 1;
    }
    return count;
  },

  // R-18: a `&&` or `||` whose left operand is a string literal.
  stringLiteralLogicalOperand(ast) {
    const analysis = analysisOf(ast);
    return analysis
      .of("LogicalExpression")
      .filter((node) => (node.operator === "&&" || node.operator === "||") && isStringLiteral(node.left)).length;
  },

  // R-19: a call to `String(...)` or to a `.toString()` method, including the
  // computed `["toString"]()` spelling.
  stringConversionCall(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("CallExpression")) {
      const callee = node.callee;
      if (callee.type === "Identifier" && callee.name === "String") count += 1;
      else if (isNamedPropertyEitherSpelling(callee, "toString")) count += 1;
    }
    return count;
  },

  // R-20: a call to `JSON.stringify(...)`, including the computed
  // `JSON["stringify"](...)` spelling.
  jsonStringifyCall(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("CallExpression")) {
      const callee = node.callee;
      if (!isNamedPropertyEitherSpelling(callee, "stringify")) continue;
      if (callee.object.type === "Identifier" && callee.object.name === "JSON") count += 1;
    }
    return count;
  },

  // R-23: a `typeof` whose operand is not a bare literal.
  typeofNonLiteralOperand(ast) {
    const analysis = analysisOf(ast);
    return analysis.of("UnaryExpression").filter((node) => node.operator === "typeof" && !isLiteral(node.argument))
      .length;
  },

  // R-24: a call to `Object.freeze(...)` or `Object.isFrozen(...)` whose
  // argument is an already-bound identifier rather than an inline literal.
  objectFreezeOnBoundIdentifier(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("CallExpression")) {
      const callee = node.callee;
      if (!isNamedProperty(callee, "freeze") && !isNamedProperty(callee, "isFrozen")) continue;
      if (callee.object.type !== "Identifier" || callee.object.name !== "Object") continue;
      const argument = node.arguments[0];
      if (argument && argument.type === "Identifier") count += 1;
    }
    return count;
  },

  // R-25: an array literal containing a spread element.
  arraySpreadElement(ast) {
    const analysis = analysisOf(ast);
    return analysis.of("ArrayExpression").filter((node) => node.elements.some((e) => e && e.type === "SpreadElement"))
      .length;
  },

  // R-26: a unary `+` whose operand is not a numeric literal.
  unaryPlusOnNonNumericOperand(ast) {
    const analysis = analysisOf(ast);
    return analysis.of("UnaryExpression").filter((node) => node.operator === "+" && !isNumericLiteral(node.argument))
      .length;
  },

  // R-27: a sequence (comma) expression in value position. A `for` header's
  // comma clauses and a bare comma statement discard the value the defect
  // loses, so they are not value position.
  sequenceExpression(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("SequenceExpression")) {
      const parent = analysis.parentOf.get(node);
      if (!parent) continue;
      if (parent.type === "ExpressionStatement") continue;
      if (parent.type === "ForStatement" && (parent.init === node || parent.update === node)) continue;
      count += 1;
    }
    return count;
  },

  // R-28: a unary `-` applied to the numeric literal `0`.
  negativeZeroLiteral(ast) {
    const analysis = analysisOf(ast);
    return analysis
      .of("UnaryExpression")
      .filter((node) => node.operator === "-" && isNumericLiteral(node.argument) && node.argument.value === 0).length;
  },

  // R-47: a `for...of` whose iterable is an identifier declared by `let` with
  // an array-literal initializer.
  forOfOverLetArrayBinding(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("ForOfStatement")) {
      if (node.right.type !== "Identifier") continue;
      const binding = analysis.binding(node.right);
      if (binding && binding.kind === "let" && binding.init && binding.init.type === "ArrayExpression") count += 1;
    }
    return count;
  },

  // R-48: an assignment of an array to an object field that the object literal
  // initialized with a numeric literal.
  arrayStoreIntoScalarObjectField(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("AssignmentExpression")) {
      const target = node.left;
      if (!target || target.type !== "MemberExpression") continue;
      if (node.right.type !== "ArrayExpression") continue;
      if (target.object.type !== "Identifier") continue;
      const field = target.computed
        ? isStringLiteral(target.property)
          ? target.property.value
          : null
        : target.property.type === "Identifier"
          ? target.property.name
          : null;
      if (field === null) continue;
      const binding = analysis.binding(target.object);
      if (!binding || !binding.init || binding.init.type !== "ObjectExpression") continue;
      const initialized = binding.init.properties.some((property) => {
        if (property.type !== "Property") return false;
        const key = property.computed
          ? isStringLiteral(property.key)
            ? property.key.value
            : null
          : property.key.type === "Identifier"
            ? property.key.name
            : isLiteral(property.key)
              ? String(property.key.value)
              : null;
        return key === field && isNumericLiteral(property.value);
      });
      if (initialized) count += 1;
    }
    return count;
  },

  // R-53: a `for...of` with a `var` or `let` loop variable whose iterable is an
  // array literal.
  forOfVarOrLetOverArrayLiteral(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("ForOfStatement")) {
      if (node.left.type !== "VariableDeclaration") continue;
      if (node.left.kind !== "var" && node.left.kind !== "let") continue;
      if (node.right.type === "ArrayExpression") count += 1;
    }
    return count;
  },

  // R-29: an assignment whose target is an identifier declared `const` in an
  // enclosing scope.
  assignmentToConstBinding(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("AssignmentExpression")) {
      if (node.left.type !== "Identifier") continue;
      const binding = analysis.binding(node.left);
      if (binding && binding.kind === "const") count += 1;
    }
    return count;
  },

  // R-30: a single-argument direct `console.log` whose argument is any boolean
  // producer other than an inline `true`/`false` literal -- a comparison,
  // `!`/`!!`, `&&`/`||`, a conditional, a literal-selecting `??`, a call to a
  // user function, a parameter read, a `var`-bound identifier, or a `const`
  // object-field read. A plain `const` scalar binding is excluded: it now
  // renders correctly.
  consoleLogOfNonInlineBooleanProducer(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("CallExpression")) {
      if (!isConsoleCall(node, "log")) continue;
      if (node.arguments.length !== 1) continue;
      const argument = node.arguments[0];
      if (!argument || argument.type === "SpreadElement") continue;
      if (isBooleanLiteral(argument)) continue; // the inline literal renders correctly

      let matched = false;
      if (argument.type === "BinaryExpression" && COMPARISON_OPERATORS.has(argument.operator)) matched = true;
      else if (argument.type === "UnaryExpression" && argument.operator === "!") matched = true;
      else if (argument.type === "LogicalExpression" && (argument.operator === "&&" || argument.operator === "||")) {
        matched = true;
      } else if (argument.type === "LogicalExpression" && argument.operator === "??") {
        // Only the literal-selecting `??`. A `??` over an unprovable operand is
        // R-08 residual 6, not this entry.
        const selected = nullishSelectedOperand(argument);
        matched = Boolean(selected) && isBooleanLiteral(selected);
      } else if (argument.type === "ConditionalExpression") matched = true;
      else if (argument.type === "CallExpression" && argument.callee.type === "Identifier") {
        matched = Boolean(userFunctionOf(argument.callee, analysis));
      } else if (argument.type === "Identifier") {
        const binding = analysis.binding(argument);
        matched = Boolean(binding) && (binding.kind === "param" || binding.kind === "var");
      } else if (argument.type === "MemberExpression" && argument.object.type === "Identifier") {
        const binding = analysis.binding(argument.object);
        matched = Boolean(binding) && binding.kind === "const" && Boolean(binding.init) && binding.init.type === "ObjectExpression";
      }
      if (matched) count += 1;
    }
    return count;
  },

  // R-31: a `console.log` argument, or a `+` concat operand, that is an array
  // or object literal or an identifier bound to one.
  consoleLogOfArrayOrObjectValue(ast) {
    const analysis = analysisOf(ast);
    const isAggregate = (node) => {
      if (!node) return false;
      if (node.type === "ArrayExpression" || node.type === "ObjectExpression") return true;
      if (node.type !== "Identifier") return false;
      const binding = analysis.binding(node);
      return (
        Boolean(binding) &&
        Boolean(binding.init) &&
        (binding.init.type === "ArrayExpression" || binding.init.type === "ObjectExpression")
      );
    };
    let count = 0;
    for (const node of analysis.of("CallExpression")) {
      if (!isConsoleCall(node, "log")) continue;
      for (const argument of node.arguments) if (isAggregate(argument)) count += 1;
    }
    for (const node of analysis.of("BinaryExpression")) {
      if (node.operator !== "+") continue;
      if (isAggregate(node.left)) count += 1;
      if (isAggregate(node.right)) count += 1;
    }
    return count;
  },

  // R-32: a numeric literal whose magnitude is at least 1e21, or is nonzero and
  // below 1e-6.
  numericLiteralOutsideExponentThresholds(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("Literal")) {
      if (typeof node.value !== "number") continue;
      const magnitude = Math.abs(node.value);
      if (magnitude >= 1e21) count += 1;
      else if (magnitude !== 0 && magnitude < 1e-6) count += 1;
    }
    return count;
  },

  // R-33: a call to `console.warn(...)`.
  consoleWarnCall(ast) {
    const analysis = analysisOf(ast);
    return analysis.of("CallExpression").filter((node) => isConsoleCall(node, "warn")).length;
  },

  // R-34: a call to a user function whose `return` yields a comparison, `!`, or
  // logical expression, used as a `+` concat operand or as a non-first
  // `console.log` argument.
  booleanReturningFunctionCallInStringPosition(ast) {
    const analysis = analysisOf(ast);
    const booleanReturning = (fn) =>
      returnedExpressions(fn).some(
        (expression) =>
          (expression.type === "BinaryExpression" && COMPARISON_OPERATORS.has(expression.operator)) ||
          (expression.type === "UnaryExpression" && expression.operator === "!") ||
          expression.type === "LogicalExpression",
      );

    // Every call that sits in a string position, by position rather than by node
    // type, so a call can be found once and only once.
    const inStringPosition = new Set();
    for (const node of analysis.of("BinaryExpression")) {
      if (node.operator !== "+") continue;
      for (const side of [node.left, node.right]) if (side.type === "CallExpression") inStringPosition.add(side);
    }
    for (const node of analysis.of("CallExpression")) {
      if (!isConsoleCall(node, "log")) continue;
      for (let index = 1; index < node.arguments.length; index += 1) {
        const argument = node.arguments[index];
        if (argument.type === "CallExpression") inStringPosition.add(argument);
      }
    }

    let count = 0;
    for (const call of inStringPosition) {
      if (call.callee.type !== "Identifier") continue;
      const fn = userFunctionOf(call.callee, analysis);
      if (fn && booleanReturning(fn)) count += 1;
    }
    return count;
  },

  // R-56: an object-literal property whose key is a STRING literal whose own
  // text begins and ends with a double quote and whose inner text reads as a
  // number -- `{'"5"': 1}`, `{"\"5\"": 1}`. That text is byte-identical to what
  // `lower_property_name` writes for the NUMERIC key `{5: 1}`, which is the
  // collision the entry is about.
  //
  // Upper bound, per the record: the exact condition is that the inner text lies
  // in Rust's `Display for f64` image, which is not reproducible from an acorn
  // AST -- `Number.isFinite(Number(inner))` is a strictly wider test, so
  // `{'"1e21"': 1}` and `{'"05"': 1}` are counted here and are NOT this defect.
  // The disclosure is in `count.mjs`'s UPPER_BOUNDS, beside the number.
  //
  // Computed keys are excluded: `{["\"5\""]: 1}` does not reach the key slot as
  // a `PropertyName::String`, and only that slot carries the marker.
  objectLiteralQuotedNumericStringKey(ast) {
    const analysis = analysisOf(ast);
    let count = 0;
    for (const node of analysis.of("ObjectExpression")) {
      for (const property of node.properties) {
        if (property.type !== "Property" || property.computed) continue;
        const key = property.key;
        if (!isLiteral(key) || typeof key.value !== "string") continue;
        const text = key.value;
        if (text.length < 2 || !text.startsWith('"') || !text.endsWith('"')) continue;
        const inner = text.slice(1, -1);
        if (inner.trim() === "" || !Number.isFinite(Number(inner))) continue;
        count += 1;
      }
    }
    return count;
  },
};

/** Every matcher's count for one source string. */
export function countAll(source) {
  const ast = parse(source);
  const out = {};
  for (const [name, matcher] of Object.entries(MATCHERS)) {
    out[name] = matcher(ast);
  }
  return out;
}

// --------------------------------------------------------------------------
// Disclosure instruments
//
// Neither of these is a matcher, and neither is in MATCHERS -- the
// catalogue/matcher agreement gate in count.mjs must stay exact in both
// directions. They exist so the caveats a reader needs in order to interpret a
// count are DERIVED and published beside it, rather than living only in a
// report under `.superpowers/`.
// --------------------------------------------------------------------------

/**
 * Where a record states its shape two ways and the readings disagree on the
 * frozen corpus, the count under the reading that was NOT published, so a
 * consumer can see that the published figure rests on an interpretation and by
 * how much.
 */
export const ALTERNATE_READINGS = {
  "R-02": {
    matcher: "callThroughNonConstFunctionBinding",
    publishedReading:
      "the complement clause -- any resolved binding that is neither a function declaration nor " +
      "a `const` bound DIRECTLY to a function literal",
    alternateReading:
      "the role list read as exhaustive -- a `let`/`var` binding, a parameter, or a `const` " +
      "whose initializer is an identifier",
    whyPublishedReadingWasChosen:
      "the register's broken list (register:616-627) includes 'a function RETURNED from a " +
      "function and called' (`const f = mk(); f()`), a `const` bound to neither a name nor a " +
      "function literal, which the role list does not name; and the record's exclusion clause " +
      "says a `const` bound DIRECTLY to a function literal, an adverb that only does work if " +
      "the rest of the `const` family is counted.",
    /** The role list, read as exhaustive. */
    count(ast) {
      const analysis = analysisOf(ast);
      let count = 0;
      for (const node of analysis.of("CallExpression")) {
        if (node.callee.type !== "Identifier") continue;
        const binding = analysis.binding(node.callee);
        if (!binding) continue;
        if (binding.kind === "let" || binding.kind === "var" || binding.kind === "param") count += 1;
        else if (binding.kind === "const" && binding.init && binding.init.type === "Identifier") count += 1;
      }
      return count;
    },
  },

  "R-07": {
    matcher: "constWithNonLiteralInitializer",
    publishedReading:
      "the main clause -- a `const` declarator whose initializer is not a literal",
    alternateReading:
      "the appositive dash-list read as exhaustive -- an identifier, binary, unary, " +
      "conditional, member or call expression only (`parenthesized` is unmatchable: acorn " +
      "emits no ParenthesizedExpression without `preserveParens`)",
    whyPublishedReadingWasChosen:
      "the main clause governs and the appositive illustrates; the register bounds the damage " +
      "the other way round -- 'a `const` bound to a literal is correct' (register:1048-1049) -- " +
      "and its Repro F is titled a shape SURVEY, not a closed list. But the disputed forms " +
      "(`new`, object literal, array literal) are named in neither the record's list nor the " +
      "register's survey, so the choice is an interpretation and is disclosed as one.",
    /** The dash-list, read as exhaustive. */
    count(ast) {
      const analysis = analysisOf(ast);
      const ENUMERATED = new Set([
        "Identifier",
        "BinaryExpression",
        "UnaryExpression",
        "ConditionalExpression",
        "MemberExpression",
        "CallExpression",
      ]);
      let count = 0;
      for (const node of analysis.of("VariableDeclaration")) {
        if (node.kind !== "const") continue;
        for (const declarator of node.declarations) {
          if (declarator.init && ENUMERATED.has(declarator.init.type)) count += 1;
        }
      }
      return count;
    },
  },
};

/**
 * Sub-counts that say what an undisclosed upper bound is actually made of.
 *
 * R-13's record is "computed member access whose key expression is not a
 * literal", with no disclosure clause -- so ordinary array indexing `a[i]`
 * counts, and the register's repro is an OBJECT read with a variable key. The
 * breakdown is computed with the module's own scope resolution, not a flat
 * per-file name map: a parameter and a module binding can share a name, and a
 * flat map silently reclassifies the receiver.
 */
export const BREAKDOWNS = {
  "R-13": {
    matcher: "computedMemberNonLiteralKey",
    of: "the same sites `computedMemberNonLiteralKey` counts, classified by receiver and by position",
    count(ast) {
      const analysis = analysisOf(ast);
      const stores = new Set();
      for (const node of analysis.of("AssignmentExpression")) stores.add(node.left);
      for (const node of analysis.of("UpdateExpression")) stores.add(node.argument);

      const out = { total: 0, objectLiteralReceiver: 0, arrayLikeReceiver: 0, storeTarget: 0 };
      for (const node of analysis.of("MemberExpression")) {
        if (!node.computed || node.property.type === "Literal") continue;
        out.total += 1;
        if (stores.has(node)) out.storeTarget += 1;
        if (node.object.type !== "Identifier") continue;
        const binding = analysis.binding(node.object);
        const init = binding && binding.init;
        if (!init) continue;
        if (init.type === "ObjectExpression") out.objectLiteralReceiver += 1;
        else if (
          init.type === "ArrayExpression" ||
          (init.type === "NewExpression" && init.callee.type === "Identifier" && init.callee.name === "Array")
        ) {
          out.arrayLikeReceiver += 1;
        }
      }
      return out;
    },
  },
};
