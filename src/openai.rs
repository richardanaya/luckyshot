use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::vec::Vec;

#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChatChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

pub async fn get_openai_chat_completion(prompt: &str, system_prompt: &str, api_key: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let request = OpenAIChatRequest {
        model: "gpt-4o-mini".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ],
        temperature: 0.7,
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    let chat_response: OpenAIChatResponse = response.json().await?;
    Ok(chat_response.choices[0].message.content.clone())
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

    let embedding_response: EmbeddingResponse = response.json().await?;
    Ok(embedding_response.data[0].embedding.clone())
}
