use crate::scan::FileVectorStore;
use std::fs;

#[derive(Debug)]
pub struct FileMatch {
    pub filename: String,
    pub similarity: f32,
}

pub async fn find_related_files(
    query_text: &str,
    api_key: &str,
    filter_similarity: f32,
    verbose: bool,
    debug: bool,
    file_contents: bool,
    count: usize,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Load the vectors file
    let vectors_content = match fs::read_to_string(".luckyshot.file.vectors.v1") {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading vectors file: {}", e);
            return Ok(Vec::new());
        }
    };

    // Parse the JSON
    let store: FileVectorStore = match serde_json::from_str(&vectors_content) {
        Ok(embeddings) => embeddings,
        Err(e) => {
            eprintln!("Error parsing vectors file: {}", e);
            return Ok(Vec::new());
        }
    };

    // Perform BM25 ranking
    let bm25_results = crate::bm25_ranker::rank_documents(&store, query_text, store.bm25_avgdl);

    // Get query embedding and calculate similarity for each file
    let query_embedding = match crate::openai::get_embedding(query_text, api_key).await {
        Ok(embedding) => embedding,
        Err(e) => {
            eprintln!("Error getting query embedding: {}", e);
            return Ok(Vec::new());
        }
    };

    let mut matches: Vec<FileMatch> = store
        .rag_vectors
        .iter()
        .map(|embedding| {
            let similarity =
                crate::similarity::dot_product_similarity(&query_embedding, &embedding.vector);
            FileMatch {
                filename: embedding.filename.clone(),
                similarity,
            }
        })
        .collect();

    // Sort matches by similarity
    matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

    if debug {
        // print out the BM25 results
        println!("\nBM25 ranks:");
        for scored_doc in &bm25_results {
            let doc_index = scored_doc.id as usize;
            if doc_index < store.bm25_files.len() {
                println!(
                    "{}: {}",
                    scored_doc.score, store.bm25_files[doc_index].filename
                );
            }
        }
        println!("\n");

        //print out the RAG vector distances and filename, in descending
        println!("RAG distances:");
        for m in &matches {
            println!("{} {}", m.similarity, m.filename);
        }
        println!("\n");
    }

    // I want you to normalize the BM25 numbers, but I need you to do it in way that gives range 1 to -1
    // Find min and max BM25 scores
    let min_bm25 = bm25_results
        .iter()
        .map(|m| m.score)
        .fold(f32::INFINITY, f32::min);
    let max_bm25 = bm25_results
        .iter()
        .map(|m| m.score)
        .fold(f32::NEG_INFINITY, f32::max);
    // greatest
    let min_abs = min_bm25.abs();
    let max_abs = max_bm25.abs();
    let max_extent = min_abs.max(max_abs);
    let min_bm25 = -max_extent;
    let max_bm25 = max_extent;

    // Normalize BM25 scores to -1 to 1 range
    for m in matches.iter_mut() {
        // Find corresponding BM25 score by matching filename
        if let Some(bm25_result) = bm25_results.iter().find(|r| r.id.to_string() == m.filename) {
            m.similarity = (bm25_result.score - min_bm25) / (max_bm25 - min_bm25) * 2.0 - 1.0;
        }
    }

    // for each missing file in bm25_results, add it to the matches with a similarity of 0
    for bm25_result in &bm25_results {
        if !matches
            .iter()
            .any(|m| m.filename == bm25_result.id.to_string())
        {
            matches.push(FileMatch {
                filename: bm25_result.id.to_string(),
                similarity: 0.0,
            });
        }
    }

    // Find min and max similarities for normalization
    let min_similarity = matches
        .iter()
        .map(|m| m.similarity)
        .fold(f32::INFINITY, f32::min);
    let max_similarity = matches
        .iter()
        .map(|m| m.similarity)
        .fold(f32::NEG_INFINITY, f32::max);

    // Normalize similarities to 0-1 range
    if (max_similarity - min_similarity).abs() > f32::EPSILON {
        for m in &mut matches {
            m.similarity = (m.similarity - min_similarity) / (max_similarity - min_similarity);
        }
    }

    if debug {
        // print out the normalized bm25
        println!("Normalized BM25:");
        for m in &matches {
            println!("{} {}", m.similarity, m.filename);
        }
        println!("\n");

        // print out the normalized rag
        println!("Normalized RAG:");
        for m in &matches {
            println!("{} {}", m.similarity, m.filename);
        }
    }

    // First filter by similarity threshold
    let similarity_filtered: Vec<&FileMatch> = matches
        .iter()
        .filter(|m| m.similarity >= filter_similarity)
        .collect();

    // For non-verbose, non-file-contents mode, deduplicate filenames before count limiting
    let deduplicated: Vec<&FileMatch> = if !verbose && !file_contents {
        let mut seen = std::collections::HashSet::new();
        similarity_filtered
            .iter()
            .filter(|m| seen.insert(&m.filename))
            .copied()
            .collect()
    } else {
        similarity_filtered
    };

    // Then limit by count if specified
    let final_matches: Vec<_> = if count > 0 {
        deduplicated.iter().take(count).cloned().collect()
    } else {
        deduplicated
    };

    // Return early if no matches
    if final_matches.is_empty() {
        return Ok(Vec::new());
    }

    // Print results according to flags
    if verbose {
        println!("Score,File,Type,Offset,Size");
        for m in &final_matches {
            let embedding = store
                .rag_vectors
                .iter()
                .find(|e| e.filename == m.filename)
                .unwrap();
            println!(
                "{:.3},{},{},{},{}",
                m.similarity,
                m.filename,
                if embedding.is_full_file {
                    "full"
                } else {
                    "chunk"
                },
                embedding.chunk_offset,
                embedding.chunk_size
            );
        }
    } else if file_contents {
        for m in &final_matches {
            let embedding = store
                .rag_vectors
                .iter()
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
                            chunk_content,
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
