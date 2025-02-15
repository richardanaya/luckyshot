use std::path::PathBuf;
use glob_match::glob_match;
use ignore::WalkBuilder;

pub fn find_matching_files(pattern: &str) -> Vec<PathBuf> {
    let mut matches = Vec::new();
    let current_dir = std::env::current_dir().unwrap_or_default();

    let walker = WalkBuilder::new(&current_dir)
        .hidden(false) // Include hidden files/directories
        .git_ignore(true) // Respect .gitignore
        .build();

    for result in walker {
        if let Ok(entry) = result {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                let path = entry.path();
                let relative_path = path.strip_prefix(&current_dir).unwrap_or(path);
                if glob_match(pattern, &relative_path.to_string_lossy()) {
                    matches.push(path.to_path_buf());
                }
            }
        }
    }

    matches
}
