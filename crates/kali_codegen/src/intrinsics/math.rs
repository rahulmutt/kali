//! Static recognition and constant-folding of Math intrinsic call shapes.
use crate::*;

impl<'a> FunctionEmitter<'a> {
    pub(crate) fn math_max_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "max" {
            Some(MATH_MAX_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_min_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "min" {
            Some(MATH_MIN_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_abs_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "abs" {
            Some(MATH_ABS_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_sign_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "sign" {
            Some(MATH_SIGN_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_imul_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "imul" {
            Some(MATH_IMUL_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_round_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "round" {
            Some(MATH_ROUND_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_clz32_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "clz32" {
            Some(MATH_CLZ32_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_pow_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "pow" {
            Some(MATH_POW_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_member_method<'b>(&self, callee_node: &'b LirNode) -> Option<&'b str> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) {
            Some(method)
        } else {
            None
        }
    }

    pub(crate) fn math_exp_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value == 0 {
            Some(1)
        } else {
            None
        }
    }

    pub(crate) fn math_log_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value == 1 {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn math_exp2_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if !(0..=62).contains(&value) {
            return None;
        }

        Some(1_i64 << (value as u32))
    }

    pub(crate) fn math_expm1_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value == 0 {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn math_log1p_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value == 0 {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn math_fround_zero_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        if value == 0.0 {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn math_sin_cos_zero_constant_value(
        &self,
        method: &str,
        arg: LirNodeId,
    ) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        if value != 0.0 {
            return None;
        }

        Some(if method == "cos" { 1 } else { 0 })
    }

    pub(crate) fn math_hyperbolic_zero_constant_value(
        &self,
        method: &str,
        arg: LirNodeId,
    ) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        if value != 0.0 {
            return None;
        }

        Some(if method == "cosh" { 1 } else { 0 })
    }

    pub(crate) fn math_inverse_trig_constant_value(
        &self,
        method: &str,
        arg: LirNodeId,
    ) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;

        match method {
            "asin" | "atan" if value == 0.0 => Some(0),
            "acos" if value == 1.0 => Some(0),
            _ => None,
        }
    }

    pub(crate) fn math_atan2_zero_slice_value(&self, y: LirNodeId, x: LirNodeId) -> Option<i64> {
        let y = self.render_static_value(y)?;
        let x = self.render_static_value(x)?;
        let y = parse_numeric_literal_value(&y)?;
        let x = parse_numeric_literal_value(&x)?;
        if y == 0.0 && x.is_finite() && x >= 0.0 {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn math_inverse_hyperbolic_constant_value(
        &self,
        method: &str,
        arg: LirNodeId,
    ) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;

        match method {
            "acosh" if value == 1.0 => Some(0),
            "asinh" | "atanh" if value == 0.0 => Some(0),
            _ => None,
        }
    }

    pub(crate) fn math_sqrt_constant_root(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value < 0 {
            return None;
        }

        let root = (value as f64).sqrt() as i64;
        if root.checked_mul(root) == Some(value) {
            Some(root)
        } else {
            None
        }
    }

    pub(crate) fn math_cbrt_constant_root(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        let root = (value as f64).cbrt().round() as i64;
        if i128::from(root).pow(3) == i128::from(value) {
            Some(root)
        } else {
            None
        }
    }

    pub(crate) fn math_log2_constant_exponent(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value <= 0 {
            return None;
        }

        let value = value as u64;
        if value.is_power_of_two() {
            Some(i64::from(value.trailing_zeros()))
        } else {
            None
        }
    }

    pub(crate) fn math_log10_constant_exponent(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let mut value = parse_number_literal(&rendered)?;
        if value <= 0 {
            return None;
        }

        let mut exponent = 0;
        while value % 10 == 0 {
            value /= 10;
            exponent += 1;
        }

        if value == 1 {
            Some(exponent)
        } else {
            None
        }
    }

    pub(crate) fn math_hypot_constant_root(&self, args: &[LirNodeId]) -> Option<i64> {
        if args.is_empty() {
            return Some(0);
        }

        let mut sum = 0_i128;
        for arg in args {
            let rendered = self.render_static_value(*arg)?;
            let value = parse_numeric_literal_value(&rendered)?;
            if !value.is_finite()
                || value.fract() != 0.0
                || value < i64::MIN as f64
                || value > i64::MAX as f64
            {
                return None;
            }

            let value = value as i128;
            sum = sum.checked_add(value.checked_mul(value)?)?;
        }

        self.perfect_square_root_i128(sum)
    }

    pub(crate) fn math_round_like_static_literal_value(
        &self,
        method: &str,
        arg: LirNodeId,
    ) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        let folded = match method {
            "round" => {
                if value.fract() == 0.0 {
                    value
                } else {
                    (value + 0.5).floor()
                }
            }
            "trunc" => value.trunc(),
            "ceil" => value.ceil(),
            "floor" => value.floor(),
            _ => return None,
        };

        if !folded.is_finite() || folded < i64::MIN as f64 || folded > i64::MAX as f64 {
            return None;
        }

        Some(folded as i64)
    }

    pub(crate) fn math_extrema_static_literal_value(
        &self,
        method: &str,
        args: &[LirNodeId],
    ) -> Option<i64> {
        let mut values = args.iter().map(|arg| {
            let rendered = self.render_static_value(*arg)?;
            let value = parse_numeric_literal_value(&rendered)?;
            if !value.is_finite()
                || value.fract() != 0.0
                || value < i64::MIN as f64
                || value > i64::MAX as f64
            {
                return None;
            }
            Some(value as i64)
        });

        let mut folded = values.next().flatten()?;
        for value in values {
            let value = value?;
            folded = if method == "max" {
                folded.max(value)
            } else {
                folded.min(value)
            };
        }

        Some(folded)
    }

    pub(crate) fn math_abs_static_literal_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        parse_number_literal(&rendered)?.checked_abs()
    }

    pub(crate) fn math_imul_static_literal_value(
        &self,
        left: LirNodeId,
        right: LirNodeId,
    ) -> Option<i64> {
        let rendered_left = self.render_static_value(left)?;
        let rendered_right = self.render_static_value(right)?;
        let left = parse_number_literal(&rendered_left)? as i32;
        let right = parse_number_literal(&rendered_right)? as i32;
        Some(i64::from(left.wrapping_mul(right)))
    }

    pub(crate) fn math_clz32_static_literal_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        let uint32 = self.to_uint32_literal_value(value)?;
        Some(i64::from(uint32.leading_zeros()))
    }

    pub(crate) fn math_sign_static_literal_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        Some(if value == 0.0 {
            0
        } else if value.is_sign_negative() {
            -1
        } else {
            1
        })
    }
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod math_tests;
