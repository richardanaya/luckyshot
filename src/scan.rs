use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct FileVectorStore {
    pub rag_vectors: Vec<RagEmbeddedFileChunk>,
    pub bm25_files: Vec<Bm25EmbeddedFile>,
    pub pattern: String,
    pub chunk_size: usize,
    pub overlap_size: usize,
    pub embed_metadata: bool,
    pub date: u64,
    pub bm25_avgdl: f32,
    pub doc_count: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Bm25EmbeddedFile {
    pub filename: String,
    pub bm25_indices: Vec<u32>,
    pub bm25_values: Vec<f32>,
    pub tokens: Vec<String>,
    pub token_count: usize,
    pub last_modified: u64,
    pub has_metadata: bool,  // Whether metadata was included in the embedding
}

#[derive(Serialize, Deserialize)]
pub struct RagEmbeddedFileChunk {
    pub filename: String,
    pub vector: Vec<f32>,
    pub last_modified: u64,
    pub chunk_offset: usize, // Starting position of chunk in file
    pub chunk_size: usize,   // Size of this chunk (might be smaller for last chunk)
    pub is_full_file: bool,  // Whether this is a full file embedding or a chunk
    pub has_metadata: bool,  // Whether metadata was included in the embedding
}

pub async fn scan_files(
    pattern: &str,
    api_key: &str,
    chunk_size: usize,
    overlap_size: usize,
    embed_metadata: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if chunk_size > 0 && overlap_size >= chunk_size {
        return Err("overlap_size must be less than chunk_size".into());
    }
    println!("Scanning for files matching pattern: {}", pattern);
    let mut store = FileVectorStore {
        rag_vectors: Vec::new(),
        bm25_files: Vec::new(),
        pattern: pattern.to_string(),
        chunk_size: chunk_size,
        overlap_size: overlap_size,
        embed_metadata: embed_metadata,
        date: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        bm25_avgdl: 0.0,
        doc_count: 0,
    };

    // For calculating average document length
    let mut total_tokens = 0;
    let mut doc_count = 0;

    fn create_chunks(
        content: &str,
        chunk_size: usize,
        overlap_size: usize,
    ) -> Vec<(usize, String)> {
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
        store: &mut FileVectorStore,
        api_key: &str,
        chunk_size: usize,
        overlap_size: usize,
        embed_metadata: bool,
        total_tokens: &mut usize,
        doc_count: &mut usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Processing: {}", path_str);

        // Read the file contents
        let contents = fs::read_to_string(path)?;
        let metadata = fs::metadata(path)?;
        let last_modified = metadata
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        // Phase 1: Create BM25 embedding for the entire file
        let content_to_embed = if embed_metadata {
            let size = metadata.len();
            crate::metadata::prepend_metadata(path_str, last_modified, size, &contents)
        } else {
            contents.clone()
        };

        // Generate BM25 vector for the entire file
        let bm25_vec = crate::bm25_embedder::create_bm25_vector(&content_to_embed, 200.0);

        // Tokenize and deduplicate tokens for BM25
        let tokens = crate::tokenize_code::tokenize_code(&content_to_embed, path_str);
        let deduped_tokens = crate::tokenize_code::deduplicate_tokens(tokens);
        
        // Update token counts for avgdl
        *total_tokens += deduped_tokens.len();
        *doc_count += 1;
        store.bm25_avgdl = *total_tokens as f32 / *doc_count as f32;
        store.doc_count = *doc_count;

        // Store the BM25 embedding
        store.bm25_files.push(Bm25EmbeddedFile {
            filename: path_str.to_string(),
            bm25_indices: bm25_vec.indices,
            bm25_values: bm25_vec.values,
            tokens: deduped_tokens.clone(),
            token_count: deduped_tokens.len(),
            last_modified,
            has_metadata: embed_metadata,
        });

        // Phase 2: Create RAG embeddings for chunks
        let chunks = create_chunks(&contents, chunk_size, overlap_size);

        for (offset, chunk_content) in chunks {
            let chunk_to_embed = if embed_metadata {
                let size = metadata.len();
                crate::metadata::prepend_metadata(path_str, last_modified, size, &chunk_content)
            } else {
                chunk_content.clone()
            };

            match crate::openai::get_embedding(&chunk_to_embed, api_key).await {
                Ok(embedding) => {
                    store.rag_vectors.push(RagEmbeddedFileChunk {
                        filename: path_str.to_string(),
                        vector: embedding,
                        last_modified,
                        chunk_offset: offset,
                        chunk_size: chunk_content.len(),
                        is_full_file: chunk_size == 0,
                        has_metadata: embed_metadata,
                    });

                    if chunk_size > 0 {
                        println!(
                            "Got embedding for {} (chunk offset: {})",
                            path_str, offset
                        );
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

        process_file(
            &path,
            &path_str,
            &mut store,
            api_key,
            chunk_size,
            overlap_size,
            embed_metadata,
            &mut total_tokens,
            &mut doc_count,
        )
        .await?;
    }

    // Save embeddings to file
    match serde_json::to_string_pretty(&store) {
        Ok(json) => {
            if let Err(e) = fs::write(".luckyshot.file.vectors.v1", json) {
                eprintln!("Error writing vectors file: {}", e);
            } else {
                println!(
                    "Successfully saved vectors for {} chunks",
                    store.rag_vectors.len()
                );

                // Print celebratory figlet
                use colored::*;
                use figlet_rs::FIGfont;

                let standard_font = FIGfont::standard().unwrap();
                let figure = standard_font.convert("Yee-haw!").unwrap();
                println!("\n{}", figure.to_string().bright_yellow());
            }
        }
        Err(e) => eprintln!("Error serializing vectors: {}", e),
    }

    Ok(())
}
