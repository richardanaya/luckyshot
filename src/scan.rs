use std::collections::HashMap;
use std::fs;
use glob_match::glob_match;

pub async fn scan_files(pattern: &str, api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Scanning for files matching pattern: {}", pattern);
    let mut file_embeddings: HashMap<String, Vec<f32>> = HashMap::new();

    fn visit_dirs(dir: &std::path::Path, pattern: &str, file_embeddings: &mut HashMap<String, Vec<f32>>, api_key: &str) {
        if dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_dir() {
                            visit_dirs(&path, pattern, file_embeddings, api_key);
                        } else {
                            let relative_path = path.strip_prefix(".").unwrap_or(&path);
                            let path_str = relative_path.to_string_lossy().to_string();
                            
                            // Skip the vectors file itself
                            if path_str.ends_with(".luckyshot.file.vectors.v1") {
                                continue;
                            }
                            
                            // Only process files that match the pattern
                            if !glob_match(pattern, &path_str) {
                                continue;
                            }

                            process_file(&path, &path_str, file_embeddings, api_key);
                        }
                    }
                }
            }
        }
    }

    fn process_file(path: &std::path::Path, path_str: &str, file_embeddings: &mut HashMap<String, Vec<f32>>, api_key: &str) {
        println!("Processing: {}", path_str);
        
        match fs::read_to_string(path) {
            Ok(contents) => {
                match tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(crate::openai::get_embedding(&contents, api_key))
                {
                    Ok(embedding) => {
                        println!("Got embedding for {} (length {})", path_str, embedding.len());
                        file_embeddings.insert(path_str.to_string(), embedding);
                    }
                    Err(e) => eprintln!("Error getting embedding for {}: {}", path_str, e),
                }
            }
            Err(e) => eprintln!("Error reading file {}: {}", path_str, e),
        }
    }

    // Start recursive directory traversal from current directory
    visit_dirs(std::path::Path::new("."), pattern, &mut file_embeddings, api_key);

    // Save embeddings to file
    match serde_json::to_string_pretty(&file_embeddings) {
        Ok(json) => {
            if let Err(e) = fs::write(".luckyshot.file.vectors.v1", json) {
                eprintln!("Error writing vectors file: {}", e);
            } else {
                println!(
                    "Successfully saved vectors for {} files",
                    file_embeddings.len()
                );
            }
        }
        Err(e) => eprintln!("Error serializing vectors: {}", e),
    }

    Ok(())
}
