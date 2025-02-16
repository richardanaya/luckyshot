/// Tokenizes code into a vector of strings, handling common code patterns
pub fn tokenize_code(input: &str) -> Vec<String> {
    // Split on whitespace and common code delimiters
    input
        .split(|c: char| {
            c.is_whitespace() || 
            c == '(' || c == ')' ||
            c == '{' || c == '}' ||
            c == '[' || c == ']' ||
            c == '.' || c == ',' ||
            c == ';' || c == ':' ||
            c == '"' || c == '\''
        })
        .filter(|s| !s.is_empty()) // Remove empty strings
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_code() {
        let code = "fn hello_world() { println!(\"Hello, world!\"); }";
        let tokens = tokenize_code(code);
        assert_eq!(
            tokens,
            vec!["fn", "hello_world", "println", "Hello", "world"]
        );
    }
}
