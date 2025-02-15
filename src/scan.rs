use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub async fn scan_files(pattern: &str, api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Scanning for files matching pattern: {}", pattern);
    let mut file_embeddings: HashMap<String, Vec<f32>> = HashMap::new();

    async fn process_file(path: &std::path::Path, path_str: &str, file_embeddings: &mut HashMap<String, Vec<f32>>, api_key: &str) {
        println!("Processing: {}", path_str);
        
        match fs::read_to_string(path) {
            Ok(contents) => {
                match crate::openai::get_embedding(&contents, api_key).await {
                    Ok(embedding) => {
                        println!("Got embedding for {}", path_str);
                        file_embeddings.insert(path_str.to_string(), embedding);
                    }
                    Err(e) => eprintln!("Error getting embedding for {}: {}", path_str, e),
                }
            }
            Err(e) => eprintln!("Error reading file {}: {}", path_str, e),
        }
    }

    // Find all matching files
    let matching_files = crate::files::find_matching_files(pattern);
    
    // Process each file
    for path in matching_files {
        let relative_path = path.strip_prefix(std::env::current_dir()?).unwrap_or(&path);
        let path_str = relative_path.to_string_lossy().to_string();
        
        // Skip the vectors file
        if path_str.ends_with(".luckyshot.file.vectors.v1") {
            continue;
        }
        
        process_file(&path, &path_str, &mut file_embeddings, api_key).await;
    }

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
