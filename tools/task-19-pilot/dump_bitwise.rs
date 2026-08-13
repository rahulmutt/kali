// Rule 8: resolve `bitwise_operators_runtime.rs`'s `format!` fixtures by
// EXECUTING the real code, never by hand-applying substitution. PACK is copied
// verbatim from bitwise_operators_runtime.rs:50; the format! calls are copied
// verbatim from its eight #[test] fns.
const PACK: &str = "let byte = 0;\nfor (let i = 0; i < 8; i = i + 1) { byte = (byte << 1) | 1; }\n";

fn emit(label: &str, s: &str) {
    println!("====={label}");
    print!("{s}");
    println!("=====END");
}

fn main() {
    emit("shift_left_and_or_pack_bits", &format!("{PACK}console.log(\"\" + byte);"));
    emit("bitwise_and", &format!("{PACK}console.log(\"\" + (byte & 15));"));
    emit("bitwise_or", &format!("{PACK}console.log(\"\" + (byte | 256));"));
    emit("bitwise_xor", &format!("{PACK}console.log(\"\" + (byte ^ 255));"));
    emit("shift_right_arithmetic", &format!("{PACK}console.log(\"\" + (byte >> 4));"));
    emit("shift_right_arithmetic_negative", &format!("{PACK}let neg = 0 - byte;\nconsole.log(\"\" + (neg >> 1));"));
    emit("unsigned_shift_zero_extends", &format!("{PACK}let neg = 0 - byte;\nconsole.log(\"\" + (neg >>> 0));"));
    // bitwise_on_float_operand_is_rejected uses a plain &str, no format!
    emit("bitwise_on_float_operand_is_rejected", "let x = 0.0;\nfor (let i = 0; i < 3; i = i + 1) { x = x + 1.5; }\nconsole.log(\"\" + (x & 1));");
}
