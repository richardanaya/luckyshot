use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
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

    println!("Response status: {}", response.status());
    println!("Response headers: {:#?}", response.headers());
    
    let response_text = response.text().await?;
    println!("Raw response: {}", response_text);
    let embedding_response: EmbeddingResponse = serde_json::from_str(&response_text)?;
    Ok(embedding_response.data[0].embedding.clone())
}
