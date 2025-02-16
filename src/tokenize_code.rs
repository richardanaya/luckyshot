use std::path::Path;
use std::collections::HashSet;

/// Deduplicates tokens while preserving their original order
pub fn deduplicate_tokens(tokens: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    tokens.into_iter().filter(|token| seen.insert(token.clone())).collect()
}

/// Tokenizes code into a vector of strings, with special handling for Rust files
pub fn tokenize_code(input: &str, file_path: &str) -> Vec<String> {
    let path = Path::new(file_path);
    if let Some(extension) = path.extension() {
        match extension.to_str().unwrap_or("") {
            "rs" => return tokenize_rust_code(input),
            "html" | "htm" => return tokenize_html_code(input),
            _ => return tokenize_generic_code(input),
        }
    }
    tokenize_generic_code(input)
}

/// Tokenizes HTML code into a vector of strings
fn tokenize_html_code(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '<' => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                chars.next();
            },
            '>' => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                chars.next();
            },
            c if c.is_whitespace() => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                chars.next();
            },
            '"' | '\'' => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                chars.next();
                // Collect the attribute value
                while let Some(&next_c) = chars.peek() {
                    chars.next();
                    if next_c == c {
                        break;
                    }
                    current_token.push(next_c);
                }
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
            },
            '=' | '/' => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                chars.next();
            },
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
            "const", "result", "=", "calculateSum", "10", "20",
            "console", "log", "Result", "result"
        ];
        
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_tokenize_html_code() {
        let code = r#"
            <div class="container">
                <h1>Hello World</h1>
                <p id="main-text">This is a test</p>
                <input type="text" value="search" />
            </div>
        "#;
        
        let tokens = tokenize_code(code, "test.html");
        let expected = vec![
            "div", "class", "container",
            "h1", "Hello", "World", "h1",
            "p", "id", "main-text", "This", "is", "a", "test", "p",
            "input", "type", "text", "value", "search", "div"
        ];
        
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_deduplicate_tokens() {
        let tokens = vec![
            "fn".to_string(),
            "main".to_string(),
            "let".to_string(),
            "x".to_string(),
            "let".to_string(),
            "y".to_string(),
            "fn".to_string(),
        ];
        
        let deduped = deduplicate_tokens(tokens);
        let expected = vec![
            "fn".to_string(),
            "main".to_string(),
            "let".to_string(),
            "x".to_string(),
            "y".to_string(),
        ];
        
        assert_eq!(deduped, expected);
    }
}
