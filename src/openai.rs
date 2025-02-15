use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::vec::Vec;
use std::fs;
use std::collections::HashMap;

#[derive(Debug)]
pub struct FileMatch {
    pub filename: String,
    pub similarity: f32,
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
}

pub async fn find_related_files(query_embedding: Vec<f32>) -> Vec<String> {
    // Load the vectors file
    let vectors_content = match fs::read_to_string(".luckyshot.file.vectors.v1") {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading vectors file: {}", e);
            return Vec::new();
        }
    };

    // Parse the JSON
    let file_embeddings: HashMap<String, Vec<f32>> = match serde_json::from_str(&vectors_content) {
        Ok(map) => map,
        Err(e) => {
            eprintln!("Error parsing vectors file: {}", e);
            return Vec::new();
        }
    };

    // Calculate similarity for each file
    let mut matches: Vec<FileMatch> = file_embeddings
        .iter()
        .map(|(filename, embedding)| {
            let similarity = bm25_similarity(&query_embedding, embedding);
            FileMatch {
                filename: filename.clone(),
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

    // Print matches with normalized similarity scores
    for m in &matches {
        println!("Similarity {:.3}: {}", m.similarity, m.filename);
    }

    // Return filenames only
    matches.iter()
        .map(|m| m.filename.clone())
        .collect()
}

fn bm25_similarity(query: &[f32], doc: &[f32]) -> f32 {
    const K1: f32 = 1.5;  // Term frequency saturation parameter
    const B: f32 = 0.75;  // Length normalization parameter
    const EPSILON: f32 = 1e-10;  // Small value to prevent division by zero

    // Calculate average document length (in this case, it's always the embedding dimension)
    let avg_dl = doc.len() as f32;
    let doc_len = doc.len() as f32;

    // Calculate BM25 score
    let mut score = 0.0;
    for (q, d) in query.iter().zip(doc.iter()) {
        // Treat the embedding components as term frequencies
        let tf = d.abs();
        
        // IDF-like component using the query values
        let idf = (q.abs() + EPSILON).ln();
        
        // BM25 formula
        let numerator = tf * (K1 + 1.0);
        let denominator = tf + K1 * (1.0 - B + B * doc_len / avg_dl);
        
        score += idf * numerator / denominator;
    }

    score
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    model: String,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: i32,
    object: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: i32,
    total_tokens: i32,
}

pub async fn get_embedding(text: &str, api_key: &str) -> Result<Vec<f32>, Box<dyn Error>> {
    let client = Client::new();
    let request = EmbeddingRequest {
        input: text.to_string(),
        model: "text-embedding-ada-002".to_string(),
    };

    let response = client
        .post("https://api.openai.com/v1/embeddings")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    let response_text = response.text().await?;
    let embedding_response: EmbeddingResponse = serde_json::from_str(&response_text)?;
    Ok(embedding_response.data[0].embedding.clone())
}
