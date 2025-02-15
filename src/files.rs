use std::path::PathBuf;
use glob_match::glob_match;

pub fn find_matching_files(pattern: &str) -> Vec<PathBuf> {
    let mut matches = Vec::new();
    let current_dir = std::env::current_dir().unwrap_or_default();
    
    fn visit_dir(dir: &std::path::Path, pattern: &str, matches: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        visit_dir(&path, pattern, matches);
                    } else {
                        let relative_path = path.strip_prefix(&std::env::current_dir().unwrap_or_default())
                            .unwrap_or(&path);
                        if glob_match(pattern, &relative_path.to_string_lossy()) {
                            matches.push(path);
                        }
                    }
                }
            }
        }
    }

    visit_dir(&current_dir, pattern, &mut matches);
    matches
}
