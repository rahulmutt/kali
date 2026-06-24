//! WASM code generation for the Kali compiler.

mod ctx;
mod emit;
mod emitter;
mod intrinsics;
mod lower;
pub use ctx::{CodegenCtx, CodegenResult, TargetConfig};
pub(crate) use intrinsics::{quote_string_literal, strip_string_delimiters, parse_number_literal, parse_numeric_literal_value, is_supported_static_ascii_char_code, static_parse_float_ascii_integer, static_parse_int_ascii};
use emitter::{
    ControlFlowLabelKind, EmittedValue, FunctionEmitter, FunctionPlan, LoopFrame,
    ObjectEnumerationMode, ValueShape,
};
use ctx::{
    StaticArrayAtResult, StaticArraySearchResult, StaticIndexMemberResult,
    StaticObjectIdentityValue, StaticStringAtResult, StringPool,
};

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashSet},
    path::PathBuf,
};

use kali_common::generator_function_yield_lowering_unavailable_message;
use kali_error::{
    _error_codes::{e3, e5, e8},
    Diagnostic, DiagnosticContext, DiagnosticContextOrigin,
};
use kali_lir::{FunctionFlavor, LirNode, LirNodeId, LirNodeKind, LirProgram};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use wasm_encoder::{
    BlockType, CodeSection, ConstExpr, CustomSection, DataSection, EntityType, ExportKind,
    ExportSection, Function, FunctionSection, ImportSection, Instruction, MemorySection,
    MemoryType, Module, TypeSection, ValType,
};

const TEST_REGISTER_IMPORT_INDEX: u32 = 0;
const CONSOLE_LOG_IMPORT_INDEX: u32 = 1;
const CONSOLE_ERROR_IMPORT_INDEX: u32 = 2;
const CONSOLE_WARN_IMPORT_INDEX: u32 = 3;
const CONSOLE_INFO_IMPORT_INDEX: u32 = 4;
const CONSOLE_DEBUG_IMPORT_INDEX: u32 = 5;
const ARGS_LEN_IMPORT_INDEX: u32 = 6;
const MATH_MAX_IMPORT_INDEX: u32 = 7;
const MATH_MIN_IMPORT_INDEX: u32 = 8;
const MATH_ABS_IMPORT_INDEX: u32 = 9;
const MATH_SIGN_IMPORT_INDEX: u32 = 10;
const MATH_IMUL_IMPORT_INDEX: u32 = 11;
const MATH_ROUND_IMPORT_INDEX: u32 = 12;
const PROCESS_PID_IMPORT_INDEX: u32 = 13;
const CWD_IMPORT_INDEX: u32 = 14;
const MATH_CLZ32_IMPORT_INDEX: u32 = 15;
const MATH_POW_IMPORT_INDEX: u32 = 16;
const COVERAGE_HIT_IMPORT_INDEX: u32 = 17;
const FUNCTION_INDEX_OFFSET: u32 = 17;
const ENV_GET_BUFFER_RESERVED: u32 = 4096;
const STRING_HANDLE_TAG: u64 = 0x8000_0000_0000_0000;

pub use lower::lower_lir_to_wasm;
pub(crate) use lower::{
    collect_function_locals, collect_function_locals_from_node, collect_functions,
    collect_functions_from_node, emit_literal, encode_string_handle,
    function_plan, generator_lowering_unavailable_message, is_function_like,
    is_process_root, process_env_property_key, program_uses_cwd_set,
    program_uses_env_delete, program_uses_env_get, program_uses_env_has,
    program_uses_env_set, program_uses_process_exit, top_level_children,
};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
