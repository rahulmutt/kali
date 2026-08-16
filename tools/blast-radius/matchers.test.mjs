// tools/blast-radius/matchers.test.mjs
//
// One positive-and-negative test per matcher: a program containing the shape,
// and a program containing a near-miss that must NOT count. Plus the two
// whole-module gates the brief specifies -- every matcher returns 0 on a
// program with none of its shapes, and a syntax error is thrown rather than
// silently counted as zero.

import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import { ALTERNATE_READINGS, BREAKDOWNS, countAll, MATCHERS, parse } from "./matchers.mjs";

const CATALOGUE = JSON.parse(fs.readFileSync(new URL("./predicates.json", import.meta.url), "utf8"));

/** Assert one matcher's count on a source string. */
function count(name, source) {
  return countAll(source)[name];
}

// ---------------------------------------------------------------- worked examples

test("computedMemberNonLiteralKey counts variable keys and not literal ones", () => {
  const src = `
    var o = {a: 1};
    var k = "a";
    console.log(o[k]);      // counts
    console.log(o["a"]);    // literal key, does not count
    console.log(o.a);       // not computed, does not count
    console.log(o[k + ""]); // non-literal expression, counts
  `;
  assert.equal(countAll(src).computedMemberNonLiteralKey, 2);
});

test("shadowingBlockDeclaration needs an enclosing binding of the same name", () => {
  const shadows = `var x = 1; { let x = 2; } console.log(x);`;
  const distinct = `var x = 1; { let y = 2; } console.log(x);`;
  assert.equal(countAll(shadows).shadowingBlockDeclaration, 1);
  assert.equal(countAll(distinct).shadowingBlockDeclaration, 0);
});

test("memberReadOnCallResult counts reads on a call, not calls on a member", () => {
  const src = `
    function f() { return [1, 2]; }
    console.log(f()[0]);   // counts
    console.log(f().a);    // counts
    console.log([1,2][0]); // literal receiver, does not count
    o.m();                 // a call ON a member, not a read on a call
  `;
  assert.equal(countAll(src).memberReadOnCallResult, 2);
});

// ---------------------------------------------------------------- R-10 ruling

test("shadowingBlockDeclaration counts function-body shadowing, not just nested blocks", () => {
  // Ruling 4: every R-10 occurrence in the extension corpus is function-body
  // shadowing. A matcher that only considered module-scope nested blocks would
  // collapse this entry to zero for matcher reasons.
  const body = `var total = 0; function f() { let total = 1; return total; }`;
  assert.equal(countAll(body).shadowingBlockDeclaration, 1);
  const param = `function f(n) { { const n = 2; return n; } }`;
  assert.equal(countAll(param).shadowingBlockDeclaration, 1);
  const distinct = `var total = 0; function f() { let sum = 1; return sum; }`;
  assert.equal(countAll(distinct).shadowingBlockDeclaration, 0);
});

// ---------------------------------------------------------------- per matcher

test("functionWithDefaultParameter counts declarations and expressions, not arrows", () => {
  assert.equal(count("functionWithDefaultParameter", `function f(a, b = 1) { return a + b; }`), 1);
  assert.equal(count("functionWithDefaultParameter", `var f = function (b = 1) { return b; };`), 1);
  assert.equal(count("functionWithDefaultParameter", `var f = (b = 1) => b;`), 0);
  assert.equal(count("functionWithDefaultParameter", `function f(a, b) { return a + b; }`), 0);
});

test("callThroughNonConstFunctionBinding counts every callee but a declaration or a const function literal", () => {
  const src = `
    function g() { return 1; }
    const h = function () { return 2; };
    let i = function () { return 3; };
    var j = function () { return 4; };
    const k = g;
    const m = g();
    function outer(cb) { return cb(); }
    g(); h(); i(); j(); k(); m();
  `;
  // i (let), j (var), k (const alias), m (const over a returned function),
  // cb (parameter) = 5. The record excludes only a function declaration and a
  // const bound DIRECTLY to a function literal.
  assert.equal(count("callThroughNonConstFunctionBinding", src), 5);
  assert.equal(
    count("callThroughNonConstFunctionBinding", `function g() { return 1; } const h = function () { return 2; }; g(); h();`),
    0,
  );
});

test("forEachOrExpressionBodiedFilterCall counts forEach and expression-bodied filter", () => {
  const src = `
    var a = [1, 2];
    a.forEach(function (x) { console.log(x); });
    a.filter((x) => x > 1);
  `;
  assert.equal(count("forEachOrExpressionBodiedFilterCall", src), 2);
  const near = `var a = [1, 2]; a.filter(function (x) { return x > 1; }); a.map((x) => x + 1);`;
  assert.equal(count("forEachOrExpressionBodiedFilterCall", near), 0);
});

test("consoleCallWithNonLiteralArgument needs two-plus args and a non-literal one", () => {
  assert.equal(count("consoleCallWithNonLiteralArgument", `var n = 1; console.log("n", n);`), 1);
  assert.equal(count("consoleCallWithNonLiteralArgument", `console.log("a", "b", \`c\`);`), 0);
  assert.equal(count("consoleCallWithNonLiteralArgument", `var n = 1; console.log(n);`), 0);
});

test("objectLiteralFunctionProperty counts value-position functions, not shorthand methods", () => {
  assert.equal(count("objectLiteralFunctionProperty", `var o = {f: function () { return 1; }, g: () => 2};`), 2);
  assert.equal(count("objectLiteralFunctionProperty", `var o = {f() { return 1; }, n: 3};`), 0);
});

test("statementAfterSwitchInNestedBlock excludes a top-level switch", () => {
  const nested = `function f(x) { switch (x) { case 1: break; } return 2; }`;
  assert.equal(count("statementAfterSwitchInNestedBlock", nested), 1);
  const top = `var x = 1; switch (x) { case 1: break; } console.log(x);`;
  assert.equal(count("statementAfterSwitchInNestedBlock", top), 0);
  const last = `function f(x) { switch (x) { case 1: break; } }`;
  assert.equal(count("statementAfterSwitchInNestedBlock", last), 0);
});

test("optionalCallExpression counts f?.() and not a?.b", () => {
  assert.equal(count("optionalCallExpression", `var f = null; f?.(1);`), 1);
  assert.equal(count("optionalCallExpression", `var a = {b: 1}; a?.b; a.b;`), 0);
});

test("forWithMisclassifiedClauseArity counts only the misread omissions", () => {
  assert.equal(count("forWithMisclassifiedClauseArity", `for (var i = 0; ; i++) { break; }`), 1);
  assert.equal(count("forWithMisclassifiedClauseArity", `for (; i < 3; i++) { break; }`), 1);
  const exempt = `for (var i = 0; i < 3; ) { i++; } for (; i < 5; ) { i++; } for (;;) { break; } for (var j = 0; j < 3; j++) { }`;
  assert.equal(count("forWithMisclassifiedClauseArity", exempt), 0);
});

test("nonConstObjectOrArrayLiteralInitializer counts var/let, not const", () => {
  assert.equal(count("nonConstObjectOrArrayLiteralInitializer", `var a = [1]; let o = {b: 2};`), 2);
  assert.equal(count("nonConstObjectOrArrayLiteralInitializer", `const a = [1]; var n = 3;`), 0);
});

test("constWithNonLiteralInitializer counts non-literal initializers only", () => {
  assert.equal(count("constWithNonLiteralInitializer", `var n = 1; const a = n; const b = n + 1;`), 2);
  assert.equal(count("constWithNonLiteralInitializer", `const a = 1; const b = "s"; let c = a + 1;`), 0);
});

test("equalityOrNullishWithNullLikeOperand counts null-like comparisons and every ??", () => {
  const src = `var a = 1, b = null; if (a === null) { } if (a != undefined) { } if (a === 1) { } var c = b ?? 2;`;
  assert.equal(count("equalityOrNullishWithNullLikeOperand", src), 4);
  assert.equal(count("equalityOrNullishWithNullLikeOperand", `var a = "x", b = "y"; if (a === b) { } if (a < 1) { }`), 0);
});

test("continueInUnfaithfulLoop counts by the innermost loop's faithfulness", () => {
  const unfaithful = `for (var i = 0; i < 3; i++) { continue; } do { continue; } while (false); for (var k in {}) { continue; }`;
  assert.equal(count("continueInUnfaithfulLoop", unfaithful), 3);
  const faithful = `var i = 0; while (i < 3) { i++; continue; } for (var x of [1]) { continue; } for (var j = 0; j < 3; ) { j++; continue; }`;
  assert.equal(count("continueInUnfaithfulLoop", faithful), 0);
});

test("bitwiseCompoundAssignment counts bitwise compounds only", () => {
  assert.equal(count("bitwiseCompoundAssignment", `var a = 1; a &= 2; a |= 3; a ^= 4; a <<= 1; a >>= 1; a >>>= 1;`), 6);
  assert.equal(count("bitwiseCompoundAssignment", `var a = 1; a += 2; a = a & 3;`), 0);
});

test("silentArrayElementStore counts the alias lane in both scopes and the module-scope direct lane", () => {
  const alias = `function f() { const a = [1, 2]; const b = a; b[0] = 7; return b[0]; }`;
  assert.equal(count("silentArrayElementStore", alias), 1);
  const moduleDirect = `const a = [1, 2]; a[0] = 7;`;
  assert.equal(count("silentArrayElementStore", moduleDirect), 1);
  // The un-aliased in-function form fails closed E5506 -- loud, so not counted.
  const inFunctionDirect = `function f() { const a = [1, 2]; a[0] = 7; return a[0]; }`;
  assert.equal(count("silentArrayElementStore", inFunctionDirect), 0);
  const notArray = `const o = {a: 1}; const b = o; b["a"] = 7;`;
  assert.equal(count("silentArrayElementStore", notArray), 0);
});

test("staticSplitElementInConcatPosition needs receiver, separator and index all literal", () => {
  assert.equal(count("staticSplitElementInConcatPosition", `console.log("x=" + "abc".split("")[0]);`), 1);
  const near = `var s = "abc", sep = "", i = 0; console.log("x=" + s.split("")[0]); console.log("y=" + "abc".split(sep)[0]); console.log("z=" + "abc".split("")[i]); var q = "abc".split("")[0];`;
  assert.equal(count("staticSplitElementInConcatPosition", near), 0);
});

test("stringMethodResultInConcatPosition counts the four methods in concat position only", () => {
  const src = `var s = "abc"; console.log("a" + s.slice(1) + s.charAt(0) + s.toUpperCase() + s.repeat(2));`;
  assert.equal(count("stringMethodResultInConcatPosition", src), 4);
  const near = `var s = "abc"; console.log("a" + s.substring(1)); var q = s.slice(1);`;
  assert.equal(count("stringMethodResultInConcatPosition", near), 0);
});

test("stringLiteralLogicalOperand needs the left operand to be a string literal", () => {
  assert.equal(count("stringLiteralLogicalOperand", `var a = "" || 1; var b = "x" && 2;`), 2);
  assert.equal(count("stringLiteralLogicalOperand", `var s = "x"; var a = s || 1; var b = 1 || "x";`), 0);
});

test("stringConversionCall counts String() and toString(), including the computed spelling", () => {
  assert.equal(count("stringConversionCall", `var n = 1; String(n); n.toString(); n["toString"]();`), 3);
  assert.equal(count("stringConversionCall", `var n = 1; Number(n); n.toFixed(2);`), 0);
});

test("jsonStringifyCall counts both spellings and not JSON.parse", () => {
  assert.equal(count("jsonStringifyCall", `var o = {a: 1}; JSON.stringify(o); JSON["stringify"](o);`), 2);
  assert.equal(count("jsonStringifyCall", `JSON.parse("{}");`), 0);
});

test("typeofNonLiteralOperand excludes a bare literal operand", () => {
  assert.equal(count("typeofNonLiteralOperand", `var x = 1; typeof x; typeof x.y;`), 2);
  assert.equal(count("typeofNonLiteralOperand", `typeof 1; typeof "s";`), 0);
});

test("objectFreezeOnBoundIdentifier needs a bound identifier, not an inline literal", () => {
  assert.equal(count("objectFreezeOnBoundIdentifier", `var o = {a: 1}; Object.freeze(o); Object.isFrozen(o);`), 2);
  assert.equal(count("objectFreezeOnBoundIdentifier", `Object.freeze({a: 1}); Object.keys({a: 1});`), 0);
});

test("arraySpreadElement counts the array literal that holds a spread", () => {
  assert.equal(count("arraySpreadElement", `var a = [1]; var b = [0, ...a];`), 1);
  assert.equal(count("arraySpreadElement", `var a = [1]; var b = [0, a]; function f(...rest) { return rest; }`), 0);
});

test("unaryPlusOnNonNumericOperand excludes a numeric literal operand", () => {
  assert.equal(count("unaryPlusOnNonNumericOperand", `var s = "42"; var a = +s; var b = +"7"; var c = +s.length;`), 3);
  assert.equal(count("unaryPlusOnNonNumericOperand", `var a = +1; var b = 1 + 2;`), 0);
});

test("sequenceExpression counts value position, not a for header or a bare statement", () => {
  assert.equal(count("sequenceExpression", `var a = (1, 2); console.log((3, 4));`), 2);
  assert.equal(count("sequenceExpression", `var i, j; for (i = 0, j = 1; i < 3; i++, j--) { } i++, j--;`), 0);
});

test("negativeZeroLiteral counts -0 and not -1 or 0", () => {
  assert.equal(count("negativeZeroLiteral", `var a = -0; var b = 1 / -0;`), 2);
  assert.equal(count("negativeZeroLiteral", `var a = 0; var b = -1; var z = 0; var c = -z;`), 0);
});

test("forOfOverLetArrayBinding needs a let binding with an array-literal initializer", () => {
  assert.equal(count("forOfOverLetArrayBinding", `let a = [1, 2]; for (const x of a) { console.log(x); }`), 1);
  const near = `const a = [1, 2]; for (const x of a) { } let b = 1; for (const y of [1]) { }`;
  assert.equal(count("forOfOverLetArrayBinding", near), 0);
});

test("arrayStoreIntoScalarObjectField needs the field to have been a numeric literal", () => {
  assert.equal(count("arrayStoreIntoScalarObjectField", `var o = {n: 0}; o.n = [1, 2];`), 1);
  const near = `var o = {n: [0], s: "x"}; o.n = [1, 2]; o.s = [3]; var p = {n: 0}; p.n = 4;`;
  assert.equal(count("arrayStoreIntoScalarObjectField", near), 0);
});

test("forOfVarOrLetOverArrayLiteral needs a var/let loop variable over a literal", () => {
  assert.equal(count("forOfVarOrLetOverArrayLiteral", `for (var x of [1, 2]) { } for (let y of [3]) { }`), 2);
  const near = `var a = [1]; for (const x of [1, 2]) { } for (var y of a) { }`;
  assert.equal(count("forOfVarOrLetOverArrayLiteral", near), 0);
});

test("assignmentToConstBinding counts only assignments whose target is const", () => {
  assert.equal(count("assignmentToConstBinding", `const c = 1; c = 2; c += 3;`), 2);
  assert.equal(count("assignmentToConstBinding", `let l = 1; l = 2; const o = {a: 1}; o.a = 2;`), 0);
});

test("consoleLogOfNonInlineBooleanProducer excludes an inline boolean literal", () => {
  const src = `
    var flag = true;
    const cfg = {on: true};
    function isBig(n) { return n > 1; }
    function f(p) { console.log(p); }
    console.log(1 === 1);
    console.log(!flag);
    console.log(flag && true);
    console.log(flag ? true : false);
    console.log(null ?? true);
    console.log(isBig(2));
    console.log(flag);
    console.log(cfg.on);
  `;
  // comparison, !, &&, conditional, literal-selecting ??, user call, parameter,
  // var binding, const object field = 9
  assert.equal(count("consoleLogOfNonInlineBooleanProducer", src), 9);
  const near = `
    const scalar = true;
    var x = 1;
    console.log(true);
    console.log(false);
    console.log(scalar);
    console.log(x ?? true);
    console.log(1 === 1, 2);
  `;
  assert.equal(count("consoleLogOfNonInlineBooleanProducer", near), 0);
});

test("consoleLogOfArrayOrObjectValue counts log arguments and concat operands", () => {
  const src = `var a = [1, 2]; var o = {b: 1}; console.log(a); console.log({c: 2}); var s = "x" + o;`;
  assert.equal(count("consoleLogOfArrayOrObjectValue", src), 3);
  const near = `var n = 1; var s = "x"; console.log(n); console.log("lit"); var t = "x" + n;`;
  assert.equal(count("consoleLogOfArrayOrObjectValue", near), 0);
});

test("numericLiteralOutsideExponentThresholds counts only past the thresholds", () => {
  assert.equal(count("numericLiteralOutsideExponentThresholds", `var a = 1e21; var b = 1e-7;`), 2);
  assert.equal(count("numericLiteralOutsideExponentThresholds", `var a = 1e20; var b = 1e-6; var c = 0; var d = 1.5;`), 0);
});

test("consoleWarnCall counts console.warn and no other sink", () => {
  assert.equal(count("consoleWarnCall", `console.warn("a"); console.warn("b", 1);`), 2);
  assert.equal(count("consoleWarnCall", `console.log("a"); console.error("b");`), 0);
});

test("booleanReturningFunctionCallInStringPosition needs a boolean-returning callee in string position", () => {
  const src = `
    function isBig(n) { return n > 1; }
    var s = "big=" + isBig(2);
    console.log("big=", isBig(3));
  `;
  assert.equal(count("booleanReturningFunctionCallInStringPosition", src), 2);
  const near = `
    function isBig(n) { return n > 1; }
    function twice(n) { return n * 2; }
    var s = "n=" + twice(2);
    console.log(isBig(3));
    var b = isBig(4);
  `;
  assert.equal(count("booleanReturningFunctionCallInStringPosition", near), 0);
});

// ---------------------------------------------------------------- module gates

test("a known-answer file counts exactly what it declares", () => {
  // Every matcher must return 0 on a program containing none of its shapes.
  const empty = `console.log("hello");`;
  const counts = countAll(empty);
  for (const name of Object.keys(MATCHERS)) {
    assert.equal(counts[name], 0, `${name} found a shape in a program with none`);
  }
});

test("a syntax error is thrown, never silently counted as zero", () => {
  assert.throws(() => countAll(`function ( {`), /parse/i);
});

test("the module exports exactly the catalogue's countable matchers, by name", () => {
  // By NAME, in both directions: a count alone would let a rename pass, and a
  // renamed matcher counts for no entry while its entry silently reads zero.
  // count.mjs enforces the same agreement at measurement time; this is the
  // same gate at test time, so a rename fails here first.
  const countable = CATALOGUE.entries.filter((entry) => entry.kind === "countable");
  const catalogueNames = countable.map((entry) => entry.matcher).sort();
  assert.deepEqual(Object.keys(MATCHERS).sort(), catalogueNames);
  assert.equal(catalogueNames.length, 38);
});

test("objectLiteralQuotedNumericStringKey counts only the colliding key spelling", () => {
  // Positive: a string key whose own text is a quoted number -- the text HIR
  // also writes for the numeric key `{5: 1}`. Negatives: an ordinary string key,
  // a numeric key, a quoted NON-number, and a computed key (which never reaches
  // the marker-carrying slot).
  const src = `
    var a = {'"5"': 1};      // counts
    var b = {"\\"1.5\\"": 1}; // counts
    var c = {"5": 1};        // ordinary string key, does not count
    var d = {5: 1};          // numeric key, does not count
    var e = {'"d"': 1};      // quoted non-number, does not count
    var f = {["\\"5\\""]: 1}; // computed, does not count
  `;
  assert.equal(count("objectLiteralQuotedNumericStringKey", src), 2);
});

test("the disclosure instruments name real entries and stay out of MATCHERS", () => {
  // ALTERNATE_READINGS and BREAKDOWNS must never leak into MATCHERS: count.mjs
  // requires the catalogue and MATCHERS to agree exactly in both directions,
  // and a stray export would abort the counter.
  const ids = new Set(CATALOGUE.entries.map((entry) => entry.id));
  for (const [id, reading] of Object.entries(ALTERNATE_READINGS)) {
    assert.ok(ids.has(id), `ALTERNATE_READINGS names ${id}, which is not a catalogue entry`);
    assert.ok(reading.matcher in MATCHERS, `${id}'s alternate reading names an unknown matcher`);
    assert.equal(typeof reading.count, "function");
    assert.ok(!(id in MATCHERS));
  }
  for (const [id, breakdown] of Object.entries(BREAKDOWNS)) {
    assert.ok(ids.has(id), `BREAKDOWNS names ${id}, which is not a catalogue entry`);
    assert.ok(breakdown.matcher in MATCHERS, `${id}'s breakdown names an unknown matcher`);
    assert.ok(!(id in MATCHERS));
  }
});

test("R-07's two readings differ exactly where the record's two statements differ", () => {
  // The published reading is the main clause; the alternate is the appositive
  // dash-list read as exhaustive. `new`, object and array literals are the
  // forms named in neither, and are the whole of the difference.
  const src = `
    const a = n;            // identifier -- both readings
    const b = n + 1;        // binary -- both readings
    const c = new Array(2); // NewExpression -- published only
    const d = {k: 1};       // object literal -- published only
    const e = [1, 2];       // array literal -- published only
    const f = 1;            // literal -- neither reading
  `;
  // Published reading: every non-literal initializer.
  assert.equal(countAll(src).constWithNonLiteralInitializer, 5);
  // Alternate reading: the enumerated forms only.
  assert.equal(ALTERNATE_READINGS["R-07"].count(parse(src)), 2);
});

test("R-02's alternate reading is the role list read as exhaustive", () => {
  const src = `function g() { return 1; } const m = g(); m();`;
  assert.equal(countAll(src).callThroughNonConstFunctionBinding, 1); // published: the complement
  assert.equal(ALTERNATE_READINGS["R-02"].count(parse(src)), 0); // alternate: the role list
});

test("R-13's breakdown classifies receivers with real scope resolution", () => {
  // A flat per-file name map lets a same-named parameter overwrite the
  // declarator and silently reclassify the receiver -- which is exactly how the
  // first hand-run of this breakdown undercounted it.
  const src = `
    const cfg = {a: 1};
    var k = "a";
    function f(cfg) { return cfg[k]; }
    console.log(cfg[k]);
  `;
  const breakdown = BREAKDOWNS["R-13"].count(parse(src));
  assert.equal(breakdown.total, 2);
  // Only the module-scope read has an object-literal binding; the parameter
  // read must not inherit it.
  assert.equal(breakdown.objectLiteralReceiver, 1);
  assert.equal(breakdown.storeTarget, 0);
});

test("R-13's breakdown counts a store target as a store, not a read", () => {
  const src = `var a = [1]; var i = 0; a[i] = 2; console.log(a[i]);`;
  const breakdown = BREAKDOWNS["R-13"].count(parse(src));
  assert.equal(breakdown.total, 2);
  assert.equal(breakdown.storeTarget, 1);
  assert.equal(breakdown.arrayLikeReceiver, 2);
});
