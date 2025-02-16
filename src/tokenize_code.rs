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
        let code = r#"
            #[derive(Debug)]
            struct Point<T> {
                x: T,
                y: T,
            }
            
            impl<T> Point<T> {
                fn new(x: T, y: T) -> Self {
                    Point { x, y }
                }
            }
            
            fn main() {
                let point = Point::new(10.5, 20.0);
                println!("Point: {:?}", point);
            }
        "#;
        
        let tokens = tokenize_code(code);
        let expected = vec![
            "derive", "Debug", "struct", "Point", "T",
            "x", "T", "y", "T", "impl", "T", "Point",
            "T", "fn", "new", "x", "T", "y", "T", "Self",
            "Point", "x", "y", "fn", "main", "let", "point",
            "Point", "new", "10.5", "20.0", "println",
            "Point", "point"
        ];
        
        assert_eq!(tokens, expected);
    }
}
