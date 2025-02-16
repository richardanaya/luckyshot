use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::vec::Vec;
use std::fs;

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

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

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

pub async fn find_related_files(query_embedding: Vec<f32>, filter_similarity: f32, verbose: bool, file_contents: bool, count: usize) -> Vec<String> {
    // Load the vectors file
    let vectors_content = match fs::read_to_string(".luckyshot.file.vectors.v1") {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading vectors file: {}", e);
            return Vec::new();
        }
    };

    // Parse the JSON
    let file_embeddings: Vec<crate::scan::FileEmbedding> = match serde_json::from_str(&vectors_content) {
        Ok(embeddings) => embeddings,
        Err(e) => {
            eprintln!("Error parsing vectors file: {}", e);
            return Vec::new();
        }
    };

    // Calculate similarity for each file
    let mut matches: Vec<FileMatch> = file_embeddings
        .iter()
        .map(|embedding| {
            let similarity = bm25_similarity(&query_embedding, &embedding.vector);
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

    // Filter matches by similarity threshold and limit count if specified
    let filtered_matches: Vec<_> = matches.iter()
        .filter(|m| m.similarity >= filter_similarity)
        .take(if count > 0 { count } else { matches.len() })
        .collect();

    // Return early if no matches
    if filtered_matches.is_empty() {
        return Vec::new();
    }

    // Print header for verbose output
    if verbose {
        println!("Score,File,Type,Offset,Size");
    }

    // Return filenames only from filtered matches and print their details
    let result: Vec<String> = filtered_matches.iter()
        .map(|m| {
            let embedding = file_embeddings.iter()
                .find(|e| e.filename == m.filename)
                .unwrap();
            
            if verbose {
                println!("{:.3},{},{},{},{}",
                    m.similarity,
                    m.filename,
                    if embedding.is_full_file { "full" } else { "chunk" },
                    embedding.chunk_offset,
                    embedding.chunk_size
                );
            } else {
                println!("{}", m.filename);
            }
            m.filename.clone()
        })
        .collect();

    // Print file contents for filtered matches
    if file_contents {
        for m in &filtered_matches {
            let embedding = file_embeddings.iter()
                .find(|e| e.filename == m.filename)
                .unwrap();
            
            if let Ok(contents) = std::fs::read_to_string(&embedding.filename) {
                let start = embedding.chunk_offset;
                let end = start + embedding.chunk_size;
                if end <= contents.len() {
                    println!("\n--- Content from {} ---", embedding.filename);
                    println!("{}", &contents[start..end]);
                    println!("--- End content ---\n");
                }
            }
        }
    }
    
    result
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

pub async fn get_chat_completion(prompt: &str, api_key: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let request = ChatRequest {
        model: "gpt-4o-mini".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    let chat_response: ChatResponse = response.json().await?;
    Ok(chat_response.choices[0].message.content.clone())
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

    let response_text = response.text().await?;
    let embedding_response: EmbeddingResponse = serde_json::from_str(&response_text)?;
    Ok(embedding_response.data[0].embedding.clone())
}
