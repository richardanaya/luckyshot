use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct FileEmbedding {
    pub filename: String,
    pub vector: Vec<f32>,
    pub last_modified: u64,
    pub chunk_offset: usize,  // Starting position of chunk in file
    pub chunk_size: usize,    // Size of this chunk (might be smaller for last chunk)
    pub is_full_file: bool,   // Whether this is a full file embedding or a chunk
}

pub async fn scan_files(pattern: &str, api_key: &str, chunk_size: usize, overlap_size: usize, embed_metadata: bool) -> Result<(), Box<dyn std::error::Error>> {
    if chunk_size > 0 && overlap_size >= chunk_size {
        return Err("overlap_size must be less than chunk_size".into());
    }
    println!("Scanning for files matching pattern: {}", pattern);
    let mut file_embeddings: Vec<FileEmbedding> = Vec::new();

    fn create_chunks(content: &str, chunk_size: usize, overlap_size: usize) -> Vec<(usize, String)> {
        if chunk_size == 0 {
            return vec![(0, content.to_string())];
        }

        let mut chunks = Vec::new();
        let content_len = content.len();
        let mut offset = 0;

        while offset < content_len {
            let end = (offset + chunk_size).min(content_len);
            let chunk = content[offset..end].to_string();
            chunks.push((offset, chunk));
        
            if end == content_len {
                break;
            }
        
            offset += chunk_size - overlap_size;
        }

        chunks
    }

    async fn process_file(
        path: &std::path::Path,
        path_str: &str,
        file_embeddings: &mut Vec<FileEmbedding>,
        api_key: &str,
        chunk_size: usize,
        overlap_size: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Processing: {}", path_str);
    
        match fs::read_to_string(path) {
            Ok(contents) => {
                let chunks = create_chunks(&contents, chunk_size, overlap_size);
            
                for (offset, chunk_content) in chunks {
                    let content_to_embed = if embed_metadata {
                        // Include file metadata in the content
                        let metadata = fs::metadata(path)?;
                        let modified = metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs();
                        let size = metadata.len();
                        format!(
                            "File: {}\nLast Modified: {}\nSize: {}\nContent:\n{}", 
                            path_str, modified, size, chunk_content
                        )
                    } else {
                        chunk_content.clone()
                    };
                    
                    match crate::openai::get_embedding(&content_to_embed, api_key).await {
                        Ok(embedding) => {
                            let metadata = fs::metadata(path)?;
                            let last_modified = metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs();
                        
                            file_embeddings.push(FileEmbedding {
                                filename: path_str.to_string(),
                                vector: embedding,
                                last_modified,
                                chunk_offset: offset,
                                chunk_size: chunk_content.len(),
                                is_full_file: chunk_size == 0,
                            });
                        
                            if chunk_size > 0 {
                                println!("Got embedding for {} (chunk offset: {})", path_str, offset);
                            } else {
                                println!("Got embedding for {}", path_str);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error getting embedding for {}: {}", path_str, e);
                            return Err(e);
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("Error reading file {}: {}", path_str, e);
                Err(Box::new(e))
            }
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
        
        process_file(&path, &path_str, &mut file_embeddings, api_key, chunk_size, overlap_size).await?;
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
