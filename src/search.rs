use crate::scan::FileEmbedding;
use std::fs;

#[derive(Debug)]
pub struct FileMatch {
    pub filename: String,
    pub similarity: f32,
}

pub async fn find_related_files(query_embedding: Vec<f32>, filter_similarity: f32, verbose: bool, file_contents: bool, count: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Load the vectors file
    let vectors_content = match fs::read_to_string(".luckyshot.file.vectors.v1") {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading vectors file: {}", e);
            return Ok(Vec::new());
        }
    };

    // Parse the JSON
    let file_embeddings: Vec<FileEmbedding> = match serde_json::from_str(&vectors_content) {
        Ok(embeddings) => embeddings,
        Err(e) => {
            eprintln!("Error parsing vectors file: {}", e);
            return Ok(Vec::new());
        }
    };

    // Calculate similarity for each file
    let mut matches: Vec<FileMatch> = file_embeddings
        .iter()
        .map(|embedding| {
            let similarity = crate::bm25::bm25_similarity(&query_embedding, &embedding.vector);
            FileMatch {
                filename: embedding.filename.clone(),
                similarity,
            }
        })
        .collect();

    // Find min and max similarities for normalization
    let min_similarity = matches.iter().map(|m| m.similarity).fold(f32::INFINITY, f32::min);
    let max_similarity = matches.iter().map(|m| m.similarity).fold(f32::NEG_INFINITY, f32::max);
    
    // Normalize similarities to 0-1 range
    if (max_similarity - min_similarity).abs() > f32::EPSILON {
        for m in &mut matches {
            m.similarity = (m.similarity - min_similarity) / (max_similarity - min_similarity);
        }
    }

    // Sort by normalized similarity (highest first)
    matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

    // First filter by similarity threshold
    let similarity_filtered: Vec<_> = matches.iter()
        .filter(|m| m.similarity >= filter_similarity)
        .collect();

    // Then limit by count if specified
    let final_matches: Vec<_> = if count > 0 {
        similarity_filtered.iter().take(count).cloned().collect()
    } else {
        similarity_filtered
    };

    // Return early if no matches
    if final_matches.is_empty() {
        return Ok(Vec::new());
    }

    // Print results according to flags
    if verbose {
        println!("Score,File,Type,Offset,Size");
        for m in &final_matches {
            let embedding = file_embeddings.iter()
                .find(|e| e.filename == m.filename)
                .unwrap();
            println!("{:.3},{},{},{},{}",
                m.similarity,
                m.filename,
                if embedding.is_full_file { "full" } else { "chunk" },
                embedding.chunk_offset,
                embedding.chunk_size
            );
        }
    } else if file_contents {
        for m in &final_matches {
            let embedding = file_embeddings.iter()
                .find(|e| e.filename == m.filename)
                .unwrap();
            
            if let Ok(contents) = std::fs::read_to_string(&embedding.filename) {
                let start = embedding.chunk_offset;
                let end = start + embedding.chunk_size;
                if end <= contents.len() {
                    let chunk_content = &contents[start..end];
                    
                    // If metadata was included in the embedding, reconstruct it for display
                    let display_content = if embedding.has_metadata {
                        crate::metadata::prepend_metadata(
                            &embedding.filename,
                            embedding.last_modified,
                            std::fs::metadata(&embedding.filename)?.len(),
                            chunk_content
                        )
                    } else {
                        chunk_content.to_string()
                    };

                    println!("\n--- Content from {} ---", embedding.filename);
                    println!("{}", display_content);
                    println!("--- End content ---\n");
                }
            }
            println!("{}", m.filename);
        }
    } else {
        // Just print filenames as the default case
        for m in &final_matches {
            println!("{}", m.filename);
        }
    }

    // Return just the filenames
    Ok(final_matches.iter().map(|m| m.filename.clone()).collect())
}
