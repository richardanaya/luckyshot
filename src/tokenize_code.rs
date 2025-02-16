use std::path::Path;

/// Tokenizes code into a vector of strings, with special handling for Rust files
pub fn tokenize_code(input: &str, file_path: &str) -> Vec<String> {
    let path = Path::new(file_path);
    if let Some(extension) = path.extension() {
        if extension == "rs" {
            return tokenize_rust_code(input);
        }
    }
    tokenize_generic_code(input)
}

/// Tokenizes generic code into a vector of strings
fn tokenize_generic_code(input: &str) -> Vec<String> {
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
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Tokenizes Rust code into a vector of strings, handling Rust-specific patterns
fn tokenize_rust_code(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            // Skip whitespace
            c if c.is_whitespace() => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                chars.next();
            },
            // Handle numbers (including floating point)
            c if c.is_numeric() || c == '.' => {
                if current_token.is_empty() || current_token.chars().next().unwrap().is_numeric() {
                    current_token.push(c);
                } else {
                    tokens.push(current_token.clone());
                    current_token.clear();
                    current_token.push(c);
                }
                chars.next();
            },
            // Handle special characters and operators
            '<' | '>' | '(' | ')' | '{' | '}' | '[' | ']' | ',' | ';' | ':' | '"' | '\'' | '=' | '?' => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                chars.next();
            },
            // Handle special tokens
            '-' => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                chars.next();
                if chars.peek() == Some(&'>') {
                    tokens.push("->".to_string());
                    chars.next();
                }
            },
            '!' => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                chars.next();
            },
            '#' => {
                chars.next();
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
            },
            // Build identifiers and other tokens
            _ => {
                current_token.push(c);
                chars.next();
            }
        }
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    tokens.into_iter()
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_rust_code() {
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
        
        let tokens = tokenize_code(code, "test.rs");
        let expected = vec![
            "derive", "Debug", "struct", "Point", "T",
            "x", "T", "y", "T", "impl", "T", "Point",
            "T", "fn", "new", "x", "T", "y", "T", "->", "Self",
            "Point", "x", "y", "fn", "main", "let", "point",
            "Point", "new", "10.5", "20.0", "println",
            "Point", "point"
        ];
        
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_tokenize_generic_code() {
        let code = r#"
            function calculateSum(a, b) {
                return a + b;
            }
            
            const result = calculateSum(10, 20);
            console.log("Result:", result);
        "#;
        
        let tokens = tokenize_code(code, "test.js");
        let expected = vec![
            "function", "calculateSum", "a", "b",
            "return", "a", "+", "b",
            "const", "result", "calculateSum", "10", "20",
            "console", "log", "Result", "result"
        ];
        
        assert_eq!(tokens, expected);
    }
}
