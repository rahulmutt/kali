//! Shared integer-vs-float representation model for the `number` type.
//!
//! Every `number`-typed program point is `I64` unless the representation
//! inference in `kali_types` unifies it with a float seed. The resulting
//! `ReprTable` is threaded to codegen, which uses it to pick wasm signatures,
//! locals, and per-operand arithmetic instructions.

use std::collections::{HashMap, HashSet};

/// Interned identity of a fixed object layout: an ordered list of
/// `(field name, field repr)`. Field `i` lives at byte offset `i * 8`
/// (every field is one 8-byte slot; objects have no header word).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct ShapeId(pub u32);

/// Machine representation chosen for a `number` value.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Repr {
    /// Two's-complement 64-bit integer (the default for every `number`).
    #[default]
    I64,
    /// IEEE-754 double.
    F64,
    /// Pointer (i64) to a fixed-shape heap object in linear memory.
    Object(ShapeId),
    /// Tagged linear-memory string handle (`STRING_HANDLE_TAG | offset << 32 | len`).
    String,
}

/// Representation decisions for a whole program, keyed by function + binding.
///
/// All lookups default to [`Repr::I64`]; only float and string decisions are
/// stored, so an empty table means "no floats and no strings anywhere" and
/// codegen can keep its i64 fast path.
#[derive(Clone, Debug, Default)]
pub struct ReprTable {
    scalars: HashMap<(String, String), Repr>,
    array_elements: HashMap<(String, String), Repr>,
    returns: HashMap<String, Repr>,
    params: HashMap<(String, usize), Repr>,
    /// `(func, binding)` pairs the inference treated as arrays (subscripted,
    /// `.length`/`.fill`, `new Array`, array literal, or a pass-through array
    /// param). Lets codegen register array PARAMETERS as array bindings — an i64
    /// array's element repr is unset (== default I64), so this is the only way to
    /// distinguish an i64 array param from a scalar param.
    array_bindings: HashSet<(String, String)>,
    /// `(func, param)` parameters that interprocedural call-site flow shows may
    /// receive a NON-SCALAR argument. This taint covers EXACTLY the DIRECT array
    /// shapes visible at the call site: a bare-identifier array binding, or a
    /// syntactic array literal `[..]` / `new Array(..)` / `Array(..)` passed
    /// directly. Such a param holds a heap handle, not a number/string, so a
    /// compound (`+=`) or update (`++`) assignment on it has no lowering —
    /// codegen's numeric/string compound arm would silently do integer
    /// arithmetic on the raw handle (a miscompile). (Object arguments need no
    /// entry here: they propagate `Repr::Object` onto the param scalar, which
    /// the compound/update allowlist rejects directly.)
    ///
    /// This taint does NOT see INDIRECT array delivery — an array via a call
    /// return (`f(g())`), a pass-through param chain (`f(a)` then `h(a)`), or a
    /// member expression (`f(o.a)`). Those are closed instead by the
    /// POSITIVE-PROOF gate ([`params_lacking_scalar_inflow`]): a param is
    /// admitted for compound/update only when actual flow proved it receives a
    /// scalar, so an indirectly-delivered array (which leaves the param at the
    /// default I64, unproven) fails closed. This taint is retained because it
    /// (a) reports a sharper diagnostic for the direct array shapes and (b) is
    /// used elsewhere. NOTE: array call-return / pass-through DELIVERY is not
    /// currently functional at runtime; if it ever becomes so, revisit whether
    /// this taint should be extended to the indirect shapes (or whether the
    /// positive-proof gate remains sufficient on its own).
    non_scalar_params: HashSet<(String, String)>,
    /// `(func, param)` parameters for which NO call-site flow ever supplied a
    /// provably-scalar argument (a numeric/string/boolean literal, an
    /// arithmetic/unary/update/template expression, or a bare identifier that
    /// is itself a scalar-inflow param). Such a param's `Repr::I64` is the
    /// DEFAULT — left unconstrained — not positive proof it holds a number, so
    /// an array/object could have reached it through an INDIRECT call shape
    /// (`f(g())`, a pass-through chain, `f(o.a)`) the `non_scalar_params` taint
    /// cannot see. The param compound/update gate rejects these fail-closed:
    /// admission requires a POSITIVELY-proven scalar repr, not the default.
    /// Only PARAMETERS are ever recorded here (var locals carry declarator-based
    /// repr evidence and are unaffected). A never-called function's params all
    /// land here (no flow evidence at all) and correctly reject.
    params_lacking_scalar_inflow: HashSet<(String, String)>,
    /// `(func, binding)` var/let/const locals whose declarator RHS is an
    /// object literal. Object shape inference only assigns `Repr::Object`
    /// when the object is "materialized" (reached by a field read somewhere
    /// in the program); an object-initialized binding that is never
    /// field-read (e.g. `var o = {x:1}; o += 1;`) stays at the default
    /// `Repr::I64`, so the repr-allowlist check in the compound/update gate
    /// alone cannot see it — it would admit `o += 1` and silently do integer
    /// arithmetic on the never-materialized value (miscompile: prints `1`
    /// where node prints `[object Object]1`). This taint is independent of
    /// materialization: it fires on the syntactic shape of the declarator RHS
    /// alone, so the gate can reject fail-closed regardless of whether the
    /// object ever gets a shape. Mirrors [`non_scalar_params`](Self::non_scalar_params)
    /// (fasta Spec 7 Task 2).
    object_initialized_bindings: HashSet<(String, String)>,
    any_float: bool,
    any_string: bool,
    /// `(func, binding)` scalars/params whose `Repr::String` value is a FRESH
    /// runtime `string_concat` handle (reachable from a `+`, interpolated
    /// template, or string `+=`), NOT an interned literal constant. Codegen may
    /// identity-compare (`==`/`!=`) or truthiness-test an interned handle
    /// correctly, but a tainted (concat-derived) handle must be rejected in
    /// those positions — its fresh handle does not equal the interned handle of
    /// the same text. Populated only when the value is also proven `String`.
    string_concat_tainted: HashSet<(String, String)>,
    /// Functions whose `Repr::String` RETURN is a fresh runtime concat handle.
    string_concat_tainted_returns: HashSet<String>,
    /// Bindings whose string value may contain non-ASCII text (byte-length
    /// handles diverge from JS UTF-16 semantics): `(function, binding)`.
    string_non_ascii: HashSet<(String, String)>,
    /// Functions whose string return value may contain non-ASCII text.
    string_non_ascii_returns: HashSet<String>,
    /// Arrays whose ELEMENTS may contain non-ASCII string text: `(function, array binding)`.
    array_element_non_ascii: HashSet<(String, String)>,
    /// Arrays whose ELEMENTS may hold runtime-concat-derived strings.
    array_element_concat_tainted: HashSet<(String, String)>,
    /// Interned object layouts; `ShapeId` indexes this list.
    shapes: Vec<Vec<(String, Repr)>>,
    /// Gate messages from the shape inference (contradictory or unsupported
    /// object usage). Any entry makes compilation fail with E5506.
    shape_conflicts: Vec<String>,
}

impl ReprTable {
    pub fn scalar(&self, func: &str, binding: &str) -> Repr {
        self.scalars
            .get(&(func.to_string(), binding.to_string()))
            .copied()
            .unwrap_or_default()
    }

    pub fn array_element(&self, func: &str, binding: &str) -> Repr {
        self.array_elements
            .get(&(func.to_string(), binding.to_string()))
            .copied()
            .unwrap_or_default()
    }

    pub fn return_repr(&self, func: &str) -> Repr {
        self.returns.get(func).copied().unwrap_or_default()
    }

    pub fn param(&self, func: &str, index: usize) -> Repr {
        self.params
            .get(&(func.to_string(), index))
            .copied()
            .unwrap_or_default()
    }

    pub fn set_scalar(&mut self, func: &str, binding: &str, repr: Repr) {
        if repr == Repr::F64 {
            self.any_float = true;
        }
        if repr == Repr::String {
            self.any_string = true;
        }
        self.scalars
            .insert((func.to_string(), binding.to_string()), repr);
    }

    pub fn set_array_element(&mut self, func: &str, binding: &str, repr: Repr) {
        if repr == Repr::F64 {
            self.any_float = true;
        }
        if repr == Repr::String {
            self.any_string = true;
        }
        self.array_elements
            .insert((func.to_string(), binding.to_string()), repr);
    }

    pub fn set_return(&mut self, func: &str, repr: Repr) {
        if repr == Repr::F64 {
            self.any_float = true;
        }
        if repr == Repr::String {
            self.any_string = true;
        }
        self.returns.insert(func.to_string(), repr);
    }

    pub fn set_param(&mut self, func: &str, index: usize, repr: Repr) {
        if repr == Repr::F64 {
            self.any_float = true;
        }
        if repr == Repr::String {
            self.any_string = true;
        }
        self.params.insert((func.to_string(), index), repr);
    }

    /// Mark `(func, binding)` (scalar or param) as a runtime-concat-derived
    /// (tainted) string. Only meaningful together with a `Repr::String` entry.
    pub fn mark_string_concat_tainted(&mut self, func: &str, binding: &str) {
        self.string_concat_tainted
            .insert((func.to_string(), binding.to_string()));
    }

    /// Mark `func`'s return as a runtime-concat-derived (tainted) string.
    pub fn mark_string_concat_tainted_return(&mut self, func: &str) {
        self.string_concat_tainted_returns.insert(func.to_string());
    }

    /// True when `(func, binding)` holds a fresh runtime concat handle (a
    /// tainted string). Defaults to false — an interned literal string is not
    /// tainted, so identity-comparison/truthiness on it stays allowed.
    pub fn is_string_concat_tainted(&self, func: &str, binding: &str) -> bool {
        self.string_concat_tainted
            .contains(&(func.to_string(), binding.to_string()))
    }

    /// True when `func`'s return is a fresh runtime concat handle.
    pub fn is_string_concat_tainted_return(&self, func: &str) -> bool {
        self.string_concat_tainted_returns.contains(func)
    }

    /// Mark `(func, binding)` (scalar or param) as a non-ASCII string.
    /// Only meaningful together with a `Repr::String` entry.
    pub fn mark_string_non_ascii(&mut self, func: &str, binding: &str) {
        self.string_non_ascii
            .insert((func.to_string(), binding.to_string()));
    }

    /// Mark `func`'s return as a non-ASCII string.
    pub fn mark_string_non_ascii_return(&mut self, func: &str) {
        self.string_non_ascii_returns.insert(func.to_string());
    }

    /// True when `(func, binding)` holds a non-ASCII string.
    /// Defaults to false — an ASCII-only string does not need special handling.
    pub fn is_string_non_ascii(&self, func: &str, binding: &str) -> bool {
        self.string_non_ascii
            .contains(&(func.to_string(), binding.to_string()))
    }

    /// True when `func`'s return is a non-ASCII string.
    pub fn is_string_non_ascii_return(&self, func: &str) -> bool {
        self.string_non_ascii_returns.contains(func)
    }

    /// Mark `(func, binding)` array element as non-ASCII.
    pub fn mark_array_element_non_ascii(&mut self, func: &str, binding: &str) {
        self.array_element_non_ascii
            .insert((func.to_string(), binding.to_string()));
    }

    /// True when `(func, binding)` array element may contain non-ASCII text.
    pub fn is_array_element_non_ascii(&self, func: &str, binding: &str) -> bool {
        self.array_element_non_ascii
            .contains(&(func.to_string(), binding.to_string()))
    }

    /// Mark `(func, binding)` array element as runtime-concat-derived (tainted).
    pub fn mark_array_element_concat_tainted(&mut self, func: &str, binding: &str) {
        self.array_element_concat_tainted
            .insert((func.to_string(), binding.to_string()));
    }

    /// True when `(func, binding)` array element holds a fresh runtime concat handle.
    pub fn is_array_element_concat_tainted(&self, func: &str, binding: &str) -> bool {
        self.array_element_concat_tainted
            .contains(&(func.to_string(), binding.to_string()))
    }

    /// Record that `(func, binding)` is an array (any element repr). Additive;
    /// does not affect [`is_empty`](Self::is_empty) (an all-integer program with
    /// arrays still has no float decisions).
    pub fn set_array_binding(&mut self, func: &str, binding: &str) {
        self.array_bindings
            .insert((func.to_string(), binding.to_string()));
    }

    /// True when `(func, binding)` was recorded as an array by the inference.
    /// Defaults to false, so a scalar binding/param reports false.
    pub fn is_array_binding(&self, func: &str, binding: &str) -> bool {
        self.array_bindings
            .contains(&(func.to_string(), binding.to_string()))
    }

    /// Record that param `binding` of `func` may receive a non-scalar (array)
    /// argument at some call site — see [`non_scalar_params`](Self::non_scalar_params).
    pub fn mark_non_scalar_param(&mut self, func: &str, binding: &str) {
        self.non_scalar_params
            .insert((func.to_string(), binding.to_string()));
    }

    /// True when param `binding` of `func` may receive a non-scalar (array)
    /// argument. Defaults to false — a param only ever passed numbers/strings
    /// reports false, so its scalar compound/update lowering stays admitted.
    pub fn is_non_scalar_param(&self, func: &str, binding: &str) -> bool {
        self.non_scalar_params
            .contains(&(func.to_string(), binding.to_string()))
    }

    /// Record that param `binding` of `func` was NEVER shown, by any call-site
    /// flow, to receive a provably-scalar argument — see
    /// [`params_lacking_scalar_inflow`](Self::params_lacking_scalar_inflow).
    pub fn mark_param_lacking_scalar_inflow(&mut self, func: &str, binding: &str) {
        self.params_lacking_scalar_inflow
            .insert((func.to_string(), binding.to_string()));
    }

    /// True when `binding` is a PARAM of `func` whose scalar repr was never
    /// positively constrained by an actual scalar-argument call edge (default
    /// I64, unconstrained). The param compound/update gate rejects these. Only
    /// parameters are ever recorded, so this is always false for a var local.
    pub fn param_lacks_scalar_inflow(&self, func: &str, binding: &str) -> bool {
        self.params_lacking_scalar_inflow
            .contains(&(func.to_string(), binding.to_string()))
    }

    /// Record that `(func, binding)`'s declarator RHS is an object literal —
    /// see [`object_initialized_bindings`](Self::object_initialized_bindings).
    pub fn mark_object_initialized_binding(&mut self, func: &str, binding: &str) {
        self.object_initialized_bindings
            .insert((func.to_string(), binding.to_string()));
    }

    /// True when `(func, binding)` was declared with an object-literal
    /// initializer. Defaults to false — a non-object binding reports false,
    /// so its scalar compound/update lowering stays gated on repr alone.
    pub fn object_initialized_binding(&self, func: &str, binding: &str) -> bool {
        self.object_initialized_bindings
            .contains(&(func.to_string(), binding.to_string()))
    }

    /// True when no float representation, string representation, object shape,
    /// or shape conflict was ever recorded (codegen may keep its all-i64 fast
    /// paths).
    pub fn is_empty(&self) -> bool {
        !self.any_float
            && !self.any_string
            && self.shapes.is_empty()
            && self.shape_conflicts.is_empty()
    }

    pub fn intern_shape(&mut self, fields: Vec<(String, Repr)>) -> ShapeId {
        if let Some(index) = self.shapes.iter().position(|shape| *shape == fields) {
            return ShapeId(index as u32);
        }
        self.shapes.push(fields);
        ShapeId((self.shapes.len() - 1) as u32)
    }

    pub fn shape_fields(&self, shape: ShapeId) -> &[(String, Repr)] {
        &self.shapes[shape.0 as usize]
    }

    /// If every field of `shape` shares one repr, return it; else `None`.
    /// Dynamic (runtime-ordinal) computed access `obj[c]` selects a field by a
    /// runtime index, so it can only lower when a single element type covers
    /// every slot — a mixed-repr shape must fail closed.
    pub fn shape_is_uniform_repr(&self, shape: ShapeId) -> Option<Repr> {
        let fields = self.shape_fields(shape);
        let first = fields.first()?.1;
        if fields.iter().all(|(_, r)| *r == first) {
            Some(first)
        } else {
            None
        }
    }

    /// `(field index, field repr)` for `name` in `shape`; `None` for an
    /// unknown field (callers gate, never miscompile).
    pub fn shape_field(&self, shape: ShapeId, name: &str) -> Option<(usize, Repr)> {
        self.shape_fields(shape)
            .iter()
            .enumerate()
            .find(|(_, (field, _))| field == name)
            .map(|(index, (_, repr))| (index, *repr))
    }

    pub fn add_shape_conflict(&mut self, message: String) {
        self.shape_conflicts.push(message);
    }

    pub fn shape_conflicts(&self) -> &[String] {
        &self.shape_conflicts
    }
}

/// Disjoint-set forest whose sets carry a sticky "is float" bit.
#[derive(Default)]
pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u32>,
    float: Vec<bool>,
}

impl UnionFind {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate a new singleton node and return its id.
    pub fn fresh(&mut self) -> usize {
        let id = self.parent.len();
        self.parent.push(id);
        self.rank.push(0);
        self.float.push(false);
        id
    }

    pub fn find(&mut self, x: usize) -> usize {
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        // Path compression.
        let mut cur = x;
        while self.parent[cur] != cur {
            let next = self.parent[cur];
            self.parent[cur] = root;
            cur = next;
        }
        root
    }

    pub fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return;
        }
        let float = self.float[ra] || self.float[rb];
        let root = if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
            rb
        } else if self.rank[ra] > self.rank[rb] {
            self.parent[rb] = ra;
            ra
        } else {
            self.parent[rb] = ra;
            self.rank[ra] += 1;
            ra
        };
        self.float[root] = float;
    }

    pub fn seed_float(&mut self, x: usize) {
        let r = self.find(x);
        self.float[r] = true;
    }

    pub fn is_float(&mut self, x: usize) -> bool {
        let r = self.find(x);
        self.float[r]
    }
}

#[cfg(test)]
#[path = "repr_tests.rs"]
mod repr_tests;
