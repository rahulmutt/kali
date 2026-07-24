//! Static recognition and constant-folding of JS/host intrinsic call shapes.
mod array;
mod collections;
mod host;
mod math;
mod number;
mod object;
mod string;
pub(crate) use host::{SchedulingCallback, SchedulingSurface, EVENT_CTORS};
pub(crate) use number::{
    is_supported_static_ascii_char_code, parse_number_literal, parse_numeric_literal_value,
};
pub(crate) use string::{quote_string_literal, strip_string_delimiters};
