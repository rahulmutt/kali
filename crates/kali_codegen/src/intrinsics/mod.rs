//! Static recognition and constant-folding of JS/host intrinsic call shapes.
use crate::*;
mod string;
mod array;
mod math;
mod number;
pub(crate) use string::{quote_string_literal, strip_string_delimiters};
pub(crate) use number::{parse_number_literal, parse_numeric_literal_value, is_supported_static_ascii_char_code, static_parse_float_ascii_integer, static_parse_int_ascii};
