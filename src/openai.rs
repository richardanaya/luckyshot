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

    // Calculate cosine similarity for each file
    let mut matches: Vec<FileMatch> = file_embeddings
        .iter()
        .map(|(filename, embedding)| {
            let similarity = cosine_similarity(&query_embedding, embedding);
            FileMatch {
                filename: filename.clone(),
                similarity,
            }
        })
        .collect();

    // Sort by similarity (highest first)
    matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

    // Print matches with similarity scores
    for m in &matches {
        println!("Similarity {:.3}: {}", m.similarity, m.filename);
    }

    // Return filenames only
    matches.iter()
        .map(|m| m.filename.clone())
        .collect()
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
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
