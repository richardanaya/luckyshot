pub fn prepend_metadata(path_str: &str, modified: u64, size: u64, content: &str) -> String {
    format!(
        "File: {}\nLast Modified: {}\nSize: {}\nContent:\n{}",
        path_str, modified, size, content
    )
}
