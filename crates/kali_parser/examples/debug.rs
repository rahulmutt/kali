fn main() {
    println!("=== Lexing 'foo()' ===");
    
    let source = "foo();".to_string();
    let chars: Vec<char> = source.chars().collect();
    let mut position = 0;
    
    println!("Source length: {}", chars.len());
    
    while position < chars.len() {
        println!("Position {}: {:?}", position, chars[position]);
        
        let ch = chars[position];
        
        if ch.is_ascii_alphabetic() {
            println!("  -> consuming identifier");
            position += 1;
            while position < chars.len() && (chars[position].is_ascii_alphanumeric() || chars[position] == '_' || chars[position] == '$') {
                position += 1;
            }
        } else {
            println!("  -> consuming single char");
            position += 1;
        }
    }
    
    println!("Position at EOF: {}", position);
    println!("=== Done ===");
}
