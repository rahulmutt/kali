fn main() {
    println!("Creating source...");
    let source = "foo();".to_string();
    println!("Source created: {}", source);
    
    println!("Collecting chars...");
    let chars: Vec<char> = source.chars().collect();
    println!("Chars collected: {:?}", chars);
    
    println!("Starting lex loop...");
    let mut position = 0;
    while position < chars.len() {
        println!("Position iteration {}:", position);
        let ch = chars[position];
        println!("  char at {}: {:?}", position, ch);
        
        if ch.is_ascii_alphabetic() {
            println!("  -> Identifier branch");
            let start = position;
            while position < chars.len() && (chars[position].is_ascii_alphanumeric() || chars[position] == '_' || chars[position] == '$') {
                position += 1;
            }
            println!("  Identifier from {} to {}: {:?}", start, position, &chars[start..position]);
        } else {
            println!("  -> Non-identifier branch");
            position += 1;
        }
         println!("  New position: {}, len: {}", position, chars.len());
         
         if position >= chars.len() {
             println!("Done!");
             break;
         }
     }
    
    println!("Final position: {}, expected: {}", position, chars.len());
    assert_eq!(position, chars.len());
}
